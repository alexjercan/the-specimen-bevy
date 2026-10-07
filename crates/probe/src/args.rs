use bevy::{math::UVec2, window::PresentMode};
use clap::Args;

use crate::{
    parse_label, parse_present_mode, parse_resolution, Output, ProbeConfig, DEFAULT_FRAMES,
    DEFAULT_LABEL, DEFAULT_PRESENT_MODE, DEFAULT_RESOLUTION, DEFAULT_WARMUP,
};

#[derive(Args, Clone, Debug)]
pub struct ProbeArgs {
    #[arg(long)]
    pub probe: bool,
    #[arg(long = "probe-warmup", requires = "probe", default_value_t = DEFAULT_WARMUP)]
    pub warmup: u32,
    #[arg(
        long = "probe-frames",
        requires = "probe",
        default_value_t = DEFAULT_FRAMES,
        value_parser = clap::value_parser!(u32).range(1..)
    )]
    pub frames: u32,
    #[arg(
        long = "probe-res",
        requires = "probe",
        default_value = DEFAULT_RESOLUTION,
        value_parser = parse_resolution
    )]
    pub resolution: UVec2,
    #[arg(
        long = "probe-present",
        requires = "probe",
        default_value = DEFAULT_PRESENT_MODE,
        value_parser = parse_present_mode
    )]
    pub present_mode: PresentMode,
    #[arg(
        long = "probe-label",
        requires = "probe",
        default_value = DEFAULT_LABEL,
        value_parser = parse_label
    )]
    pub label: String,
    #[arg(long = "probe-out", requires = "probe", value_parser = Output::parse)]
    pub out: Option<Output>,
}

impl ProbeArgs {
    pub fn config(&self) -> Option<ProbeConfig> {
        self.probe.then(|| ProbeConfig {
            label: self.label.clone(),
            warmup: self.warmup,
            frames: self.frames,
            resolution: self.resolution,
            present_mode: self.present_mode,
            out: self.out.clone(),
            ..ProbeConfig::default()
        })
    }
}
