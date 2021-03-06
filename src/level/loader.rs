use crate::AppState;
use bevy::prelude::*;
use rand::prelude::*;

use super::{
    maze_level::{AxisChanged, PositionChanged},
    MazeLevel,
};

#[derive(Clone, Debug)]
pub struct LoadLevel {
    pub rng_source: RngSource,
    pub dimensions: DimensionLength,
}

#[derive(Clone, Debug)]
pub enum RngSource {
    Seeded(u64),
}

// Remove this once construction methods for dimensions are found.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum DimensionLength {
    Two([u8; 2]),
    Three([u8; 3]),
    Four([u8; 4]),
    Five([u8; 5]),
    Six([u8; 6]),
}

impl Default for LoadLevel {
    fn default() -> Self {
        Self {
            rng_source: RngSource::Seeded(123456789),
            dimensions: DimensionLength::Two([2, 2]),
        }
    }
}

pub fn level_load_system(
    mut c: Commands,
    mut events: EventReader<LoadLevel>,
    mut app_state: ResMut<State<AppState>>,
) {
    for level_loader in events.iter() {
        let mut rng = match level_loader.rng_source {
            RngSource::Seeded(seed) => StdRng::seed_from_u64(seed),
        };
        c.insert_resource(match level_loader.dimensions {
            DimensionLength::Two(lengths) => MazeLevel::new(&lengths, &mut rng),
            DimensionLength::Three(lengths) => MazeLevel::new(&lengths, &mut rng),
            DimensionLength::Four(lengths) => MazeLevel::new(&lengths, &mut rng),
            DimensionLength::Five(lengths) => MazeLevel::new(&lengths, &mut rng),
            DimensionLength::Six(lengths) => MazeLevel::new(&lengths, &mut rng),
        });
        app_state.push(AppState::InMaze).unwrap();
    }
}

pub fn initial_events_on_load(
    maze: Res<MazeLevel>,
    mut position_changed: EventWriter<PositionChanged>,
    mut axis_changed: EventWriter<AxisChanged>,
) {
    position_changed.send(PositionChanged {
        position: maze.pos(),
    });
    axis_changed.send(AxisChanged { axis: maze.axis() });
}

pub fn load_maze_assets(
    mut c: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    c.insert_resource(MazeAssets {
        joint: meshes.add(Mesh::from(shape::Box::new(0.2, 1.0, 0.2))),
        wall: meshes.add(Mesh::from(shape::Box::new(0.1, 0.6, 1.0))),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    });
}

pub fn spawn_player(
    mut c: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    c.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 0.3,
            ..default()
        })),
        material: materials.add(Color::rgb(0.5, 0.5, 0.8).into()),
        ..Default::default()
    });
}

#[derive(Component)]
pub struct MazeAssets {
    joint: Handle<Mesh>,
    wall: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl MazeAssets {
    pub fn wall(&self, transform: Transform) -> PbrBundle {
        PbrBundle {
            mesh: self.wall.clone(),
            material: self.material.clone(),
            transform,
            ..Default::default()
        }
    }

    pub fn joint(&self, transform: Transform) -> PbrBundle {
        PbrBundle {
            mesh: self.joint.clone(),
            material: self.material.clone(),
            transform,
            ..Default::default()
        }
    }
}
