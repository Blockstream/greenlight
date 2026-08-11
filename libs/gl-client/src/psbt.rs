use crate::bitcoin::consensus::encode::VarInt;
use crate::bitcoin::consensus::Decodable;
use crate::bitcoin::{OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness};
use base64::{engine::general_purpose, Engine as _};
use std::io::Cursor;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateFunding {
    pub outpoint: OutPoint,
    pub txout: TxOut,
    pub sign_splice_tx_input_index: u32,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid base64 PSBT: {0}")]
    InvalidBase64(#[source] base64::DecodeError),
    #[error("invalid PSBT (v0: {v0}; v2: {v2})")]
    InvalidPsbt { v0: String, v2: String },
    #[error("invalid PSBT framing: {0}")]
    InvalidFraming(String),
    #[error("PSBT contains trailing data")]
    TrailingData,
    #[error("PSBT inputs require incompatible lock times")]
    IncompatibleLockTime,
    #[error("PSBT input {input_index} has no UTXO")]
    MissingUtxo { input_index: usize },
    #[error(
        "PSBT input {input_index} non-witness UTXO txid mismatch: expected {expected}, got {actual}"
    )]
    NonWitnessUtxoTxidMismatch {
        input_index: usize,
        expected: Txid,
        actual: Txid,
    },
    #[error(
        "PSBT input {input_index} references output {vout}, but its non-witness UTXO has {output_count} outputs"
    )]
    UtxoOutputMissing {
        input_index: usize,
        vout: u32,
        output_count: usize,
    },
    #[error("PSBT input {input_index} has inconsistent witness and non-witness UTXOs")]
    InconsistentUtxo { input_index: usize },
    #[error("splice PSBT does not spend old funding outpoint {0}")]
    FundingOutpointNotSpent(OutPoint),
    #[error("splice PSBT spends old funding outpoint {0} more than once")]
    FundingOutpointSpentMultipleTimes(OutPoint),
    #[error("splice funding output index {vout} is missing")]
    FundingOutputMissing { vout: u32 },
    #[error("splice transaction has too many inputs to represent the funding input index")]
    FundingInputIndexOverflow,
}

#[derive(Clone, Debug)]
enum ParsedPsbtInner {
    V0(psbt_v2::v0::Psbt),
    V2(psbt_v2::v2::Psbt),
}

#[derive(Clone, Debug)]
pub struct ParsedPsbt {
    inner: ParsedPsbtInner,
    unsigned_tx: Transaction,
}

impl ParsedPsbt {
    pub fn from_base64(encoded: &str) -> Result<Self, Error> {
        let bytes = general_purpose::STANDARD
            .decode(encoded)
            .map_err(Error::InvalidBase64)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let v0_error = match psbt_v2::v0::Psbt::deserialize(bytes) {
            Ok(psbt) => {
                ensure_complete_psbt(bytes, psbt.inputs.len(), psbt.outputs.len())?;
                let unsigned_tx = psbt.unsigned_tx.clone();
                return Ok(Self {
                    inner: ParsedPsbtInner::V0(psbt),
                    unsigned_tx,
                });
            }
            Err(error) => format!("{error:?}"),
        };

        match psbt_v2::v2::Psbt::deserialize(bytes) {
            Ok(psbt) => {
                ensure_complete_psbt(bytes, psbt.global.input_count, psbt.global.output_count)?;
                let unsigned_tx = v2_unsigned_tx(&psbt)?;
                Ok(Self {
                    inner: ParsedPsbtInner::V2(psbt),
                    unsigned_tx,
                })
            }
            Err(error) => Err(Error::InvalidPsbt {
                v0: v0_error,
                v2: format!("{error:?}"),
            }),
        }
    }

    pub fn unsigned_tx(&self) -> &Transaction {
        &self.unsigned_tx
    }

    /// Return the version-independent unsigned transaction fingerprint.
    pub fn fingerprint(&self) -> Txid {
        self.unsigned_tx.compute_txid()
    }

    pub fn input_outpoints(&self) -> Vec<OutPoint> {
        self.unsigned_tx
            .input
            .iter()
            .map(|input| input.previous_output)
            .collect()
    }

    pub fn funding_utxos(&self) -> Result<Vec<TxOut>, Error> {
        match &self.inner {
            ParsedPsbtInner::V0(psbt) => self
                .unsigned_tx
                .input
                .iter()
                .zip(&psbt.inputs)
                .enumerate()
                .map(|(input_index, (txin, input))| {
                    validated_funding_utxo(
                        input_index,
                        txin.previous_output,
                        input.witness_utxo.as_ref(),
                        input.non_witness_utxo.as_ref(),
                    )
                })
                .collect(),
            ParsedPsbtInner::V2(psbt) => psbt
                .inputs
                .iter()
                .enumerate()
                .map(|(input_index, input)| {
                    validated_funding_utxo(
                        input_index,
                        OutPoint::new(input.previous_txid, input.spent_output_index),
                        input.witness_utxo.as_ref(),
                        input.non_witness_utxo.as_ref(),
                    )
                })
                .collect(),
        }
    }

    pub fn candidate_funding(
        &self,
        funding_txid: Txid,
        funding_vout: u32,
        old_funding_outpoint: OutPoint,
    ) -> Result<CandidateFunding, Error> {
        let mut matching_inputs = self
            .unsigned_tx
            .input
            .iter()
            .enumerate()
            .filter(|(_, input)| input.previous_output == old_funding_outpoint);
        let (input_index, _) = matching_inputs
            .next()
            .ok_or(Error::FundingOutpointNotSpent(old_funding_outpoint))?;
        if matching_inputs.next().is_some() {
            return Err(Error::FundingOutpointSpentMultipleTimes(
                old_funding_outpoint,
            ));
        }

        let sign_splice_tx_input_index =
            u32::try_from(input_index).map_err(|_| Error::FundingInputIndexOverflow)?;
        let txout = self
            .unsigned_tx
            .output
            .get(funding_vout as usize)
            .cloned()
            .ok_or(Error::FundingOutputMissing { vout: funding_vout })?;

        Ok(CandidateFunding {
            outpoint: OutPoint::new(funding_txid, funding_vout),
            txout,
            sign_splice_tx_input_index,
        })
    }
}

fn v2_unsigned_tx(psbt: &psbt_v2::v2::Psbt) -> Result<Transaction, Error> {
    let lock_time = psbt
        .determine_lock_time()
        .map_err(|_| Error::IncompatibleLockTime)?;
    let input = psbt
        .inputs
        .iter()
        .map(|input| TxIn {
            previous_output: OutPoint::new(input.previous_txid, input.spent_output_index),
            script_sig: ScriptBuf::new(),
            sequence: input.sequence.unwrap_or(Sequence::MAX),
            witness: Witness::new(),
        })
        .collect();
    let output = psbt
        .outputs
        .iter()
        .map(|output| TxOut {
            value: output.amount,
            script_pubkey: output.script_pubkey.clone(),
        })
        .collect();

    Ok(Transaction {
        version: psbt.global.tx_version,
        lock_time,
        input,
        output,
    })
}

fn validated_funding_utxo(
    input_index: usize,
    previous_output: OutPoint,
    witness_utxo: Option<&TxOut>,
    non_witness_utxo: Option<&Transaction>,
) -> Result<TxOut, Error> {
    let non_witness_output = non_witness_utxo
        .map(|transaction| {
            let actual = transaction.compute_txid();
            if actual != previous_output.txid {
                return Err(Error::NonWitnessUtxoTxidMismatch {
                    input_index,
                    expected: previous_output.txid,
                    actual,
                });
            }
            transaction
                .output
                .get(previous_output.vout as usize)
                .ok_or(Error::UtxoOutputMissing {
                    input_index,
                    vout: previous_output.vout,
                    output_count: transaction.output.len(),
                })
        })
        .transpose()?;

    match (witness_utxo, non_witness_output) {
        (Some(witness), Some(non_witness)) if witness != non_witness => {
            Err(Error::InconsistentUtxo { input_index })
        }
        (Some(witness), _) => Ok(witness.clone()),
        (None, Some(non_witness)) => Ok(non_witness.clone()),
        (None, None) => Err(Error::MissingUtxo { input_index }),
    }
}

fn ensure_complete_psbt(
    bytes: &[u8],
    input_count: usize,
    output_count: usize,
) -> Result<(), Error> {
    if !bytes.starts_with(b"psbt\xff") {
        return Err(Error::InvalidFraming("invalid magic bytes".to_string()));
    }

    // psbt-v2 0.3.0 accepts trailing bytes. This cursor only validates map
    // framing and leaves all PSBT key/value parsing to the crate.
    let mut cursor = Cursor::new(bytes);
    cursor.set_position(5);
    consume_map(&mut cursor)?;
    for _ in 0..input_count {
        consume_map(&mut cursor)?;
    }
    for _ in 0..output_count {
        consume_map(&mut cursor)?;
    }

    if cursor.position() != bytes.len() as u64 {
        return Err(Error::TrailingData);
    }
    Ok(())
}

fn consume_map(cursor: &mut Cursor<&[u8]>) -> Result<(), Error> {
    loop {
        let key_length = read_compact_size(cursor)?;
        if key_length == 0 {
            return Ok(());
        }
        skip_bytes(cursor, key_length)?;
        let value_length = read_compact_size(cursor)?;
        skip_bytes(cursor, value_length)?;
    }
}

fn read_compact_size(cursor: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    VarInt::consensus_decode(cursor)
        .map(|value| value.0)
        .map_err(|error| Error::InvalidFraming(error.to_string()))
}

fn skip_bytes(cursor: &mut Cursor<&[u8]>, count: u64) -> Result<(), Error> {
    let end = cursor
        .position()
        .checked_add(count)
        .ok_or_else(|| Error::InvalidFraming("map length overflow".to_string()))?;
    if end > cursor.get_ref().len() as u64 {
        return Err(Error::InvalidFraming(
            "map entry extends past end of PSBT".to_string(),
        ));
    }
    cursor.set_position(end);
    Ok(())
}

#[cfg(test)]
pub(crate) const CLN_PSBT_V2: &str = "cHNidP8BAgQCAAAAAQQBAQEFAQIB+wQCAAAAAAEOIAsK2SFBnByHGXNdctxzn56p4GONH+TB7vD5lECEgV/IAQ8EAAAAAAABAwgACK8vAAAAAAEEFgAUxDD2TEdW2jENvRoIVXLvKZkmJywAAQMIi73rCwAAAAABBBYAFE3Rk6yWSlasG54cyoRU/i9HT4UTAA==";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitcoin::absolute::LockTime;
    use crate::bitcoin::transaction::Version;
    use crate::bitcoin::{Amount, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};
    use base64::engine::general_purpose;
    use std::str::FromStr;

    fn unsigned_tx(previous_output: OutPoint) -> Transaction {
        Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output,
                script_sig: ScriptBuf::new(),
                sequence: Sequence::MAX,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(75_000),
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }

    fn v0_psbt(previous_output: OutPoint) -> psbt_v2::v0::Psbt {
        psbt_v2::v0::Psbt::from_unsigned_tx(unsigned_tx(previous_output)).unwrap()
    }

    #[test]
    fn parses_cln_v2_with_bip370_sequence_default() {
        let psbt = ParsedPsbt::from_base64(CLN_PSBT_V2).unwrap();

        assert_eq!(psbt.unsigned_tx().input[0].sequence, Sequence::MAX);
        assert_eq!(psbt.input_outpoints().len(), 1);
        assert_eq!(psbt.fingerprint(), psbt.unsigned_tx().compute_txid());
    }

    #[test]
    fn equivalent_v0_and_v2_psbts_have_the_same_fingerprint() {
        let v2 = ParsedPsbt::from_base64(CLN_PSBT_V2).unwrap();
        let v0 = psbt_v2::v0::Psbt::from_unsigned_tx(v2.unsigned_tx().clone()).unwrap();
        let v0 = ParsedPsbt::from_bytes(&v0.serialize()).unwrap();

        assert_eq!(v0.fingerprint(), v2.fingerprint());
    }

    #[test]
    fn rejects_trailing_data_for_both_versions() {
        let previous_output = OutPoint::new(Txid::from_str(&"11".repeat(32)).unwrap(), 0);
        let mut v0 = v0_psbt(previous_output).serialize();
        v0.push(0);
        let mut v2 = general_purpose::STANDARD.decode(CLN_PSBT_V2).unwrap();
        v2.push(0);

        assert!(matches!(
            ParsedPsbt::from_bytes(&v0),
            Err(Error::TrailingData)
        ));
        assert!(matches!(
            ParsedPsbt::from_bytes(&v2),
            Err(Error::TrailingData)
        ));
    }

    #[test]
    fn rejects_noncanonical_map_framing() {
        let raw = general_purpose::STANDARD.decode(CLN_PSBT_V2).unwrap();
        assert_eq!(raw[5], 1);
        let mut non_minimal = Vec::with_capacity(raw.len() + 2);
        non_minimal.extend_from_slice(&raw[..5]);
        non_minimal.extend_from_slice(&[0xfd, 0x01, 0x00]);
        non_minimal.extend_from_slice(&raw[6..]);

        assert!(ParsedPsbt::from_bytes(&non_minimal).is_err());
    }

    #[test]
    fn rejects_incompatible_v2_locktimes() {
        let raw = general_purpose::STANDARD.decode(CLN_PSBT_V2).unwrap();
        let mut v2 = psbt_v2::v2::Psbt::deserialize(&raw).unwrap();
        v2.inputs[0].min_height =
            Some(crate::bitcoin::absolute::Height::from_consensus(1).unwrap());
        let mut time_locked_input = v2.inputs[0].clone();
        time_locked_input.min_height = None;
        time_locked_input.min_time =
            Some(crate::bitcoin::absolute::Time::from_consensus(500_000_000).unwrap());
        v2.inputs.push(time_locked_input);
        v2.global.input_count += 1;

        assert!(matches!(
            ParsedPsbt::from_bytes(&v2.serialize()),
            Err(Error::IncompatibleLockTime)
        ));
    }

    #[test]
    fn validates_and_returns_funding_utxos() {
        let previous_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![TxOut {
                value: Amount::from_sat(80_000),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        let previous_output = OutPoint::new(previous_tx.compute_txid(), 0);
        let mut v0 = v0_psbt(previous_output);
        v0.inputs[0].non_witness_utxo = Some(previous_tx.clone());
        v0.inputs[0].witness_utxo = Some(previous_tx.output[0].clone());

        let parsed = ParsedPsbt::from_bytes(&v0.serialize()).unwrap();

        assert_eq!(parsed.funding_utxos().unwrap(), previous_tx.output);
    }

    #[test]
    fn rejects_missing_funding_utxo() {
        let previous_output = OutPoint::new(Txid::from_str(&"11".repeat(32)).unwrap(), 0);
        let parsed = ParsedPsbt::from_bytes(&v0_psbt(previous_output).serialize()).unwrap();

        assert!(matches!(
            parsed.funding_utxos(),
            Err(Error::MissingUtxo { input_index: 0 })
        ));
    }

    #[test]
    fn rejects_missing_non_witness_output() {
        let previous_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![TxOut {
                value: Amount::from_sat(80_000),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        let mut v0 = v0_psbt(OutPoint::new(previous_tx.compute_txid(), 1));
        v0.inputs[0].non_witness_utxo = Some(previous_tx);
        let parsed = ParsedPsbt::from_bytes(&v0.serialize()).unwrap();

        assert!(matches!(
            parsed.funding_utxos(),
            Err(Error::UtxoOutputMissing {
                input_index: 0,
                vout: 1,
                output_count: 1,
            })
        ));
    }

    #[test]
    fn rejects_mismatched_non_witness_utxo_even_with_witness_utxo() {
        let previous_output = OutPoint::new(Txid::from_str(&"11".repeat(32)).unwrap(), 0);
        let mut v0 = v0_psbt(previous_output);
        let wrong_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![TxOut {
                value: Amount::from_sat(80_000),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        v0.inputs[0].non_witness_utxo = Some(wrong_tx.clone());
        v0.inputs[0].witness_utxo = Some(wrong_tx.output[0].clone());

        let parsed = ParsedPsbt::from_bytes(&v0.serialize()).unwrap();

        assert!(matches!(
            parsed.funding_utxos(),
            Err(Error::NonWitnessUtxoTxidMismatch { input_index: 0, .. })
        ));
    }

    #[test]
    fn rejects_inconsistent_witness_and_non_witness_utxos() {
        let previous_tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![TxOut {
                value: Amount::from_sat(80_000),
                script_pubkey: ScriptBuf::new(),
            }],
        };
        let mut v0 = v0_psbt(OutPoint::new(previous_tx.compute_txid(), 0));
        v0.inputs[0].non_witness_utxo = Some(previous_tx);
        v0.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(79_999),
            script_pubkey: ScriptBuf::new(),
        });

        let parsed = ParsedPsbt::from_bytes(&v0.serialize()).unwrap();

        assert!(matches!(
            parsed.funding_utxos(),
            Err(Error::InconsistentUtxo { input_index: 0 })
        ));
    }
}
