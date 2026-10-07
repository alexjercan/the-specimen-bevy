use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{FrameStats, ProbeError};

pub const CSV_HEADER: &str = "label,frames,warmup,mean_ms,p50_ms,p95_ms,p99_ms,min_ms,max_ms,mean_fps,width,height,physical_width,physical_height,scale_factor,present_mode,adapter,backend,profile";

#[derive(Clone, Debug, PartialEq)]
pub struct ProbeReport {
    pub label: String,
    pub stats: FrameStats,
    pub warmup: u32,
    pub width: u32,
    pub height: u32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f32,
    pub present_mode: String,
    pub adapter: String,
    pub backend: String,
    pub profile: String,
}

impl ProbeReport {
    pub fn summary_line(&self) -> String {
        let stats = &self.stats;
        format!(
            "probe: label={} frames={} warmup={} mean_ms={:.3} p50_ms={:.3} p95_ms={:.3} p99_ms={:.3} min_ms={:.3} max_ms={:.3} mean_fps={:.1} res={}x{} physical={}x{} present={} adapter={:?} backend={} profile={}",
            self.label,
            stats.frames,
            self.warmup,
            stats.mean_ms,
            stats.p50_ms,
            stats.p95_ms,
            stats.p99_ms,
            stats.min_ms,
            stats.max_ms,
            stats.mean_fps,
            self.width,
            self.height,
            self.physical_width,
            self.physical_height,
            self.present_mode,
            self.adapter,
            self.backend,
            self.profile,
        )
    }

    pub fn csv_row(&self) -> String {
        let stats = &self.stats;
        [
            csv_cell(&self.label),
            stats.frames.to_string(),
            self.warmup.to_string(),
            format!("{:.4}", stats.mean_ms),
            format!("{:.4}", stats.p50_ms),
            format!("{:.4}", stats.p95_ms),
            format!("{:.4}", stats.p99_ms),
            format!("{:.4}", stats.min_ms),
            format!("{:.4}", stats.max_ms),
            format!("{:.2}", stats.mean_fps),
            self.width.to_string(),
            self.height.to_string(),
            self.physical_width.to_string(),
            self.physical_height.to_string(),
            self.scale_factor.to_string(),
            csv_cell(&self.present_mode),
            csv_cell(&self.adapter),
            csv_cell(&self.backend),
            csv_cell(&self.profile),
        ]
        .join(",")
    }

    pub fn to_json(&self) -> String {
        let stats = &self.stats;
        format!(
            "{{\n  \"label\": {},\n  \"frames\": {},\n  \"warmup\": {},\n  \"mean_ms\": {:.4},\n  \"p50_ms\": {:.4},\n  \"p95_ms\": {:.4},\n  \"p99_ms\": {:.4},\n  \"min_ms\": {:.4},\n  \"max_ms\": {:.4},\n  \"mean_fps\": {:.2},\n  \"width\": {},\n  \"height\": {},\n  \"physical_width\": {},\n  \"physical_height\": {},\n  \"scale_factor\": {},\n  \"present_mode\": {},\n  \"adapter\": {},\n  \"backend\": {},\n  \"profile\": {}\n}}\n",
            json_string(&self.label),
            stats.frames,
            self.warmup,
            stats.mean_ms,
            stats.p50_ms,
            stats.p95_ms,
            stats.p99_ms,
            stats.min_ms,
            stats.max_ms,
            stats.mean_fps,
            self.width,
            self.height,
            self.physical_width,
            self.physical_height,
            self.scale_factor,
            json_string(&self.present_mode),
            json_string(&self.adapter),
            json_string(&self.backend),
            json_string(&self.profile),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Csv(PathBuf),
    Json(PathBuf),
}

impl Output {
    pub fn parse(value: &str) -> Result<Self, ProbeError> {
        Self::from_path(value)
    }

    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, ProbeError> {
        let path = path.into();
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("csv") => Ok(Self::Csv(path)),
            Some("json") => Ok(Self::Json(path)),
            _ => Err(ProbeError::UnsupportedOutput(path)),
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Csv(path) | Self::Json(path) => path,
        }
    }

    pub fn write(&self, report: &ProbeReport) -> Result<(), ProbeError> {
        let path = self.path();
        let io = |error: std::io::Error| ProbeError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        };
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(io)?;
        }
        let contents = match self {
            Self::Json(_) => report.to_json(),
            Self::Csv(_) => {
                let mut contents = match fs::read_to_string(path) {
                    Ok(contents) => contents,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                    Err(error) => return Err(io(error)),
                };
                if contents.is_empty() {
                    contents.push_str(CSV_HEADER);
                    contents.push('\n');
                } else {
                    if contents.lines().next() != Some(CSV_HEADER) {
                        return Err(ProbeError::CsvHeaderMismatch(path.to_path_buf()));
                    }
                    if !contents.ends_with('\n') {
                        contents.push('\n');
                    }
                }
                contents.push_str(&report.csv_row());
                contents.push('\n');
                contents
            }
        };
        let mut temporary = path.as_os_str().to_owned();
        temporary.push(".tmp");
        let temporary = PathBuf::from(temporary);
        fs::write(&temporary, contents)
            .and_then(|()| fs::rename(&temporary, path))
            .map_err(|error| {
                let _ = fs::remove_file(&temporary);
                io(error)
            })
    }
}

fn csv_cell(value: &str) -> String {
    value
        .chars()
        .map(|char| match char {
            ',' | '"' => ';',
            char if char.is_control() => ' ',
            char => char,
        })
        .collect()
}

fn json_string(value: &str) -> String {
    let mut text = String::with_capacity(value.len() + 2);
    text.push('"');
    for char in value.chars() {
        match char {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            char if char.is_control() => text.push_str(&format!("\\u{:04x}", char as u32)),
            char => text.push(char),
        }
    }
    text.push('"');
    text
}
