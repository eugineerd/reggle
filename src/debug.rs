use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};

use crate::launcher::Launcher;
use crate::trajectory::TrajectoryLine;
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new().filter::<Without<TrajectoryLine>>())
            .register_inspectable::<Launcher>();
    }
}
