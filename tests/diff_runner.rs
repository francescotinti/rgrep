use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct Testsuite {
    cases: Vec<String>,
}

#[derive(Deserialize)]
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
    #[serde(default)]
    expected_to_fail: bool,
}

#[derive(Debug, PartialEq)]
enum DiffOutcome {
    Match,
    Differ { rgrep: String, oracle: String },
    BothFail { rgrep_code: i32, oracle_code: i32 },
    OnlyRgrepOk { oracle_code: i32 },
    OnlyOracleOk { rgrep_code: i32 },
}

fn is_bsd_grep() -> bool {
    let output = Command::new("grep")
        .arg("--version")
        .output()
        .expect("Failed to execute grep --version");
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    let version_err = String::from_utf8_lossy(&output.stderr);
    
    !version_str.contains("GNU") && !version_err.contains("GNU")
}

fn run_command_with_stdin(cmd: &str, args: &[String], stdin_data: &str) -> (i32, String, String) {
    let mut command = Command::new(cmd);
    for arg in args {
        command.arg(arg);
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().unwrap_or_else(|_| panic!("Failed to spawn {}", cmd));

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_data.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    let code = output.status.code().unwrap_or(1);
    let stdout_str = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();
    
    (code, stdout_str, stderr_str)
}

#[test]
fn test_differential() {
    let manifest_path = Path::new("tests/testsuite.toml");
    let manifest_content = fs::read_to_string(manifest_path).expect("Failed to read testsuite.toml");
    let testsuite: Testsuite = toml::from_str(&manifest_content).expect("Failed to parse testsuite.toml");

    let is_bsd = is_bsd_grep();
    
    let rgrep_bin = env!("CARGO_BIN_EXE_rgrep");

    for case_path in testsuite.cases {
        let case_content = fs::read_to_string(Path::new("tests").join(&case_path))
            .unwrap_or_else(|_| panic!("Failed to read {}", case_path));
        let case: TestCase = toml::from_str(&case_content)
            .unwrap_or_else(|_| panic!("Failed to parse {}", case_path));

        if case.skip_if_bsd && is_bsd {
            eprintln!("[skip] {} (bsd grep) - {}", case.name, case.skip_reason);
            continue;
        }

        let (oracle_code, oracle_stdout, _oracle_stderr) = run_command_with_stdin("grep", &case.args, &case.stdin);
        
        let (rgrep_code, rgrep_stdout, rgrep_stderr) = run_command_with_stdin(rgrep_bin, &case.args, &case.stdin);

        let outcome = if rgrep_code == 0 && oracle_code == 0 {
            if rgrep_stdout == oracle_stdout {
                DiffOutcome::Match
            } else {
                DiffOutcome::Differ { rgrep: rgrep_stdout.clone(), oracle: oracle_stdout.clone() }
            }
        } else if rgrep_code != 0 && oracle_code != 0 {
            if rgrep_code == oracle_code {
                DiffOutcome::Match
            } else {
                DiffOutcome::BothFail { rgrep_code, oracle_code }
            }
        } else if rgrep_code == 0 && oracle_code != 0 {
            DiffOutcome::OnlyRgrepOk { oracle_code }
        } else {
            DiffOutcome::OnlyOracleOk { rgrep_code }
        };

        if case.expected_to_fail {
            continue; // Se è marcato come expected_to_fail, saltiamo l'assert
        }

        if outcome != DiffOutcome::Match {
            panic!(
                "Test '{}' failed! Outcome: {:?}\nArgs: {:?}\nExpected Code: {}\nOracle Code: {}\nRgrep Code: {}\nOracle Stdout:\n{}\nRgrep Stdout:\n{}\nRgrep Stderr:\n{}",
                case.name, outcome, case.args, case.expected_exit_code, oracle_code, rgrep_code, oracle_stdout, rgrep_stdout, rgrep_stderr
            );
        }
        
        assert_eq!(oracle_code, case.expected_exit_code, "Oracle grep produced unexpected exit code");
        assert_eq!(oracle_stdout, case.expected_stdout, "Oracle grep produced unexpected output");
    }
}
