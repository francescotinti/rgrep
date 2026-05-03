use proptest::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

fn is_bsd_grep() -> bool {
    let output = Command::new("grep")
        .arg("--version")
        .output()
        .expect("Failed to execute grep --version");
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    let version_err = String::from_utf8_lossy(&output.stderr);
    
    version_str.contains("BSD grep") || (!version_str.contains("GNU") && !version_err.contains("GNU"))
}

fn run_rgrep(args: &[&str], stdin: &[u8]) -> (i32, Vec<u8>) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rgrep"));
    cmd.args(args)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    let mut child = cmd.spawn().unwrap();
    if let Some(mut stdin_pipe) = child.stdin.take() {
        stdin_pipe.write_all(stdin).unwrap();
    }
    
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(2), out.stdout)
}

fn run_oracle(args: &[&str], stdin: &[u8]) -> (i32, Vec<u8>) {
    let mut cmd = Command::new("grep");
    cmd.args(args)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    let mut child = cmd.spawn().unwrap();
    if let Some(mut stdin_pipe) = child.stdin.take() {
        stdin_pipe.write_all(stdin).unwrap();
    }
    
    let out = child.wait_with_output().unwrap();
    (out.status.code().unwrap_or(2), out.stdout)
}

proptest! {
    // 1024 cases - expanded range!
    #![proptest_config(ProptestConfig::with_cases(1024))]
    
    #[test]
    fn prop_literal_match(
        pattern in "[a-zA-Z0-9]{1,8}",
        input in "[a-zA-Z0-9\n ]{0,200}",
    ) {
        if is_bsd_grep() { return Ok(()); }
        
        let rgrep_out = run_rgrep(&[&pattern], input.as_bytes());
        let oracle_out = run_oracle(&[&pattern], input.as_bytes());
        
        prop_assert_eq!(rgrep_out, oracle_out);
    }
    
    #[test]
    fn prop_anchored(
        anchor in prop_oneof![Just("^"), Just("$"), Just("^.*$")],
        body in "[a-zA-Z0-9]{0,5}",
        input in "[a-zA-Z0-9 \n]{0,150}",
    ) {
        if is_bsd_grep() { return Ok(()); }
        
        let pat = format!("{}{}", anchor, body);
        let rgrep_out = run_rgrep(&[&pat], input.as_bytes());
        let oracle_out = run_oracle(&[&pat], input.as_bytes());
        
        prop_assert_eq!(rgrep_out, oracle_out);
    }
    
    #[test]
    fn prop_char_class(
        class_body in "[a-zA-Z0-9]{1,5}",
        negate in any::<bool>(),
        input in "[a-zA-Z0-9 \n]{0,150}",
    ) {
        if is_bsd_grep() { return Ok(()); }
        
        let pat = if negate {
            format!("[^{}]", class_body)
        } else {
            format!("[{}]", class_body)
        };
        let rgrep_out = run_rgrep(&[&pat], input.as_bytes());
        let oracle_out = run_oracle(&[&pat], input.as_bytes());
        
        prop_assert_eq!(rgrep_out, oracle_out);
    }
}
