
use rand_pcg::Mcg128Xsl64;
use rand_core::RngCore;

use crate::{utils::{Arr2d, TWO_POW_15_F32, Vec2, ReducedArrayWrapper, ColorMapArray}, erosion::erode, settings::{GenerationOptions}};



// genere un nombre aleatoire dans un interval adequat pour le diamondsquare
pub fn generate_noise(w: usize, id: usize, h: f32, max_height: f32, rng: &mut Mcg128Xsl64) -> f32 {

    let n = (rng.next_u32() >> 16) as f32 - TWO_POW_15_F32;

    let noise = (n as f32) / TWO_POW_15_F32;
    
    noise * get_multiplier(w, id, h, max_height)    

}

// trouve l'interval
pub fn get_multiplier(w: usize, id: usize, h: f32, max_height: f32) -> f32 {
    h * (id as f32) * max_height / w as f32

}


pub fn diamond_square_2(arr: &Arr2d<f32>, output: &mut Arr2d<f32>, power_of_two: usize, reduced_output: &mut Arr2d<f32>, scaling: usize, mut h: f32,
    n_iteration_difference: u32, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray, settings: &GenerationOptions) {

    assert_eq!(arr.get_height(), arr.get_width());
    assert_eq!(output.get_height(), output.get_width());

    let input_w = arr.get_height();  // taille du tableau d'entree (template)

    let w = output.get_height();  // taille du tableau de sortie
    
    // rempli le tableau de sortie avec les valeurs de la template
    for x in 0..input_w {
        for y in 0..input_w {
            if let Some(output_point_value) = output.get_mut(x * scaling, y * scaling) {
                if let Some(input_point_value) = arr.get(x, y) {
                    *output_point_value = *input_point_value;

                }
            }
        }
    }

    println!("{}", output.get_width());

    let mut number_of_step_to_skip = settings.template_power_of_two; // nombre d'etape a sauter pour ne pas perdre les donnees de la template


    let mut i = w - 1;  // 2^n

    let reduced_output_step = 2_usize.pow(n_iteration_difference);  // distance entre les valeurs du tableau reduit

    let mut id: usize;  // 2^(n-1) 
    let mut offset: usize;  // variable utilitaire pour l'etape square

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

                    // équivaut à "output[x, y] = center_value + generate_noise(w, id, h, settings.max_terrain_height, rng)" en python
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



