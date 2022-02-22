use rand_pcg::Mcg128Xsl64;

use crate::utils::{ReducedArrayWrapper, generate_in_interval, linear_interpolation, bilinear_interpolation, ColorMapArray};







pub fn erode(heightmap: &mut ReducedArrayWrapper<f32>, n_iteration: u32, ref_height: f32, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray) {

    println!("starting erosion");

    let mut x: usize;
    let mut y: usize;

    let mut pos: [f32; 2];

    let max = 2_u16.pow(heightmap.get_reduced_n());

    let mut water_remaining: u16;
    let mut sediment_stocked: f32;

    let mut current_height: f32;
    let mut current_height_mut: &mut f32;

    let mut next_height: f32;
    let mut current_lowest_relativ_position: [usize; 2];

    let mut vx: f32;
    let mut vy: f32;

    let mut speed: f32;

    let mut h2: f32;

    let mut slope_left: f32;
    let mut slope_right: f32;
    let mut slope_middle_axis_y: f32;
    let mut slope_middle_axis_x: f32;

    let mut capacity: f32;

    let mut pixel: (&mut f32, &mut f32, &mut f32);


    for _ in 0..n_iteration {
        water_remaining = 200;
        sediment_stocked = 0.0;
        capacity = 0.08;

        speed = 0.0;

        current_lowest_relativ_position = [0, 0];

        x = generate_in_interval(max, rng) as usize;
        y = generate_in_interval(max, rng) as usize;

        pos = [x as f32, y as f32];

        vx = 0.0;
        vy = 0.0;
        
        while water_remaining > 0 {
            
            vx *= 0.9;
            vy *= 0.9;

            // slope_left = heightmap.get(x, y + 1).unwrap() - heightmap.get(x, y).unwrap();
            // slope_right = heightmap.get(x + 1, y + 1).unwrap() - heightmap.get(x + 1, y).unwrap();

            // slope_middle_axis_y = (slope_left + slope_right) / 2.0;
            // slope_middle_axis_x = linear_interpolation(0.5, *heightmap.get(x, y + 1).unwrap(), *heightmap.get(x, y).unwrap()) - linear_interpolation(0.5, *heightmap.get(x + 1, y + 1).unwrap(), *heightmap.get(x + 1, y).unwrap());

            current_height = *heightmap.get(x, y).unwrap();
            for dx in 0..=1 {
                for dy in 0..=1 {
                    if *heightmap.get(x + dx, y + dy).unwrap() <= current_height {
                        current_lowest_relativ_position = [dx, dy];
                    }
                }
            }


            vx += (current_lowest_relativ_position[0] as f32) * 2.0 - 1.0;
            vy += (current_lowest_relativ_position[1] as f32) * 2.0 - 1.0;

            speed = (vx * vx + vy * vy).sqrt();
            
            current_height_mut = heightmap.get_mut(x, y).unwrap();

            sediment_stocked += 2.0;
            *current_height_mut -= 2.0;

            if sediment_stocked > capacity * water_remaining as f32 {
                *current_height_mut += sediment_stocked - capacity * water_remaining as f32;
            }

            pos[0] = (pos[0] + vx / speed).clamp(0.0, max as f32 - 2.0);
            pos[1] = (pos[1] + vy / speed).clamp(0.0, max as f32 - 2.0);

            x = pos[0] as usize;
            y = pos[1] as usize;


            pixel = color_map.get_mut_pixel(heightmap.convert(y), heightmap.convert(x)).unwrap();
            *pixel.0 += 1.0 / n_iteration as f32 * 1000.0;
            *pixel.1 += (water_remaining as f32) / 200.0 / n_iteration as f32 * 1000.0;

            water_remaining -= 1;


        }







        





    }
}






