/// A simple staging mechanism for incoming requests so we can invert from
/// pull to push. Used by `hsmproxy` to stage requests that can then
/// asynchronously be retrieved and processed by one or more client
/// devices.
///
/// The stage also tracks [`Presence`]: whether a signer is attached and
/// answering right now. Dialling a peer while no signer answers freezes
/// `connectd` (its ECDH is a blocking hsmd call), so callers gate dials
/// on [`Stage::check_answering`].
use crate::pb;
use anyhow::{anyhow, Error};
use futures::future::{BoxFuture, FutureExt, Shared};
use log::{debug, info, trace, warn};
use std::collections;
use std::sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc, Mutex as SyncMutex, Once, Weak,
};
use tokio::sync::{broadcast, mpsc, watch, Mutex};
use tokio::time::{Duration, Instant};

/// A signer that has not replied for this long while a request is
/// outstanding is considered [`Presence::Silent`]. The same window
/// decides whether a recent reply is fresh enough to skip a probe.
pub const SILENT_AFTER: Duration = Duration::from_secs(10);

/// How long [`Stage::check_answering`] waits for a probe reply.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// How often the watchdog checks for overdue requests.
const WATCHDOG_INTERVAL: Duration = Duration::from_secs(1);

/// Probes use request ids `1..=PROBE_ID_RANGE`. `hsmproxy` numbers its
/// requests from 1000 upwards, and the attach heartbeat uses 0, so the
/// two never collide.
const PROBE_ID_RANGE: u32 = 999;

/// Whether a signer is attached and answering right now. Unlike
/// "a signer attached at some point", this can go back to `Silent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    /// No signer is attached, or the attached one stopped replying.
    Silent,
    /// A signer is attached and has replied since it attached.
    Answering,
}

type Probe = Shared<BoxFuture<'static, Presence>>;

pub struct Stage {
    requests: Arc<Mutex<collections::HashMap<u32, Request>>>,
    notify: broadcast::Sender<Request>,
    hsm_connections: Arc<AtomicUsize>,
    presence: Arc<watch::Sender<Presence>>,
    last_reply: Arc<SyncMutex<Option<Instant>>>,
    /// The probe in flight, shared by concurrent callers.
    probe: SyncMutex<Option<Probe>>,
    probe_counter: AtomicU32,
    watchdog: Once,
}

impl std::fmt::Debug for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stage")
            .field("hsm_connections", &self.hsm_connections)
            .field("presence", &*self.presence.borrow())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub request: pb::HsmRequest,
    pub response: mpsc::Sender<pb::HsmResponse>,
    pub start_time: tokio::time::Instant,
}

impl Stage {
    pub fn new() -> Self {
        let (notify, _) = broadcast::channel(1000);
        let (presence, _) = watch::channel(Presence::Silent);
        Stage {
            requests: Arc::new(Mutex::new(collections::HashMap::new())),
            notify: notify,
            hsm_connections: Arc::new(AtomicUsize::new(0)),
            presence: Arc::new(presence),
            last_reply: Arc::new(SyncMutex::new(None)),
            probe: SyncMutex::new(None),
            probe_counter: AtomicU32::new(0),
            watchdog: Once::new(),
        }
    }

    pub async fn send(
        &self,
        request: pb::HsmRequest,
    ) -> Result<mpsc::Receiver<pb::HsmResponse>, Error> {
        let mut requests = self.requests.lock().await;
        let (response, receiver): (
            mpsc::Sender<pb::HsmResponse>,
            mpsc::Receiver<pb::HsmResponse>,
        ) = mpsc::channel(1);

        let r = Request {
            request,
            response,
            start_time: tokio::time::Instant::now(),
        };

        requests.insert(r.request.request_id, r.clone());

        if let Err(_) = self.notify.send(r) {
            warn!("Error notifying hsmd request stream, likely lost connection.");
        }

        Ok(receiver)
    }

    pub async fn mystream(&self) -> StageStream {
        self.watchdog.call_once(|| self.spawn_watchdog());
        let requests = self.requests.lock().await;
        self.hsm_connections.fetch_add(1, Ordering::SeqCst);
        StageStream {
            backlog: requests.values().map(|e| e.clone()).collect(),
            bcast: self.notify.subscribe(),
            hsm_connections: self.hsm_connections.clone(),
            presence: self.presence.clone(),
        }
    }

    pub async fn respond(&self, response: pb::HsmResponse) -> Result<(), Error> {
        self.note_reply();
        let mut requests = self.requests.lock().await;
        match requests.remove(&response.request_id) {
            Some(req) => {
                debug!(
                    "Response for request_id={}, signer_rtt={}s, outstanding requests count={}",
                    response.request_id,
                    req.start_time.elapsed().as_secs_f64(),
                    requests.len()
                );
                if let Err(e) = req.response.send(response).await {
                    Err(anyhow!("Error sending request to requester: {:?}", e))
                } else {
                    Ok(())
                }
            }
            None => {
                trace!(
                    "Request {} not found, is this a duplicate result?",
                    response.request_id
                );
                Ok(())
            }
        }
    }

    /// Subscribe to the signer presence. It starts `Silent`.
    pub fn presence(&self) -> watch::Receiver<Presence> {
        self.presence.subscribe()
    }

    /// Record that a signer replied, whatever the reply was. An error
    /// reply still proves the signer is there.
    pub fn note_reply(&self) {
        *self.last_reply.lock().unwrap() = Some(Instant::now());
        if self.hsm_connections.load(Ordering::SeqCst) > 0 {
            set_presence(&self.presence, Presence::Answering);
        }
    }

    fn replied_within(&self, window: Duration) -> bool {
        self.last_reply
            .lock()
            .unwrap()
            .map_or(false, |t| t.elapsed() <= window)
    }

    /// The dial gate: is a signer answering right now?
    ///
    /// A reply within [`SILENT_AFTER`] answers yes without asking the
    /// signer. Otherwise we stage a `GetHeartbeat` probe and wait up to
    /// [`PROBE_TIMEOUT`] for its reply. Concurrent callers share one
    /// probe. With no signer attached, the answer is `Silent` at once.
    pub async fn check_answering(self: &Arc<Self>) -> Presence {
        if self.hsm_connections.load(Ordering::SeqCst) == 0 {
            return Presence::Silent;
        }
        if self.replied_within(SILENT_AFTER) {
            return Presence::Answering;
        }
        let probe = {
            let mut slot = self.probe.lock().unwrap();
            match &*slot {
                Some(p) => p.clone(),
                None => {
                    let this = self.clone();
                    let p = async move { this.run_probe().await }.boxed().shared();
                    *slot = Some(p.clone());
                    p
                }
            }
        };
        probe.await
    }

    /// Wait until [`Stage::check_answering`] says `Answering`, for at
    /// most `within`. Returns whether it did.
    pub async fn await_answering(self: &Arc<Self>, within: Duration) -> bool {
        let deadline = Instant::now() + within;
        let mut rx = self.presence.subscribe();
        loop {
            match tokio::time::timeout_at(deadline, self.check_answering()).await {
                Ok(Presence::Answering) => return true,
                Ok(Presence::Silent) => {}
                Err(_) => return false,
            }
            // Silent: wait for a reply to flip the presence before
            // asking again, rather than probing in a loop.
            loop {
                match tokio::time::timeout_at(deadline, rx.changed()).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) | Err(_) => return false,
                }
                if *rx.borrow_and_update() == Presence::Answering {
                    break;
                }
            }
        }
    }

    async fn run_probe(self: Arc<Self>) -> Presence {
        use vls_protocol::msgs::SerBolt;
        let request_id = 1 + self.probe_counter.fetch_add(1, Ordering::Relaxed) % PROBE_ID_RANGE;
        trace!("Probing signer presence with request_id={}", request_id);
        let req = pb::HsmRequest {
            request_id,
            context: None,
            raw: vls_protocol::msgs::GetHeartbeat {}.as_vec(),
            signer_state: vec![],
            requests: vec![],
        };
        let replied = match self.send(req).await {
            Ok(mut rx) => matches!(
                tokio::time::timeout(PROBE_TIMEOUT, rx.recv()).await,
                Ok(Some(_))
            ),
            Err(_) => false,
        };
        let presence = if replied {
            Presence::Answering
        } else {
            // Don't leave the probe behind for a later signer to answer.
            self.requests.lock().await.remove(&request_id);
            info!(
                "Signer did not answer a presence probe within {:?}, marking it silent",
                PROBE_TIMEOUT
            );
            set_presence(&self.presence, Presence::Silent);
            Presence::Silent
        };
        *self.probe.lock().unwrap() = None;
        presence
    }

    /// Flip `Answering` to `Silent` once a request has gone unanswered
    /// for [`SILENT_AFTER`]. This catches a backgrounded app whose
    /// stream is half-open: it still accepts requests but never replies.
    fn spawn_watchdog(&self) {
        let presence = Arc::downgrade(&self.presence);
        let requests = self.requests.clone();
        let last_reply = self.last_reply.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(WATCHDOG_INTERVAL);
            loop {
                tick.tick().await;
                let Some(presence) = Weak::upgrade(&presence) else {
                    break;
                };
                if *presence.borrow() != Presence::Answering {
                    continue;
                }
                let last = *last_reply.lock().unwrap();
                let starts: Vec<Instant> = requests
                    .lock()
                    .await
                    .values()
                    .map(|r| r.start_time)
                    .collect();
                if overdue(Instant::now(), last, starts) {
                    warn!(
                        "A signer request is unanswered for more than {:?}, marking the signer silent",
                        SILENT_AFTER
                    );
                    set_presence(&presence, Presence::Silent);
                }
            }
        });
    }

    /// Returns the HSM request types currently queued that block node progress.
    /// An empty vec means the node is not stuck.
    pub async fn stuck_request_types(&self) -> Vec<u16> {
        let sticky_types: Vec<u16> = vec![
            1,   // WIRE_HSMD_ECDH (connectd, blocks all peer I/O)
            5,   // WIRE_HSMD_SIGN_COMMITMENT_TX
            12,  // WIRE_HSMD_SIGN_DELAYED_PAYMENT_TO_US
            13,  // WIRE_HSMD_SIGN_REMOTE_HTLC_TO_US
            14,  // WIRE_HSMD_SIGN_PENALTY_TO_US
            18,  // WIRE_HSMD_GET_PER_COMMITMENT_POINT (onchaind key derivation)
            20,  // WIRE_HSMD_SIGN_REMOTE_HTLC_TX
            21,  // WIRE_HSMD_SIGN_MUTUAL_CLOSE_TX
            28,  // WIRE_HSMD_CHECK_PUBKEY
            34,  // WIRE_HSMD_FORGET_CHANNEL (lightningd delete_channel, sync)
            142, // WIRE_HSMD_SIGN_ANY_DELAYED_PAYMENT_TO_US
            143, // WIRE_HSMD_SIGN_ANY_REMOTE_HTLC_TO_US
            144, // WIRE_HSMD_SIGN_ANY_PENALTY_TO_US
            146, // WIRE_HSMD_SIGN_ANY_LOCAL_HTLC_TX
            147, // WIRE_HSMD_SIGN_ANCHORSPEND
            149, // WIRE_HSMD_SIGN_HTLC_TX_MINGLE
        ];
        let types: Vec<u16> = self
            .requests
            .lock()
            .await
            .values()
            .filter_map(|r| {
                let head: [u16; 2] = [r.request.raw[0].into(), r.request.raw[1].into()];
                let typ = head[0] << 8 | head[1];
                if sticky_types.contains(&typ) {
                    Some(typ)
                } else {
                    None
                }
            })
            .collect();

        trace!("Found stuck request types: {:?}", types);
        types
    }

    pub async fn is_stuck(&self) -> bool {
        !self.stuck_request_types().await.is_empty()
    }
}

fn set_presence(tx: &watch::Sender<Presence>, value: Presence) {
    tx.send_if_modified(|p| {
        if *p == value {
            false
        } else {
            info!("Signer presence changed from {:?} to {:?}", *p, value);
            *p = value;
            true
        }
    });
}

/// Is any outstanding request overdue? A request counts from when it
/// was staged or from the last reply, whichever is later: a signer that
/// just attached and replied is not blamed for the backlog it inherited,
/// and one that keeps replying is not blamed for a request it rejected
/// (an error reply never clears the request from the stage).
fn overdue(now: Instant, last_reply: Option<Instant>, starts: Vec<Instant>) -> bool {
    starts.into_iter().any(|start| {
        let since = last_reply.map_or(start, |r| r.max(start));
        now.saturating_duration_since(since) > SILENT_AFTER
    })
}

pub struct StageStream {
    backlog: Vec<Request>,
    bcast: broadcast::Receiver<Request>,
    hsm_connections: Arc<AtomicUsize>,
    presence: Arc<watch::Sender<Presence>>,
}

impl StageStream {
    pub async fn next(&mut self) -> Result<Request, Error> {
        if self.backlog.len() > 0 {
            let req = self.backlog[0].clone();
            self.backlog.remove(0);
            Ok(req)
        } else {
            match self.bcast.recv().await {
                Ok(r) => Ok(r),
                Err(e) => Err(anyhow!("error waiting for more requests: {:?}", e)),
            }
        }
    }
}

impl Drop for StageStream {
    fn drop(&mut self) {
        if self.hsm_connections.fetch_sub(1, Ordering::SeqCst) == 1 {
            set_presence(&self.presence, Presence::Silent);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep as delay_for;

    #[tokio::test]
    async fn test_live_stream() {
        let stage = Stage::new();

        let mut responses = vec![];

        for i in 0..10 {
            responses.push(
                stage
                    .send(pb::HsmRequest {
                        request_id: i,
                        context: None,
                        raw: vec![],
                        signer_state: vec![],
                        requests: vec![],
                    })
                    .await
                    .unwrap(),
            );
        }

        let mut s1 = stage.mystream().await;
        let mut s2 = stage.mystream().await;
        let f1 = tokio::spawn(async move {
            while let Ok(r) = s1.next().await {
                eprintln!("hsmd {} is handling request {}", 1, r.request.request_id);
                match r
                    .response
                    .send(pb::HsmResponse {
                        request_id: r.request.request_id,
                        raw: vec![],
                        signer_state: vec![],
                        error: "".into(),
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!("errror {:?}", e),
                }
                delay_for(Duration::from_millis(10)).await;
            }
        });
        let f2 = tokio::spawn(async move {
            while let Ok(r) = s2.next().await {
                eprintln!("hsmd {} is handling request {}", 2, r.request.request_id);
                match r
                    .response
                    .send(pb::HsmResponse {
                        request_id: r.request.request_id,
                        raw: vec![],
                        signer_state: vec![],
                        error: "".into(),
                    })
                    .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!("errror {:?}", e),
                }
                delay_for(Duration::from_millis(10)).await;
            }
        });

        for i in 10..100 {
            responses.push(
                stage
                    .send(pb::HsmRequest {
                        request_id: i,
                        context: None,
                        raw: vec![],
                        signer_state: vec![],
                        requests: vec![],
                    })
                    .await
                    .unwrap(),
            );
        }

        for mut r in responses {
            let resp = r.recv().await.unwrap();
            eprintln!("Got response {:?}", resp);
        }

        drop(stage);
        f1.await.unwrap();
        f2.await.unwrap();
    }

    #[tokio::test]
    async fn test_stuck_request_types() {
        // WIRE_HSMD_FORGET_CHANNEL (34) is queued but never answered: this is
        // the bgsync deadlock, lightningd's delete_channel() blocks on it
        // synchronously, so the node must report itself as stuck.
        let stage = Stage::new();
        stage
            .send(pb::HsmRequest {
                request_id: 0,
                context: None,
                raw: vec![0, 34, 0, 0, 0, 0, 0, 0, 0, 1],
                signer_state: vec![],
                requests: vec![],
            })
            .await
            .unwrap();
        assert_eq!(stage.stuck_request_types().await, vec![34]);
        assert!(stage.is_stuck().await);

        // WIRE_HSMD_DERIVE_SECRET (27) is not on the sticky list, a queued
        // request of this type alone must not mark the node as stuck.
        let stage = Stage::new();
        stage
            .send(pb::HsmRequest {
                request_id: 0,
                context: None,
                raw: vec![0, 27, 0, 0, 0, 0, 0, 0, 0, 1],
                signer_state: vec![],
                requests: vec![],
            })
            .await
            .unwrap();
        assert_eq!(stage.stuck_request_types().await, Vec::<u16>::new());
        assert!(!stage.is_stuck().await);
    }

    fn req(request_id: u32, raw: Vec<u8>) -> pb::HsmRequest {
        pb::HsmRequest {
            request_id,
            context: None,
            raw,
            signer_state: vec![],
            requests: vec![],
        }
    }

    fn reply(request_id: u32) -> pb::HsmResponse {
        pb::HsmResponse {
            request_id,
            raw: vec![],
            signer_state: vec![],
            error: "".into(),
        }
    }

    /// Attach a signer and answer the attach heartbeat (request_id 0,
    /// which is streamed directly and never staged).
    async fn attach_and_answer(stage: &Stage) -> StageStream {
        let stream = stage.mystream().await;
        stage.respond(reply(0)).await.unwrap();
        stream
    }

    /// A signer that answers every staged request.
    fn spawn_answering_signer(stage: Arc<Stage>, mut stream: StageStream) {
        tokio::spawn(async move {
            while let Ok(r) = stream.next().await {
                stage.respond(reply(r.request.request_id)).await.unwrap();
            }
        });
    }

    #[tokio::test(start_paused = true)]
    async fn presence_answering_after_attach_and_reply() {
        let stage = Stage::new();
        let rx = stage.presence();
        assert_eq!(*rx.borrow(), Presence::Silent);

        let _stream = stage.mystream().await;
        // Attached but not replied yet.
        assert_eq!(*rx.borrow(), Presence::Silent);

        stage.respond(reply(0)).await.unwrap();
        assert_eq!(*rx.borrow(), Presence::Answering);
    }

    #[tokio::test(start_paused = true)]
    async fn presence_silent_after_request_outstanding_too_long() {
        let stage = Stage::new();
        let rx = stage.presence();
        let _stream = attach_and_answer(&stage).await;
        let _pending = stage.send(req(1000, vec![0, 5])).await.unwrap();

        delay_for(SILENT_AFTER - Duration::from_secs(1)).await;
        assert_eq!(*rx.borrow(), Presence::Answering);

        delay_for(Duration::from_secs(3)).await;
        assert_eq!(*rx.borrow(), Presence::Silent);
    }

    #[tokio::test(start_paused = true)]
    async fn presence_ignores_backlog_inherited_on_attach() {
        let stage = Stage::new();
        let rx = stage.presence();
        let _pending = stage.send(req(1000, vec![0, 5])).await.unwrap();
        delay_for(Duration::from_secs(60)).await;

        // The backlog is a minute old, but the signer just replied.
        let _stream = attach_and_answer(&stage).await;
        delay_for(Duration::from_secs(5)).await;
        assert_eq!(*rx.borrow(), Presence::Answering);

        // It is blamed once it stays quiet for the full window.
        delay_for(SILENT_AFTER).await;
        assert_eq!(*rx.borrow(), Presence::Silent);
    }

    #[tokio::test(start_paused = true)]
    async fn presence_silent_when_stream_exits() {
        let stage = Stage::new();
        let rx = stage.presence();
        let s1 = attach_and_answer(&stage).await;
        let s2 = stage.mystream().await;

        // One of two signers leaving keeps the other one answering.
        drop(s1);
        assert_eq!(*rx.borrow(), Presence::Answering);

        drop(s2);
        assert_eq!(*rx.borrow(), Presence::Silent);

        // A late reply from a detached signer doesn't revive presence.
        stage.respond(reply(0)).await.unwrap();
        assert_eq!(*rx.borrow(), Presence::Silent);
    }

    #[tokio::test(start_paused = true)]
    async fn check_answering_fresh_reply_sends_no_probe() {
        let stage = Arc::new(Stage::new());
        let _stream = attach_and_answer(&stage).await;
        delay_for(SILENT_AFTER - Duration::from_secs(1)).await;

        assert_eq!(stage.check_answering().await, Presence::Answering);
        assert!(stage.requests.lock().await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn check_answering_without_signer_is_silent_at_once() {
        let stage = Arc::new(Stage::new());
        let start = Instant::now();
        assert_eq!(stage.check_answering().await, Presence::Silent);
        assert_eq!(start.elapsed(), Duration::ZERO);
        assert!(stage.requests.lock().await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn check_answering_probe_timeout_is_silent() {
        let stage = Arc::new(Stage::new());
        let rx = stage.presence();
        // A half-open stream: attached, answered once, then quiet.
        let _stream = attach_and_answer(&stage).await;
        delay_for(SILENT_AFTER + Duration::from_secs(1)).await;
        assert_eq!(*rx.borrow(), Presence::Answering);

        let start = Instant::now();
        assert_eq!(stage.check_answering().await, Presence::Silent);
        assert_eq!(start.elapsed(), PROBE_TIMEOUT);
        assert_eq!(*rx.borrow(), Presence::Silent);
        // The unanswered probe is not left in the stage.
        assert!(stage.requests.lock().await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn check_answering_probe_reply_is_answering() {
        let stage = Arc::new(Stage::new());
        let stream = attach_and_answer(&stage).await;
        delay_for(SILENT_AFTER + Duration::from_secs(1)).await;

        let mut seen = stage.notify.subscribe();
        spawn_answering_signer(stage.clone(), stream);
        let (a, b) = tokio::join!(stage.check_answering(), stage.check_answering());
        assert_eq!((a, b), (Presence::Answering, Presence::Answering));

        // Both callers shared one probe, a GetHeartbeat below hsmproxy's ids.
        let probe = seen.try_recv().unwrap();
        assert!((1..1000).contains(&probe.request.request_id));
        use vls_protocol::msgs::SerBolt;
        assert_eq!(
            probe.request.raw,
            vls_protocol::msgs::GetHeartbeat {}.as_vec()
        );
        assert!(seen.try_recv().is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn await_answering_waits_for_attach() {
        let stage = Arc::new(Stage::new());
        let s = stage.clone();
        tokio::spawn(async move {
            delay_for(Duration::from_secs(3)).await;
            let stream = attach_and_answer(&s).await;
            std::mem::forget(stream);
        });
        let start = Instant::now();
        assert!(stage.await_answering(Duration::from_secs(10)).await);
        assert_eq!(start.elapsed(), Duration::from_secs(3));
    }

    #[tokio::test(start_paused = true)]
    async fn await_answering_times_out_without_signer() {
        let stage = Arc::new(Stage::new());
        let start = Instant::now();
        assert!(!stage.await_answering(Duration::from_secs(10)).await);
        assert_eq!(start.elapsed(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_ecdh_is_stuck() {
        // An unanswered WIRE_HSMD_ECDH (1) freezes all of connectd.
        let stage = Stage::new();
        let _pending = stage.send(req(1000, vec![0, 1, 0, 0])).await.unwrap();
        assert_eq!(stage.stuck_request_types().await, vec![1]);
    }
}
