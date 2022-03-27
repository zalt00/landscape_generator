use core::num;
use std::convert::TryInto;

use rand_core::RngCore;
use rand_pcg::Mcg128Xsl64;

use crate::{utils::{Arr2d, ColorMapArray, HALF_PI, PI, TWO_POW_15_F32, bilinear_interpolation}, settings::GenerationOptions};

pub fn generate_f32_2(h: f32, rng: &mut Mcg128Xsl64) -> f32 {
    let n = (rng.next_u32() >> 16) as f32 - TWO_POW_15_F32;

    ((n as f32) / TWO_POW_15_F32) * h
}

pub fn coin_flip_with_probability(p: f32, rng: &mut Mcg128Xsl64) -> bool {
    generate_f32_2(1.0, rng).abs() <= p
}

pub fn generate_terrain_texture(output: &mut ColorMapArray, heightmap: &Arr2d<f32>, gradientmap: &Arr2d<f32>, scale_divisor: usize,
     width: usize, ref_height: f32, shadow_direction: u8, sun_angle: f32, rng: &mut Mcg128Xsl64) {

    let mut noise: f32;

    add_environment_coloration(output, heightmap, gradientmap, width, scale_divisor, ref_height, rng);

    for x in 0..width {
        for y in 0..width {

            if let Some(pixel) = output.get_mut_pixel(y, x) {

                noise = generate_f32_2(0.01, rng);

                *pixel.0 += noise;
                *pixel.1 += noise;
                *pixel.2 += noise;
            }


        }
    }

    add_shadow(output, heightmap, width, shadow_direction, sun_angle, ref_height);

}

fn above_threshold(v: f32, t: f32) -> f32 {
    if v >= t {
        1.0
    } else {
        v
    }
}

fn below_threshold(v: f32, t: f32) -> f32 {
    if v <= t {
        0.0
    } else {
        v
    }
}



pub fn add_environment_coloration2(output: &mut ColorMapArray, heightmap: &Arr2d<f32>, width: usize, settings: &GenerationOptions, rng: &mut Mcg128Xsl64) {

    



}




pub fn add_environment_coloration(output: &mut ColorMapArray, heightmap: &Arr2d<f32>,
     gradientmap: &Arr2d<f32>, width: usize, scale_divisor: usize, ref_height: f32, rng: &mut Mcg128Xsl64) {
    
    let mut sum_of_heights: f32;
    let mut n: f32;
    let mut current_height: &f32;

    let color_for_environements: [[f32;3]; 3] = [
        [87.0 / 255.0, 93.0 / 255.0, 98.0 / 255.0],  // roche sombre
        [185.0 / 255.0, 180.0 / 255.0, 171.0 / 255.0], // roche
        [97.0 / 255.0, 109.0 / 255.0, 74.0 / 255.0] // vegetation
    ];

    let mut current_amounts: [f32; 3];

    let mut snowy = false;

    let mut gradient: f32;
    let mut max_gradient = -78_f32;
    let mut min_gradient = 78_f32;

    let ref_height_squared = ref_height.powi(2);

    // let mut x2_result: Result<usize, _>;
    // let mut y2_result: Result<usize, _>;

    let mut color: [f32; 3] = [0.0, 0.0, 0.0];

    let mut scaled_height: f32;

    let gradient_width = 40;

    let mut previous_value: f32;

    let mut v_: f32;

    // for x in 0..width {
    //     previous_value = 0.0;
    //     for y in 0..width {
    //         if let Some(value) = buffer.get_mut(x, y) {
    //             if let Some(height) = heightmap.get(x, y) {
    //                 scaled_height = *height;

    //                 *value = scaled_height + previous_value;

    //                 if let Some(prev_height) = heightmap.get(x, y.checked_sub(gradient_width * 2).unwrap_or(999999)) {
    //                     *value -= prev_height;
    //                 }

    //                 previous_value = *value;
    //             }
    //         }
    //     }
    // }
    
    let mut x_gradientmap: usize;
    let mut y_gradientmap: usize;

    let mut gradient_total: f32 = 0.0;
    let mut number_of_points: f32 = 0.0;

    for x in 0..width {
        for y in 0..width {

            if let Some(pixel) = output.get_mut_pixel(y, x) {

                current_height = heightmap.get(x, y).unwrap();
                scaled_height = current_height / ref_height;

                x_gradientmap = x.div_euclid(scale_divisor);
                y_gradientmap = y.div_euclid(scale_divisor);

                if let Some(v11) = gradientmap.get(x_gradientmap + 1, y_gradientmap + 1) {
                    gradient = bilinear_interpolation(
                        (x - x_gradientmap * scale_divisor) as f32 / scale_divisor as f32,
                        (y - y_gradientmap * scale_divisor) as f32 / scale_divisor as f32,
                        *gradientmap.get(x_gradientmap, y_gradientmap).unwrap_or_else(|| {println!("setting gradient to default at {} {}", x_gradientmap, y_gradientmap); &0.0}),
                        *gradientmap.get(x_gradientmap, y_gradientmap + 1).unwrap_or_else(|| {println!("setting gradient to default at {} {}", x_gradientmap, y_gradientmap + 1); &0.0}), 
                        *gradientmap.get(x_gradientmap + 1, y_gradientmap).unwrap_or_else(|| {println!("setting gradient to default at {} {}", x_gradientmap + 1, y_gradientmap); &0.0}), 
                        *v11)
                } else {
                    gradient = *gradientmap.get(x_gradientmap, y_gradientmap).unwrap_or_else(|| {println!("setting gradient to default at {} {}", x_gradientmap, y_gradientmap); &0.0});
                }

                gradient += generate_f32_2(0.1, rng);
                gradient *= 2.0;

                // gradient = sum_of_heights / n / ref_height * width as f32 / 17.0 / 8.0; // moyenne des différences, mise à l'échelle
                
                // sum_of_heights = *buffer.get(x, (y + gradient_width).min(width - 1)).unwrap();
                
                // if y > 20 && false {
                //     assert_eq!(sum_of_heights, {

                //         let mut v_ = 0_f32;

                //         for dy in -(gradient_width as i16)..gradient_width as i16 {
                //             v_ += heightmap.get(x, (y as i16 + dy) as usize).unwrap()
                //         }
                //         v_
    
                //     });                }




                // gradient = (sum_of_heights / (gradient_width * 2) as f32 - current_height).abs() / ref_height * width as f32 / 10.0 / 8.0;
                
                gradient_total += gradient;
                number_of_points += 1.0;

                current_amounts = [0.0, 0.0, 0.0];

                current_amounts[1] += gradient.round();
                current_amounts[2] += (1.0 - gradient).round();

                if current_amounts[1] >= 0.0 {
                    current_amounts[0] += generate_f32_2(0.2 + ((scaled_height * PI * 50.0).sin() + (scaled_height * PI * 30.0 + 1.0).sin() + (scaled_height * PI * 20.0 + 2.0).sin()) / 30.0, rng).abs();
                    current_amounts[1] -= current_amounts[0];
                }

                color = [0.0, 0.0, 0.0];
                for channel in 0..=2 {
                    for j in 0..color_for_environements.len() {
                        color[channel] += color_for_environements[j][channel] * current_amounts[j].min(1.0).max(0.0);
                    }
                }

                *pixel.0 += color[0] * 0.0 + 1.0;
                *pixel.1 += color[1] * 0.0 + 1.0;
                *pixel.2 += color[2] * 0.0 + 1.0;


            }


        }
    }

    println!("gradient mean: {}", gradient_total / number_of_points);
}


fn get_exposition(slope: f32, sun_angle: f32) -> f32 {
    let CBA = slope.atan().abs();
    let CBF = CBA + sun_angle;
    CBF.sin().abs()
}

fn get_pos_with_direction(i: usize, j: usize, direction: u8, width: usize) -> [usize; 2] {
    match direction {
        0 => {
            [i, j]
        }
        1 => {
            [j, i]
        }
        2 => {
            [width - i - 1, j]
        }
        _ => {
            [j, width - i - 1]
        }
    }
}

fn checked_get_pos_with_direction(i: i32, j: i32, direction: u8, width: usize) -> Option<[usize; 2]> {
    if i >= width as i32 || i < 0 || j >= width as i32 || j < 0{
        None
    } else {
        Some(get_pos_with_direction(i as usize, j as usize, direction, width))
    }


}

//67 104 156
pub fn add_shadow(output: &mut ColorMapArray, heightmap: &Arr2d<f32>, width: usize, direction: u8, angle: f32, ref_height: f32) {

    let mut current_max_per_line: Vec<f32> = vec![0.0_f32;width];
    let mut pos: [usize; 2];

    let coef = ((angle  % HALF_PI).tan()).abs() * ref_height / width as f32;

    let mut pixel;
    let mut exposition: f32;
    let mut local_height: f32;

    let mut slope: f32;
    let mut exposition_sum: f32;
    let mut n: u8;


    for i in 0..width {
        for j in 0..width {

            exposition_sum = 0.0;
            n = 0;

            pos = get_pos_with_direction(i, j, direction, width);

            pixel = output.get_mut_pixel(pos[1], pos[0]).unwrap();
            local_height = *heightmap.get(pos[0], pos[1]).unwrap(); 

            exposition = 1.0;

            if local_height >= current_max_per_line[j] {
                current_max_per_line[j] = local_height;

                for di in [-1, 1] {
                    if let Some(pos) = checked_get_pos_with_direction(i as i32 + di, j as i32, direction, width) {
                        if let Some(height) = heightmap.get(pos[0], pos[1]) {
                            exposition_sum += get_exposition((height - local_height) * di as f32 * ref_height / width as f32, angle);
                            n += 1;
                        }
                    }
                }
                exposition = 1.0 - (1.0 - exposition_sum / n as f32) * 0.6;


            } else {
                exposition -= (local_height - current_max_per_line[j]).abs() / ref_height * 0.1 + 0.60;
            }

            *pixel.0 -= exposition;
            *pixel.1 -= exposition;
            *pixel.2 -= exposition;

            if i == 0 {
                *pixel.0 += 0.5;
            }

            current_max_per_line[j] -= coef;








        }
    }


}






















