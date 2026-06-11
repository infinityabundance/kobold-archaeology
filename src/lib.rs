#![forbid(unsafe_code)]
//! # kobold-archaeology
//!
//! Corpus archaeology as a reusable Apache-2.0 library: a public/private COBOL **dataset registry**, a
//! surface-frequency **gap board**, and a generalized COBOL **feature-terrain scanner** that maps real
//! source against the court board ("what does this estate actually exercise, and where are the gaps?").
//!
//! Part of the KOBOLD ecosystem (independently-authored tooling; no GnuCOBOL source). Dependency rule:
//! kobold-* MAY depend on gnucobol-rs; gnucobol-rs MUST NOT depend on kobold-*.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Tolerate explicit JSON `null` (and absence) for integer fields -> 0.
fn de_i64_lenient<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<i64>::deserialize(d)?.unwrap_or(0))
}

/// One indexed public/private COBOL corpus.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Corpus {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, deserialize_with = "de_i64_lenient")]
    pub tier: i64,
    #[serde(default, deserialize_with = "de_i64_lenient")]
    pub priority: i64,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub status: String,
}

/// The corpus registry (e.g. a public-corpus-index).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorpusIndex {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub doctrine: String,
    pub corpora: Vec<Corpus>,
    #[serde(default)]
    pub best_first_10: Vec<String>,
}

impl CorpusIndex {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
    pub fn by_tier(&self, tier: i64) -> Vec<&Corpus> {
        self.corpora.iter().filter(|c| c.tier == tier).collect()
    }
}

/// One surface on the gap board (a COBOL construct classified vs the court map).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Surface {
    pub surface: String,
    #[serde(default, deserialize_with = "de_i64_lenient")]
    pub occurrences: i64,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub court: String,
}

/// The surface-frequency gap board.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapBoard {
    #[serde(default)]
    pub schema: String,
    #[serde(default, deserialize_with = "de_i64_lenient")]
    pub files_scanned: i64,
    pub surfaces: Vec<Surface>,
}

impl GapBoard {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
    pub fn with_status<'a>(&'a self, status: &str) -> Vec<&'a Surface> {
        self.surfaces.iter().filter(|s| s.status == status).collect()
    }
    /// The missing-court board: exercised surfaces with no court yet, hottest first.
    pub fn missing_hottest(&self) -> Vec<&Surface> {
        let mut v = self.with_status("missing");
        v.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        v
    }
}

/// A surface pattern for the feature scanner: a name + the COBOL word(s) that signal it.
#[derive(Debug, Clone)]
pub struct SurfacePattern {
    pub name: &'static str,
    pub needles: &'static [&'static str],
}

/// A built-in COBOL surface vocabulary so `scan` works out of the box. Generalize/extend per estate.
pub fn default_surfaces() -> Vec<SurfacePattern> {
    macro_rules! s {
        ($n:expr, $($x:expr),*) => { SurfacePattern { name: $n, needles: &[$($x),*] } };
    }
    vec![
        s!("MOVE", "MOVE"),
        s!("ACCEPT/DISPLAY", "ACCEPT", "DISPLAY"),
        s!("PERFORM", "PERFORM"),
        s!("CALL/linkage", "CALL", "CANCEL"),
        s!("IF/EVALUATE", "IF", "EVALUATE"),
        s!("arithmetic", "COMPUTE", "ADD", "SUBTRACT", "MULTIPLY", "DIVIDE"),
        s!("STRING/UNSTRING", "STRING", "UNSTRING"),
        s!("INSPECT", "INSPECT"),
        s!("INITIALIZE", "INITIALIZE"),
        s!("SEARCH", "SEARCH"),
        s!("SORT/MERGE", "SORT", "MERGE"),
        s!("file-io", "OPEN", "READ", "WRITE", "REWRITE", "DELETE", "START", "CLOSE"),
        s!("COPY/REPLACING", "COPY", "REPLACING", "REPLACE"),
        s!("REDEFINES", "REDEFINES"),
        s!("OCCURS/ODO", "OCCURS"),
        s!("GO TO", "GO TO"),
        s!("ALTER", "ALTER"),
        s!("DECLARATIVES", "DECLARATIVES"),
        s!("SCREEN SECTION", "SCREEN SECTION"),
        s!("REPORT WRITER", "REPORT SECTION"),
        s!("EXEC SQL/CICS", "EXEC SQL", "EXEC CICS"),
    ]
}

/// Scan COBOL source text for surface occurrences. Word-boundary aware (no regex): single-word needles
/// match whole tokens; multi-word needles match within the normalized token stream. Returns
/// `(surface_name, count)` hottest first, zero-count surfaces dropped.
pub fn scan(text: &str, patterns: &[SurfacePattern]) -> Vec<(String, usize)> {
    let tokens: Vec<String> = text
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_uppercase() } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(String::from)
        .collect();
    let joined = format!(" {} ", tokens.join(" "));
    let mut out: Vec<(String, usize)> = patterns
        .iter()
        .map(|p| {
            let count: usize = p
                .needles
                .iter()
                .map(|needle| {
                    let nu = needle.to_ascii_uppercase();
                    if nu.contains(' ') {
                        joined.matches(&format!(" {nu} ")).count()
                    } else {
                        tokens.iter().filter(|t| **t == nu).count()
                    }
                })
                .sum();
            (p.name.to_string(), count)
        })
        .filter(|(_, c)| *c > 0)
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_and_gap_fixtures() {
        let ci = CorpusIndex::load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/corpus-index.json")).unwrap();
        assert!(ci.corpora.len() >= 2);
        assert_eq!(ci.by_tier(1).len(), 1);
        let gb = GapBoard::load(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/gap-board.json")).unwrap();
        assert_eq!(gb.missing_hottest().first().map(|s| s.surface.as_str()), Some("CALL/linkage"));
    }

    #[test]
    fn scanner_counts_surfaces_word_aware() {
        let src = "PROCEDURE DIVISION.\n  MOVE A TO B.\n  MOVE C TO D.\n  PERFORM P.\n  DISPLAY X.\n  CALL 'SUB'.\n";
        let r = scan(src, &default_surfaces());
        let map: std::collections::HashMap<_, _> = r.iter().cloned().collect();
        assert_eq!(map.get("MOVE"), Some(&2));
        assert_eq!(map.get("PERFORM"), Some(&1));
        assert_eq!(map.get("ACCEPT/DISPLAY"), Some(&1));
        assert_eq!(map.get("CALL/linkage"), Some(&1));
        // "ALREADY" must NOT count as READ (word-boundary aware)
        assert!(scan("ALREADY DONE", &default_surfaces()).is_empty());
    }
}
