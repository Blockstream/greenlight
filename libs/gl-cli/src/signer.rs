use crate::error::{Error, Result};
use crate::util;
use clap::{Subcommand, ValueEnum};
use core::fmt::Debug;
use gl_client::signer::{
    RecoverableChannel, Signer, SignerBackupSnapshot, SignerBackupStrategy, SignerConfig,
    StateSignatureMode, StateSignatureOverrideConfig,
};
use lightning_signer::bitcoin::Network;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::{join, signal};
use util::{CREDENTIALS_FILE_NAME, SEED_FILE_NAME};

pub struct Config<P: AsRef<Path>> {
    pub data_dir: P,
    pub network: Network,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StateSignatureModeArg {
    Off,
    Soft,
    Hard,
}

impl Default for StateSignatureModeArg {
    fn default() -> Self {
        Self::Soft
    }
}

impl From<StateSignatureModeArg> for StateSignatureMode {
    fn from(value: StateSignatureModeArg) -> Self {
        match value {
            StateSignatureModeArg::Off => StateSignatureMode::Off,
            StateSignatureModeArg::Soft => StateSignatureMode::Soft,
            StateSignatureModeArg::Hard => StateSignatureMode::Hard,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum BackupInspectFormat {
    Json,
    Text,
}

impl Default for BackupInspectFormat {
    fn default() -> Self {
        Self::Json
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Starts a signer that connects to greenlight
    Run {
        #[arg(long, value_enum, default_value_t = StateSignatureModeArg::Soft)]
        state_signature_mode: StateSignatureModeArg,
        #[arg(long = "state-override")]
        state_override: Option<String>,
        #[arg(long = "state-override-note")]
        state_override_note: Option<String>,
    },
    /// Inspects a local signer backup file
    InspectBackup {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = BackupInspectFormat::Json)]
        format: BackupInspectFormat,
    },
    /// Prints the version of the signer used
    Version,
}

pub async fn command_handler<P: AsRef<Path>>(cmd: Command, config: Config<P>) -> Result<()> {
    match cmd {
        Command::Run {
            state_signature_mode,
            state_override,
            state_override_note,
        } => {
            run_handler(
                config,
                state_signature_mode,
                state_override,
                state_override_note,
            )
            .await
        }
        Command::InspectBackup { path, format } => inspect_backup(&path, format),
        Command::Version => version(config).await,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BackupInspectionReport {
    pub version: u32,
    pub created_at: String,
    pub node_id: String,
    pub strategy: SignerBackupStrategy,
    pub total_channels: usize,
    pub complete_channels: usize,
    pub incomplete_channels: usize,
    pub channels: Vec<RecoverableChannel>,
}

fn inspect_backup(path: &Path, format: BackupInspectFormat) -> Result<()> {
    let report = inspect_backup_report(path)?;

    match format {
        BackupInspectFormat::Json => {
            let output = serde_json::to_string_pretty(&report).map_err(Error::custom)?;
            println!("{output}");
        }
        BackupInspectFormat::Text => print!("{}", format_backup_report_text(&report)),
    }

    Ok(())
}

fn inspect_backup_report(path: &Path) -> Result<BackupInspectionReport> {
    let snapshot = SignerBackupSnapshot::read(path).map_err(|e| {
        Error::custom(format!(
            "failed to read signer backup {}: {}",
            path.display(),
            e
        ))
    })?;
    let channels = snapshot.recovery_data().map_err(|e| {
        Error::custom(format!(
            "failed to inspect signer backup {}: {}",
            path.display(),
            e
        ))
    })?;

    Ok(backup_inspection_report(snapshot, channels))
}

fn backup_inspection_report(
    snapshot: SignerBackupSnapshot,
    channels: Vec<RecoverableChannel>,
) -> BackupInspectionReport {
    let complete_channels = channels.iter().filter(|channel| channel.complete).count();
    let total_channels = channels.len();

    BackupInspectionReport {
        version: snapshot.version,
        created_at: snapshot.created_at,
        node_id: snapshot.node_id,
        strategy: snapshot.strategy,
        total_channels,
        complete_channels,
        incomplete_channels: total_channels - complete_channels,
        channels,
    }
}

fn format_backup_report_text(report: &BackupInspectionReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("Signer backup {}\n", report.node_id));
    output.push_str(&format!("version: {}\n", report.version));
    output.push_str(&format!("created_at: {}\n", report.created_at));
    output.push_str(&format!("strategy: {}\n", format_strategy(report.strategy)));
    output.push_str(&format!(
        "channels: total={} complete={} incomplete={}\n",
        report.total_channels, report.complete_channels, report.incomplete_channels
    ));

    for channel in &report.channels {
        let status = if channel.complete {
            "complete"
        } else {
            "incomplete"
        };
        let peer_addr = channel.peer_addr.as_deref().unwrap_or("missing");
        let warnings = if channel.warnings.is_empty() {
            "none".to_string()
        } else {
            channel.warnings.join(",")
        };

        output.push_str(&format!("\nchannel: {}\n", channel.channel_key));
        output.push_str(&format!("  status: {status}\n"));
        output.push_str(&format!("  peer_id: {}\n", channel.peer_id));
        output.push_str(&format!("  peer_addr: {peer_addr}\n"));
        output.push_str(&format!(
            "  funding: {}:{}\n",
            channel.funding_outpoint.txid, channel.funding_outpoint.vout
        ));
        output.push_str(&format!("  funding_sats: {}\n", channel.funding_sats));
        output.push_str(&format!("  opener: {:?}\n", channel.opener));
        output.push_str(&format!(
            "  remote_to_self_delay: {}\n",
            channel.remote_to_self_delay
        ));
        output.push_str(&format!("  commitment_type: {}\n", channel.commitment_type));
        output.push_str(&format!("  warnings: {warnings}\n"));
    }

    output
}

fn format_strategy(strategy: SignerBackupStrategy) -> String {
    match strategy {
        SignerBackupStrategy::NewChannelsOnly => "new_channels_only".to_string(),
        SignerBackupStrategy::Periodic { updates } => format!("periodic(updates={updates})"),
    }
}

async fn run_handler<P: AsRef<Path>>(
    config: Config<P>,
    state_signature_mode: StateSignatureModeArg,
    state_override: Option<String>,
    state_override_note: Option<String>,
) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("Seed not found");
        return Err(Error::SeedNotFoundError(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // Initialize a signer and scheduler with default credentials.
    let creds_path = config.data_dir.as_ref().join(CREDENTIALS_FILE_NAME);
    let creds = match util::read_credentials(&creds_path) {
        Some(c) => c,
        None => {
            return Err(Error::CredentialsNotFoundError(format!(
                "could not read from {}",
                creds_path.display()
            )))
        }
    };

    if state_override.is_none() && state_override_note.is_some() {
        return Err(Error::custom(
            "--state-override-note requires --state-override",
        ));
    }

    let state_signature_override = state_override.map(|ack| {
        StateSignatureOverrideConfig {
            ack,
            note: state_override_note,
        }
    });

    let signer = Signer::new_with_config(
        seed,
        config.network,
        creds.clone(),
        SignerConfig {
            state_signature_mode: state_signature_mode.into(),
            state_signature_override,
            backup: None,
        },
    )
    .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;

    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let handle = tokio::spawn(async move {
        let _ = signer.run_forever(rx).await;
    });

    _ = signal::ctrl_c().await.map_err(|e| Error::custom(e))?;

    _ = tx.send(()).await;
    _ = join!(handle);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        format_backup_report_text, inspect_backup_report, BackupInspectFormat, Command,
        StateSignatureModeArg,
    };
    use clap::{Parser, Subcommand};
    use serde_json::json;
    use std::path::Path;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        cmd: Command,
    }

    #[derive(Subcommand, Debug)]
    enum RootCommand {
        #[command(subcommand)]
        Signer(Command),
    }

    #[test]
    fn parse_run_mode_flag() {
        let cli = TestCli::parse_from(["test", "run", "--state-signature-mode", "hard"]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Hard);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn run_mode_defaults_to_soft() {
        let cli = TestCli::parse_from(["test", "run"]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Soft);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn signer_subcommand_parses_mode_flag() {
        #[derive(Parser, Debug)]
        struct WrapperCli {
            #[command(subcommand)]
            cmd: RootCommand,
        }

        let cli =
            WrapperCli::parse_from(["test", "signer", "run", "--state-signature-mode", "off"]);
        match cli.cmd {
            RootCommand::Signer(Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            }) => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Off);
                assert!(state_override.is_none());
                assert!(state_override_note.is_none());
            }
            _ => panic!("expected signer run"),
        }
    }

    #[test]
    fn parse_override_flags() {
        let cli = TestCli::parse_from([
            "test",
            "run",
            "--state-signature-mode",
            "hard",
            "--state-override",
            "I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE",
            "--state-override-note",
            "debug session",
        ]);
        match cli.cmd {
            Command::Run {
                state_signature_mode,
                state_override,
                state_override_note,
            } => {
                assert_eq!(state_signature_mode, StateSignatureModeArg::Hard);
                assert_eq!(
                    state_override.as_deref(),
                    Some("I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE")
                );
                assert_eq!(state_override_note.as_deref(), Some("debug session"));
            }
            _ => panic!("expected run command"),
        }
    }

    #[test]
    fn parse_inspect_backup_defaults_to_json() {
        let cli = TestCli::parse_from(["test", "inspect-backup", "--path", "backup.json"]);
        match cli.cmd {
            Command::InspectBackup { path, format } => {
                assert_eq!(path, Path::new("backup.json"));
                assert_eq!(format, BackupInspectFormat::Json);
            }
            _ => panic!("expected inspect-backup command"),
        }
    }

    #[test]
    fn parse_inspect_backup_text_format() {
        let cli = TestCli::parse_from([
            "test",
            "inspect-backup",
            "--path",
            "backup.json",
            "--format",
            "text",
        ]);
        match cli.cmd {
            Command::InspectBackup { path, format } => {
                assert_eq!(path, Path::new("backup.json"));
                assert_eq!(format, BackupInspectFormat::Text);
            }
            _ => panic!("expected inspect-backup command"),
        }
    }

    #[test]
    fn parse_inspect_backup_rejects_invalid_format() {
        assert!(TestCli::try_parse_from([
            "test",
            "inspect-backup",
            "--path",
            "backup.json",
            "--format",
            "yaml",
        ])
        .is_err());
    }

    #[test]
    fn inspect_backup_report_counts_channels_and_warnings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let peer_a = peer_id(0xaa);
        let peer_b = peer_id(0xbb);
        write_json(
            &path,
            backup_json(
                json!({
                    channel_key(&peer_a, 1): [1, channel(recovery_setup("00", 0, 1000, true))],
                    channel_key(&peer_a, 2): [1, channel(recovery_setup("11", 1, 2000, false))],
                    channel_key(&peer_b, 3): [1, channel(recovery_setup("22", 2, 3000, false))]
                }),
                json!([peerlist_entry(&peer_a, "127.0.0.1:9735")]),
                json!("new_channels_only"),
            ),
        );

        let report = inspect_backup_report(&path).unwrap();

        assert_eq!(report.version, 1);
        assert_eq!(report.node_id, hex::encode([2u8; 33]));
        assert_eq!(report.total_channels, 3);
        assert_eq!(report.complete_channels, 2);
        assert_eq!(report.incomplete_channels, 1);
        assert_eq!(report.channels[0].funding_outpoint.txid, "00");
        assert_eq!(
            report.channels[0].peer_addr.as_deref(),
            Some("127.0.0.1:9735")
        );
        let serialized = serde_json::to_value(&report).unwrap();
        assert_eq!(serialized["total_channels"], 3);
        assert!(serialized["channels"][0]["remote_basepoints"].is_object());
        assert!(serialized.get("state").is_none());
        assert!(serialized.get("peerlist").is_none());
        let incomplete = report
            .channels
            .iter()
            .find(|channel| channel.peer_id == peer_b)
            .unwrap();
        assert!(!incomplete.complete);
        assert_eq!(incomplete.peer_addr, None);
        assert_eq!(incomplete.warnings, vec!["missing_peer_addr".to_string()]);
    }

    #[test]
    fn inspect_backup_report_accepts_periodic_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        write_json(
            &path,
            backup_json(
                json!({}),
                json!([]),
                json!({ "periodic": { "updates": 5 } }),
            ),
        );

        let report = inspect_backup_report(&path).unwrap();

        assert_eq!(report.total_channels, 0);
        assert_eq!(
            serde_json::to_value(report.strategy).unwrap()["periodic"]["updates"],
            5
        );
    }

    #[test]
    fn inspect_backup_report_rejects_unsupported_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let mut backup = backup_json(json!({}), json!([]), json!("new_channels_only"));
        backup["version"] = json!(2);
        write_json(&path, backup);

        let err = inspect_backup_report(&path).unwrap_err().to_string();

        assert!(err.contains("unsupported signer backup version 2"));
    }

    #[test]
    fn inspect_backup_report_rejects_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        std::fs::write(&path, "not-json").unwrap();

        let err = inspect_backup_report(&path).unwrap_err().to_string();

        assert!(err.contains("parsing signer backup"));
    }

    #[test]
    fn inspect_backup_report_rejects_malformed_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        write_json(
            &path,
            backup_json(
                json!({
                    "channels/not-state-entry": "not-a-state-entry"
                }),
                json!([]),
                json!("new_channels_only"),
            ),
        );

        let err = inspect_backup_report(&path).unwrap_err().to_string();

        assert!(err.contains("parsing signer backup"));
    }

    #[test]
    fn backup_report_text_includes_recovery_fields_and_warnings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        let peer = peer_id(0xaa);
        write_json(
            &path,
            backup_json(
                json!({
                    channel_key(&peer, 1): [1, channel(recovery_setup("00", 0, 1000, true))]
                }),
                json!([]),
                json!("new_channels_only"),
            ),
        );
        let report = inspect_backup_report(&path).unwrap();

        let text = format_backup_report_text(&report);

        assert!(text.contains("version: 1"));
        assert!(text.contains("channels: total=1 complete=0 incomplete=1"));
        assert!(text.contains(&format!("peer_id: {peer}")));
        assert!(text.contains("peer_addr: missing"));
        assert!(text.contains("funding: 00:0"));
        assert!(text.contains("funding_sats: 1000"));
        assert!(text.contains("warnings: missing_peer_addr"));
    }

    fn backup_json(
        channels: serde_json::Value,
        peerlist: serde_json::Value,
        strategy: serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "version": 1,
            "created_at": "2026-04-29T00:00:00Z",
            "node_id": hex::encode([2u8; 33]),
            "strategy": strategy,
            "state": {
                "values": channels
            },
            "peerlist": peerlist
        })
    }

    fn peerlist_entry(peer_id: &str, addr: &str) -> serde_json::Value {
        json!({
            "peer_id": peer_id,
            "addr": addr,
            "direction": "out",
            "features": "",
            "generation": 7,
            "raw_datastore_string": format!(
                r#"{{"id":"{peer_id}","direction":"out","addr":"{addr}","features":""}}"#
            )
        })
    }

    fn peer_id(byte: u8) -> String {
        let mut bytes = vec![byte; 33];
        bytes[0] = 2;
        hex::encode(bytes)
    }

    fn channel_key(peer_id: &str, oid: u64) -> String {
        let mut raw = vec![3u8; 33];
        raw.extend(hex::decode(peer_id).unwrap());
        raw.extend(oid.to_le_bytes());
        format!("channels/{}", hex::encode(raw))
    }

    fn channel(channel_setup: serde_json::Value) -> serde_json::Value {
        json!({
            "channel_setup": channel_setup,
            "id": {
                "id": "00"
            }
        })
    }

    fn recovery_setup(txid: &str, vout: u32, sats: u64, is_outbound: bool) -> serde_json::Value {
        json!({
            "channel_value_sat": sats,
            "commitment_type": "AnchorsZeroFeeHtlc",
            "counterparty_points": {
                "delayed_payment_basepoint": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "funding_pubkey": "02bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "htlc_basepoint": "02cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                "payment_point": "02dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                "revocation_basepoint": "02eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            },
            "counterparty_selected_contest_delay": 144,
            "counterparty_shutdown_script": null,
            "funding_outpoint": {
                "txid": txid,
                "vout": vout
            },
            "holder_selected_contest_delay": 144,
            "holder_shutdown_script": null,
            "is_outbound": is_outbound,
            "push_value_msat": 0
        })
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
}

async fn version<P: AsRef<Path>>(config: Config<P>) -> Result<()> {
    // Check if we can find a seed file, if we can not find one, we need to register first.
    let seed_path = config.data_dir.as_ref().join(SEED_FILE_NAME);
    let seed = util::read_seed(&seed_path);
    if seed.is_none() {
        println!("Seed not found");
        return Err(Error::SeedNotFoundError(format!(
            "could not read from {}",
            seed_path.display()
        )));
    }

    let seed = seed.unwrap(); // we checked if it is none before.

    // Initialize a signer and scheduler with default credentials.
    let creds = gl_client::credentials::Nobody::new();
    let signer = gl_client::signer::Signer::new(seed, config.network, creds.clone())
        .map_err(|e| Error::custom(format!("Failed to create signer: {}", e)))?;
    println!("{}", signer.version());
    Ok(())
}
