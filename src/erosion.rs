use std::convert::TryInto;

use rand_pcg::Mcg128Xsl64;

use crate::{utils::{ReducedArrayWrapper, ColorMapArray, next_random_number, Vec2}, settings::{GenerationOptions}};




// calcul l'altitude de la goutte d'eau et la pente sur l'axe des x et des y
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

// précalcul les points de sorte à faire une sphère autour d'une position quelconque
pub fn compute_points_in_range(relative_points_table: &mut Vec<RelativePoint>, radius: i32) {

    let mut weight: f64;
    let mut weight_sum = 0_f64;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            weight = radius as f64 - f64::sqrt((dx * dx + dy * dy) as f64);
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



pub fn erode(heightmap: &mut ReducedArrayWrapper<f32>, rng: &mut Mcg128Xsl64, _color_map: &mut ColorMapArray, settings: &GenerationOptions) {

    println!("starting erosion, number of iterations: {}", settings.number_of_erosion_iterations);

    let initial_lifetime: usize = settings.initial_lifetime as usize;
    
    let mut pos_x: f64;
    let mut pos_y: f64;

    let mut arr_pos_x: usize;
    let mut arr_pos_y: usize;

    let mut offset_x: f64;
    let mut offset_y: f64;

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

        for _current_lifetime in (0..initial_lifetime).rev() {

            arr_pos_x = pos_x as usize;
            arr_pos_y = pos_y as usize;

            offset_x = pos_x - arr_pos_x as f64;
            offset_y = pos_y - arr_pos_y as f64;

            compute_height_and_slopes(heightmap, offset_x, offset_y, arr_pos_x, arr_pos_y, &mut height, &mut horizontal_slope, &mut vertical_slope);

            velocity.x = velocity.x * settings.inertia + horizontal_slope * (1.0 - settings.inertia);
            velocity.y = velocity.y * settings.inertia + vertical_slope * (1.0 - settings.inertia);

            match velocity.normalize_ip() {  // le vecteur vitesse est toujours normalisé à 1 pour éviter des bonds
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
                  // si la goutte remonte une pente, dépose une partie du sédiment pour essayer de faire une zone plate
                to_depose = f64::min(height_difference, sediment_stocked);

            } else {
                capacity = f64::max(min_capacity, quantity_of_water * -height_difference * settings.capacity_factor);

                if capacity < sediment_stocked {
                    // dépose 30% de la différence entre capacité et stockage
                    to_depose = (sediment_stocked - capacity) * 0.3

                } else {
                    // érode en forme de sphère autour de la position
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






