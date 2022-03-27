

use core::num;
use std::{convert::TryInto, ops::Add};

use rand_pcg::Mcg128Xsl64;
use rand_core::RngCore;

use crate::{utils::{Arr2d, TWO_POW_15_F32, TWO_POW_31, Vec2, ReducedArrayWrapper, ColorMapArray}, erosion::erode, settings::{Settings, GenerationOptions}};






pub fn generate_f32(h: f32, rng: &mut Mcg128Xsl64) -> f32 {
    let a = (generate_f32_2(h, rng) + h) / 2.0;
    println!("{}", a);
    a
}

pub fn generate_f32_2(h: f32, rng: &mut Mcg128Xsl64) -> f32 {
    let n = (rng.next_u32() >> 16) as f32 - TWO_POW_15_F32;

    ((n as f32) / TWO_POW_15_F32).tanh() * h
}


pub fn normalize(n: f32) -> f32 {
    f32::exp(- (n * n) / (TWO_POW_31 as f32))
}

pub fn generate_noise(w: usize, id: usize, h: f32, max_height: f32, rng: &mut Mcg128Xsl64) -> f32 {
    let n = (rng.next_u32() >> 16) as f32 - TWO_POW_15_F32;

    let noise = ((n as f32) / TWO_POW_15_F32);
    
    noise * get_multiplier(w, id, h, max_height)    

}

pub fn get_multiplier(w: usize, id: usize, h: f32, max_height: f32) -> f32 {
    h * (id as f32) * max_height / w as f32

}


pub fn diamond_square_2(arr: &Arr2d<f32>, output: &mut Arr2d<f32>, power_of_two: usize, reduced_output: &mut Arr2d<f32>, scaling: usize, mut h: f32,
    n_iteration_difference: u32, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray, settings: &GenerationOptions) {

    assert_eq!(arr.get_height(), arr.get_width());
    assert_eq!(output.get_height(), output.get_width());

    let input_w = arr.get_height();

    let w = output.get_height();
    
    
    for x in 0..input_w {
        for y in 0..input_w {
            if let Some(output_point_value) = output.get_mut(x * scaling, y * scaling) {
                if let Some(input_point_value) = arr.get(x, y) {
                    *output_point_value = *input_point_value;

                    // println!("{} {}  {} {}", x, y, x*scaling, y*scaling);

                }
            }
        }
    }

    println!("{}", output.get_width());


    let mut number_of_step_to_skip = settings.template_power_of_two;


    let mut i = w - 1;

    let reduced_output_step = 2_usize.pow(n_iteration_difference);

    let mut id: usize;
    let mut offset: usize;

    let mut sum: f32;
    let mut n: usize;


    while i > 1 {

        id = i / 2;

        if number_of_step_to_skip == 0 {
            // diamond step
            for x in (id..w).step_by(i) {
                for y in (id..w).step_by(i) {

                    let center_value = (
                        output.get(x - id, y - id).unwrap()   // équivaut à output[x - id, y - id] en python
                        + output.get(x - id, y + id).unwrap()
                        + output.get(x + id, y + id).unwrap()
                        + output.get(x + id, y - id).unwrap()
                    ) / 4.0;

                    *(output.get_mut(x, y).unwrap()) = center_value + generate_noise(w, id, h, settings.max_terrain_height, rng);
                }
            }

            // square step
            offset = 0;
            for x in (0..w).step_by(id) {
                offset = if offset == 0 {id} else {0};

                for y in (offset..w).step_by(i) {
                    sum = 0.0;
                    n = 0;

                    if x >= id {sum += output.get(x - id, y).unwrap(); n += 1}
                    if x + id < w {sum += output.get(x + id, y).unwrap(); n += 1}
                    if y >= id {sum += output.get(x, y - id).unwrap(); n += 1}
                    if y + id < w {sum += output.get(x, y + id).unwrap(); n += 1}

                    *(output.get_mut(x, y).unwrap()) = sum / (n as f32) + generate_noise(w, id, h, settings.max_terrain_height, rng);
                };
            }
        }

        else {
            number_of_step_to_skip -= 1;
        }

        i = id;
        
        if i == reduced_output_step {
            erode(&mut ReducedArrayWrapper::new(output, power_of_two as u32, power_of_two as u32 - n_iteration_difference), rng, color_map, settings);
            h = 0.0;
        }


    }

    for (rx, x) in (0..w).step_by(reduced_output_step).enumerate() {
        for (ry, y) in (0..w).step_by(reduced_output_step).enumerate() {
            *reduced_output.get_mut(rx, ry).unwrap() = *output.get(x, y).unwrap();
        }
    }


    println!("generation done.");




}




pub fn generate_demisphere_heightmap(arr: &mut Arr2d<f32>, w: usize) {
    let radius = w / 2 - 1;
    let radius_squared = radius.pow(2) as f32;


    let center: Vec2<f32> = Vec2 {x: (radius + 1) as f32, y: (radius + 1) as f32};

    let mut squared_distance_from_center: f32;

    for x in 0..w {
        for y in 0..w {
            squared_distance_from_center = (x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2);
            
            squared_distance_from_center = f32::min(radius_squared, squared_distance_from_center);

            *( arr.get_mut(x, y).unwrap() ) = (radius_squared - squared_distance_from_center).sqrt();

        }
    }


}




/*
fonction diamant-carré (tableau t)
    h = t.coté()
    t[0, 0] = rand(-h, h)  /* initialisation des coins */
    t[0, h-1] = rand(-h, h)
    t[h-1, h-1] = rand(-h, h)
    t[h-1, 0] = rand(-h, h)
    i = h-1
    tant_que i > 1
        id = i/2
        pour x allant de id à h avec un pas de i  /* début de la phase du diamant */
            pour y allant de id à h avec un pas de i
                moyenne = (t[x - id, y - id] + t[x - id, y + id] + t[x + id, y + id] + t[x + id, y - id]) / 4
                t[x, y] = moyenne + rand(-id, id)    
            fin pour
        fin pour
        décalage = 0
        pour x allant de 0 à h avec un pas de id  /* début de la phase du carré */
            si décalage = 0 alors
                décalage = id
            sinon
                décalage = 0
            fin si
            pour y allant de décalage à h avec un pas de i
                somme = 0
                n = 0
                si x >= id alors
                    somme = somme + t[x - id, y]
                    n = n+1
                fin si
                si x + id < h alors
                    somme = somme + t[x + id, y]
                    n = n+1
                fin si
                si y >= id alors
                    somme = somme + t[x, y - id]
                    n = n+1
                fin si
                si y + id < h alors
                    somme = somme + t[x, y + id]
                    n = n+1
                fin si
                t[x, y] = somme / n + rand(-id, id)
            fin pour
        fin pour
        i = id
    fin tant_que
fin fonction



*/



