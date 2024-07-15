use bevy::{ecs::system::Command, prelude::*};
use hexx::*;

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Component, Deref, DerefMut)]
pub struct Position(Hex);

impl From<Hex> for Position {
    fn from(value: Hex) -> Self {
        Self(value)
    }
}

impl Position {
    pub fn ray(self, direction: Direction, limit: u32) -> Vec<Position> {
        let mut ray = Vec::new();
        let mut steps = 1;
        let mut next_neighbor = self;
        while steps < limit {
            next_neighbor = next_neighbor.neighbor(direction.to_hex()).into();
            ray.push(next_neighbor);
            steps += 1;
        }
        ray
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Component, Reflect)]
pub enum Direction {
    North,
    South,
    Northeast,
    Southeast,
    Northwest,
    Southwest,
}

impl Direction {
    fn to_hex(&self) -> EdgeDirection {
        match self {
            Self::North => EdgeDirection::FLAT_NORTH,
            Self::South => EdgeDirection::FLAT_SOUTH,
            Self::Northeast => EdgeDirection::FLAT_NORTH_EAST,
            Self::Southeast => EdgeDirection::FLAT_SOUTH_EAST,
            Self::Northwest => EdgeDirection::FLAT_NORTH_WEST,
            Self::Southwest => EdgeDirection::FLAT_SOUTH_WEST,
        }
    }
}

pub fn neighbor_along_ray(
    source_query: Query<(&Position, &Direction)>,
    hit_query: Query<&Position>,
) {
    for (source_position, source_direction) in &source_query {
        let ray = source_position.ray(*source_direction, 100);

        for hit_position in &hit_query {
            if ray.contains(hit_position) {}
        }
    }
}

pub trait Tile {
    fn activate(&self) -> impl Command;

    fn on_hit(&self) -> Option<impl Command> {None}
}



pub trait MerchAppExt {
    fn define_merchandise<T>(&mut self)
    where
        T: Component + GetTypeRegistration + Merchandise,
    {
    }
}

impl MerchAppExt for App {
    fn define_merchandise<T>(&mut self)
    where
        T: Component + GetTypeRegistration + Merchandise,
    {
        let mut registry = self.world_mut().resource_mut::<MerchRegistry>();
        // registry.register::<T>().unwrap();
        self.add_systems(Update, MySystemSet::handle_hit_sinks::<T>)
    }
}

impl  {
    fn handle_hit_sinks<T: Tile>(mut commands: Commands, query: Query<(&Sink, &Position, &dyn Tile)>) {
        // on hit
        commands.add(t.activate());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyTile {
        value: usize,
    }

    impl Tile for MyTile {
        fn activate(&self) -> impl Command {
            TileCommand { value: self.value }
        }
    }

    struct TileCommand {
        value: usize,
    }

    impl Command for TileCommand {
        fn apply(self, world: &mut World) {
            world.insert_resource(CommandComplete);
        }
    }

    #[derive(Resource)]
    pub struct CommandComplete;

    #[test]
    fn test_command() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);

        fn system(mut commands: Commands) {
            commands.add(MyTile { value: 10 }.activate());
        }

        app.add_systems(Startup, system);
        app.update();
        app.world.resource::<CommandComplete>();
    }
}
