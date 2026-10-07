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
fn probe_uses_defaults_without_a_flag() {
    assert_eq!(parse(&[]).unwrap().config(), ProbeConfig::default());
}

#[test]
fn options_build_the_config_without_a_probe_flag() {
    let config = parse(&[
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
    .config();
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
fn redundant_probe_flag_is_rejected() {
    assert!(parse(&["--probe"]).is_err());
}

#[test]
fn invalid_values_are_rejected() {
    for args in [
        &["--probe-frames", "0"][..],
        &["--probe-frames", "-1"],
        &["--probe-res", "0x720"],
        &["--probe-res", "1280"],
        &["--probe-present", "vsync"],
        &["--probe-label", ""],
        &["--probe-label", "a,b"],
        &["--probe-out", "run.txt"],
    ] {
        assert!(parse(args).is_err(), "{args:?} parsed");
    }
}
