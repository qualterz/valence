use bevy_ecs::prelude::*;

pub const MIN_WEATHER_LEVEL: f32 = 0_f32;
pub const MAX_WEATHER_LEVEL: f32 = 1_f32;

/// The weather state representation.
#[derive(Component)]
pub struct Weather {
    /// Contains the raining level.
    /// Should be between [`MIN_WEATHER_LEVEL`] and [`MAX_WEATHER_LEVEL`].
    ///
    /// The [`None`] value means no raining event.
    pub raining: Option<f32>,
    /// Contains the thunder level.
    /// Should be between [`MIN_WEATHER_LEVEL`] and [`MAX_WEATHER_LEVEL`].
    ///
    /// The [`None`] value means no thunder.
    pub thunder: Option<f32>,
}
