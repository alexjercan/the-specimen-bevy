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
    assert!(String::from_utf8_lossy(&output.stdout).contains("--seed <SEED>"));
}

#[test]
fn non_numeric_seed_is_rejected() {
    let output = game(&["--norender", "--transport", "--seed", "abc"], "");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn headless_transport_runs_with_a_seed() {
    let output = game(
        &[
            "--norender",
            "--transport",
            "--seed",
            "18446744073709551615",
        ],
        "{\"tick\":1,\"input\":{}}\n",
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"tick\":1"));
}
