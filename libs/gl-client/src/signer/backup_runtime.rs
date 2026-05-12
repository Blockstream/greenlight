#[cfg(feature = "backup")]
mod enabled {
    use super::super::backup::{self, SignerBackupConfig};
    use super::super::SignerConfig;
    use crate::persist::State;
    use anyhow::Result;
    use log::error;
    use std::sync::{Arc, Mutex};

    pub(crate) type Pending = Option<(SignerBackupConfig, State)>;

    #[derive(Clone)]
    pub(crate) struct Runtime {
        config: Option<SignerBackupConfig>,
        runtime: Arc<Mutex<backup::BackupRuntime>>,
    }

    impl Runtime {
        pub(crate) fn new(config: &SignerConfig) -> Result<Self> {
            if let Some(backup) = &config.backup {
                backup.validate()?;
            }

            Ok(Self {
                config: config.backup.clone(),
                runtime: Arc::new(Mutex::new(backup::BackupRuntime::default())),
            })
        }

        pub(crate) fn before_request(&self, state: &State) -> State {
            state.clone()
        }

        pub(crate) fn after_request(&self, before: &State, final_state: &State) -> Pending {
            self.config.as_ref().and_then(|config| {
                let mut runtime = match self.runtime.lock() {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        error!("Signer backup runtime lock failed; skipping backup snapshot: {e}");
                        return None;
                    }
                };
                runtime
                    .observe(config.strategy, before, final_state)
                    .then(|| (config.clone(), final_state.omit_tombstones()))
            })
        }

        pub(crate) fn write_pending(&self, node_id: &[u8], pending: Pending) {
            if let Some((config, state)) = pending {
                match backup::write_snapshot(&config, node_id, state.clone()) {
                    Ok(()) => match self.runtime.lock() {
                        Ok(mut runtime) => runtime.snapshot_succeeded(&state),
                        Err(e) => {
                            error!(
                                "Signer backup runtime lock failed after successful snapshot: {e}"
                            )
                        }
                    },
                    Err(e) => {
                        error!("Signer backup failed; continuing without backup snapshot: {e}");
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "backup"))]
mod disabled {
    use super::super::SignerConfig;
    use crate::persist::State;
    use anyhow::Result;

    pub(crate) type Pending = ();

    #[derive(Clone)]
    pub(crate) struct Runtime;

    impl Runtime {
        pub(crate) fn new(_config: &SignerConfig) -> Result<Self> {
            Ok(Self)
        }

        pub(crate) fn before_request(&self, _state: &State) {}

        pub(crate) fn after_request(&self, _before: &(), _final_state: &State) -> Pending {}

        pub(crate) fn write_pending(&self, _node_id: &[u8], _pending: Pending) {}
    }
}

#[cfg(not(feature = "backup"))]
pub(crate) use disabled::Runtime;
#[cfg(feature = "backup")]
pub(crate) use enabled::Runtime;
