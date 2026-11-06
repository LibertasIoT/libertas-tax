//! Ordered complete capture comparison and final-estimate review.
mod capture;
pub mod capture_budget;
pub mod compact;
pub use compact::{compact_event, validate_compact_event};
mod compare;
mod debug;
mod final_figures;
mod json;
mod structure;
mod timing;
mod verify_compact;

use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(message.into().into())
}

static SERIAL: AtomicU64 = AtomicU64::new(0);
fn unique() -> u64 {
    SERIAL.fetch_add(1, Ordering::Relaxed)
}
struct TempDir(PathBuf);
impl TempDir {
    fn new(parent: &Path) -> Result<Self> {
        for _ in 0..100 {
            let path = parent.join(format!("tax-figures-rust-{}-{}", std::process::id(), unique()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        fail("unable to create a temporary comparison directory")
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
struct Args {
    command: String,
    options: BTreeMap<String, String>,
    positional: Vec<String>,
    shard: (i64, i64),
    jobs: usize,
    differences_only: bool,
}
impl Args {
    fn parse() -> Result<Self> {
        let mut argv = std::env::args().skip(1);
        let command=argv.next().ok_or("usage: tax-paired-figures compare --old OLD --new NEW --output PREFIX [--differences-only] | final OLD NEW [DELTAS_OUT]")?;
        if command != "compare"
            && command != "final"
            && command != "final-figures"
            && command != "compress"
            && command != "reduce"
            && command != "verify-compact"
        {
            return fail(format!(
                "unknown command: {command}; expected compare, final, compress, reduce or verify-compact"
            ));
        }
        let mut result = Self {
            command,
            options: BTreeMap::new(),
            positional: Vec::new(),
            shard: (0, 1),
            jobs: 12,
            differences_only: false,
        };
        while let Some(arg) = argv.next() {
            if arg == "--differences-only" {
                if result.command != "compare" {
                    return fail("--differences-only is only supported by compare");
                }
                result.differences_only = true;
            } else if arg == "--test-shard" {
                let index: i64 = argv.next().ok_or("test shard requires INDEX COUNT")?.parse()?;
                let count: i64 = argv.next().ok_or("test shard requires INDEX COUNT")?.parse()?;
                result.shard = (index, count);
            } else if arg == "--jobs" {
                result.jobs = argv.next().ok_or("--jobs requires COUNT")?.parse()?;
                if result.jobs == 0 {
                    return fail("--jobs must be positive");
                }
            } else if let Some(name) = arg.strip_prefix("--") {
                if ![
                    "old",
                    "new",
                    "output",
                    "index",
                    "line-map",
                    "reasons",
                    "structure-map",
                    "capture-index-dir",
                    "pairs-compression",
                    "memory-budget-mib",
                ]
                .contains(&name)
                {
                    return fail(format!("unknown option: {arg}"));
                }
                result.options.insert(
                    name.into(),
                    argv.next().ok_or_else(|| format!("missing value for {arg}"))?,
                );
            } else {
                result.positional.push(arg);
            }
        }
        if let Some(codec) = result.options.get("pairs-compression")
            && (result.command != "compare" || !matches!(codec.as_str(), "gzip" | "zstd"))
        {
            return fail("--pairs-compression requires compare and gzip or zstd");
        }
        if let Some(budget) = result.options.get("memory-budget-mib") {
            let budget: usize = budget.parse()?;
            if result.command != "compare"
                || budget == 0
                || budget.checked_mul(1024 * 1024).is_none()
            {
                return fail(
                    "--memory-budget-mib requires compare and a positive byte-representable budget",
                );
            }
        }
        if result.options.contains_key("capture-index-dir")
            && !matches!(result.command.as_str(), "compare" | "final" | "final-figures")
        {
            return fail("--capture-index-dir requires compare or final");
        }
        if result.command == "compare" && !result.positional.is_empty() {
            return fail("compare accepts named options only");
        }
        if result.command == "compare" {
            for key in ["old", "new", "output"] {
                result.required(key)?;
            }
        } else if result.command == "verify-compact" {
            if result.positional.len() != 2 || !result.options.is_empty() {
                return fail("verify-compact requires REFERENCE_DIR CANDIDATE_DIR");
            }
        } else if matches!(result.command.as_str(), "compress" | "reduce")
            && result.positional.len() != 2
        {
            return fail("compress/reduce requires OLD_EVENTS OUTPUT.zst");
        } else if !matches!(result.command.as_str(), "compress" | "reduce")
            && !(2..=3).contains(&result.positional.len())
        {
            return fail("final requires OLD_EVENTS NEW_EVENTS [DELTAS_OUT]");
        }
        Ok(result)
    }
    fn required(&self, key: &str) -> Result<&str> {
        self.options.get(key).map(String::as_str).ok_or_else(|| format!("missing --{key}").into())
    }
    fn read_json(&self, key: &str) -> Result<Value> {
        self.options.get(key).map_or(Ok(serde_json::json!({})), |path| {
            Ok(serde_json::from_reader(std::fs::File::open(path)?)?)
        })
    }
}
fn spaced_json(value: &Value) -> String {
    fn render(value: &Value, out: &mut String) {
        match value {
            Value::Object(map) => {
                out.push('{');
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&json::ascii_string(key));
                    out.push_str(": ");
                    render(value, out);
                }
                out.push('}');
            }
            Value::Array(items) => {
                out.push('[');
                for (i, value) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    render(value, out);
                }
                out.push(']');
            }
            Value::String(s) => out.push_str(&json::ascii_string(s)),
            Value::Number(number) => out.push_str(&json::number_text(number)),
            _ => out.push_str(&value.to_string()),
        }
    }
    let mut out = String::new();
    render(value, &mut out);
    out
}
pub fn run_cli() {
    let args = match Args::parse() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("tax-paired-figures: {error}");
            std::process::exit(2);
        }
    };
    let total = timing::PhaseTimer::new(format!("{}_total", args.command));
    let result = match args.command.as_str() {
        "compare" => compare::run(&args),
        "compress" => capture::compress(&args),
        "reduce" => capture::reduce(&args),
        "verify-compact" => verify_compact::run(&args),
        _ => final_figures::run(&args),
    };
    total.finish();
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("tax-paired-figures: {error}");
            std::process::exit(1);
        }
    }
}

// Python compares JSON scalar booleans and numbers by numeric value in deltas.
fn py_equal(a: &Value, b: &Value) -> bool {
    fn number(a: &serde_json::Number, b: &serde_json::Number) -> bool {
        let integer = |n: &serde_json::Number| {
            n.as_i64().map(i128::from).or_else(|| n.as_u64().map(i128::from))
        };
        match (integer(a), integer(b)) {
            (Some(x), Some(y)) => x == y,
            (Some(x), None) => b.as_f64().is_some_and(|y| {
                y.is_finite()
                    && y.fract() == 0.0
                    && y >= i128::MIN as f64
                    && y < i128::MAX as f64
                    && x == y as i128
            }),
            (None, Some(_)) => number(b, a),
            (None, None) => a.as_f64() == b.as_f64(),
        }
    }
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => number(x, y),
        (Value::Bool(x), Value::Number(y)) => number(&serde_json::Number::from(u8::from(*x)), y),
        (Value::Number(_), Value::Bool(_)) => py_equal(b, a),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| py_equal(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, a)| y.get(k).is_some_and(|b| py_equal(a, b)))
        }
        _ => a == b,
    }
}
