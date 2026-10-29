use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use autopilot::AutopilotStatus;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use game::prelude::{AppBuilder, FrameStep, Frontend, VideoProfile};

mod credits;
mod escape_run;

use credits::rendered_credits_layout;
use escape_run::escape_run;
use gameplay::facility::Escaped;

const SEED: u64 = 1;
const MAX_FRAMES: u32 = 20_000;
// NOTE: generous on purpose. Tighten it when a real regression sets a number.
const FRAME_BUDGET_P99: Duration = Duration::from_millis(50);
// NOTE: time for the run itself on top of the profile encode deadline.
const RECORD_MARGIN: Duration = Duration::from_secs(180);

fn run_headless_escape(app: &mut App) -> Result<Vec<Duration>, String> {
    app.finish();
    app.cleanup();
    let mut frame_times = Vec::new();
    while *app.world().resource::<AutopilotStatus>() == AutopilotStatus::Running {
        let start = Instant::now();
        app.update();
        frame_times.push(start.elapsed());
        if frame_times.len() > MAX_FRAMES as usize {
            return Err(format!("autopilot still running after {MAX_FRAMES} frames"));
        }
    }
    match app.world().resource::<AutopilotStatus>() {
        AutopilotStatus::Success => Ok(frame_times),
        AutopilotStatus::Failed(reason) => Err(reason.clone()),
        AutopilotStatus::Running => unreachable!(),
    }
}

fn headless_escape_run() -> Result<(), String> {
    let mut app = AppBuilder::new(Frontend::Headless).with_seed(SEED).build();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        FrameStep::DEFAULT.duration(),
    ))
    .add_plugins(escape_run(SEED));
    run_headless_escape(&mut app)?;
    let escaped = app
        .world_mut()
        .query_filtered::<Entity, With<Escaped>>()
        .iter(app.world())
        .count();
    if escaped != 1 {
        return Err(format!("expected one escaped player, found {escaped}"));
    }
    Ok(())
}

fn percentile(sorted: &[Duration], percent: f64) -> Duration {
    let last = sorted.len().saturating_sub(1);
    let rank = ((percent / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(last)]
}

fn headless_frame_budget() -> Result<(), String> {
    let mut app = AppBuilder::new(Frontend::Headless).with_seed(SEED).build();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        FrameStep::DEFAULT.duration(),
    ))
    .add_plugins(escape_run(SEED));
    let mut frame_times = run_headless_escape(&mut app)?;
    if frame_times.is_empty() {
        return Err("the run produced no frames".into());
    }
    frame_times.sort_unstable();
    let p50 = percentile(&frame_times, 50.0);
    let p99 = percentile(&frame_times, 99.0);
    println!(
        "e2e headless_frame_budget: {} frames, p50 {:.2} ms, p99 {:.2} ms, budget {} ms",
        frame_times.len(),
        p50.as_secs_f64() * 1000.0,
        p99.as_secs_f64() * 1000.0,
        FRAME_BUDGET_P99.as_millis()
    );
    if p99 > FRAME_BUDGET_P99 {
        return Err(format!(
            "p99 frame time {:.2} ms exceeds the {} ms budget",
            p99.as_secs_f64() * 1000.0,
            FRAME_BUDGET_P99.as_millis()
        ));
    }
    Ok(())
}

fn rendered_escape_run() -> Result<(), String> {
    let mut app = AppBuilder::new(Frontend::Windowed { menu: false })
        .with_seed(SEED)
        .build();
    app.add_plugins(escape_run(SEED));
    match app.run() {
        AppExit::Success => Ok(()),
        AppExit::Error(code) => Err(format!("rendered run exited with code {code}")),
    }
}

fn record_run(profile: &str, deadline: Duration, video: &Path) -> Result<(), String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_the-specimen-bevy"))
        .args(["--stdio", "--seed", "1", "--record"])
        .arg(video)
        .args(["--video-profile", profile])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("spawn game: {error}"))?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let mut replies = BufReader::new(child.stdout.take().expect("piped stdout")).lines();
    let mut reply = |expected: u64| -> Result<(), String> {
        let line = replies
            .next()
            .ok_or("game closed stdout")?
            .map_err(|error| format!("read reply: {error}"))?;
        let needle = format!("\"tick\":{expected}");
        if line.contains(&needle) {
            Ok(())
        } else {
            Err(format!("expected tick {expected}, got {line}"))
        }
    };
    reply(0)?;
    for target in [30, 60, 90] {
        writeln!(
            stdin,
            "{{\"version\":1,\"tick\":{target},\"input\":{{\"w\":true}}}}"
        )
        .map_err(|error| format!("write command: {error}"))?;
        reply(target)?;
    }
    drop(stdin);
    let deadline = Instant::now() + deadline + RECORD_MARGIN;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                return Err("game did not finalize the recording in time".into());
            }
            Err(error) => return Err(format!("wait for game: {error}")),
        }
    };
    if !status.success() {
        return Err(format!("game exited with {status}"));
    }
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name",
            "-of",
            "csv=p=0",
        ])
        .arg(video)
        .output()
        .map_err(|error| format!("ffprobe unavailable: {error}"))?;
    let streams = String::from_utf8_lossy(&probe.stdout);
    if !streams.contains("vp9,video") || !streams.contains("opus,audio") {
        return Err(format!(
            "{profile} recording streams: {}",
            streams.trim().replace('\n', " ")
        ));
    }
    Ok(())
}

fn rendered_record_run() -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!(
        "specimen-e2e-record-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).map_err(|error| format!("create {}: {error}", dir.display()))?;
    let result = [
        ("preview", VideoProfile::PREVIEW),
        ("final", VideoProfile::FINAL),
    ]
    .iter()
    .try_for_each(|(name, profile)| {
        record_run(name, profile.deadline, &dir.join(format!("{name}.webm")))
    });
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn main() -> ExitCode {
    let switch = |name: &str| std::env::args().any(|argument| argument == name);
    let scenarios: &[(&str, fn() -> Result<(), String>)] = if switch("--record") {
        &[("rendered_record_run", rendered_record_run)]
    } else if switch("--rendered") {
        &[("rendered_escape_run", rendered_escape_run)]
    } else if switch("--credits") {
        &[("rendered_credits_layout", rendered_credits_layout)]
    } else {
        &[
            ("headless_escape_run", headless_escape_run),
            ("headless_frame_budget", headless_frame_budget),
        ]
    };
    let mut failed = 0;
    for (name, scenario) in scenarios {
        match scenario() {
            Ok(()) => println!("e2e {name}: ok"),
            Err(reason) => {
                failed += 1;
                println!("e2e {name}: FAILED: {reason}");
            }
        }
    }
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
