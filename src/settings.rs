use config::{ConfigError, Config, File, Environment};
use std::env;
use serde_derive::Deserialize;


#[derive(Debug, Deserialize)]
pub struct LaunchOptions {
    pub generate_sky_heightmap: bool,
    pub generate_sky_texture: bool,
    pub generate_terrain_heightmap: bool,
    pub generate_terrain_texture: bool
}


#[derive(Debug, Deserialize)]
pub struct GenerationOptions {
    pub seed: u64,

    pub max_terrain_height: f32,
    pub irregularity: f32,

    pub sun_angle: f32,
    pub sun_size: f32,
    pub atmosphere_radius: f32,
    pub planet_radius: f32,

    pub ambient_sky_light: f32,

    pub shadow_direction: u8,

    pub number_of_erosion_iterations: u32,
    pub inertia: f64,
    pub radius: u8,
    pub capacity_factor: f64
}


#[derive(Debug, Deserialize)]
pub struct Settings {
    pub launch_options: LaunchOptions,
    pub generation_options: GenerationOptions
}


impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("Settings"))?;

        s.try_into()
    }
}









