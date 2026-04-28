use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use regex::Regex;
use serde::Serialize;
use uuid::Uuid;

use opensim_next::scripting::executor::tree_walk::TreeWalkExecutor;
use opensim_next::scripting::executor::ScriptExecutor;

#[derive(Serialize)]
struct TestReport {
    timestamp: String,
    total_scripts: usize,
    parsed_ok: usize,
    parse_failed: usize,
    success_rate: String,
    duration_secs: f64,
    functions_used: HashMap<String, FunctionUsage>,
    failures: Vec<FailureEntry>,
    top_errors: Vec<(String, usize)>,
}

#[derive(Serialize)]
struct FunctionUsage {
    count: usize,
    scripts: Vec<String>,
}

#[derive(Serialize)]
struct FailureEntry {
    file: String,
    error: String,
    lines: usize,
}

fn find_lsl_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    fn walk(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("lsl") {
                    files.push(path);
                }
            }
        }
    }
    walk(dir, &mut files);
    files.sort();
    files
}

fn strip_source(source: &str) -> String {
    source.replace('\0', "").replace('\u{FEFF}', "")
}

fn extract_ll_functions(source: &str) -> Vec<String> {
    let re = Regex::new(r"\bll[A-Z][A-Za-z0-9]*").unwrap();
    let mut funcs: Vec<String> = re
        .find_iter(source)
        .map(|m| m.as_str().to_string())
        .collect();
    funcs.sort();
    funcs.dedup();
    funcs
}

fn main() -> Result<()> {
    let start = Instant::now();

    let args: Vec<String> = std::env::args().collect();
    let dir_arg = if args.len() > 1 {
        &args[1]
    } else {
        "content/LSL-Scripts-master"
    };
    let base_dir = Path::new(dir_arg);
    if !base_dir.exists() {
        eprintln!("ERROR: Script directory not found: {}", base_dir.display());
        eprintln!("Run from opensim-next/ root directory");
        std::process::exit(1);
    }

    println!("LSL Script Test Harness");
    println!("=======================");
    println!("Scanning: {}", base_dir.display());
    println!();

    let files = find_lsl_files(base_dir);
    let total = files.len();
    println!("Found {} .lsl files", total);

    let executor = TreeWalkExecutor::new();

    let mut parsed_ok = 0usize;
    let mut parse_failed = 0usize;
    let mut failures: Vec<FailureEntry> = Vec::new();
    let mut error_counts: HashMap<String, usize> = HashMap::new();
    let mut function_map: HashMap<String, FunctionUsage> = HashMap::new();

    for (i, path) in files.iter().enumerate() {
        if (i + 1) % 200 == 0 || i + 1 == total {
            eprint!(
                "\rProcessing: {}/{} ({:.1}%)",
                i + 1,
                total,
                (i + 1) as f64 / total as f64 * 100.0
            );
        }

        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                parse_failed += 1;
                let rel = path.strip_prefix(base_dir).unwrap_or(path);
                failures.push(FailureEntry {
                    file: rel.display().to_string(),
                    error: format!("Read error: {}", e),
                    lines: 0,
                });
                *error_counts
                    .entry(format!("Read error: {}", e))
                    .or_insert(0) += 1;
                continue;
            }
        };

        let cleaned = strip_source(&source);
        let line_count = cleaned.lines().count();
        let script_id = Uuid::new_v4();

        match executor.compile(&cleaned, script_id) {
            Ok(_compiled) => {
                parsed_ok += 1;

                let funcs = extract_ll_functions(&cleaned);
                let rel = path.strip_prefix(base_dir).unwrap_or(path);
                let script_name = rel.display().to_string();

                for func in funcs {
                    let entry = function_map.entry(func).or_insert_with(|| FunctionUsage {
                        count: 0,
                        scripts: Vec::new(),
                    });
                    entry.count += 1;
                    if entry.scripts.len() < 5 {
                        entry.scripts.push(script_name.clone());
                    }
                }
            }
            Err(e) => {
                parse_failed += 1;
                let rel = path.strip_prefix(base_dir).unwrap_or(path);
                let err_msg = format!("{}", e);
                let short_err = err_msg.lines().next().unwrap_or(&err_msg).to_string();
                let short_err = if short_err.len() > 120 {
                    format!("{}...", &short_err[..120])
                } else {
                    short_err
                };

                failures.push(FailureEntry {
                    file: rel.display().to_string(),
                    error: short_err.clone(),
                    lines: line_count,
                });
                *error_counts.entry(short_err).or_insert(0) += 1;
            }
        }
    }
    eprintln!();

    let duration = start.elapsed();
    let success_rate = if total > 0 {
        format!("{:.1}%", parsed_ok as f64 / total as f64 * 100.0)
    } else {
        "0.0%".to_string()
    };

    println!();
    println!("Results");
    println!("-------");
    println!("Scanned:   {} scripts", total);
    println!(
        "Parsed OK: {} ({:.1}%)",
        parsed_ok,
        parsed_ok as f64 / total.max(1) as f64 * 100.0
    );
    println!(
        "Failed:    {} ({:.1}%)",
        parse_failed,
        parse_failed as f64 / total.max(1) as f64 * 100.0
    );
    println!("Duration:  {:.2}s", duration.as_secs_f64());

    println!();
    println!("Top 15 Errors:");
    let mut sorted_errors: Vec<_> = error_counts.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(a.1));
    for (err, count) in sorted_errors.iter().take(15) {
        let truncated = if err.len() > 60 { &err[..60] } else { err };
        println!("  {:>4} — {}", count, truncated);
    }

    println!();
    println!("Top 25 ll* Functions:");
    let mut sorted_funcs: Vec<_> = function_map.iter().collect();
    sorted_funcs.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    for (name, usage) in sorted_funcs.iter().take(25) {
        println!("  {:<30} — {} scripts", name, usage.count);
    }

    let top_errors: Vec<(String, usize)> = sorted_errors
        .iter()
        .take(30)
        .map(|(k, v)| (k.to_string(), **v))
        .collect();

    let report = TestReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_scripts: total,
        parsed_ok,
        parse_failed,
        success_rate,
        duration_secs: duration.as_secs_f64(),
        functions_used: function_map,
        failures,
        top_errors,
    };

    let report_path = "content/lsl_test_report.json";
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(report_path, &json)?;
    println!();
    println!("Report saved to: {}", report_path);

    Ok(())
}
