use bevy::prelude::{
    Commands, Component, Deref, Entity, EventReader, FixedUpdate, Gizmos, IntoSystemConfigs,
    Plugin, Query, Ray2d, SystemSet, Timer, TimerMode, Update, Vec2,
};

use lasers::LaserPathEvent;
use tilemap::TilemapLayout;

pub struct LaserVisualPlugin;

impl Plugin for LaserVisualPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (Self::spawn_laser_drawing, Self::clean_laser_drawing).in_set(LaserVisualSystems),
        )
        .add_systems(FixedUpdate, Self::draw_lasers.in_set(LaserVisualSystems));
    }
}

impl LaserVisualPlugin {
    fn spawn_laser_drawing(
        mut commands: Commands,
        mut laser_path_events: EventReader<LaserPathEvent>,
        tilemaps: Query<&TilemapLayout>,
    ) {
        let all_paths = laser_path_events
            .read()
            .map(|laser_path| laser_path.path.clone())
            .collect::<Vec<_>>();
        for tilemap in &tilemaps {
            let paths = all_paths
                .iter()
                .map(|laser_path| {
                    LaserPath::new(
                        laser_path
                            .iter()
                            .map(|hex_position| tilemap.hex_to_world_pos(**hex_position))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();

            commands.spawn(LaserDrawSimulation {
                paths,
                timer: Timer::from_seconds(100., TimerMode::Once),
                remaining_paths: laser_path_events.read().len() as u32,
            });
        }
    }

    fn draw_lasers(mut draw_simulations: Query<&mut LaserDrawSimulation>, mut gizmos: Gizmos) {
        for mut simulation in &mut draw_simulations {
            let time = simulation.timer.elapsed_secs();
            // Store the data for each start, end of all the laser segments
            let all_laser_segments = simulation
                .paths
                .iter()
                .map(|laser_path| {
                    laser_path
                        .windows(2)
                        .map(|segment| {
                            let start = segment[0];
                            let end = segment[1];
                            [start, end]
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            for laser_path in all_laser_segments {
                let mut laser_time = time;
                let number_of_segments = laser_path.len();
                for (index, segment) in laser_path.iter().enumerate() {
                    let start = segment[0];
                    let end = segment[1];
                    let ray = Ray2d::new(start, end - start);
                    let point_reached =
                        ray.get_point(laser_time * LaserDrawSimulation::LASER_SPEED);
                    if point_reached.distance(start) > end.distance(start) {
                        gizmos.arrow_2d(start, end, bevy::color::palettes::css::RED);
                        laser_time -= end.distance(start) / LaserDrawSimulation::LASER_SPEED;
                        if index == number_of_segments - 1 {
                            simulation.remaining_paths -= 1;
                        }
                    } else {
                        gizmos.arrow_2d(start, point_reached, bevy::color::palettes::css::RED);
                        break;
                    }
                }
            }
        }
    }

    fn clean_laser_drawing(
        mut commands: Commands,
        draw_simulations: Query<(Entity, &LaserDrawSimulation)>,
    ) {
        for (entity, simulation) in &draw_simulations {
            if simulation.remaining_paths == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct LaserVisualSystems;

#[derive(Clone, Debug)]
#[derive(Component, Deref)]
pub struct LaserPath(Vec<Vec2>);

impl LaserPath {
    pub fn new(points: Vec<Vec2>) -> Self {
        LaserPath(points)
    }
}

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct LaserDrawSimulation {
    pub paths: Vec<LaserPath>,
    pub timer: Timer,
    pub remaining_paths: u32,
}

impl LaserDrawSimulation {
    const LASER_SPEED: f32 = 3.;
}
