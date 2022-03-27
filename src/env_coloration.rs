use std::convert::TryInto;

use rand_pcg::Mcg128Xsl64;

use crate::{utils::{Arr2d, ColorMapArray, Vec2, next_random_number, ReducedArrayWrapper}, settings::GenerationOptions, erosion::{RelativePoint, compute_points_in_range, compute_height_and_slopes, PointsInRangeIterator}};




pub fn erode(heightmap: &ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray, settings: &GenerationOptions) {

    println!("starting erosion (environment generation), number of iterations: {}", settings.number_of_erosion_iterations);

    const initial_lifetime: usize = 30;
    
    let mut pos_x: f64;
    let mut pos_y: f64;

    let mut arr_pos_x: usize;
    let mut arr_pos_y: usize;

    let mut offset_x: f64;
    let mut offset_y: f64;

    let mut height_00: f64;
    let mut height_01: f64;
    let mut height_10: f64;
    let mut height_11: f64;

    let mut height: f64 = 42.0;

    let mut horizontal_slope: f64 = 0.0;
    let mut vertical_slope: f64 = 0.0;

    let mut velocity: Vec2<f64>;

    let mut quantity_of_water: f64;
    let mut capacity: f64;
    let min_capacity: f64 = 0.01;

    let mut new_arr_pos_x: usize;
    let mut new_arr_pos_y: usize;
    let mut new_offset_x: f64;
    let mut new_offset_y: f64;

    let mut new_height: f64 = 42.0;
    let mut height_difference: f64;

    let array_max_index = heightmap.get_reduced_width() as u32 - 1;

    let mut to_depose: f64;
    let mut to_erode: f64;
    let mut sediment_eroded: f64;

    // // let mut pixel;

    let mut sediment_stocked: f64;

    let mut height_ref: &f32;

    let mut speed = 1.0;

    let mut relative_points_table: Vec<RelativePoint> = Vec::with_capacity((settings.radius.pow(2) * 4) as usize);
    compute_points_in_range(&mut relative_points_table, settings.radius as i32);

    for iteration in 0..settings.number_of_erosion_iterations {

        if (iteration + 1) % 50000 == 0 {
            println!("{} iterations done.", iteration + 1)
        }


        pos_x = next_random_number(array_max_index as u64, rng).try_into().expect("Error while converting");
        pos_y = next_random_number(array_max_index as u64, rng).try_into().expect("Error while converting");

        velocity = Vec2 {x: 0.0, y: 0.0};
        quantity_of_water = 1.0;

        sediment_stocked = 0.0;

        for current_lifetime in (0..initial_lifetime).rev() {

            arr_pos_x = pos_x as usize;
            arr_pos_y = pos_y as usize;

            offset_x = pos_x - arr_pos_x as f64;
            offset_y = pos_y - arr_pos_y as f64;

            compute_height_and_slopes(heightmap, offset_x, offset_y, arr_pos_x, arr_pos_y, &mut height, &mut horizontal_slope, &mut vertical_slope);

            velocity.x = velocity.x * settings.inertia + horizontal_slope * (1.0 - settings.inertia);
            velocity.y = velocity.y * settings.inertia + vertical_slope * (1.0 - settings.inertia);

            match velocity.normalize_ip() {
                Ok(()) => (),
                Err(()) => break
            }

            pos_x -= velocity.x;
            pos_y -= velocity.y;

            if pos_x < 0.0 || pos_y < 0.0 || pos_x > (array_max_index - 1) as f64 || pos_y > (array_max_index - 1) as f64 { break }

            new_arr_pos_x = pos_x as usize;
            new_arr_pos_y = pos_y as usize;

            new_offset_x = pos_x - new_arr_pos_x as f64;
            new_offset_y = pos_y - new_arr_pos_y as f64;
            
            compute_height_and_slopes(heightmap, new_offset_x, new_offset_y, new_arr_pos_x, new_arr_pos_y, &mut new_height, &mut horizontal_slope, &mut vertical_slope);


            height_difference = new_height - height;

            to_erode = 0.0;
            to_depose = 0.0;

            if height_difference > 0.0 {
                to_depose = f64::min(height_difference, sediment_stocked);

            } else {
                capacity = f64::max(min_capacity, quantity_of_water * -height_difference * settings.capacity_factor);

                if capacity < sediment_stocked {
                    to_depose = (sediment_stocked - capacity) * 0.3

                } else {
                    to_erode = f64::min(-height_difference, (capacity - sediment_stocked) * 0.3);

                    for point in PointsInRangeIterator::new(&relative_points_table, array_max_index as usize + 1, arr_pos_x as i32, arr_pos_y as i32) {
                        height_ref = heightmap.get(point.x, point.y).unwrap();
                        sediment_eroded = f64::min(to_erode * point.weight, *height_ref as f64);

                        sediment_stocked += sediment_eroded;
                    }

                }

            }
            assert!(to_depose >= 0.0);
            assert!(to_erode >= 0.0);
            sediment_stocked -= to_depose;
            assert!(sediment_stocked >= 0.0);

            quantity_of_water *= 0.95;
            speed = f64::sqrt(speed * speed + height_difference.abs() / settings.max_terrain_height as f64 * 1.0);

            
            // // pixel = color_map.get_mut_pixel(heightmap.convert(arr_pos_y), heightmap.convert(arr_pos_x)).unwrap();
            // // *pixel.0 += 1.0 / settings.number_of_erosion_iterations as f32 * 1000.0;
            // // *pixel.1 += (current_lifetime as f32) / 200.0 / settings.number_of_erosion_iterations as f32 * 1000.0;
         
        }

    }

}




