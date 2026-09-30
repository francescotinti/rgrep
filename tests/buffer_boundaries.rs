//! Observable CLI regressions for streamed records crossing internal buffers.
use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], input: Vec<u8>) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rgrep"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || {
        if let Err(error) = stdin.write_all(&input) {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    });
    let result = child.wait_with_output().unwrap();
    writer.join().unwrap();
    result
}

#[test]
fn long_lines_preserve_offsets_crlf_and_final_record() {
    let long = format!("{}needle", "x".repeat(20_003));
    let input = format!("skip\r\n{long}\r\nneedle");
    let output = run(&["-anbF", "needle"], input.into_bytes());
    let expected = format!("2:6:{long}\n3:{}:needle\n", 6 + long.len() + 2);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn nul_records_preserve_count_and_only_matching() {
    let input = format!("{}needle\0skip\0needle", "x".repeat(20_003)).into_bytes();
    for (args, expected) in [
        (vec!["-zcF", "needle"], b"2\n".as_slice()),
        (vec!["-zoF", "needle"], b"needle\0needle\0".as_slice()),
    ] {
        let output = run(&args, input.clone());
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn quiet_and_count_handle_empty_and_unterminated_input() {
    for (args, input, code, expected) in [
        (vec!["-cF", "needle"], b"".as_slice(), 1, b"0\n".as_slice()),
        (vec!["-acF", "needle"], b"\xffneedle", 0, b"1\n"),
        (vec!["-qF", "needle"], b"needle\nrest", 0, b""),
        (vec!["-qF", "needle"], b"absent", 1, b""),
    ] {
        let output = run(&args, input.to_vec());
        assert_eq!(output.status.code(), Some(code));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}
