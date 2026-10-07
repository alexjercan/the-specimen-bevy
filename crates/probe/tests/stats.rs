use probe::{FrameStats, ProbeError};

#[test]
fn uniform_window_is_exact() {
    let stats = FrameStats::from_samples(&[10.0; 10]).unwrap();
    assert_eq!(stats.frames, 10);
    assert_eq!(stats.mean_ms, 10.0);
    assert_eq!(stats.p50_ms, 10.0);
    assert_eq!(stats.p99_ms, 10.0);
    assert_eq!(stats.mean_fps, 100.0);
}

#[test]
fn percentiles_use_nearest_rank() {
    let samples: Vec<f64> = (1..=100).map(f64::from).rev().collect();
    let stats = FrameStats::from_samples(&samples).unwrap();
    assert_eq!(stats.min_ms, 1.0);
    assert_eq!(stats.max_ms, 100.0);
    assert_eq!(stats.p50_ms, 50.0);
    assert_eq!(stats.p95_ms, 95.0);
    assert_eq!(stats.p99_ms, 99.0);
    assert_eq!(stats.mean_ms, 50.5);
}

#[test]
fn single_frame_is_every_percentile() {
    let stats = FrameStats::from_samples(&[4.0]).unwrap();
    assert_eq!((stats.p50_ms, stats.p95_ms, stats.p99_ms), (4.0, 4.0, 4.0));
}

#[test]
fn empty_window_fails() {
    assert_eq!(FrameStats::from_samples(&[]), Err(ProbeError::NoFrames));
}

#[test]
fn invalid_samples_fail() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            FrameStats::from_samples(&[5.0, value]),
            Err(ProbeError::InvalidSample { index: 1, .. })
        ));
    }
}
