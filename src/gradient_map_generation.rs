use crate::utils::Arr2d;




pub fn generate_gradient_map(heightmap: &Arr2d<f32>, output: &mut Arr2d<f32>, ref_height: f32, w: usize, step: usize, offset: i32) {

    let mut number_of_points: usize;
    let mut current_sum: f32;
    let mut current_max = -f32::INFINITY;

    // gradient à une position: moyenne des valeurs absolues des différences entre la hauteur à cette position position 
    // et celles des positions d'une distance inférieure ou égale à step
    for x in 0..w {
        for y in 0..w {
            number_of_points = 0;
            current_sum = 0.0;

            if let Some(value) = output.get_mut(x, y) {
                if let Some(h) = heightmap.get(x, y) {

                    for dx in (-offset..=offset).step_by(step) {
                        for dy in (-offset..=offset).step_by(step) {
                            if let Some(oh) = heightmap.geti(x as i32 + dx, y as i32 + dy) {
                                number_of_points += 1;
                                current_sum += f32::abs(oh - h);
                            }
                        }
                    }
                }
                *value = current_sum / number_of_points as f32 / offset as f32 * w as f32 / ref_height;
                current_max = f32::max(current_max, *value);
            }
        }
    }

    for x in 0..w {
        for y in 0..w {
            
            if let Some(value) = output.get_mut(x, y) {
                *value /= current_max;
            }
        }
    }
}




