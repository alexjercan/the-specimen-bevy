use std::{
    fs,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const MIN_SENSITIVITY: f32 = 0.0005;
pub const MAX_SENSITIVITY: f32 = 0.01;
pub const DEFAULT_SENSITIVITY: f32 = 0.002;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsQuality {
    Low,
    Medium,
    #[default]
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Windowed,
    #[default]
    Fullscreen,
}

impl DisplayMode {
    pub fn next(self) -> Self {
        match self {
            Self::Windowed => Self::Fullscreen,
            Self::Fullscreen => Self::Windowed,
        }
    }
}

impl GraphicsQuality {
    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MovementKeys {
    pub forward: String,
    pub left: String,
    pub backward: String,
    pub right: String,
    pub interact: String,
    pub flashlight: String,
    pub flashbang: String,
}

impl Default for MovementKeys {
    fn default() -> Self {
        Self {
            forward: "KeyW".into(),
            left: "KeyA".into(),
            backward: "KeyS".into(),
            right: "KeyD".into(),
            interact: "KeyF".into(),
            flashlight: "MouseLeft".into(),
            flashbang: "MouseRight".into(),
        }
    }
}

impl MovementKeys {
    pub fn unique(&self) -> bool {
        let keys = [
            &self.forward,
            &self.left,
            &self.backward,
            &self.right,
            &self.interact,
            &self.flashlight,
            &self.flashbang,
        ];
        keys[..5].iter().all(|key| parse_key(key).is_some())
            && keys[5..].iter().all(|key| parse_binding(key).is_some())
            && keys
                .iter()
                .enumerate()
                .all(|(index, key)| keys[index + 1..].iter().all(|other| other != key))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

pub fn parse_binding(name: &str) -> Option<InputBinding> {
    match name {
        "MouseLeft" => Some(InputBinding::Mouse(MouseButton::Left)),
        "MouseRight" => Some(InputBinding::Mouse(MouseButton::Right)),
        "MouseMiddle" => Some(InputBinding::Mouse(MouseButton::Middle)),
        _ => parse_key(name).map(InputBinding::Key),
    }
}

pub fn parse_key(name: &str) -> Option<KeyCode> {
    Some(match name {
        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "Space" => KeyCode::Space,
        _ => return None,
    })
}

fn default_volume() -> f32 {
    1.0
}
fn default_sensitivity() -> f32 {
    DEFAULT_SENSITIVITY
}

#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    #[serde(default = "default_volume")]
    pub master: f32,
    #[serde(default = "default_volume")]
    pub sfx: f32,
    #[serde(default = "default_volume")]
    pub music: f32,
    #[serde(default = "default_sensitivity")]
    pub mouse_sensitivity: f32,
    pub keys: MovementKeys,
    pub graphics: GraphicsQuality,
    pub display_mode: DisplayMode,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            sfx: 1.0,
            music: 1.0,
            mouse_sensitivity: DEFAULT_SENSITIVITY,
            keys: MovementKeys::default(),
            graphics: GraphicsQuality::High,
            display_mode: DisplayMode::Fullscreen,
        }
    }
}

impl GameSettings {
    pub fn sanitize(&mut self) {
        for level in [&mut self.master, &mut self.sfx, &mut self.music] {
            *level = if level.is_finite() {
                level.clamp(0.0, 1.0)
            } else {
                1.0
            };
        }
        self.mouse_sensitivity = if self.mouse_sensitivity.is_finite() {
            self.mouse_sensitivity
                .clamp(MIN_SENSITIVITY, MAX_SENSITIVITY)
        } else {
            DEFAULT_SENSITIVITY
        };
        if !self.keys.unique() {
            self.keys = MovementKeys::default();
        }
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = fs::read(path)?;
        let mut settings: Self = serde_json::from_slice(&data).map_err(std::io::Error::other)?;
        settings.sanitize();
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        fs::write(&temp, data)?;
        fs::rename(&temp, path)
    }
}

pub fn default_path() -> Option<PathBuf> {
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(root.join("horror-game-bevy").join("settings.json"))
}

#[derive(Resource, Default)]
pub struct SettingsPath(pub Option<PathBuf>);

#[derive(Resource, Default)]
pub struct SettingsDirty(pub bool);

pub struct GameSettingsPlugin {
    pub persist: bool,
}

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        let path = if self.persist { default_path() } else { None };
        let settings = path.as_deref().map(GameSettings::load).transpose();
        let settings = match settings {
            Ok(Some(settings)) => settings,
            Ok(None) => GameSettings::default(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => GameSettings::default(),
            Err(error) => {
                warn!("settings load failed: {error}");
                GameSettings::default()
            }
        };
        app.insert_resource(settings)
            .insert_resource(SettingsPath(path))
            .init_resource::<SettingsDirty>()
            .add_systems(Last, save_dirty);
    }
}

fn save_dirty(
    settings: Res<GameSettings>,
    path: Res<SettingsPath>,
    mut dirty: ResMut<SettingsDirty>,
) {
    if !dirty.0 {
        return;
    }
    let Some(path) = &path.0 else {
        dirty.0 = false;
        return;
    };
    match settings.save(path) {
        Ok(()) => dirty.0 = false,
        Err(error) => warn!("settings save failed: {error}"),
    }
}
