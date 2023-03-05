use bevy_ecs::prelude::*;
use bevy_ecs::schedule::SystemConfigs;
use valence_protocol::packet::s2c::play::game_state_change::GameEventKind;
use valence_protocol::packet::s2c::play::GameStateChangeS2c;

use super::Instance;
use crate::client::Client;

pub const WEATHER_LEVEL_MIN: f32 = 0_f32;
pub const WEATHER_LEVEL_MAX: f32 = 1_f32;

/// The weather state representation.
#[derive(Component)]
pub struct Weather {
    /// Contains the rain level.
    /// Should be between [`WEATHER_LEVEL_MIN`] and [`WEATHER_LEVEL_MAX`].
    ///
    /// The [`None`] value means no rain level.
    pub rain: Option<f32>,
    /// Contains the thunder level.
    /// Should be between [`WEATHER_LEVEL_MIN`] and [`WEATHER_LEVEL_MAX`].
    ///
    /// The [`None`] value means no thunder level.
    pub thunder: Option<f32>,
}

impl Instance {
    /// Sends the begin rain event to all players in the instance.
    pub fn begin_raining(&mut self) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::BeginRaining,
            value: f32::default(),
        });
    }

    /// Sends the end rain event to all players in the instance.
    pub fn end_raining(&mut self) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::EndRaining,
            value: f32::default(),
        });
    }

    /// Sends the set rain level event to all players in the instance.
    pub fn set_rain_level(&mut self, level: f32) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::RainLevelChange,
            value: level.clamp(WEATHER_LEVEL_MIN, WEATHER_LEVEL_MAX),
        });
    }

    /// Sends the set thunder level event to all players in the instance.
    pub fn set_thunder_level(&mut self, level: f32) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::ThunderLevelChange,
            value: level.clamp(WEATHER_LEVEL_MIN, WEATHER_LEVEL_MAX),
        });
    }

    /// Sends weather level events to all players in the instance.
    pub fn set_weather(&mut self, weather: &Weather) {
        if let Some(rain_level) = weather.rain {
            self.set_rain_level(rain_level)
        }

        if let Some(thunder_level) = weather.thunder {
            self.set_thunder_level(thunder_level)
        }
    }
}

impl Client {
    /// Sends the begin rain event to the client.
    pub fn begin_raining(&mut self) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::BeginRaining,
            value: f32::default(),
        });
    }

    /// Sends the end rain event to the client.
    pub fn end_raining(&mut self) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::EndRaining,
            value: f32::default(),
        });
    }

    /// Sends the set rain level event to the client.
    pub fn set_rain_level(&mut self, level: f32) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::RainLevelChange,
            value: level.clamp(WEATHER_LEVEL_MIN, WEATHER_LEVEL_MAX),
        });
    }

    /// Sends the set thunder level event to the client.
    pub fn set_thunder_level(&mut self, level: f32) {
        self.write_packet(&GameStateChangeS2c {
            kind: GameEventKind::ThunderLevelChange,
            value: level.clamp(WEATHER_LEVEL_MIN, WEATHER_LEVEL_MAX),
        });
    }

    /// Sends weather level events to the client.
    pub fn set_weather(&mut self, weather: &Weather) {
        if let Some(rain_level) = weather.rain {
            self.set_rain_level(rain_level)
        }

        if let Some(thunder_level) = weather.thunder {
            self.set_thunder_level(thunder_level)
        }
    }
}

fn handle_weather_begin_per_instance(mut query: Query<&mut Instance, Added<Weather>>) {
    query.par_iter_mut().for_each_mut(|mut instance| {
        instance.begin_raining();
    });
}

fn handle_weather_end_per_instance(
    mut query: Query<&mut Instance>,
    mut removed: RemovedComponents<Weather>,
) {
    removed.iter().for_each(|entity| {
        if let Ok(mut instance) = query.get_mut(entity) {
            instance.end_raining();
        }
    })
}

fn handle_weather_change_per_instance(
    mut query: Query<(&mut Instance, &Weather), Changed<Weather>>,
) {
    query
        .par_iter_mut()
        .for_each_mut(|(mut instance, weather)| {
            instance.set_weather(weather);
        });
}

pub(crate) fn update_weather() -> SystemConfigs {
    (
        handle_weather_begin_per_instance,
        handle_weather_end_per_instance,
        handle_weather_change_per_instance,
    )
        .into_configs()
}

#[cfg(test)]
mod test {
    use anyhow::Ok;
    use bevy_app::App;
    use valence_protocol::packet::S2cPlayPacket;

    use super::*;
    use crate::unit_test::util::scenario_single_client;
    use crate::{assert_packet_count, assert_packet_order};

    #[test]
    fn test_weather_events_emit() -> anyhow::Result<()> {
        let mut app = App::new();
        let (_, mut client_helper) = scenario_single_client(&mut app);

        // Process a tick to get past the "on join" logic.
        app.update();
        client_helper.clear_sent();

        // Insert a weather component to the instance
        let weather = Weather {
            rain: Some(1_f32),
            thunder: Some(1_f32),
        };

        let instance_ent = app
            .world
            .iter_entities()
            .find(|e| e.contains::<Instance>())
            .expect("could not find instance")
            .id();

        app.world.entity_mut(instance_ent).insert(weather);

        // Make sure that all event packets are sent
        for _ in 0..3 {
            app.update();
        }

        // Remove the weather component from the instance
        app.world.entity_mut(instance_ent).remove::<Weather>();
        app.update();

        // Make assertions
        let sent_packets = client_helper.collect_sent()?;

        assert_packet_count!(sent_packets, 4, S2cPlayPacket::GameStateChangeS2c(_));

        assert_packet_order!(
            sent_packets,
            S2cPlayPacket::GameStateChangeS2c(GameStateChangeS2c {
                kind: GameEventKind::BeginRaining,
                value: _
            }),
            S2cPlayPacket::GameStateChangeS2c(GameStateChangeS2c {
                kind: GameEventKind::RainLevelChange,
                value: _
            }),
            S2cPlayPacket::GameStateChangeS2c(GameStateChangeS2c {
                kind: GameEventKind::ThunderLevelChange,
                value: _
            }),
            S2cPlayPacket::GameStateChangeS2c(GameStateChangeS2c {
                kind: GameEventKind::EndRaining,
                value: _
            })
        );

        Ok(())
    }
}
