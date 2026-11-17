use super::{DeviceStorage, Error};
use crate::persist::State;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Entry {
    version: u64,
    value_hash: [u8; 32],
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Watermark {
    format_version: u32,
    entries: BTreeMap<String, Entry>,
}

impl Watermark {
    fn from_state(state: &State) -> Result<Self> {
        let entries = state
            .entry_fingerprints()
            .map(|entry| {
                let (key, version, value_hash) = entry?;
                Ok((
                    key.to_owned(),
                    Entry {
                        version,
                        value_hash,
                    },
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            format_version: FORMAT_VERSION,
            entries,
        })
    }

    fn check(&self, candidate: &Self) -> Result<()> {
        for (key, previous) in &self.entries {
            let entry = candidate
                .entries
                .get(key)
                .with_context(|| format!("watermark: missing state entry {key}"))?;
            if entry.version < previous.version {
                bail!(
                    "watermark: state entry {key} fell from version {} to {}",
                    previous.version,
                    entry.version
                );
            }
            if entry.version == previous.version && entry.value_hash != previous.value_hash {
                bail!(
                    "watermark: state entry {key} differs at version {}",
                    entry.version
                );
            }
        }
        Ok(())
    }
}

pub(super) struct WatermarkRuntime {
    store: Arc<dyn DeviceStorage>,
    key: String,
    committed: Option<Watermark>,
    initialized: bool,
    blocked: bool,
}

impl WatermarkRuntime {
    pub(super) fn load(store: Arc<dyn DeviceStorage>, key: String) -> Result<Self> {
        let committed = store
            .load(&key)
            .context("loading watermark")?
            .map(|bytes| -> Result<Watermark> {
                let watermark: Watermark =
                    serde_json::from_slice(&bytes).context("decoding watermark")?;
                if watermark.format_version != FORMAT_VERSION || watermark.entries.is_empty() {
                    bail!("invalid watermark format version or empty entries");
                }
                Ok(watermark)
            })
            .transpose()?;
        Ok(Self {
            store,
            key,
            committed,
            initialized: false,
            blocked: false,
        })
    }

    pub(super) fn has_checkpoint(&self) -> bool {
        self.committed.is_some()
    }

    /// Check the first full snapshot before the signer merges its seed defaults.
    pub(super) fn admit(&self, incoming: &State) -> Result<bool, Error> {
        if self.blocked {
            return Err(Error::WatermarkStorage(anyhow::anyhow!(
                "watermark storage failed; reconstruct the signer to reload it"
            )));
        }
        if self.initialized {
            return Ok(false);
        }
        if let Some(committed) = &self.committed {
            let snapshot = Watermark::from_state(incoming).map_err(Error::Other)?;
            committed
                .check(&snapshot)
                .map_err(Error::WatermarkViolation)?;
        }
        Ok(true)
    }

    /// Save the signed candidate before the signer commits it or releases signatures.
    pub(super) fn advance(&mut self, state: &State) -> Result<(), Error> {
        let watermark = Watermark::from_state(state).map_err(Error::Other)?;
        if let Some(committed) = &self.committed {
            committed
                .check(&watermark)
                .map_err(Error::WatermarkViolation)?;
        }
        if !self.initialized || self.committed.as_ref() != Some(&watermark) {
            let bytes = serde_json::to_vec(&watermark)
                .context("encoding watermark")
                .map_err(Error::WatermarkStorage)?;
            if let Err(error) = self.store.save(&self.key, &bytes) {
                // The write can fail after replacing the old bytes.
                self.blocked = true;
                return Err(Error::WatermarkStorage(error.context("saving watermark")));
            }
            self.committed = Some(watermark);
        }
        self.initialized = true;
        Ok(())
    }
}
