// use crate::common::GameStats;
// use crate::launcher::Launcher;
// use crate::trajectory::TrajectoryLine;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::RapierDebugRenderPlugin;
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(WorldInspectorPlugin)
            .add_plugin(RapierDebugRenderPlugin::default());
    }
}
