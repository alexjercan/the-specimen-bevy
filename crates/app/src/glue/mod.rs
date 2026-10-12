mod facility_assets;
mod interaction_assets;
mod menu_assets;
mod monster_assets;
mod settings;
mod sound;
mod sounds;

pub(crate) use facility_assets::FacilityAssetsPlugin;
pub(crate) use interaction_assets::InteractionAssetsPlugin;
pub(crate) use menu_assets::MenuAssetsPlugin;
pub(crate) use monster_assets::MonsterAssetsPlugin;
pub(crate) use settings::SettingsGluePlugin;
pub(crate) use sound::SoundGluePlugin;
pub(crate) use sounds::FeatureSoundsPlugin;
