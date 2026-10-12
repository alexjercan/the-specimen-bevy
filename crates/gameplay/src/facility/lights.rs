use std::ops::{Mul, MulAssign};

use bevy::prelude::*;

use super::builder::LightEffect;

// NOTE: nits are cd/m^2, the unit of GlobalAmbientLight brightness.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct AmbientNits(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct LightScale(pub f32);

impl LightScale {
    pub fn of(self, intensity: f32) -> f32 {
        intensity * self.0
    }
}

impl Mul<f32> for LightScale {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        Self(self.0 * factor)
    }
}

impl MulAssign<f32> for LightScale {
    fn mul_assign(&mut self, factor: f32) {
        self.0 *= factor;
    }
}

impl Mul<LightScale> for AmbientNits {
    type Output = Self;

    fn mul(self, scale: LightScale) -> Self {
        Self(scale.of(self.0))
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct LightingSettings {
    pub ambient_brightness: AmbientNits,
    pub outage_brightness: AmbientNits,
    pub ambient_modifier: LightScale,
    pub facility_modifier: LightScale,
    pub menu_modifier: LightScale,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            ambient_brightness: AmbientNits(6.0),
            outage_brightness: AmbientNits(0.35),
            ambient_modifier: LightScale(1.0),
            facility_modifier: LightScale(0.45),
            menu_modifier: LightScale(1.0),
        }
    }
}

#[derive(Component)]
pub struct FacilityLight;

#[derive(Component)]
pub struct MainMenuAccentLight;

#[derive(Component, Clone, Copy, Default)]
pub struct LightConfig {
    pub position: Option<Vec3>,
    pub intensity: Option<f32>,
    pub effect: Option<LightEffect>,
}

impl LightConfig {
    pub const fn intensity(intensity: f32) -> Self {
        Self {
            position: None,
            intensity: Some(intensity),
            effect: None,
        }
    }

    pub const fn flicker(phase: f32) -> Self {
        Self {
            position: None,
            intensity: None,
            effect: Some(LightEffect::Flicker(phase)),
        }
    }
}
