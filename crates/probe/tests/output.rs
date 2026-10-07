use std::path::PathBuf;

use probe::{FrameStats, Output, ProbeError, ProbeReport, CSV_HEADER};

fn report(label: &str) -> ProbeReport {
    ProbeReport {
        label: label.to_string(),
        stats: FrameStats::from_samples(&[8.0, 10.0, 12.0]).unwrap(),
        warmup: 5,
        width: 1280,
        height: 720,
        physical_width: 1280,
        physical_height: 720,
        scale_factor: 1.0,
        present_mode: "autonovsync".to_string(),
        adapter: "Fake, \"GPU\"".to_string(),
        backend: "vulkan".to_string(),
        profile: "debug".to_string(),
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("probe-output-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn extension_selects_format() {
    assert_eq!(
        Output::parse("a/run.csv"),
        Ok(Output::Csv("a/run.csv".into()))
    );
    assert_eq!(
        Output::parse("run.json"),
        Ok(Output::Json("run.json".into()))
    );
    assert_eq!(
        Output::parse("run.txt"),
        Err(ProbeError::UnsupportedOutput("run.txt".into()))
    );
    assert!(Output::parse("run").is_err());
}

#[test]
fn csv_row_matches_header_width() {
    let row = report("a").csv_row();
    assert_eq!(row.split(',').count(), CSV_HEADER.split(',').count());
    assert!(row.contains("Fake; ;GPU;"));
}

#[test]
fn csv_creates_header_then_appends() {
    let dir = scratch("csv");
    let path = dir.join("nested/runs.csv");
    let output = Output::Csv(path.clone());
    output.write(&report("before")).unwrap();
    output.write(&report("after")).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], CSV_HEADER);
    assert!(lines[1]
        .starts_with("before,3,5,10.0000,10.0000,12.0000,12.0000,8.0000,12.0000,100.00,1280,720"));
    assert!(lines[2].starts_with("after,"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn csv_refuses_a_foreign_header() {
    let dir = scratch("foreign");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("runs.csv");
    std::fs::write(&path, "label,fps\nold,60\n").unwrap();
    assert_eq!(
        Output::Csv(path.clone()).write(&report("a")),
        Err(ProbeError::CsvHeaderMismatch(path.clone()))
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "label,fps\nold,60\n"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn json_overwrites_one_escaped_object() {
    let dir = scratch("json");
    let path = dir.join("run.json");
    let output = Output::Json(path.clone());
    output.write(&report("first")).unwrap();
    output.write(&report("second")).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with("{\n  \"label\": \"second\",\n"));
    assert!(text.contains("\"adapter\": \"Fake, \\\"GPU\\\"\""));
    assert!(text.contains("\"p95_ms\": 12.0000"));
    assert!(!text.contains("first"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_write_leaves_no_partial_file() {
    let dir = scratch("blocked");
    let path = dir.join("run.json");
    std::fs::create_dir_all(&path).unwrap();
    assert!(matches!(
        Output::Json(path.clone()).write(&report("a")),
        Err(ProbeError::Io { .. })
    ));
    assert!(path.is_dir());
    assert!(!dir.join("run.json.tmp").exists());
    std::fs::remove_dir_all(dir).unwrap();
}
