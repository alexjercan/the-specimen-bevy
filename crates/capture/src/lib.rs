use bevy::prelude::*;

pub mod loops;
pub mod screenshot;

#[derive(Default)]
pub struct CapturePlugin {
    pub loops: loops::LoopCapturePlugin,
    pub screenshot: screenshot::ScreenshotCapturePlugin,
}

impl CapturePlugin {
    pub fn new(fps: u32) -> Self {
        Self {
            loops: loops::LoopCapturePlugin::new(fps),
            screenshot: screenshot::ScreenshotCapturePlugin,
        }
    }
}

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaptureState>()
            .add_plugins((self.screenshot, self.loops));
    }
}

#[derive(Resource, Default)]
pub struct CaptureState {
    pending: usize,
    failure: Option<String>,
}

impl CaptureState {
    pub fn register(&mut self) {
        self.pending += 1;
    }

    pub fn finish(&mut self) {
        assert!(self.pending > 0, "no capture collector is pending");
        self.pending -= 1;
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    pub fn fail(&mut self, reason: String) {
        self.failure = Some(reason);
    }

    pub fn failed(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

#[cfg(test)]
#[path = "../tests/unit/state.rs"]
mod tests;
