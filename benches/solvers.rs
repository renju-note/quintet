//! Macro benchmark for the mate solvers.
//!
//! Runs every case in `benches/cases/`, checks its result, and reports the
//! nodes searched, the memo entries left and the wall-clock time of each.
//! Nodes and memo entries are deterministic and are the numbers to compare
//! solver changes by; time is only meaningful between runs on one machine.
//!
//! ```text
//! cargo bench --bench solvers -- [OPTIONS] [NAME_FILTER...]
//!
//!   --tag TAG         only cases tagged TAG (repeatable: all must match)
//!   --skip-tag TAG    leave out cases tagged TAG (repeatable)
//!   --all             also run the cases tagged `heavy`, which are left out
//!                     unless this or `--tag heavy` is given
//!   --mode MODE       run every case in MODE instead of its own; the verdict
//!                     is still checked, the exact line is not
//!   --runs N          time each case N times and report the median (default 3)
//!   --save PATH       write the results as TSV, to be passed to --baseline
//!   --baseline PATH   compare with results saved earlier by --save
//!   --cases DIR       read the cases from DIR (default benches/cases)
//! ```
//!
//! The format of a case file is described in `docs/07-benchmarks.en.md`.

use quintet::board::{Board, Player, Points};
use quintet::mate::{SolveLimits, SolveMode, SolveResult, solve_with_stats};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};
use std::{env, fs};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expect {
    /// Proven by exactly this line.
    Line(String),
    /// Proven, by whatever line.
    Proven,
    Disproven,
    Aborted,
}

#[derive(Clone)]
struct Case {
    name: String,
    mode: String,
    attacker: Player,
    limits: SolveLimits,
    tags: Vec<String>,
    expect: Expect,
    board: Board,
}

fn parse_case(name: &str, text: &str) -> Result<Case, String> {
    let mut fields = HashMap::new();
    let mut lines = text.lines();
    let mut board_text = None;
    for line in lines.by_ref() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("expected `key: value`, got `{line}`"))?;
        let (key, value) = (key.trim(), value.trim());
        if key == "board" {
            board_text = Some(if value.is_empty() {
                lines.by_ref().collect::<Vec<_>>().join("\n")
            } else {
                value.to_string()
            });
            break;
        }
        fields.insert(key.to_string(), value.to_string());
    }
    let board_text = board_text.ok_or("missing `board:`")?;
    let field = |key: &str| fields.get(key).map(String::as_str);
    let required = |key: &str| field(key).ok_or_else(|| format!("missing `{key}:`"));
    let number = |key: &str| -> Result<Option<u64>, String> {
        field(key)
            .map(|v| v.parse::<u64>().map_err(|e| format!("`{key}`: {e}")))
            .transpose()
    };
    let depth = |key: &str| -> Result<Option<u8>, String> {
        number(key)?
            .map(|v| u8::try_from(v).map_err(|e| format!("`{key}`: {e}")))
            .transpose()
    };

    let mode = required("mode")?.to_string();
    mode.parse::<SolveMode>()
        .map_err(|e| format!("`mode`: {e}"))?;
    let attacker = required("attacker")?
        .parse::<Player>()
        .map_err(|e| format!("`attacker`: {e}"))?;
    let mut limits = SolveLimits::new(depth("limit")?.ok_or("missing `limit:`")?);
    if let Some(v) = depth("threat_limit")? {
        limits = limits.with_threat_limit(v);
    }
    if let Some(v) = depth("defender_vcf_depth")? {
        limits = limits.with_defender_vcf_depth(v);
    }
    if let Some(v) = number("max_nodes")? {
        limits = limits.with_max_nodes(v);
    }
    let tags = field("tags")
        .unwrap_or("")
        .split_whitespace()
        .map(String::from)
        .collect();
    let expect = match required("expect")? {
        "proven" => Expect::Proven,
        "disproven" => Expect::Disproven,
        "aborted" => Expect::Aborted,
        line => Expect::Line(line.split_whitespace().collect()),
    };
    let board = board_text
        .parse::<Board>()
        .map_err(|e| format!("`board`: {e}"))?;
    Ok(Case {
        name: name.to_string(),
        mode,
        attacker,
        limits,
        tags,
        expect,
        board,
    })
}

fn load_cases(dir: &Path) -> Result<Vec<Case>, String> {
    let mut paths = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    paths.retain(|p| p.extension().is_some_and(|ext| ext == "txt"));
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let name = path.file_stem().unwrap().to_string_lossy();
            let text = fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
            parse_case(&name, &text).map_err(|e| format!("{name}: {e}"))
        })
        .collect()
}

/// One row of the results, as printed and as saved by `--save`.
#[derive(Debug, Clone)]
struct Row {
    name: String,
    mode: String,
    verdict: String,
    nodes: u64,
    memo_len: usize,
    time: Duration,
}

impl Row {
    fn key(&self) -> (String, String) {
        (self.name.clone(), self.mode.clone())
    }
}

/// The tag of the cases too slow to run by default.
const HEAVY: &str = "heavy";

const TSV_HEADER: &str = "name\tmode\tverdict\tnodes\tmemo_len\ttime_us";

fn to_tsv(rows: &[Row]) -> String {
    let mut out = String::from(TSV_HEADER);
    out.push('\n');
    for r in rows {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}",
            r.name,
            r.mode,
            r.verdict,
            r.nodes,
            r.memo_len,
            r.time.as_micros()
        );
    }
    out
}

fn from_tsv(text: &str) -> Result<Vec<Row>, String> {
    let mut lines = text.lines();
    if lines.next() != Some(TSV_HEADER) {
        return Err("not a results file written by --save".into());
    }
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            let [name, mode, verdict, nodes, memo_len, time_us] = cols[..] else {
                return Err(format!("bad row: `{line}`"));
            };
            let bad = |e: std::num::ParseIntError| format!("bad row `{line}`: {e}");
            Ok(Row {
                name: name.into(),
                mode: mode.into(),
                verdict: verdict.into(),
                nodes: nodes.parse().map_err(bad)?,
                memo_len: memo_len.parse().map_err(bad)?,
                time: Duration::from_micros(time_us.parse().map_err(bad)?),
            })
        })
        .collect()
}

fn verdict(result: &SolveResult) -> &'static str {
    match result {
        SolveResult::Proven(_) => "proven",
        SolveResult::Disproven => "disproven",
        SolveResult::Aborted => "aborted",
    }
}

/// Checks `result` against the case; `exact` also checks the line.
fn check(expect: &Expect, result: &SolveResult, exact: bool) -> Result<(), String> {
    let got = verdict(result);
    let want = match expect {
        Expect::Line(_) | Expect::Proven => "proven",
        Expect::Disproven => "disproven",
        Expect::Aborted => "aborted",
    };
    if got != want {
        return Err(format!("expected {want}, got {got}"));
    }
    if let (Expect::Line(line), SolveResult::Proven(mate), true) = (expect, result, exact) {
        let path = Points(mate.path.clone()).to_string();
        if &path != line {
            return Err(format!("expected line {line}, got {path}"));
        }
    }
    Ok(())
}

fn run_case(case: &Case, mode: &str, exact: bool, runs: usize) -> Result<Row, String> {
    let solve_mode = mode.parse::<SolveMode>()?;
    let mut times = Vec::with_capacity(runs);
    let mut first = None;
    for _ in 0..runs {
        let start = Instant::now();
        let (result, stats) = solve_with_stats(solve_mode, &case.board, case.attacker, case.limits);
        times.push(start.elapsed());
        check(&case.expect, &result, exact)?;
        match first {
            None => first = Some((verdict(&result), stats)),
            Some(f) if f != (verdict(&result), stats) => {
                return Err(format!(
                    "not deterministic: {f:?} then {:?}",
                    (verdict(&result), stats)
                ));
            }
            Some(_) => {}
        }
    }
    times.sort();
    let (verdict, stats) = first.unwrap();
    Ok(Row {
        name: case.name.clone(),
        mode: mode.to_string(),
        verdict: verdict.to_string(),
        nodes: stats.nodes,
        memo_len: stats.memo_len,
        time: times[times.len() / 2],
    })
}

fn format_time(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1e3;
    if ms >= 1e3 {
        format!("{:.2}s", ms / 1e3)
    } else {
        format!("{ms:.1}ms")
    }
}

fn format_delta(new: f64, old: f64) -> String {
    if old == new {
        "=".into()
    } else if old == 0.0 {
        "new".into()
    } else {
        format!("{:+.1}%", (new - old) / old * 100.0)
    }
}

struct Options {
    cases_dir: PathBuf,
    filters: Vec<String>,
    tags: Vec<String>,
    skip_tags: Vec<String>,
    all: bool,
    mode: Option<String>,
    runs: usize,
    save: Option<PathBuf>,
    baseline: Option<PathBuf>,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        cases_dir: Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/cases"),
        filters: vec![],
        tags: vec![],
        skip_tags: vec![],
        all: false,
        mode: None,
        runs: 3,
        save: None,
        baseline: None,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            // Passed by `cargo bench` to every bench target.
            "--bench" => {}
            "--tag" => options.tags.push(value()?),
            "--skip-tag" => options.skip_tags.push(value()?),
            "--all" => options.all = true,
            "--mode" => {
                let mode = value()?;
                mode.parse::<SolveMode>()?;
                options.mode = Some(mode);
            }
            "--runs" => {
                options.runs = value()?.parse().map_err(|e| format!("--runs: {e}"))?;
                if options.runs == 0 {
                    return Err("--runs must be at least 1".into());
                }
            }
            "--save" => options.save = Some(value()?.into()),
            "--baseline" => options.baseline = Some(value()?.into()),
            "--cases" => options.cases_dir = value()?.into(),
            _ if arg.starts_with('-') => return Err(format!("unknown option {arg}")),
            _ => options.filters.push(arg),
        }
    }
    Ok(options)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

/// Runs the selected cases; `Ok(false)` if any of them failed.
fn run() -> Result<bool, String> {
    let mut options = parse_options()?;
    if !options.all && !options.tags.iter().any(|t| t == HEAVY) {
        options.skip_tags.push(HEAVY.to_string());
    }
    let baseline: HashMap<_, _> = match &options.baseline {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            from_tsv(&text)?.into_iter().map(|r| (r.key(), r)).collect()
        }
        None => HashMap::new(),
    };
    let cases: Vec<Case> = load_cases(&options.cases_dir)?
        .into_iter()
        .filter(|c| {
            options.filters.is_empty() || options.filters.iter().any(|f| c.name.contains(f))
        })
        .filter(|c| options.tags.iter().all(|t| c.tags.contains(t)))
        .filter(|c| !options.skip_tags.iter().any(|t| c.tags.contains(t)))
        .collect();
    if cases.is_empty() {
        return Err("no case selected".into());
    }

    let compare = options.baseline.is_some();
    if compare {
        println!(
            "{:<32} {:<10} {:>12} {:>8} {:>10} {:>8} {:>10} {:>8}",
            "case", "mode", "nodes", "Δ", "memo", "Δ", "time", "Δ"
        );
    } else {
        println!(
            "{:<32} {:<10} {:>12} {:>10} {:>10}",
            "case", "mode", "nodes", "memo", "time"
        );
    }

    let mut rows = vec![];
    let mut failures = 0;
    let (mut base_nodes, mut base_time, mut compared) = (0, Duration::ZERO, 0);
    for case in &cases {
        let mode = options.mode.as_deref().unwrap_or(&case.mode);
        let exact = options.mode.is_none() || options.mode.as_deref() == Some(&case.mode);
        let row = match run_case(case, mode, exact, options.runs) {
            Ok(row) => row,
            Err(e) => {
                println!("{:<32} {:<10} FAILED: {e}", case.name, mode);
                failures += 1;
                continue;
            }
        };
        let (nodes, memo, time) = (
            row.nodes.to_string(),
            row.memo_len.to_string(),
            format_time(row.time),
        );
        match baseline.get(&row.key()) {
            Some(b) => {
                println!(
                    "{:<32} {:<10} {:>12} {:>8} {:>10} {:>8} {:>10} {:>8}",
                    row.name,
                    row.mode,
                    nodes,
                    format_delta(row.nodes as f64, b.nodes as f64),
                    memo,
                    format_delta(row.memo_len as f64, b.memo_len as f64),
                    time,
                    format_delta(row.time.as_secs_f64(), b.time.as_secs_f64()),
                );
                if b.verdict != row.verdict {
                    println!("  verdict changed: {} -> {}", b.verdict, row.verdict);
                }
                base_nodes += b.nodes;
                base_time += b.time;
                compared += 1;
            }
            None if compare => println!(
                "{:<32} {:<10} {:>12} {:>8} {:>10} {:>8} {:>10} {:>8}",
                row.name, row.mode, nodes, "new", memo, "new", time, "new"
            ),
            None => println!(
                "{:<32} {:<10} {:>12} {:>10} {:>10}",
                row.name, row.mode, nodes, memo, time
            ),
        }
        rows.push(row);
    }

    let total_nodes: u64 = rows.iter().map(|r| r.nodes).sum();
    let total_time: Duration = rows.iter().map(|r| r.time).sum();
    println!();
    println!(
        "total: {} cases, {} nodes, {}",
        rows.len(),
        total_nodes,
        format_time(total_time)
    );
    if compared > 0 {
        let (nodes, time) = rows
            .iter()
            .filter(|r| baseline.contains_key(&r.key()))
            .fold((0, Duration::ZERO), |(n, t), r| (n + r.nodes, t + r.time));
        println!(
            "vs baseline ({compared} cases in both): nodes {}, time {}",
            format_delta(nodes as f64, base_nodes as f64),
            format_delta(time.as_secs_f64(), base_time.as_secs_f64()),
        );
    }
    if failures > 0 {
        println!("{failures} case(s) FAILED");
    }

    if let Some(path) = &options.save {
        fs::write(path, to_tsv(&rows)).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("saved to {}", path.display());
    }
    Ok(failures == 0)
}
