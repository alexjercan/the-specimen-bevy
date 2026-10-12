use bevy::{platform::collections::HashMap, prelude::*, world_serialization::WorldAsset};

use super::module::Module;

#[derive(Resource)]
pub struct FacilityAssets {
    pub modules: HashMap<Module, Handle<WorldAsset>>,
}

impl FacilityAssets {
    pub fn module(&self, module: Module) -> Option<&Handle<WorldAsset>> {
        self.modules.get(&module)
    }
}
