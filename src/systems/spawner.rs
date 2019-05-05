use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    ecs::*,
    shrev::{EventChannel, ReaderId},
};

use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng, RngCore,
};

use std::f32::consts::PI;

use crate::{components::creatures::CreatureType, resources::prefabs::CreaturePrefabs};

#[derive(Debug, Clone)]
pub struct CreatureSpawnEvent {
    pub creature_type: String,
    pub transform: Transform,
}

struct CreatureTypeDistribution {
    creature_type: CreatureType,
}
impl Distribution<CreatureTypeDistribution> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CreatureTypeDistribution {
        match rng.gen_range(0, 3) {
            0 => CreatureTypeDistribution {
                creature_type: "Carnivore".to_string(),
            },
            1 => CreatureTypeDistribution {
                creature_type: "Herbivore".to_string(),
            },
            _ => CreatureTypeDistribution {
                creature_type: "Plant".to_string(),
            },
        }
    }
}

#[derive(Default)]
pub struct CreatureSpawnerSystem {
    spawn_reader_id: Option<ReaderId<CreatureSpawnEvent>>,
}

impl<'s> System<'s> for CreatureSpawnerSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, EventChannel<CreatureSpawnEvent>>,
        Read<'s, CreaturePrefabs>,
        Write<'s, LazyUpdate>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.spawn_reader_id = Some(
            res.fetch_mut::<EventChannel<CreatureSpawnEvent>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (entities, spawn_events, prefabs, lazy_update): Self::SystemData) {
        for event in spawn_events.read(self.spawn_reader_id.as_mut().unwrap()) {
            let creature_prefab = prefabs.get_prefab(&event.creature_type);
            match creature_prefab {
                Some(prefab) => {
                    lazy_update
                        .create_entity(&entities)
                        .with(prefab.clone())
                        .with(event.transform.clone())
                        .build();
                }
                None => (),
            }
        }
    }
}

// For debugging purposes this system sends spawn events regularly
#[derive(Default)]
pub struct DebugSpawnTriggerSystem {
    timer_to_next_spawn: f32,
}

impl<'s> System<'s> for DebugSpawnTriggerSystem {
    type SystemData = (Write<'s, EventChannel<CreatureSpawnEvent>>, Read<'s, Time>);

    fn run(&mut self, (mut spawn_events, time): Self::SystemData) {
        let delta_seconds = time.delta_seconds();
        self.timer_to_next_spawn -= delta_seconds;
        if self.timer_to_next_spawn <= 0.0 {
            self.timer_to_next_spawn = 1.5;
            let mut rng = thread_rng();
            let x = (rng.next_u32() % 100) as f32 / 5.0 - 10.0;
            let y = (rng.next_u32() % 100) as f32 / 5.0 - 10.0;
            let mut transform = Transform::default();
            transform.set_xyz(x, y, 0.02);
            let CreatureTypeDistribution { creature_type }: CreatureTypeDistribution =
                rand::random();

            match creature_type {
                _ if creature_type == "Carnivore" || creature_type == "Herbivore" => {
                    transform.set_scale(0.5, 0.5, 1.0);
                }
                _ if creature_type == "Plant" => {
                    let scale = (rng.next_u32() % 100) as f32 / 250.0 + 0.8;
                    let rotation = (rng.next_u32() % 100) as f32 / 100.0 * PI;
                    transform.set_z(0.0);
                    transform.set_scale(scale, scale, 1.0);
                    transform.set_rotation_euler(0.0, 0.0, rotation);
                }
                _ => {}
            }

            spawn_events.single_write(CreatureSpawnEvent {
                creature_type,
                transform,
            });
        }
    }
}
