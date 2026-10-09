mod device_hud;
mod flashlight_hud;
mod fuse_hud;
mod interaction_hint;
mod sound;
mod stamina_hud;

pub(crate) use device_hud::DeviceHudPlugin;
pub(crate) use flashlight_hud::FlashlightHudPlugin;
pub(crate) use fuse_hud::FuseHudPlugin;
pub(crate) use interaction_hint::InteractionHintPlugin;
pub(crate) use sound::SoundGluePlugin;
pub(crate) use stamina_hud::StaminaHudPlugin;

pub(crate) fn hud_binding_label(binding: &str) -> &str {
    match binding {
        "MouseLeft" => "LMB",
        "MouseRight" => "RMB",
        "MouseMiddle" => "MMB",
        _ => binding.strip_prefix("Key").unwrap_or(binding),
    }
}
