mod controller_settings;
mod device_hud;
mod flashlight_hud;
mod fps_overlay;
mod fuse_hud;
mod interaction_hint;
mod sound;
mod stamina_hud;

pub(crate) use controller_settings::ControllerSettingsPlugin;
pub(crate) use device_hud::DeviceHudPlugin;
pub(crate) use flashlight_hud::FlashlightHudPlugin;
pub(crate) use fps_overlay::FpsOverlayPlugin;
pub(crate) use fuse_hud::FuseHudPlugin;
pub(crate) use interaction_hint::InteractionHintPlugin;
pub(crate) use sound::SoundGluePlugin;
pub(crate) use stamina_hud::StaminaHudPlugin;
