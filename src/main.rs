#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(unused_variables)]


use std::{f32::INFINITY, process::Command, io::Write};

mod diamondsquare;
mod sky_generation;
mod utils;
mod image_generation;
mod terrain_texture_generation;
mod settings;
mod gradient_map_generation;
mod erosion;

use image::ImageBuffer;
use image_generation::generate_test_image;
use rand_pcg::Mcg128Xsl64;
use terrain_texture_generation::generate_terrain_texture;

use crate::{diamondsquare::{diamond_square_2, generate_demisphere_heightmap}, image_generation::{generate_colormap_image, generate_heightmap_image}, sky_generation::{LightSpectrum, generate_sky_colormap}, utils::{Arr2d, ColorMapArray, PI}, gradient_map_generation::generate_gradient_map, erosion::erode};



use crate::utils::ReducedArrayWrapper;



fn main() {

    let mut command = String::new();
    while command != "quit" {
        command.clear();

        print!("> ");
        std::io::stdout().flush().expect("welp");
        std::io::stdin().read_line(&mut command).unwrap_or_else(|_| {println!("Error - invalid input"); 0});
        command = command.trim().to_lowercase();

        if command == "display" {
            display()
        } else if command == "generate" {
            generate()
        } else if command != "quit" {
            println!("Error - unknown command: \"{}\"", command)
        }

    }
}


fn generate() {

    let settings = settings::Settings::new().expect("did not work welp");
    println!("{:?}", settings);
    if settings.launch_options.generate_terrain_heightmap {
        let template_img = image::io::Reader::open("template.png").expect("welp").decode().expect("welp");

        let n: usize = 10;
    
        let mut rng = Mcg128Xsl64::new(settings.generation_options.seed as u128);
    
        let template = Arr2d::from_dynamic_image(template_img, settings.generation_options.max_terrain_height);
    
        let scaling = usize::pow(2, 4);
        let w = 2_usize.pow(n as u32) + 1;

        let reduced_w = 2_usize.pow(n as u32 - 1) + 1;

        let mut terrain_heightmap: Arr2d<f32> = Arr2d::init_with_value(w, w, 10.0);
        let mut reduced_terrain_heightmap: Arr2d<f32> = Arr2d::init_with_value(reduced_w, reduced_w, 10.0);

        let mut terrain_colormap: ColorMapArray = ColorMapArray::new_empty(w, w);

        diamond_square_2(&template, &mut terrain_heightmap,
             n, &mut reduced_terrain_heightmap, scaling, settings.generation_options.irregularity, 1, &mut rng, &mut terrain_colormap);

        //erode(&mut ReducedArrayWrapper::new(&mut reduced_terrain_heightmap, n as u32 - 1, n as u32 - 1), 200000, settings.generation_options.max_terrain_height, &mut rng);
            
        image::save_buffer("heightmap.png",
        &generate_heightmap_image(&reduced_terrain_heightmap, false),
        reduced_w as u32, reduced_w as u32, image::ColorType::Rgb8).expect("welp");
    
        if settings.launch_options.generate_terrain_texture {

            let mut terrain_gradientmap: Arr2d<f32> = Arr2d::init_with_value(reduced_w, reduced_w, 10.0);
            generate_gradient_map(&reduced_terrain_heightmap, &mut terrain_gradientmap, settings.generation_options.max_terrain_height, reduced_w, 2, 5);

            image::save_buffer("gradientmap.png",
            &generate_heightmap_image(&terrain_gradientmap, false),
            reduced_w as u32, reduced_w as u32, image::ColorType::Rgb8).expect("welp");
    
            let mut buffer: Arr2d<f32> = Arr2d::zeros(w, w);
            generate_terrain_texture(&mut terrain_colormap, &terrain_heightmap, &terrain_gradientmap, w.div_euclid(reduced_w) + 1, w, settings.generation_options.max_terrain_height * 2_f32.powi(1),
                settings.generation_options.shadow_direction, settings.generation_options.sun_angle * PI / 180.0,  &mut rng);

            image::save_buffer("colormap.png",
            &generate_colormap_image(&terrain_colormap, w - 1), w as u32 - 1, w as u32 - 1, image::ColorType::Rgb8)
            .expect("welp");
        }


    }


    let demisphere_radius = 20 * 4;
    let demisphere_width = demisphere_radius * 8 * 2 + 1;

    let demisphere_heightmap_width = 2_usize.pow(7);
    
    if settings.launch_options.generate_sky_heightmap {
        let mut demisphere_heightmap: Arr2d<f32> = Arr2d::zeros(demisphere_heightmap_width, demisphere_heightmap_width);
        generate_demisphere_heightmap(&mut demisphere_heightmap, demisphere_heightmap_width);

        image::save_buffer("demisphere_heightmap.png",
        &generate_heightmap_image(&demisphere_heightmap, false), demisphere_heightmap_width as u32, demisphere_heightmap_width as u32, image::ColorType::Rgb8)
        .expect("welp");

    }

    
    if settings.launch_options.generate_sky_texture {
        let mut demisphere_colormap: ColorMapArray = ColorMapArray::new_empty(demisphere_width, demisphere_width);

        let sun_angle = settings.generation_options.sun_angle * PI / 180.0;
    
        let mut incident_light_spectrum = LightSpectrum::new();
    
        generate_sky_colormap(&mut demisphere_colormap, demisphere_width,
            settings.generation_options.planet_radius, settings.generation_options.atmosphere_radius, &mut incident_light_spectrum, sun_angle,
            settings.generation_options.sun_size, settings.generation_options.ambient_sky_light
        );
    
        image::save_buffer("demisphere_colormap.png",
         &generate_colormap_image(&demisphere_colormap, demisphere_width), demisphere_width as u32, demisphere_width as u32, image::ColorType::Rgb8)
         .expect("welp");
        
    }

}


fn display() {
    let output = Command::new(r"C:\Users\Hélène Le Berre\rp\mapgeneration\displayer\launcher.cmd")
        // .arg("cmd")
        // .arg(r"C:\Users\Hélène Le Berre\rp\mapgeneration\displayer\main.py")
        .output()
        .expect("failed to execute");
}


