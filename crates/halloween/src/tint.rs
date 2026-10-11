use bevy::{pbr::DistanceFog, prelude::*};

const AMBIENT_TINT: Color = Color::srgb(1.0, 0.93, 0.85);
const FOG_TINT: Color = Color::srgb(0.023, 0.019, 0.018);

pub(crate) struct TintPlugin;

impl Plugin for TintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tint_ambient)
            .add_observer(tint_camera_fog);
    }
}

fn tint_ambient(mut ambient: Option<ResMut<GlobalAmbientLight>>) {
    if let Some(ambient) = ambient.as_deref_mut() {
        if ambient.color != AMBIENT_TINT {
            ambient.color = AMBIENT_TINT;
        }
    }
}

fn tint_camera_fog(added: On<Add, DistanceFog>, mut fog: Query<&mut DistanceFog>) {
    if let Ok(mut fog) = fog.get_mut(added.entity) {
        fog.color = FOG_TINT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tint_keeps_ambient_brightness_and_fog_density() {
        let mut app = App::new();
        app.insert_resource(GlobalAmbientLight {
            brightness: 0.35,
            ..default()
        })
        .add_plugins(TintPlugin);
        let camera = app.world_mut().spawn(DistanceFog::default()).id();
        app.update();
        let ambient = app.world().resource::<GlobalAmbientLight>();
        assert_eq!(ambient.brightness, 0.35);
        assert_eq!(ambient.color, AMBIENT_TINT);
        let fog = app.world().get::<DistanceFog>(camera).unwrap();
        assert_eq!(fog.color, FOG_TINT);
    }
}
