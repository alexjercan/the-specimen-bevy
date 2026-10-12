use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn game(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_the-specimen-bevy"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn help_lists_the_seed_flag() {
    let output = game(&["--help"], "");
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--seed <SEED>"));
    assert!(help.contains("--video-profile <VIDEO_PROFILE>"));
    assert!(help.contains("preview"));
    assert!(help.contains("final"));
}

#[test]
fn non_numeric_seed_is_rejected() {
    let output = game(&["--headless", "--stdio", "--seed", "abc"], "");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn headless_stdio_runs_with_a_seed() {
    let output = game(
        &["--headless", "--stdio", "--seed", "18446744073709551615"],
        "{\"version\":1,\"tick\":1,\"input\":{}}\n",
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"version\":1,\"tick\":1"));
}

#[test]
fn headless_without_stdio_exits_after_the_bounded_run() {
    let output = game(&["--headless", "--seed", "1"], "");
    assert!(output.status.success());
}

#[test]
fn video_profile_needs_record() {
    let output = game(&["--stdio", "--video-profile", "final"], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--record"));
}

#[test]
fn record_needs_a_windowed_stdio_run() {
    let output = game(&["--headless", "--record", "run.webm"], "");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--record requires --stdio"));
}
