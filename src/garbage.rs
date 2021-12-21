

fn to_image_buffer(v: &Vec<f32>, width: usize, height: usize, sea_repartition: f32, modifier: &mut Vec<f32>) -> Vec<u8> {
    let mut output = vec![];
    
    let mut current_modifier_max: f32 = -20000.0;
    let mut current_modifier_min: f32 = 20000.0;

    for n in modifier.iter() {
        current_modifier_max = current_modifier_max.max(*n);
        current_modifier_min = current_modifier_min.min(*n);
    }

    if current_modifier_min == current_modifier_max {
        current_modifier_min = 0.0;
        if current_modifier_max == 0.0 {
            current_modifier_max = 1.0
        }
    }

    let mut v2 = v.clone();

    for i in 0..v2.len() {
        v2[i] *= (modifier[i] - current_modifier_min) / (current_modifier_max - current_modifier_min)
    }

    v2.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

    let min_value = v2.first().unwrap();
    let max_value = v2.last().unwrap();

    println!("{} {}", min_value, max_value);

    let median = (v2[((width * height) as f32 * sea_repartition) as usize] - min_value) / (max_value - min_value);


    for (i, n) in v.iter().enumerate() {

        let mut normalized_n = n * ((modifier[i] - current_modifier_min) / (current_modifier_max - current_modifier_min));

        normalized_n = (normalized_n - min_value) / (max_value - min_value);


        assert!(0.0 <= normalized_n && normalized_n <= 1.0);

        if normalized_n < median {
            output.push((normalized_n * 55.0) as u8);
            output.push((normalized_n * 105.0) as u8);
            output.push((normalized_n * 195.0) as u8);
        } else {
            output.push((normalized_n * 55.0) as u8);
            output.push((normalized_n * 195.0) as u8);
            output.push((normalized_n * 115.0) as u8);
        }
    
    };
    output
}


fn to_image_buffer_2(v: &Arr2d<f32>, sea_repartition: f32, sea_color: (u8, u8, u8), land_color: (u8, u8, u8)) -> Vec<u8> {

    println!("converting to image buffer...");

    println!("cloning array...");
    let mut vector2 = v.get_vec().clone();

    println!("sorting array...");
    vector2.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min_value = vector2[0];
    let max_value = *vector2.last().unwrap();

    let sea_level = vector2[ (sea_repartition * v.get_length() as f32 ) as usize ];

    let mut is_sea: bool;
    let mut normalized_value: f32;

    let mut output: Vec<u8> = vec![];

    println!("begin iterations.");

    for n in v.get_vec().iter() {
        is_sea = *n < sea_level;

        normalized_value = (n - min_value) / (max_value - min_value);

        assert!(0.0 <= normalized_value && normalized_value <= 1.0, "invalid value: {}", normalized_value);

        if is_sea {
            output.push((normalized_value * sea_color.0 as f32) as u8);
            output.push((normalized_value * sea_color.1 as f32) as u8);
            output.push((normalized_value * sea_color.2 as f32) as u8);
        } else {
            output.push((normalized_value * land_color.0 as f32) as u8);
            output.push((normalized_value * land_color.1 as f32) as u8);
            output.push((normalized_value * land_color.2 as f32) as u8);
        }
    

    };

    println!("conversion done.");

    output

}


pub fn generate_demisphere_callibration_colormap(colormap: &mut ColorMapArray, w: usize) {
    
    // let mut pixel: &mut (&mut f32, &mut f32, &mut f32);
    
    for x in 0..w {
        for y in 0..w {
            
            if let Some(pixel) = colormap.get_mut_pixel(x, y) {
                if w / 2 - 1 <= x && w / 2 + 1 >= x {
                    *pixel.0 = 0.8;
                }

                if x > w / 2 {
                    *pixel.2 = 0.8
                } else {
                    *pixel.2 = 0.3
                }

                if y > w / 2 {
                    *pixel.1 = 0.8
                } else {
                    *pixel.1 = 0.3
                }

                if w / 2 - 1 <= y && w / 2 + 1 >= y {
                    *pixel.0 = 0.3;
                }
            }
        }
    }
}

pub fn get_lighting_repartition(output: &mut Arr2d<f64>, w: usize, sun_apparent_proportion: f32, planet_radius: f32, atmosphere_radius: f32, angle: f32) {

    let mut ray_angle: f32;
    let mut sum = 0.0_f64;

    let center = (w / 2) as i32;
    let sun_radius = w as f32 * sun_apparent_proportion / 2.0;
    let sun_radius_squared = sun_radius.powi(2) as i32;

    let mut squared_distance_from_center: i32;
    let mut distance_rate_relative_to_sun_inv: f64;
    let mut distance_rate: f64;

    let mut lighting: f64;

    let w_squared = w.pow(2) as f64;


    for x in 0..w {
        for y in 0..w {
            if let Some(lighting_rate) = output.get_mut(x, y) {
                squared_distance_from_center = (x as i32 - center).pow(2) + (y as i32- center).pow(2);

                distance_rate_relative_to_sun_inv = sun_radius_squared as f64 / (squared_distance_from_center + 1) as f64;
                distance_rate = (squared_distance_from_center + 1) as f64 / (w / 2).pow(2) as f64;
                
                lighting = distance_rate_relative_to_sun_inv + (1.0 - distance_rate) * 2.0;
                
                sum += lighting;
                *lighting_rate = lighting;
            }
        }
    }

    sum /= w_squared;

    println!("{}", sum);

    for x in 0..w {
        for y in 0..w {
            if let Some(lighting_rate) = output.get_mut(x, y) {
                *lighting_rate /= sum;
            }
        }
    }
}


pub fn get_color_repartition(output: &mut Arr2d<f64>, w: usize, planet_radius: f32, atmosphere_radius: f32, sun_angle: f32,  color_wavelength: f32, color_amount: f64, 
    lighting_repartition: &mut Arr2d<f64>) {

    let mut angle: f32;
    let mut squared_distance_from_center: i32;
    let mut distance_from_center: f32;

    let mut theorical_distance: f32;
    let mut diffusion_rate: f32;

    let mut sum: f64 = 0.0;

    let w_squared = w.pow(2) as f64;

    let center = (w / 2) as i32;
    let radius = center as f32;


    for y in 0..w {
        for x in 0..w {
            if let Some(color_rate) = output.get_mut(x, y) {
                squared_distance_from_center = (x as i32 - center).pow(2) + (y as i32- center).pow(2);
                distance_from_center = (squared_distance_from_center as f32).sqrt();

                angle = HALF_PI - (distance_from_center / radius).atan() + sun_angle.abs() * PI / 180.0;
                theorical_distance = compute_travelling_distance(planet_radius, atmosphere_radius, angle);
                diffusion_rate = compute_diffusion_rate(theorical_distance, color_wavelength);
                
                if let Some(lighting_rate) = lighting_repartition.get(x, y) {
                    *color_rate = diffusion_rate.powi(2) as f64 * (1.0 - (squared_distance_from_center + 1) as f64 / (w / 2).pow(2) as f64);
                    sum += *color_rate;
                }

            }
        }
    }

    for x in 0..w {
        for y in 0..w {
            if let Some(color_rate) = output.get_mut(x, y) {
                *color_rate /= sum / color_amount / w_squared
            }
        }
    }

}

pub fn generate_demisphere_colormap(
    colormap: &mut ColorMapArray, w: usize,
    sun_apparent_proportion: f32, planet_radius: f32, atmosphere_radius: f32,
    incident_spectrum: &mut LightSpectrum, angle: f32) 
{

    let global_luminosity = 1.0 / compute_travelling_distance(planet_radius, atmosphere_radius, angle) / (atmosphere_radius - planet_radius);
    println!("{}" , global_luminosity);


    let diffused_spectrum = incident_spectrum.diffuse(planet_radius, atmosphere_radius, angle);
    let ambient_color = diffused_spectrum.get_sum_of_colors();
    let sun_color = incident_spectrum.get_sum_of_colors();

    let center = (w / 2) as i32;
    let sun_radius = w as f32 * sun_apparent_proportion / 2.0;
    let sun_radius_squared = sun_radius.powi(2) as i32;

    let mut squared_distance_from_center: i32;
    let mut distance_rate_relative_to_sun: f32;
    let mut distance_rate: f32;

    let sun_luminosity_h = 0.5;

    let mut lighting_repartition: Arr2d<f64> = Arr2d::zeros64(w, w);
    get_lighting_repartition(&mut lighting_repartition, w, sun_apparent_proportion, planet_radius, atmosphere_radius, angle);

    let mut color_repartitions: Vec<Arr2d<f64>> = vec![];

    {
        let mut auxiliary_array = Arr2d::zeros64(w, w);

        for (wavelength, color) in diffused_spectrum.colors.iter() {
            get_color_repartition(&mut auxiliary_array, w, planet_radius, atmosphere_radius, angle, *wavelength, 1.0_f64, 
            &mut lighting_repartition);
            color_repartitions.push(auxiliary_array.clone())
        }
    }

    let mut pix_color: (f64, f64, f64);

    for x in 0..w {
        for y in 0..w {
            if let Some(pixel) = colormap.get_mut_pixel(x, y) {
                
                pix_color = (0.0, 0.0, 0.0);
                
                for (color_repartition_array, (_, base_color)) in color_repartitions.iter().zip(diffused_spectrum.colors.iter()) {
                    if let Some(color_amount) = color_repartition_array.get(x, y) {
                        pix_color.0 += *color_amount * base_color.0 as f64;
                        pix_color.1 += *color_amount * base_color.1 as f64;
                        pix_color.2 += *color_amount * base_color.2 as f64;

                    }
                }
                
                if let Some(lighting) = lighting_repartition.get(x, y) {
                    *pixel.0 = (*lighting * global_luminosity as f64 * 0.0 + pix_color.0) as f32;
                    *pixel.1 = (*lighting * global_luminosity as f64 * 0.0 + pix_color.1) as f32;
                    *pixel.2 = (*lighting * global_luminosity as f64 * 0.0 + pix_color.2) as f32;

                    //print!("{} ", *lighting);
                } else {
                    panic!()
                }


                /*                
                squared_distance_from_center = (x as i32 - center).pow(2) + (y as i32- center).pow(2);

                if squared_distance_from_center <= sun_radius_squared {
                    *pixel.0 = sun_color.0 + ;
                    *pixel.1 = sun_color.1 + ;
                    *pixel.2 = sun_color.2 +;
                } else {

                    distance_rate_relative_to_sun = sun_radius_squared as f32 / squared_distance_from_center as f32;
                    distance_rate = squared_distance_from_center as f32 / (w / 2).pow(2) as f32;


                    *pixel.0 = (1.0 - distance_rate_relative_to_sun) * ambient_color.0 + distance_rate_relative_to_sun * (sun_color.0 + sun_luminosity_h) + global_luminosity - 0.5 * distance_rate;
                    *pixel.1 = (1.0 - distance_rate_relative_to_sun) * ambient_color.1 + distance_rate_relative_to_sun * (sun_color.1 + sun_luminosity_h) + global_luminosity - 0.5 * distance_rate;
                    *pixel.2 = (1.0 - distance_rate_relative_to_sun) * ambient_color.2 + distance_rate_relative_to_sun * (sun_color.2 + sun_luminosity_h) + global_luminosity - 0.5 * distance_rate;
                }

                */

            }
        }
    }
        

}