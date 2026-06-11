//! kobold-archaeology CLI.
//!   kobold-archaeology corpus <corpus-index.json>   -- summarize the dataset registry
//!   kobold-archaeology gap    <gap-board.json>      -- summarize the surface gap board
//!   kobold-archaeology scan   <file.cob>...         -- feature-terrain scan of COBOL source
use kobold_archaeology::{default_surfaces, scan, CorpusIndex, GapBoard};
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: kobold-archaeology <corpus|gap|scan> <path>...";
    if args.len() < 3 {
        eprintln!("{usage}");
        exit(2);
    }
    match args[1].as_str() {
        "corpus" => match CorpusIndex::load(&args[2]) {
            Ok(ci) => {
                println!("corpora: {} (schema={})", ci.corpora.len(), ci.schema);
                for c in &ci.corpora {
                    println!("  [tier {}] {:<28} {} feature(s) [{}]", c.tier, c.id, c.features.len(), c.status);
                }
            }
            Err(e) => {
                eprintln!("load error: {e}");
                exit(2);
            }
        },
        "gap" => match GapBoard::load(&args[2]) {
            Ok(gb) => {
                println!("surfaces: {} ({} files scanned)", gb.surfaces.len(), gb.files_scanned);
                for st in ["sealed", "observed", "refused", "missing"] {
                    let v = gb.with_status(st);
                    if !v.is_empty() {
                        println!("  {st}: {}", v.len());
                    }
                }
                let miss = gb.missing_hottest();
                if !miss.is_empty() {
                    println!("hottest MISSING (exercised, no court):");
                    for s in miss.iter().take(10) {
                        println!("  {:<28} {} occurrences", s.surface, s.occurrences);
                    }
                }
            }
            Err(e) => {
                eprintln!("load error: {e}");
                exit(2);
            }
        },
        "scan" => {
            let mut all = String::new();
            for f in &args[2..] {
                match std::fs::read_to_string(f) {
                    Ok(s) => all.push_str(&s),
                    Err(e) => {
                        eprintln!("read {f}: {e}");
                        exit(2);
                    }
                }
            }
            println!("feature terrain ({} file(s)):", args.len() - 2);
            for (name, count) in scan(&all, &default_surfaces()) {
                println!("  {name:<20} {count}");
            }
        }
        _ => {
            eprintln!("{usage}");
            exit(2);
        }
    }
}
