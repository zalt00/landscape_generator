use std::convert::TryInto;

use rand_pcg::Mcg128Xsl64;

use crate::{utils::{ReducedArrayWrapper, generate_in_interval, linear_interpolation, bilinear_interpolation, ColorMapArray, rand, next_random_number, Vec2, Arr2d}, settings::{Settings, GenerationOptions}};




pub fn erode2(heightmap: &mut ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray, settings: &GenerationOptions) {

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
    let mut current_lowest_relativ_position: [i32; 2];

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

    let mut to_erode: f32;

    let mut adjacent_slopes: [f32;9];

    let mut dx: i32;
    let mut dy: i32;

    let mut current_max: f32;
    let mut sum: f32;

    let mut f: f32;
    let mut counter: usize;

    let ref_height = settings.max_terrain_height;

    for _ in 0..settings.number_of_erosion_iterations {
        water_remaining = 200;
        sediment_stocked = 0.0;
        capacity = 15.0;

        speed = 0.0;
        
        adjacent_slopes = [0.0; 9];


        x = (generate_in_interval(max, rng) as usize).max(1).min(max as usize - 2);
        y = (generate_in_interval(max, rng) as usize).max(1).min(max as usize - 2);

        pos = [x as f32, y as f32];


        while water_remaining > 0 {

            current_height = *heightmap.get(x, y).unwrap();
            current_max = -999999999.0;
            sum = 0.0;
            current_lowest_relativ_position = [0, 0];
            counter = 0;


            for i in 0..9_i32 {
                dx = i.div_euclid(3) - 1;
                dy = i % 3 - 1;
                if let Some(height) = heightmap.geti(x as i32 + dx, y as i32 + dy) {
                    adjacent_slopes[i as usize] = *height - current_height;
                    current_max = f32::max(current_max, *height - current_height);
                } else {
                    println!("welp")
                }
            }

            for i in 0..9_usize {
                adjacent_slopes[i] -= current_max;
                sum += adjacent_slopes[i];
                assert!(adjacent_slopes[i] <= 0.0);
            }

            for i in 0..9_usize {
                if sum == 0.0 {
                    adjacent_slopes[i] = 1.0/9.0;
                } else {
                    adjacent_slopes[i] = (adjacent_slopes[i] / sum).abs();
                    assert!(0.0 <= adjacent_slopes[i] && adjacent_slopes[i] <= 1.0, "{}, {}", adjacent_slopes[i], sum);
    
                }

            }
            
            sum = 0.0;
            counter = 0;
            f = rand(rng);
            while f > sum {
                sum += adjacent_slopes[counter];
                counter += 1;
            }

            current_lowest_relativ_position = [counter.div_euclid(3) as i32 - 1, counter as i32 % 3 - 1];




            to_erode = (0.1 * current_height - heightmap.geti(x as i32 + current_lowest_relativ_position[0], y as i32 + current_lowest_relativ_position[1]).unwrap()).max(0.0) + 3.5;            
            
            current_height_mut = heightmap.get_mut(x, y).unwrap();

            sediment_stocked += to_erode;
            *current_height_mut -= to_erode;


            if sediment_stocked > capacity * water_remaining as f32 {
                *current_height_mut += sediment_stocked - capacity * water_remaining as f32;
                sediment_stocked = capacity * water_remaining as f32;
            }

            *current_height_mut += f32::min(sediment_stocked, 0.5);
            sediment_stocked -= f32::min(sediment_stocked, 0.5);

            /*
            // pixel = color_map.get_mut_pixel(heightmap.convert(y), heightmap.convert(x)).unwrap();
            // *pixel.0 += 1.0 / settings.number_of_erosion_iterations as f32 * 1000.0;
            // *pixel.1 += (water_remaining as f32) / 200.0 / settings.number_of_erosion_iterations as f32 * 1000.0;
            */

            water_remaining -= 1;

            pos[0] = (pos[0] + current_lowest_relativ_position[0] as f32).clamp(1.0, max as f32 - 2.0);
            pos[1] = (pos[1] + current_lowest_relativ_position[1] as f32).clamp(1.0, max as f32 - 2.0);

            x = pos[0] as usize;
            y = pos[1] as usize;


            if pos[0] == 1.0 || pos[0] == max as f32 - 2.0 || pos[1] == 1.0 || pos[1] == max as f32 - 2.0 {
                water_remaining = 0;
            }


        }







        





    }
}



pub fn compute_height_and_slopes(heightmap: &ReducedArrayWrapper<f32>, offset_x: f64, offset_y: f64, arr_pos_x: usize, arr_pos_y: usize, height: &mut f64, hslope: &mut f64, vslope: &mut f64) {

    let height_00 = *heightmap.get(arr_pos_x, arr_pos_y).unwrap() as f64;
    let height_01 = *heightmap.get(arr_pos_x, arr_pos_y + 1).unwrap() as f64;
    let height_10 = *heightmap.get(arr_pos_x + 1, arr_pos_y).unwrap() as f64;
    let height_11 = *heightmap.get(arr_pos_x + 1, arr_pos_y + 1).unwrap() as f64;

    *hslope = (height_10 - height_00) * (1.0 - offset_y) + (height_11 - height_01) * offset_y;
    *vslope = (height_01 - height_00) * (1.0 - offset_x) + (height_11 - height_10) * offset_x;


    *height = height_00 * (1.0 - offset_x) * (1.0 - offset_y) +
             height_01 * (1.0 - offset_x) * offset_y + 
             height_10 * offset_x * (1.0 - offset_y) +
             height_11 * offset_x * offset_y;

}

pub struct RelativePoint {
    dx: i32, 
    dy: i32, 
    weight: f64
}

pub struct Point {
    pub x: usize,
    pub y: usize,
    pub weight: f64
}

pub struct PointsInRangeIterator<'a> {
    relative_points_table: &'a Vec<RelativePoint>,
    height_map_width: usize,
    x: i32,
    y: i32,
    index: usize
}

impl<'a> PointsInRangeIterator<'a> {
    pub fn new(relative_points_table: &'a Vec<RelativePoint>, height_map_width: usize, x: i32, y: i32) -> PointsInRangeIterator {
        PointsInRangeIterator { relative_points_table, height_map_width, x, y, index: 0 }
    }
}

impl<'a> Iterator for PointsInRangeIterator<'a> {
    
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {

        let mut x: i32;
        let mut y: i32;

        let width = self.height_map_width as i32;

        let mut relative_point: &RelativePoint = self.relative_points_table.get(self.index)?;

        x = self.x + relative_point.dx;
        y = self.y + relative_point.dy;


        while !(0 <= x && x < width && 0 <= y && y < width) {
            self.index += 1;

            relative_point = self.relative_points_table.get(self.index)?;

            x = self.x + relative_point.dx;
            y = self.y + relative_point.dy;

        }

        self.index += 1;

        Some(Point {x: x as usize, y: y as usize, weight: relative_point.weight})

    }

}

pub fn compute_points_in_range(relative_points_table: &mut Vec<RelativePoint>, radius: i32) {

    let mut weight: f64;
    let mut weight_sum = 0_f64;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            weight = f64::sqrt((dx * dx + dy * dy) as f64);
            if weight > 0.0 {
                relative_points_table.push(RelativePoint {dx, dy, weight});
                weight_sum += weight
            }
        }
    }

    for v in relative_points_table.iter_mut() {
        v.weight /= weight_sum;
        assert!(0.0<=v.weight && v.weight <=1.0)
    }


}



pub fn erode(heightmap: &mut ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, color_map: &mut ColorMapArray, settings: &GenerationOptions) {

    println!("starting erosion, number of iterations: {}", settings.number_of_erosion_iterations);

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

    let mut mut_height_ref: &mut f32;

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
                        mut_height_ref = heightmap.get_mut(point.x, point.y).unwrap();
                        *mut_height_ref = f32::max(*mut_height_ref, 0.0);
                        sediment_eroded = f64::min(to_erode * point.weight, *mut_height_ref as f64);

                        sediment_stocked += sediment_eroded;
                        *mut_height_ref -= sediment_eroded as f32;
                    }

                }

            }
            assert!(to_depose >= 0.0);
            assert!(to_erode >= 0.0);
            sediment_stocked -= to_depose;
            assert!(sediment_stocked >= 0.0);

            *heightmap.get_mut(arr_pos_x, arr_pos_y).unwrap() += (to_depose * (1.0 - offset_x) * (1.0 - offset_y)) as f32;
            *heightmap.get_mut(arr_pos_x + 1, arr_pos_y).unwrap() += (to_depose * offset_x * (1.0 - offset_y)) as f32;
            *heightmap.get_mut(arr_pos_x, arr_pos_y + 1).unwrap() += (to_depose * (1.0 - offset_x) * offset_y) as f32;
            *heightmap.get_mut(arr_pos_x + 1, arr_pos_y + 1).unwrap() += (to_depose * offset_x * offset_y) as f32;

            quantity_of_water *= 0.95;
            speed = f64::sqrt(speed * speed + height_difference.abs() / settings.max_terrain_height as f64 * 1.0);

            
            // // pixel = color_map.get_mut_pixel(heightmap.convert(arr_pos_y), heightmap.convert(arr_pos_x)).unwrap();
            // // *pixel.0 += 1.0 / settings.number_of_erosion_iterations as f32 * 1000.0;
            // // *pixel.1 += (current_lifetime as f32) / 200.0 / settings.number_of_erosion_iterations as f32 * 1000.0;





            
                 
        }







    }




}






