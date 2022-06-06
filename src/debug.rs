use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::widgets::ResourceInspector;
use bevy_inspector_egui::{
    Inspectable, InspectorPlugin, RegisterInspectable, WorldInspectorPlugin,
};
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;

use crate::common::GameStats;
use crate::launcher::Launcher;
use crate::trajectory::TrajectoryLine;
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InspectorPlugin::<InspectableResources>::new())
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(WorldInspectorPlugin::new().filter::<Without<TrajectoryLine>>())
            .add_plugin(RapierDebugRenderPlugin::default())
            .register_inspectable::<Launcher>()
            .register_inspectable::<GameStats>();
    }
}

#[derive(Inspectable, Default)]
struct InspectableResources {
    game_stats: ResourceInspector<GameStats>,
}
