use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum ProbeError {
    NoFrames,
    InvalidSample { index: usize, value: f64 },
    ZeroFrames,
    ZeroTimeout,
    InvalidResolution(String),
    InvalidPresentMode(String),
    InvalidLabel(String),
    UnsupportedOutput(PathBuf),
    CsvHeaderMismatch(PathBuf),
    Io { path: PathBuf, message: String },
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFrames => write!(f, "no frames were captured"),
            Self::InvalidSample { index, value } => {
                write!(f, "frame {index} has an invalid frame time of {value} ms")
            }
            Self::ZeroFrames => write!(f, "the capture window needs at least one frame"),
            Self::ZeroTimeout => write!(f, "the readiness timeout must be longer than zero"),
            Self::InvalidResolution(value) => write!(
                f,
                "invalid resolution {value:?}; use WIDTHxHEIGHT with both sides above zero"
            ),
            Self::InvalidPresentMode(value) => write!(
                f,
                "invalid present mode {value:?}; use one of {}",
                crate::PRESENT_MODES.join(", ")
            ),
            Self::InvalidLabel(value) => write!(
                f,
                "invalid label {value:?}; use a non-empty name without commas, quotes or control characters"
            ),
            Self::UnsupportedOutput(path) => write!(
                f,
                "unsupported output {}; the file extension must be .csv or .json",
                path.display()
            ),
            Self::CsvHeaderMismatch(path) => write!(
                f,
                "{} has a different CSV header; write to a new file",
                path.display()
            ),
            Self::Io { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

impl std::error::Error for ProbeError {}
