use bevy::prelude::*;
use bevy_inspector_egui::widgets::ResourceInspector;
use bevy_inspector_egui::{
    Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin,
};

use crate::common::GameStats;
use crate::launcher::Launcher;
use crate::trajectory::TrajectoryLine;
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InspectorPlugin::<InspectableResources>::new())
            .add_plugin(WorldInspectorPlugin::new().filter::<Without<TrajectoryLine>>())
            .register_inspectable::<Launcher>()
            .register_inspectable::<GameStats>();
    }
}

#[derive(Inspectable, Default)]
struct InspectableResources {
    game_stats: ResourceInspector<GameStats>,
}
