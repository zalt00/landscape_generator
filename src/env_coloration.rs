use std::convert::TryInto;

use rand_pcg::Mcg128Xsl64;

use crate::{utils::{Arr2d, ColorMapArray, Vec2, next_random_number, ReducedArrayWrapper, bilinear_interpolation}, settings::GenerationOptions, erosion::{RelativePoint, compute_points_in_range, compute_height_and_slopes, PointsInRangeIterator}, diamondsquare::generate_f32_2};






pub fn add_snow_falls(heightmap: &mut ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, settings: &GenerationOptions, colormap: &mut ColorMapArray) {

    const initial_lifetime: usize = 10;
    
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

    // // let mut pixel;
    
    let mut snow_stocked: f32;

    let mut height_ref: &f32;

    let mut speed = 1.0;

    let to_depose = 1.0;

    let mut relative_points_table: Vec<RelativePoint> = Vec::with_capacity((settings.radius_2.pow(2) * 4) as usize);
    compute_points_in_range(&mut relative_points_table, settings.radius_2 as i32);

    let scaling = heightmap.get_scaling();

    for iteration in 0..settings.number_of_erosion_iterations_2 {

        speed = 1.0;

        if (iteration + 1) % 50000 == 0 {
            println!("{} iterations done.", iteration + 1)
        }


        pos_x = next_random_number(array_max_index as u64, rng).try_into().expect("Error while converting");
        pos_y = next_random_number(array_max_index as u64, rng).try_into().expect("Error while converting");

        velocity = Vec2 {x: 0.0, y: 0.0};
        quantity_of_water = 1.0;

        snow_stocked = 1.0;

        for current_lifetime in (0..initial_lifetime).rev() {

            arr_pos_x = pos_x as usize;
            arr_pos_y = pos_y as usize;

            offset_x = pos_x - arr_pos_x as f64;
            offset_y = pos_y - arr_pos_y as f64;

            compute_height_and_slopes(heightmap, offset_x, offset_y, arr_pos_x, arr_pos_y, &mut height, &mut horizontal_slope, &mut vertical_slope);

            velocity.x = velocity.x * settings.inertia_2 + horizontal_slope * (1.0 - settings.inertia_2);
            velocity.y = velocity.y * settings.inertia_2 + vertical_slope * (1.0 - settings.inertia_2);

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
        
        };

        arr_pos_x = pos_x as usize;
        arr_pos_y = pos_y as usize;

        offset_x = pos_x - arr_pos_x as f64;
        offset_y = pos_y - arr_pos_y as f64;


        for point in PointsInRangeIterator::new(&relative_points_table, array_max_index as usize + 1, arr_pos_x as i32, arr_pos_y as i32) {
            colormap.lighten_portion(point.x * scaling, point.y * scaling, scaling, point.weight as f32 * settings.capacity_factor_2 as f32);
            *heightmap.get_mut(point.x, point.y).unwrap() += point.weight as f32;

            
        }




        
    }





}





pub fn env_coloration_erosion(heightmap: &ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, concentration_map: &mut Arr2d<f32>, settings: &GenerationOptions) {

    println!("starting erosion (environment generation), number of iterations: {}", settings.number_of_erosion_iterations);

    const initial_lifetime: usize = 50;
    
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

    let mut relative_points_table: Vec<RelativePoint> = Vec::with_capacity((settings.radius_2.pow(2) * 4) as usize);
    compute_points_in_range(&mut relative_points_table, settings.radius_2 as i32);

    for iteration in 0..settings.number_of_erosion_iterations_2 {

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

            velocity.x = velocity.x * settings.inertia_2 + horizontal_slope * (1.0 - settings.inertia_2);
            velocity.y = velocity.y * settings.inertia_2 + vertical_slope * (1.0 - settings.inertia_2);

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
                capacity = f64::max(min_capacity, quantity_of_water * -height_difference * settings.capacity_factor_2);

                if capacity < sediment_stocked {
                    to_depose = (sediment_stocked - capacity) * 0.3

                } else {
                    to_erode = f64::min(-height_difference, (capacity - sediment_stocked) * 0.3);

                    for point in PointsInRangeIterator::new(&relative_points_table, array_max_index as usize + 1, arr_pos_x as i32, arr_pos_y as i32) {
                        height_ref = heightmap.get(point.x, point.y).unwrap();
                        sediment_eroded = f64::min(to_erode * point.weight, *concentration_map.get(point.x, point.y).unwrap() as f64);

                        *concentration_map.get_mut(point.x, point.y).unwrap() -= sediment_eroded as f32;

                        sediment_stocked += sediment_eroded;
                    }
                }
            }

            *concentration_map.get_mut(arr_pos_x, arr_pos_y).unwrap() += (to_depose * (1.0 - offset_x) * (1.0 - offset_y)) as f32;
            *concentration_map.get_mut(arr_pos_x + 1, arr_pos_y).unwrap() += (to_depose * offset_x * (1.0 - offset_y)) as f32;
            *concentration_map.get_mut(arr_pos_x, arr_pos_y + 1).unwrap() += (to_depose * (1.0 - offset_x) * offset_y) as f32;
            *concentration_map.get_mut(arr_pos_x + 1, arr_pos_y + 1).unwrap() += (to_depose * offset_x * offset_y) as f32;



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


pub fn apply_env_coloration(heightmap: &ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, settings: &GenerationOptions, colormap: &mut ColorMapArray) {

    let w = heightmap.get_reduced_width();

    let mut material_1_map = Arr2d::zeros(w, w);
    let mut material_2_map = Arr2d::zeros(w, w);
    let mut material_3_map = Arr2d::zeros(w, w);

    let mut height: &f32;
    let peek_size = settings.max_terrain_height / 3.0;

    let color_1 = [76.0/255.0, 117.0/255.0, 53.0/255.0];
    let color_2 = [116.0/255.0, 103.0/255.0, 95.0/255.0];
    let color_3 = [127.0/255.0, 126.0/255.0, 126.0/255.0];

    let neutral = [0.4, 0.4, 0.4];

    let mut noise: f32;

    for x in 0..w {
        for y in 0..w {
            height = heightmap.get(x, y).unwrap();

            noise = generate_f32_2(0.001, rng) * settings.max_terrain_height;
            *material_1_map.get_mut(x, y).unwrap() = f32::exp(-(height / peek_size - settings.max_terrain_height / 6.0 / peek_size + noise).powi(2)) * settings.max_terrain_height;
            
            noise = generate_f32_2(0.001, rng) * settings.max_terrain_height;
            *material_2_map.get_mut(x, y).unwrap() = f32::exp(-(height / peek_size - settings.max_terrain_height / 2.0 / peek_size + noise).powi(2)) * settings.max_terrain_height;
            
            noise = generate_f32_2(0.001, rng) * settings.max_terrain_height;
            *material_3_map.get_mut(x, y).unwrap() = f32::exp(-(height / peek_size - settings.max_terrain_height * 5.0 / 6.0 / peek_size + noise).powi(2)) * settings.max_terrain_height;
        }
    }

    env_coloration_erosion(heightmap, rng, &mut material_3_map, settings);
    env_coloration_erosion(heightmap, rng, &mut material_2_map, settings);
    env_coloration_erosion(heightmap, rng, &mut material_1_map, settings);


    let w2 = colormap.get_width();
    let mut rx: usize;
    let mut ry: usize;

    let mut t1: f32;
    let mut t2: f32;

    let mut color_00: [f32; 3];
    let mut color_10: [f32; 3];
    let mut color_01: [f32; 3];
    let mut color_11: [f32; 3];

    let mut final_color = [0.0, 0.0, 0.0];
    let mut pixel: (&mut f32, &mut f32,&mut f32);

    let scaling = heightmap.get_scaling();

    let get_color = |x: usize, y: usize| -> [f32;3] {

        let mut output = [0.0, 0.0, 0.0];

        let c1 = material_1_map.get(x, y).unwrap() / settings.max_terrain_height;
        let c2 = material_2_map.get(x, y).unwrap() / settings.max_terrain_height;
        let c3 = material_3_map.get(x, y).unwrap() / settings.max_terrain_height;

        for i in 0..3_usize {
            output[i] = c1 * color_1[i] + c2 * color_2[i] + c3 * color_3[i] + (1.0 - c1 - c2 - c3).max(0.0) * neutral[i];
        }

        output

    };

    for x in 0..w2 {
        for y in 0..w2 {

            rx = y / scaling;
            ry = x / scaling;

            if rx < w - 1 && ry < w - 1 {
                color_00 = get_color(rx, ry);
                color_01 = get_color(rx, ry + 1);
                color_11 = get_color(rx + 1, ry + 1);
                color_10 = get_color(rx + 1, ry);

                t1 = (y - rx * scaling) as f32 / scaling as f32;
                t2 = (x - ry * scaling) as f32 / scaling as f32;
                
                for i in 0..3_usize {
                    final_color[i] = bilinear_interpolation(t1, t2, color_00[i], color_01[i], color_10[i], color_11[i]);
                }

                pixel = colormap.get_mut_pixel(x, y).unwrap();
                *pixel.0 = final_color[0].min(1.0).max(0.0);
                *pixel.1 = final_color[1].min(1.0).max(0.0);
                *pixel.2 = final_color[2].min(1.0).max(0.0);

            }

        }
    }

}






