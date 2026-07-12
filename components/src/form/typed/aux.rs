//! Per-field UI state that cannot live inside the typed struct `T`:
//! unparseable in-progress text (overlay), written/pristine tracking,
//! touched, and error messages.
//!
//! Deliberately store-agnostic — keyed by dot-notation path strings, no
//! knowledge of `T` — so a future dynamic (runtime-schema) form reuses it
//! verbatim. All row-shifting logic for `Vec` fields lives here; it is the
//! single place index re-keying happens.

use std::collections::{HashMap, HashSet};

/// In-progress text that failed to parse into the field's type, plus the
/// parse error to surface once the field is touched.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayEntry {
    pub text: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuxState {
    /// Raw text overriding the typed value for display while unparseable.
    pub overlay: HashMap<String, OverlayEntry>,
    /// Per-field written override. Fields absent here fall back to
    /// [`AuxState::all_written`]. Unwritten fields display blank even though
    /// `T::default()` gives them a value.
    pub written: HashMap<String, bool>,
    /// Blanket set by `default_values`: fields without an explicit
    /// [`AuxState::written`] entry count as written.
    pub all_written: bool,
    /// Fields that have been blurred/committed at least once.
    pub touched: HashSet<String>,
    /// Current validation message per field (`None` = explicitly cleared).
    pub errors: HashMap<String, Option<String>>,
}

impl AuxState {
    pub fn is_written(&self, path: &str) -> bool {
        self.written.get(path).copied().unwrap_or(self.all_written)
    }

    pub fn is_touched(&self, path: &str) -> bool {
        self.touched.contains(path)
    }

    pub fn error(&self, path: &str) -> Option<String> {
        self.errors.get(path).cloned().flatten()
    }

    pub fn mark_written(&mut self, path: &str) {
        self.written.insert(path.to_string(), true);
    }

    pub fn unmark_written(&mut self, path: &str) {
        self.written.insert(path.to_string(), false);
    }

    pub fn touch(&mut self, path: &str) {
        if !self.touched.contains(path) {
            self.touched.insert(path.to_string());
        }
    }

    pub fn set_error(&mut self, path: &str, error: Option<String>) {
        self.errors.insert(path.to_string(), error);
    }

    pub fn set_overlay(&mut self, path: &str, text: String, message: String) {
        self.overlay
            .insert(path.to_string(), OverlayEntry { text, message });
    }

    pub fn clear_overlay(&mut self, path: &str) {
        self.overlay.remove(path);
    }

    /// Reset `path` and its descendants (`path.*`) to pristine.
    pub fn clear_field(&mut self, path: &str) {
        let prefix = format!("{path}.");
        let under = |k: &str| k == path || k.starts_with(&prefix);
        self.overlay.retain(|k, _| !under(k));
        self.touched.retain(|k| !under(k));
        self.errors.retain(|k, _| !under(k));
        self.written.retain(|k, _| !under(k));
        if self.all_written {
            self.unmark_written(path);
        }
    }

    /// Row removed at `index` under array `prefix`: drop that row's entries,
    /// shift higher rows down by one.
    pub fn remove_row(&mut self, prefix: &str, index: usize) {
        self.rekey_rows(prefix, |row| {
            if row == index {
                None
            } else if row > index {
                Some(row - 1)
            } else {
                Some(row)
            }
        });
    }

    /// Row inserted at `index` under array `prefix`: shift rows at and above
    /// `index` up by one.
    pub fn insert_row(&mut self, prefix: &str, index: usize) {
        self.rekey_rows(prefix, |row| {
            if row >= index {
                Some(row + 1)
            } else {
                Some(row)
            }
        });
    }

    /// Rows `a` and `b` swapped under array `prefix`.
    pub fn swap_rows(&mut self, prefix: &str, a: usize, b: usize) {
        self.rekey_rows(prefix, |row| {
            if row == a {
                Some(b)
            } else if row == b {
                Some(a)
            } else {
                Some(row)
            }
        });
    }

    fn rekey_rows(&mut self, prefix: &str, map_row: impl Fn(usize) -> Option<usize>) {
        rekey_map(&mut self.overlay, prefix, &map_row);
        rekey_map(&mut self.errors, prefix, &map_row);
        rekey_map(&mut self.written, prefix, &map_row);
        rekey_set(&mut self.touched, prefix, &map_row);
    }
}

/// Splits `key` as `{prefix}.{row}{rest}` where `row` is a numeric segment
/// and `rest` is empty or starts with '.'. Returns `None` when `key` is not
/// under `prefix` or the next segment is not an index.
fn split_row<'a>(key: &'a str, prefix: &str) -> Option<(usize, &'a str)> {
    let tail = key.strip_prefix(prefix)?.strip_prefix('.')?;
    let seg_end = tail.find('.').unwrap_or(tail.len());
    let (seg, rest) = tail.split_at(seg_end);
    let row: usize = seg.parse().ok()?;
    Some((row, rest))
}

enum Rekey {
    Keep,
    Drop,
    To(String),
}

fn rekeyed(key: &str, prefix: &str, map_row: &impl Fn(usize) -> Option<usize>) -> Rekey {
    match split_row(key, prefix) {
        None => Rekey::Keep,
        Some((row, rest)) => match map_row(row) {
            None => Rekey::Drop,
            Some(new_row) if new_row == row => Rekey::Keep,
            Some(new_row) => Rekey::To(format!("{prefix}.{new_row}{rest}")),
        },
    }
}

fn rekey_map<V>(
    map: &mut HashMap<String, V>,
    prefix: &str,
    map_row: &impl Fn(usize) -> Option<usize>,
) {
    let mut moved: Vec<(String, V)> = Vec::new();
    let keys: Vec<String> = map.keys().cloned().collect();
    for key in keys {
        match rekeyed(&key, prefix, map_row) {
            Rekey::Keep => {}
            Rekey::Drop => {
                map.remove(&key);
            }
            Rekey::To(new_key) => {
                if let Some(v) = map.remove(&key) {
                    moved.push((new_key, v));
                }
            }
        }
    }
    map.extend(moved);
}

fn rekey_set(set: &mut HashSet<String>, prefix: &str, map_row: &impl Fn(usize) -> Option<usize>) {
    let mut moved: Vec<String> = Vec::new();
    let keys: Vec<String> = set.iter().cloned().collect();
    for key in keys {
        match rekeyed(&key, prefix, map_row) {
            Rekey::Keep => {}
            Rekey::Drop => {
                set.remove(&key);
            }
            Rekey::To(new_key) => {
                set.remove(&key);
                moved.push(new_key);
            }
        }
    }
    set.extend(moved);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(keys: &[&str]) -> AuxState {
        let mut s = AuxState::default();
        for k in keys {
            s.mark_written(k);
            s.touch(k);
            s.set_error(k, Some(format!("err:{k}")));
            s.set_overlay(k, format!("txt:{k}"), "bad".into());
        }
        s
    }

    #[test]
    fn split_row_parses_index_segments() {
        assert_eq!(split_row("items.2.qty", "items"), Some((2, ".qty")));
        assert_eq!(split_row("items.10", "items"), Some((10, "")));
        assert_eq!(split_row("items.x.qty", "items"), None);
        assert_eq!(split_row("itemsy.2", "items"), None);
        assert_eq!(split_row("other.2", "items"), None);
        // A dot-descendant of a sibling-prefixed key must not match.
        assert_eq!(split_row("items", "items"), None);
    }

    #[test]
    fn remove_row_drops_and_shifts() {
        let mut s = state_with(&["name", "items.0.qty", "items.1.qty", "items.2.qty"]);
        s.remove_row("items", 1);

        assert!(s.is_written("name"));
        assert!(s.is_written("items.0.qty"));
        // old row 2 became row 1
        assert!(s.is_written("items.1.qty"));
        assert!(!s.is_written("items.2.qty"));
        assert_eq!(s.error("items.1.qty"), Some("err:items.2.qty".into()));
        assert_eq!(
            s.overlay.get("items.1.qty").map(|o| o.text.clone()),
            Some("txt:items.2.qty".into())
        );
        assert!(s.is_touched("items.1.qty"));
        assert!(!s.is_touched("items.2.qty"));
    }

    #[test]
    fn insert_row_shifts_up() {
        let mut s = state_with(&["items.0.qty", "items.1.qty"]);
        s.insert_row("items", 0);
        assert!(!s.is_written("items.0.qty"));
        assert!(s.is_written("items.1.qty"));
        assert!(s.is_written("items.2.qty"));
    }

    #[test]
    fn swap_rows_exchanges_keys() {
        let mut s = AuxState::default();
        s.set_error("items.0.qty", Some("a".into()));
        s.set_error("items.3.qty", Some("b".into()));
        s.swap_rows("items", 0, 3);
        assert_eq!(s.error("items.0.qty"), Some("b".into()));
        assert_eq!(s.error("items.3.qty"), Some("a".into()));
    }

    #[test]
    fn rekey_ignores_similar_prefixes() {
        let mut s = state_with(&["items_extra.1.qty", "items.1.qty"]);
        s.remove_row("items", 1);
        assert!(s.is_written("items_extra.1.qty"));
        assert!(!s.is_written("items.1.qty"));
    }

    #[test]
    fn clear_field_resets_descendants_under_blanket() {
        let mut s = AuxState::default();
        s.all_written = true;
        s.touch("address.street");
        s.set_error("address.street", Some("x".into()));
        s.clear_field("address");
        assert!(!s.is_written("address"));
        assert!(!s.is_touched("address.street"));
        assert_eq!(s.error("address.street"), None);
        // blanket still applies to unrelated fields
        assert!(s.is_written("name"));
    }

    /// Deterministic pseudo-random op sequences against a naive oracle that
    /// models rows as an actual Vec of per-row key sets.
    #[test]
    fn row_ops_match_oracle() {
        const PREFIX: &str = "items";
        const SUFFIXES: [&str; 3] = ["qty", "name", "nested.deep"];

        struct Lcg(u64);
        impl Lcg {
            fn next(&mut self, bound: usize) -> usize {
                // Numerical Recipes LCG constants.
                self.0 = self
                    .0
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((self.0 >> 33) as usize) % bound.max(1)
            }
        }

        for seed in 0..20u64 {
            let mut rng = Lcg(seed.wrapping_add(1));
            let mut aux = AuxState::default();
            // oracle: rows as Vec<HashSet<suffix>> of "touched" markers
            let mut oracle: Vec<HashSet<&'static str>> = Vec::new();

            for _step in 0..200 {
                match rng.next(4) {
                    // add row at random position with random touched suffixes
                    0 => {
                        let at = rng.next(oracle.len() + 1);
                        aux.insert_row(PREFIX, at);
                        let mut row = HashSet::new();
                        for s in SUFFIXES {
                            if rng.next(2) == 0 {
                                let key = format!("{PREFIX}.{at}.{s}");
                                aux.touch(&key);
                                row.insert(s);
                            }
                        }
                        oracle.insert(at, row);
                    }
                    // remove random row
                    1 if !oracle.is_empty() => {
                        let at = rng.next(oracle.len());
                        aux.remove_row(PREFIX, at);
                        oracle.remove(at);
                    }
                    // swap two random rows
                    2 if oracle.len() >= 2 => {
                        let a = rng.next(oracle.len());
                        let b = rng.next(oracle.len());
                        aux.swap_rows(PREFIX, a, b);
                        oracle.swap(a, b);
                    }
                    // touch a random suffix on a random row
                    3 if !oracle.is_empty() => {
                        let at = rng.next(oracle.len());
                        let s = SUFFIXES[rng.next(SUFFIXES.len())];
                        aux.touch(&format!("{PREFIX}.{at}.{s}"));
                        oracle[at].insert(s);
                    }
                    _ => {}
                }

                // full-state comparison
                let mut expected: HashSet<String> = HashSet::new();
                for (i, row) in oracle.iter().enumerate() {
                    for s in row {
                        expected.insert(format!("{PREFIX}.{i}.{s}"));
                    }
                }
                assert_eq!(
                    aux.touched, expected,
                    "seed {seed}: aux touched diverged from oracle"
                );
            }
        }
    }
}
