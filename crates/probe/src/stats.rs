use crate::ProbeError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameStats {
    pub frames: usize,
    pub mean_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub mean_fps: f64,
}

impl FrameStats {
    pub fn from_samples(samples: &[f64]) -> Result<Self, ProbeError> {
        if samples.is_empty() {
            return Err(ProbeError::NoFrames);
        }
        if let Some((index, &value)) = samples
            .iter()
            .enumerate()
            .find(|(_, value)| !value.is_finite() || **value <= 0.0)
        {
            return Err(ProbeError::InvalidSample { index, value });
        }
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        let mean_ms = sorted.iter().sum::<f64>() / sorted.len() as f64;
        Ok(Self {
            frames: sorted.len(),
            mean_ms,
            min_ms: sorted[0],
            max_ms: sorted[sorted.len() - 1],
            p50_ms: nearest_rank(&sorted, 50.0),
            p95_ms: nearest_rank(&sorted, 95.0),
            p99_ms: nearest_rank(&sorted, 99.0),
            mean_fps: 1000.0 / mean_ms,
        })
    }
}

fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    let rank = (percentile / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}
