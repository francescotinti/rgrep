// rgrep — manifest-driven differential verification against GNU/BSD grep.
// Copyright (c) 2026 Francesco Tinti <francesco.tinti@activemind.it>

mod common;

use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

#[derive(Deserialize)]
struct Testsuite {
    cases: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureFile {
    name: String,
    #[serde(default)]
    content: String,
    symlink_to: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TestCase {
    name: String,
    args: Vec<String>,
    stdin: String,
    expected_stdout: String,
    expected_exit_code: i32,
    #[serde(default)]
    skip_if_bsd: bool,
    #[serde(default)]
    skip_reason: String,
    requires_feature: Option<String>,
    forbids_feature: Option<String>,
    #[serde(default)]
    rgrep_only: bool,
    oracle_args: Option<Vec<String>>,
    gnu_difference: Option<String>,
    gnu_stdout: Option<String>,
    gnu_stderr_contains: Option<String>,
    #[serde(default)]
    fixture_files: Vec<FixtureFile>,
    env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    sort_output: bool,
    #[serde(rename = "source")]
    _source: Option<String>,
}

#[derive(Debug, PartialEq)]
enum DiffOutcome {
    Match,
    Differ { rgrep: Vec<u8>, oracle: Vec<u8> },
    BothFail { rgrep_code: i32, oracle_code: i32 },
    OnlyRgrepOk { oracle_code: i32 },
    OnlyOracleOk { rgrep_code: i32 },
}

fn normalize(bytes: &[u8], sort: bool) -> Vec<u8> {
    if !sort {
        return bytes.to_vec();
    }
    let mut lines: Vec<_> = bytes.split_inclusive(|&byte| byte == b'\n').collect();
    lines.sort_unstable();
    lines.concat()
}

fn outcome(rgrep: &Output, oracle: &Output, sort: bool) -> DiffOutcome {
    let rgrep_code = rgrep.status.code().unwrap_or(-1);
    let oracle_code = oracle.status.code().unwrap_or(-1);
    if rgrep_code == oracle_code {
        let rgrep = normalize(&rgrep.stdout, sort);
        let oracle = normalize(&oracle.stdout, sort);
        if rgrep == oracle {
            DiffOutcome::Match
        } else {
            DiffOutcome::Differ { rgrep, oracle }
        }
    } else if rgrep_code == 0 {
        DiffOutcome::OnlyRgrepOk { oracle_code }
    } else if oracle_code == 0 {
        DiffOutcome::OnlyOracleOk { rgrep_code }
    } else {
        DiffOutcome::BothFail {
            rgrep_code,
            oracle_code,
        }
    }
}

#[test]
fn nonzero_exit_still_requires_byte_exact_stdout() {
    use std::os::unix::process::ExitStatusExt;
    let make = |stdout| Output {
        status: std::process::ExitStatus::from_raw(1 << 8),
        stdout,
        stderr: Vec::new(),
    };
    let actual = make(vec![0xff, 0]);
    let reference = make(vec![0xfe, 0]);
    assert!(matches!(
        outcome(&actual, &reference, false),
        DiffOutcome::Differ { .. }
    ));
    assert_eq!(outcome(&actual, &actual, false), DiffOutcome::Match);
}

fn run(cmd: &str, args: &[String], case: &TestCase) -> Output {
    let mut command = Command::new(cmd);
    command.args(args).env("LC_ALL", "C");
    if let Some(env) = &case.env {
        command.envs(env);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("cannot spawn {cmd}: {error}"));
    let input = case.stdin.clone();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        if let Err(error) = stdin.write_all(input.as_bytes()) {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    });
    let result = child.wait_with_output().unwrap();
    writer.join().unwrap();
    result
}

#[test]
fn test_differential() {
    let suite: Testsuite =
        toml::from_str(&fs::read_to_string("tests/testsuite.toml").unwrap()).unwrap();
    let oracle = common::oracle();
    let mut failures = Vec::new();
    let (mut passed, mut local, mut skipped, mut known) = (0, 0, 0, 0);
    let total = suite.cases.len();
    for case_path in suite.cases {
        let case: TestCase =
            toml::from_str(&fs::read_to_string(Path::new("tests").join(&case_path)).unwrap())
                .unwrap_or_else(|error| panic!("invalid manifest case {case_path}: {error}"));
        let skip = if case.skip_if_bsd && oracle.is_bsd && !case.rgrep_only {
            Some(case.skip_reason.as_str())
        } else if case.requires_feature.as_deref() == Some("perl-regexp")
            && !cfg!(feature = "perl-regexp")
        {
            Some("requires perl-regexp feature")
        } else if case.forbids_feature.as_deref() == Some("perl-regexp")
            && cfg!(feature = "perl-regexp")
        {
            Some("tests a build without perl-regexp")
        } else {
            None
        };
        if let Some(reason) = skip {
            eprintln!("[skip] {}: {reason}", case.name);
            skipped += 1;
            continue;
        }

        let fixture_dir = std::env::temp_dir().join(format!(
            "rgrep-diff-{}-{}",
            std::process::id(),
            Path::new(&case_path).file_stem().unwrap().to_string_lossy()
        ));
        for file in &case.fixture_files {
            let path = fixture_dir.join(&file.name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            if let Some(target) = &file.symlink_to {
                std::os::unix::fs::symlink(target, &path).unwrap();
            } else {
                fs::write(&path, &file.content).unwrap();
            }
        }
        let expand = |value: &str| value.replace("{FIXTURES}", &fixture_dir.to_string_lossy());
        let args: Vec<_> = case.args.iter().map(|s| expand(s)).collect();
        let oracle_args: Vec<_> = case
            .oracle_args
            .as_ref()
            .unwrap_or(&case.args)
            .iter()
            .map(|s| expand(s))
            .collect();
        let actual = run(env!("CARGO_BIN_EXE_rgrep"), &args, &case);
        let reference = (!case.rgrep_only).then(|| run(&oracle.executable, &oracle_args, &case));
        if !case.fixture_files.is_empty() {
            fs::remove_dir_all(&fixture_dir).unwrap();
        }
        let expected = expand(&case.expected_stdout);
        if actual.status.code() != Some(case.expected_exit_code)
            || normalize(&actual.stdout, case.sort_output)
                != normalize(expected.as_bytes(), case.sort_output)
        {
            failures.push(format!(
                "{} ({case_path}): rgrep differs from fixture: {actual:?}",
                case.name
            ));
            continue;
        }
        let Some(reference) = reference else {
            local += 1;
            eprintln!("[rgrep-only] {}", case.name);
            continue;
        };
        if !oracle.is_bsd
            && let Some(reason) = &case.gnu_difference
        {
            let expected_gnu = expand(case.gnu_stdout.as_ref().expect("GNU output required"));
            let stderr_ok = case.gnu_stderr_contains.as_ref().is_none_or(|needle| {
                let needle = expand(needle);
                reference
                    .stderr
                    .windows(needle.len())
                    .any(|part| part == needle.as_bytes())
            });
            if reference.status.code() != Some(case.expected_exit_code)
                || reference.stdout != expected_gnu.as_bytes()
                || !stderr_ok
                || outcome(&actual, &reference, case.sort_output) == DiffOutcome::Match
                || std::env::var_os("RGREP_STRICT_GNU").is_some()
            {
                failures.push(format!(
                    "{}: GNU divergence changed, resolved, or rejected in strict mode: {reason}; {reference:?}",
                    case.name
                ));
            } else {
                known += 1;
                eprintln!("[known-gnu-divergence] {}: {reason}", case.name);
            }
            continue;
        }
        let result = outcome(&actual, &reference, case.sort_output);
        if result == DiffOutcome::Match {
            passed += 1;
            eprintln!("[pass] {}", case.name);
        } else {
            failures.push(format!(
                "{} ({case_path}): {result:?}; oracle={reference:?}",
                case.name
            ));
        }
    }
    eprintln!(
        "[summary] total={total} parity={passed} rgrep_only={local} skipped={skipped} known_gnu_divergences={known} failed={}",
        failures.len()
    );
    assert_eq!(total, passed + local + skipped + known + failures.len());
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
