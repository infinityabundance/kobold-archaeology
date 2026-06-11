# kobold-archaeology

Public/private corpus archaeology engine: dataset registry, feature-terrain scanner, copybook/program feature extraction, gap boards, and dataset-to-court mapping.

**Part of KOBOLD** -- a forensic archaeology and evidence system for legacy COBOL estates: it maps real COBOL
codebases, generated oracle witnesses, compiler-profile behavior, and migration risk into court-backed
receipts. Independently-authored tooling; contains no GnuCOBOL source.

## What it does (v0.1)
- `CorpusIndex` / `Corpus` -- load + query a public/private dataset registry (`by_tier`), null-tolerant.
- `GapBoard` / `Surface` -- load + query a surface gap board (`with_status`, `missing_hottest`).
- `scan(text, default_surfaces())` -- a generalized, word-boundary-aware COBOL feature-terrain scanner
  (no regex; built-in surface vocabulary; extend per estate).
- CLI: `kobold-archaeology corpus|gap|scan <path>...`.

```
cargo run -- corpus path/to/public-corpus-index.json
cargo run -- gap    path/to/public-gap-board.json
cargo run -- scan   prog1.cob prog2.cob
```

Roadmap: copybook/program feature extraction, dataset->court mapping, terrain diffing across estates.

## Architecture
- gnucobol-rs (separate crate) = the oracle-proven semantic primitive layer.
- kobold-* = the forensic-intelligence layer.
- kobold-* MAY depend on gnucobol-rs; gnucobol-rs MUST NOT depend on kobold-*.

## License
Apache-2.0 (see LICENSE).
