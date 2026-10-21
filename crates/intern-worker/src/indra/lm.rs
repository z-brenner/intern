//! The recognizer's character n-gram language model: Katz back-off over
//! packed keys, read from the reference package's `.npz` (spec §7.2, §7.3).
//!
//! A token is a recognizer class id `c` stored as `c + 1`; 0 is the start
//! of the line. A key packs tokens oldest first, `bits` bits each, so a key
//! of a history of `n` tokens and then a token is `n + 1` tokens wide. For
//! every history length `n` from 0 to `order - 1` the model holds the sorted
//! keys of the (history, token) pairs it saw with their natural-log
//! probabilities, and the sorted keys of the histories it saw with their
//! back-off weights (`lm.py:1-14`).
//!
//! The reference merges each kind of table over all lengths into one sorted
//! array with the length in the top bits (`lm.py:51-58`) so that numpy can
//! search every length at once. Every table in the pinned models is strictly
//! ascending and no key reaches the tag bits, so searching each length's
//! table on its own finds exactly the same entries; [`NgramLm::from_tables`]
//! refuses tables for which that would not hold.

use std::fmt;
use std::path::Path;

use super::npy::{self, NpyArray, NpyData, NpyError};

/// Bits a token takes in a key when the file does not say (`lm.py:23`).
pub const BITS: u32 = 11;
/// The reference keeps a key's history length in the bits from here up
/// (`lm.py:24`), so a key of `order` tokens must fit under it.
pub const TAG_SHIFT: u32 = 61;
/// The three tag bits hold history lengths up to 7, so the reference's
/// merged tables tell lengths apart only up to order 8.
pub const MAX_ORDER: usize = 8;

/// One history length's tables: the keys of the (history, token) pairs the
/// model saw with their log-probabilities, and the keys of the histories it
/// saw with their back-off weights. Keys are strictly ascending.
#[derive(Clone, Default, PartialEq)]
pub struct LmTable {
    pub keys: Vec<u64>,
    pub logp: Vec<f32>,
    pub histories: Vec<u64>,
    pub backoff: Vec<f32>,
}

/// A back-off n-gram model over recognizer class ids.
#[derive(Clone)]
pub struct NgramLm {
    pub order: usize,
    pub vocab_size: usize,
    pub bits: u32,
    /// The largest log-probability a token can be given; for pruning.
    pub max_logp: f64,
    /// `tables[n]` holds the pairs and histories of history length `n`.
    tables: Vec<LmTable>,
    /// `ln(vocab_size)`, the uniform floor of a token no history saw.
    ln_vocab: f64,
}

impl fmt::Debug for NgramLm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NgramLm")
            .field("order", &self.order)
            .field("vocab_size", &self.vocab_size)
            .field("bits", &self.bits)
            .field("max_logp", &self.max_logp)
            .field(
                "pairs",
                &self.tables.iter().map(|t| t.keys.len()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn invalid(message: impl Into<String>) -> NpyError {
    NpyError::Header(format!("language model: {}", message.into()))
}

/// Takes an integer scalar out of the archive.
fn scalar(arrays: &std::collections::HashMap<String, NpyArray>, name: &str) -> Option<i64> {
    arrays.get(name).and_then(NpyArray::scalar_i64)
}

/// Moves a `u64` array out of the archive without copying it.
fn take_u64(
    arrays: &mut std::collections::HashMap<String, NpyArray>,
    name: &str,
) -> Result<Vec<u64>, NpyError> {
    match arrays.remove(name) {
        Some(NpyArray {
            data: NpyData::U64(values),
            ..
        }) => Ok(values),
        Some(_) => Err(invalid(format!("{name} is not uint64"))),
        None => Err(invalid(format!("{name} is missing"))),
    }
}

/// Moves an `f32` array out of the archive without copying it.
fn take_f32(
    arrays: &mut std::collections::HashMap<String, NpyArray>,
    name: &str,
) -> Result<Vec<f32>, NpyError> {
    match arrays.remove(name) {
        Some(NpyArray {
            data: NpyData::F32(values),
            ..
        }) => Ok(values),
        Some(_) => Err(invalid(format!("{name} is not float32"))),
        None => Err(invalid(format!("{name} is missing"))),
    }
}

fn strictly_ascending(keys: &[u64]) -> bool {
    keys.windows(2).all(|pair| pair[0] < pair[1])
}

/// The index of `key` in a strictly ascending table.
fn find(keys: &[u64], key: u64) -> Option<usize> {
    keys.binary_search(&key).ok()
}

impl NgramLm {
    /// Reads `lm.npz` / `lm_<group>.npz` (`NgramLM.__init__`, `lm.py:37-61`):
    /// `order`, `vocab_size`, optional `bits` (11 when absent), and
    /// `k{n}`/`v{n}`/`hk{n}`/`hb{n}` for every history length.
    pub fn load(path: &Path) -> Result<Self, NpyError> {
        let mut arrays = npy::read_npz(path)?;
        let order = scalar(&arrays, "order").ok_or_else(|| invalid("order is missing"))?;
        let vocab_size =
            scalar(&arrays, "vocab_size").ok_or_else(|| invalid("vocab_size is missing"))?;
        let bits = match arrays.get("bits") {
            Some(array) => array
                .scalar_i64()
                .ok_or_else(|| invalid("bits is not an integer"))?,
            None => i64::from(BITS),
        };
        let order = usize::try_from(order).map_err(|_| invalid("negative order"))?;
        let vocab_size = usize::try_from(vocab_size).map_err(|_| invalid("negative vocab_size"))?;
        let bits = u32::try_from(bits).map_err(|_| invalid("bad bits"))?;
        // Checked here too, before the table names are formed from it.
        if order == 0 || order as u64 * u64::from(bits) > u64::from(TAG_SHIFT) {
            return Err(invalid(format!(
                "order {order} x {bits} bits do not fit a key"
            )));
        }
        let mut tables = Vec::with_capacity(order);
        for n in 0..order {
            tables.push(LmTable {
                keys: take_u64(&mut arrays, &format!("k{n}"))?,
                logp: take_f32(&mut arrays, &format!("v{n}"))?,
                histories: take_u64(&mut arrays, &format!("hk{n}"))?,
                backoff: take_f32(&mut arrays, &format!("hb{n}"))?,
            });
        }
        Self::from_tables(order, vocab_size, bits, tables)
    }

    /// A model from its tables, `tables[n]` for history length `n`. Refuses
    /// what the reference would refuse (`order * bits > 61`, `lm.py:41-42`)
    /// and tables whose per-length search could differ from the reference's
    /// merged one: keys not strictly ascending, or reaching the tag bits.
    pub fn from_tables(
        order: usize,
        vocab_size: usize,
        bits: u32,
        tables: Vec<LmTable>,
    ) -> Result<Self, NpyError> {
        if order == 0 || bits == 0 || order as u64 * u64::from(bits) > u64::from(TAG_SHIFT) {
            return Err(invalid(format!(
                "order {order} x {bits} bits do not fit a key"
            )));
        }
        if order > MAX_ORDER {
            return Err(invalid(format!("order {order} is over {MAX_ORDER}")));
        }
        if vocab_size == 0 {
            return Err(invalid("vocab_size is 0"));
        }
        if tables.len() != order {
            return Err(invalid(format!(
                "{} history lengths for order {order}",
                tables.len()
            )));
        }
        for (n, table) in tables.iter().enumerate() {
            if table.keys.len() != table.logp.len() || table.histories.len() != table.backoff.len()
            {
                return Err(invalid(format!(
                    "k{n}/v{n} or hk{n}/hb{n} differ in length"
                )));
            }
            if !strictly_ascending(&table.keys) || !strictly_ascending(&table.histories) {
                return Err(invalid(format!("k{n} or hk{n} is not strictly ascending")));
            }
            let top = table
                .keys
                .last()
                .copied()
                .max(table.histories.last().copied())
                .unwrap_or(0);
            if top >> TAG_SHIFT != 0 {
                return Err(invalid(format!("k{n} or hk{n} reaches the tag bits")));
            }
        }
        // `max_logp` (`lm.py:64-72`): the largest log-probability of any
        // length, or the uniform floor if larger, plus every positive largest
        // back-off weight, summed over lengths in order as Python's `sum`.
        let ln_vocab = (vocab_size as f64).ln();
        let largest = tables
            .iter()
            .flat_map(|t| t.logp.iter())
            .map(|&v| f64::from(v))
            .fold(f64::NEG_INFINITY, f64::max);
        let top = if -ln_vocab > largest {
            -ln_vocab
        } else {
            largest
        };
        let mut backoff = 0.0f64;
        for table in &tables {
            if let Some(max) = table.backoff.iter().map(|&v| f64::from(v)).reduce(f64::max) {
                backoff += if max > 0.0 { max } else { 0.0 };
            }
        }
        Ok(Self {
            order,
            vocab_size,
            bits,
            max_logp: top + backoff,
            tables,
            ln_vocab,
        })
    }

    /// `logp_keys` for one key `(history << bits) | (class + 1)` (spec §7.3,
    /// `lm.py:104-151`): Katz back-off from the longest history the model
    /// has seen. The back-off weights are summed from the longest length
    /// down, as the reference's reversed `cumsum` adds them, so the result
    /// is the reference's to the last bit.
    pub fn logp(&self, key: u64) -> f64 {
        let b = self.bits;
        let token = key & ((1u64 << b) - 1);
        let history = key >> b;
        let mask = |n: usize| (1u64 << (b as usize * n)) - 1;
        // Back-off weights of the seen histories: length 0 always counts
        // (`h_ok[:, 0] = True`), its weight 0 if the table lacks it; then
        // lengths 1, 2, ... while each is seen (`m`).
        let mut backoff = [0.0f64; MAX_ORDER];
        backoff[0] = find(&self.tables[0].histories, 0)
            .map_or(0.0, |i| f64::from(self.tables[0].backoff[i]));
        let mut longest = 0;
        for (n, (table, weight)) in self.tables.iter().zip(&mut backoff).enumerate().skip(1) {
            match find(&table.histories, history & mask(n)) {
                Some(i) => {
                    *weight = f64::from(table.backoff[i]);
                    longest = n;
                }
                None => break,
            }
        }
        // The longest seen history that saw the token, plus the back-offs
        // of the longer seen histories.
        let mut sum = 0.0f64;
        for n in (0..=longest).rev() {
            let table = &self.tables[n];
            let pair = ((history & mask(n)) << b) | token;
            if let Some(i) = find(&table.keys, pair) {
                return sum + f64::from(table.logp[i]);
            }
            sum += backoff[n];
        }
        sum - self.ln_vocab
    }

    /// The mask of a rolling history of `order - 1` tokens
    /// (`hmask`, `lm.py:180`).
    pub fn history_mask(&self) -> u64 {
        (1u64 << (self.bits as usize * (self.order - 1))) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model over classes 1..=3 (tokens 2..=4), order 3, 4 bits a token.
    pub(crate) fn tiny() -> NgramLm {
        let k = |tokens: &[u64]| tokens.iter().fold(0u64, |key, &t| (key << 4) | t);
        let mut tables = vec![LmTable::default(), LmTable::default(), LmTable::default()];
        // Unigrams of tokens 2 and 3; history () weight -0.5.
        tables[0].keys = vec![k(&[2]), k(&[3])];
        tables[0].logp = vec![-1.0, -2.0];
        tables[0].histories = vec![0];
        tables[0].backoff = vec![-0.5];
        // History (2,) seen with weight -0.25; pair (2, 3).
        tables[1].keys = vec![k(&[2, 3])];
        tables[1].logp = vec![-0.125];
        tables[1].histories = vec![k(&[2]), k(&[3])];
        tables[1].backoff = vec![-0.25, -0.75];
        // History (3, 2) seen; pair (3, 2, 4).
        tables[2].keys = vec![k(&[3, 2, 4])];
        tables[2].logp = vec![-0.0625];
        tables[2].histories = vec![k(&[3, 2])];
        tables[2].backoff = vec![-1.5];
        NgramLm::from_tables(3, 5, 4, tables).unwrap()
    }

    #[test]
    fn backs_off_as_katz() {
        let lm = tiny();
        let key = |h: &[u64], t: u64| h.iter().fold(0u64, |key, &x| (key << 4) | x) << 4 | t;
        // Seen trigram: its own value.
        assert_eq!(lm.logp(key(&[3, 2], 4)), -0.0625);
        // History (3, 2) seen but not with token 3: bigram (2, 3) plus
        // the back-off of (3, 2).
        assert_eq!(lm.logp(key(&[3, 2], 3)), -1.5 + -0.125);
        // History (1, 2): (2,) is seen, (1, 2) is not, so m = 1.
        assert_eq!(lm.logp(key(&[1, 2], 3)), -0.125);
        assert_eq!(lm.logp(key(&[1, 2], 2)), -0.25 + -1.0);
        // Unseen everywhere: every seen back-off and the uniform floor.
        assert_eq!(lm.logp(key(&[3, 2], 1)), (-1.5 + -0.25) + -0.5 - 5f64.ln());
        // A history of line starts: only length 0 is seen.
        assert_eq!(lm.logp(key(&[0, 0], 3)), -2.0);
        assert_eq!(lm.logp(key(&[0, 0], 4)), -0.5 - 5f64.ln());
        assert_eq!(lm.history_mask(), 0xff);
        // The best value is -0.0625 and no weight is positive.
        assert_eq!(lm.max_logp, -0.0625);
    }

    #[test]
    fn refuses_tables_the_reference_could_read_differently() {
        let mut tables = vec![LmTable::default()];
        tables[0].keys = vec![3, 2];
        tables[0].logp = vec![-1.0, -1.0];
        assert!(NgramLm::from_tables(1, 4, 11, tables).is_err());
        assert!(NgramLm::from_tables(6, 4, 11, vec![LmTable::default(); 6]).is_err());
        assert!(NgramLm::from_tables(5, 4, 11, vec![LmTable::default(); 5]).is_ok());
    }
}
