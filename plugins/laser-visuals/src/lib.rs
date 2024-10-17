use bevy::{
    prelude::{
        info, Commands, Component, Deref, Entity, EventReader, EventWriter, FixedUpdate, Gizmos,
        IntoSystemConfigs, Plugin, Query, Ray2d, Res, SystemSet, Timer, TimerMode, Update, Vec2,
    },
    time::Time,
};

use game_loop::{DrawingCompleteEvent, GamePhase};
use lasers::LaserPathEvent;
use tilemap::TilemapLayout;

pub struct LaserVisualPlugin;

impl Plugin for LaserVisualPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                Self::spawn_laser_drawing,
                Self::clean_laser_drawing,
                Self::tick_simulation,
            )
                .chain()
                .in_set(LaserVisualSystems),
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
            let total_paths = paths.len();
            if paths.len() > 0 {
                commands.spawn(LaserDrawSimulation {
                    paths,
                    timer: Timer::from_seconds(20., TimerMode::Once),
                    remaining_paths: total_paths as u32,
                });
                info!(
                    "Laser draw simulation spawned with {} paths!",
                    total_paths as u32
                );
            }
        }
    }

    fn draw_lasers(mut draw_simulations: Query<&mut LaserDrawSimulation>, mut gizmos: Gizmos) {
        for mut simulation in &mut draw_simulations {
            let time = simulation.timer.elapsed_secs();
            if simulation.timer.paused() {
                info!("Paused timer")
            }
            info!("Drawing lasers");
            info!("Elapsed time: {}", time);
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
                        gizmos
                            .arrow_2d(start, end, bevy::color::palettes::css::RED)
                            .with_tip_length(25.);
                        laser_time -= end.distance(start) / LaserDrawSimulation::LASER_SPEED;
                        if index == number_of_segments - 1 {
                            simulation.remaining_paths -= 1;
                        }
                    } else {
                        gizmos
                            .arrow_2d(start, point_reached, bevy::color::palettes::css::RED)
                            .with_tip_length(25.);
                        break;
                    }
                }
            }
        }
    }

    fn clean_laser_drawing(
        mut commands: Commands,
        games: Query<(Entity, &GamePhase)>,
        draw_simulations: Query<(Entity, &LaserDrawSimulation)>,
        mut events: EventWriter<DrawingCompleteEvent>,
    ) {
        let Ok((game, game_phase)) = games.get_single() else {
            return;
        };
        if draw_simulations.is_empty() && matches!(game_phase, GamePhase::Draw) {
            events.send(DrawingCompleteEvent { game });
        }
        for (entity, simulation) in &draw_simulations {
            if simulation.remaining_paths == 0 {
                commands.entity(entity).despawn();
                info!("Laser draw simulation cleared")
            }
        }
    }

    fn tick_simulation(time: Res<Time>, mut simulations: Query<&mut LaserDrawSimulation>) {
        for mut simulation in &mut simulations {
            simulation.timer.tick(time.delta());
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
    const LASER_SPEED: f32 = 300.;
}
