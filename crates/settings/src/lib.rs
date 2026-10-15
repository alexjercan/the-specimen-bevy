use std::{
    collections::HashSet,
    fmt, fs,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    reflect::enums::{DynamicEnum, DynamicVariant},
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

pub const MIN_SENSITIVITY: f32 = 0.0;
pub const MAX_SENSITIVITY: f32 = 0.004;
pub const DEFAULT_SENSITIVITY: f32 = 0.0015;

pub const MIN_FOV: f32 = 50.0;
pub const MAX_FOV: f32 = 110.0;
pub const DEFAULT_FOV: f32 = 55.0;

pub const MIN_UI_SCALE: f32 = 0.75;
pub const MAX_UI_SCALE: f32 = 1.5;
pub const DEFAULT_UI_SCALE: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsQuality {
    Low,
    Medium,
    #[default]
    High,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MsaaQuality {
    Off,
    Sample2,
    #[default]
    Sample4,
    Sample8,
}

impl MsaaQuality {
    pub const ALL: [Self; 4] = [Self::Off, Self::Sample2, Self::Sample4, Self::Sample8];

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self, 1)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FpsCap {
    #[default]
    Uncapped,
    Fps30,
    Fps60,
    Fps120,
}

impl FpsCap {
    pub const ALL: [Self; 4] = [Self::Uncapped, Self::Fps30, Self::Fps60, Self::Fps120];

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self, 1)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Windowed,
    #[default]
    Fullscreen,
}

impl DisplayMode {
    pub const ALL: [Self; 2] = [Self::Windowed, Self::Fullscreen];

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self, 1)
    }
}

impl GraphicsQuality {
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Custom];

    pub fn next(self) -> Self {
        cycle(&Self::ALL, self, 1)
    }

    pub fn preset_msaa(self) -> Option<MsaaQuality> {
        match self {
            Self::Low => Some(MsaaQuality::Off),
            Self::Medium => Some(MsaaQuality::Sample2),
            Self::High => Some(MsaaQuality::Sample4),
            Self::Custom => None,
        }
    }
}

pub fn cycle<T: Copy + PartialEq>(choices: &[T], current: T, step: isize) -> T {
    let index = choices
        .iter()
        .position(|item| *item == current)
        .unwrap_or(0) as isize;
    choices[(index + step).rem_euclid(choices.len() as isize) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl InputBinding {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "MouseLeft" => Some(Self::Mouse(MouseButton::Left)),
            "MouseRight" => Some(Self::Mouse(MouseButton::Right)),
            "MouseMiddle" => Some(Self::Mouse(MouseButton::Middle)),
            "MouseBack" => Some(Self::Mouse(MouseButton::Back)),
            "MouseForward" => Some(Self::Mouse(MouseButton::Forward)),
            _ => parse_key(name).map(Self::Key),
        }
    }

    pub fn name(self) -> String {
        match self {
            Self::Key(key) => key_name(key),
            Self::Mouse(MouseButton::Left) => "MouseLeft".to_owned(),
            Self::Mouse(MouseButton::Right) => "MouseRight".to_owned(),
            Self::Mouse(MouseButton::Middle) => "MouseMiddle".to_owned(),
            Self::Mouse(MouseButton::Back) => "MouseBack".to_owned(),
            Self::Mouse(MouseButton::Forward) => "MouseForward".to_owned(),
            Self::Mouse(MouseButton::Other(id)) => format!("MouseOther{id}"),
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Key(key) => key_label(key),
            Self::Mouse(MouseButton::Left) => "LMB".to_owned(),
            Self::Mouse(MouseButton::Right) => "RMB".to_owned(),
            Self::Mouse(MouseButton::Middle) => "MMB".to_owned(),
            Self::Mouse(MouseButton::Back) => "MB4".to_owned(),
            Self::Mouse(MouseButton::Forward) => "MB5".to_owned(),
            Self::Mouse(MouseButton::Other(id)) => format!("MB{id}"),
        }
    }

    pub fn key(self) -> Option<KeyCode> {
        match self {
            Self::Key(key) => Some(key),
            Self::Mouse(_) => None,
        }
    }
}

impl From<KeyCode> for InputBinding {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<MouseButton> for InputBinding {
    fn from(button: MouseButton) -> Self {
        Self::Mouse(button)
    }
}

impl fmt::Display for InputBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl Serialize for InputBinding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.name())
    }
}

impl<'de> Deserialize<'de> for InputBinding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Self::parse(&name).ok_or_else(|| D::Error::custom(format!("unknown binding {name}")))
    }
}

pub fn key_name(key: KeyCode) -> String {
    format!("{key:?}")
}

pub fn key_label(key: KeyCode) -> String {
    let name = key_name(key);
    name.strip_prefix("Key")
        .or_else(|| name.strip_prefix("Digit"))
        .unwrap_or(&name)
        .to_owned()
}

pub fn parse_key(name: &str) -> Option<KeyCode> {
    KeyCode::from_reflect(&DynamicEnum::new(name.to_owned(), DynamicVariant::Unit))
        .filter(|key| *key != KeyCode::Escape)
}

mod key_code {
    use super::*;

    pub fn serialize<S: Serializer>(key: &KeyCode, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&key_name(*key))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<KeyCode, D::Error> {
        let name = String::deserialize(deserializer)?;
        parse_key(&name).ok_or_else(|| D::Error::custom(format!("unknown key {name}")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MovementKeys {
    #[serde(with = "key_code")]
    pub forward: KeyCode,
    #[serde(with = "key_code")]
    pub left: KeyCode,
    #[serde(with = "key_code")]
    pub backward: KeyCode,
    #[serde(with = "key_code")]
    pub right: KeyCode,
    #[serde(with = "key_code")]
    pub interact: KeyCode,
    #[serde(with = "key_code")]
    pub sprint: KeyCode,
    pub flashlight: InputBinding,
    pub flashbang: InputBinding,
}

impl Default for MovementKeys {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            left: KeyCode::KeyA,
            backward: KeyCode::KeyS,
            right: KeyCode::KeyD,
            interact: KeyCode::KeyF,
            sprint: KeyCode::ShiftLeft,
            flashlight: InputBinding::Mouse(MouseButton::Left),
            flashbang: InputBinding::Mouse(MouseButton::Right),
        }
    }
}

impl MovementKeys {
    pub fn all(&self) -> [InputBinding; 8] {
        [
            self.forward.into(),
            self.left.into(),
            self.backward.into(),
            self.right.into(),
            self.interact.into(),
            self.sprint.into(),
            self.flashlight,
            self.flashbang,
        ]
    }

    pub fn unique(&self) -> bool {
        let all = self.all();
        all.iter().collect::<HashSet<_>>().len() == all.len()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferredMode {
    #[default]
    Normal,
    Custom,
    Insane,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CustomLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl CustomLevel {
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];

    pub fn pick<T: Copy>(self, choices: [T; 3]) -> T {
        let [low, medium, high] = choices;
        match self {
            Self::Low => low,
            Self::Medium => medium,
            Self::High => high,
        }
    }

    pub fn of<T: Copy + PartialEq>(choices: [T; 3], value: T) -> Self {
        Self::ALL
            .into_iter()
            .find(|level| level.pick(choices) == value)
            .unwrap_or_default()
    }
}

// NOTE: saved as the number 0, 1 or 2 so older settings files still load; unknown numbers mean Low.
impl Serialize for CustomLevel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
        })
    }
}

impl<'de> Deserialize<'de> for CustomLevel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match u8::deserialize(deserializer)? {
            1 => Self::Medium,
            2 => Self::High,
            _ => Self::Low,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomDifficulty {
    pub walk_speed: CustomLevel,
    pub chase_speed: CustomLevel,
    pub walk_hearing: CustomLevel,
    pub run_hearing: CustomLevel,
    pub sprint: bool,
    pub flashbangs: bool,
    pub detector: bool,
    pub blackout: bool,
}

impl Default for CustomDifficulty {
    fn default() -> Self {
        Self {
            walk_speed: CustomLevel::Low,
            chase_speed: CustomLevel::Low,
            walk_hearing: CustomLevel::Low,
            run_hearing: CustomLevel::Low,
            sprint: true,
            flashbangs: true,
            detector: true,
            blackout: false,
        }
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub master: f32,
    pub sfx: f32,
    pub music: f32,
    pub mouse_sensitivity: f32,
    pub keys: MovementKeys,
    pub graphics: GraphicsQuality,
    pub msaa: MsaaQuality,
    pub display_mode: DisplayMode,
    pub fps_overlay: bool,
    pub preferred_mode: PreferredMode,
    pub custom_difficulty: CustomDifficulty,
    pub invert_y: bool,
    pub reduce_flicker: bool,
    pub fov: f32,
    pub ui_scale: f32,
    pub vsync: bool,
    pub fps_cap: FpsCap,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            sfx: 1.0,
            music: 1.0,
            mouse_sensitivity: DEFAULT_SENSITIVITY,
            keys: MovementKeys::default(),
            graphics: GraphicsQuality::default(),
            msaa: MsaaQuality::default(),
            display_mode: DisplayMode::default(),
            fps_overlay: false,
            preferred_mode: PreferredMode::default(),
            custom_difficulty: CustomDifficulty::default(),
            invert_y: false,
            reduce_flicker: false,
            fov: DEFAULT_FOV,
            ui_scale: DEFAULT_UI_SCALE,
            vsync: true,
            fps_cap: FpsCap::default(),
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
        self.fov = if self.fov.is_finite() {
            self.fov.clamp(MIN_FOV, MAX_FOV)
        } else {
            DEFAULT_FOV
        };
        self.ui_scale = if self.ui_scale.is_finite() {
            self.ui_scale.clamp(MIN_UI_SCALE, MAX_UI_SCALE)
        } else {
            DEFAULT_UI_SCALE
        };
        if !self.keys.unique() {
            self.keys = MovementKeys::default();
        }
    }

    fn from_saved_json(data: &[u8]) -> std::io::Result<Self> {
        let value: serde_json::Value =
            serde_json::from_slice(data).map_err(std::io::Error::other)?;
        let missing_msaa = value.get("msaa").is_none();
        let mut settings: Self = serde_json::from_value(value).map_err(std::io::Error::other)?;
        if missing_msaa {
            if let Some(msaa) = settings.graphics.preset_msaa() {
                settings.msaa = msaa;
            }
        }
        settings.sanitize();
        Ok(settings)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        Self::from_saved_json(&fs::read(path)?)
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
    Some(root.join("the-specimen-bevy").join("settings.json"))
}

pub fn load_or_migrate(path: &Path) -> std::io::Result<GameSettings> {
    match GameSettings::load(path) {
        Ok(settings) => Ok(settings),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let Some(root) = path.parent().and_then(Path::parent) else {
                return Err(error);
            };
            let old_path = root.join("horror-game-bevy/settings.json");
            match GameSettings::load(&old_path) {
                Ok(settings) => {
                    if let Err(save_error) = settings.save(path) {
                        warn!("settings migration save failed: {save_error}");
                    }
                    Ok(settings)
                }
                Err(old_error) if old_error.kind() == std::io::ErrorKind::NotFound => Err(error),
                Err(old_error) => Err(old_error),
            }
        }
        Err(error) => Err(error),
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub enum SettingsStore {
    Disabled,
    File(PathBuf),
    #[cfg(target_arch = "wasm32")]
    Browser,
}

impl SettingsStore {
    pub fn persistent(persist: bool) -> Self {
        if !persist {
            return Self::Disabled;
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::Browser
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            match default_path() {
                Some(path) => Self::File(path),
                None => {
                    warn!("settings directory unavailable; settings will not persist");
                    Self::Disabled
                }
            }
        }
    }

    pub fn load(&self) -> std::io::Result<Option<GameSettings>> {
        match self {
            Self::Disabled => Ok(None),
            Self::File(path) => match load_or_migrate(path) {
                Ok(settings) => Ok(Some(settings)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error),
            },
            #[cfg(target_arch = "wasm32")]
            Self::Browser => browser::load(),
        }
    }

    pub fn save(&self, settings: &GameSettings) -> std::io::Result<()> {
        match self {
            Self::Disabled => Ok(()),
            Self::File(path) => settings.save(path),
            #[cfg(target_arch = "wasm32")]
            Self::Browser => browser::save(settings),
        }
    }
}

#[derive(Resource, Default)]
pub struct SettingsDirty(pub bool);

pub struct GameSettingsPlugin {
    pub persist: bool,
}

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        let store = SettingsStore::persistent(self.persist);
        let settings = match store.load() {
            Ok(Some(settings)) => settings,
            Ok(None) => GameSettings::default(),
            Err(error) => {
                warn!("settings load failed: {error}");
                GameSettings::default()
            }
        };
        app.insert_resource(settings)
            .insert_resource(store)
            .init_resource::<SettingsDirty>()
            .add_systems(Last, save_dirty);
    }
}

fn save_dirty(
    settings: Res<GameSettings>,
    store: Res<SettingsStore>,
    mut dirty: ResMut<SettingsDirty>,
) {
    if !dirty.0 {
        return;
    }
    match store.save(&settings) {
        Ok(()) => dirty.0 = false,
        Err(error) => warn!("settings save failed: {error}"),
    }
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::GameSettings;

    const SETTINGS_KEY: &str = "the-specimen-bevy.settings";

    fn storage() -> std::io::Result<web_sys::Storage> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .ok_or_else(|| std::io::Error::other("local storage unavailable"))
    }

    pub fn load() -> std::io::Result<Option<GameSettings>> {
        let value = storage()?
            .get_item(SETTINGS_KEY)
            .map_err(|_| std::io::Error::other("local storage read failed"))?;
        value
            .map(|value| GameSettings::from_saved_json(value.as_bytes()))
            .transpose()
    }

    pub fn save(settings: &GameSettings) -> std::io::Result<()> {
        let value = serde_json::to_string(settings).map_err(std::io::Error::other)?;
        storage()?
            .set_item(SETTINGS_KEY, &value)
            .map_err(|_| std::io::Error::other("local storage write failed"))
    }
}
