use bevy::{math::UVec2, window::PresentMode};
use clap::Parser;
use probe::{Output, ProbeArgs, ProbeConfig};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    probe: ProbeArgs,
}

fn parse(args: &[&str]) -> Result<ProbeArgs, clap::Error> {
    Cli::try_parse_from(std::iter::once("game").chain(args.iter().copied())).map(|cli| cli.probe)
}

#[test]
fn probe_is_off_by_default() {
    assert_eq!(parse(&[]).unwrap().config(), None);
}

#[test]
fn probe_flag_uses_defaults() {
    assert_eq!(
        parse(&["--probe"]).unwrap().config(),
        Some(ProbeConfig::default())
    );
}

#[test]
fn options_build_the_config() {
    let config = parse(&[
        "--probe",
        "--probe-warmup",
        "0",
        "--probe-frames",
        "60",
        "--probe-res",
        "800x600",
        "--probe-present",
        "fifo",
        "--probe-label",
        "lights-off",
        "--probe-out",
        "target/probe/runs.csv",
    ])
    .unwrap()
    .config()
    .unwrap();
    assert_eq!(config.warmup, 0);
    assert_eq!(config.frames, 60);
    assert_eq!(config.resolution, UVec2::new(800, 600));
    assert_eq!(config.present_mode, PresentMode::Fifo);
    assert_eq!(config.label, "lights-off");
    assert_eq!(
        config.out,
        Some(Output::Csv("target/probe/runs.csv".into()))
    );
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn options_require_the_probe_flag() {
    for args in [
        &["--probe-frames", "10"][..],
        &["--probe-warmup", "10"],
        &["--probe-out", "run.json"],
        &["--probe-res", "800x600"],
    ] {
        assert!(parse(args).is_err(), "{args:?} parsed without --probe");
    }
}

#[test]
fn invalid_values_are_rejected() {
    for args in [
        &["--probe", "--probe-frames", "0"][..],
        &["--probe", "--probe-frames", "-1"],
        &["--probe", "--probe-res", "0x720"],
        &["--probe", "--probe-res", "1280"],
        &["--probe", "--probe-present", "vsync"],
        &["--probe", "--probe-label", ""],
        &["--probe", "--probe-label", "a,b"],
        &["--probe", "--probe-out", "run.txt"],
    ] {
        assert!(parse(args).is_err(), "{args:?} parsed");
    }
}
