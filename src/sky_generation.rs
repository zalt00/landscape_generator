use std::{fs::File, io::Write};

use crate::utils::{ColorMapArray, HALF_PI, PI};


const LENGTH: usize = 1000;
const LENGTH_F32: f32 = LENGTH as f32;

pub struct LightSpectrum {
    colors: Vec<(f32, (f32, f32, f32))>,
    wl_repartition: Vec<f32>,
    wl_curve: Option<[f32; LENGTH]>,
    color_curves: Option<[[f32; 3]; LENGTH]>
}

impl LightSpectrum {
    pub fn new() -> LightSpectrum {

        let colors = vec![

            (0.5, (0.0, 0.2, 0.75)),
            (0.4, (0.0, 0.30 - 0.05, 0.25)),
            (0.15, (0.55, 0.28 - 0.05, 0.05)),
            (0.12, (0.85, 0.10, 0.15))
        ];

        let wl_repartition = vec![1.0;colors.len()];

        LightSpectrum {colors, wl_repartition, wl_curve: None, color_curves: None}
    }

    fn normalize(&self, wl: f32) -> f32 {
        let min_wl = self.colors.last().unwrap().0;
        let max_wl  = self.colors.first().unwrap().0;

        (wl - min_wl) / (max_wl - min_wl)
    }

    pub fn generate_color_curves(&mut self, n: usize) {
        let mut curves = [[0.0; 3]; LENGTH];
        
        let mut normalized_wl: Vec<f32> = vec![];
        for (wl, _) in self.colors.iter() {
            normalized_wl.push(self.normalize(*wl));
        }

        let mut value: f32;
        let mut closest = self.colors.len() - 1;

        let mut color = self.colors[closest].1;

        for i in 0..LENGTH {
            value = i as f32 / LENGTH_F32;
            while closest != 0 && f32::abs(normalized_wl[closest] - value) > f32::abs(normalized_wl[closest - 1] - value) {
                closest -= 1;
                color = self.colors[closest].1;
            }

            curves[i][0] = color.0;
            curves[i][1] = color.1;
            curves[i][2] = color.2;

        }

        // make the curves continuous
        let mut i2: usize;

        for layer_id in 0..3 {

            for j in (1..=n).rev() {
                for i in j..(LENGTH - j) {
                    i2 = LENGTH - 1 - i;
    
                    curves[i][layer_id] = (curves[i - j][layer_id] + curves[i + j][layer_id]) / 2.0;
                    curves[i2][layer_id] = (curves[i2 - j][layer_id] + curves[i2 + j][layer_id]) / 2.0;
                }
                
                for to_correct in 0..j {
                    curves[to_correct][layer_id] = curves[j][layer_id];
                    curves[LENGTH - to_correct - 1][layer_id] = curves[LENGTH - 1 - j][layer_id];
                }


            }
        }

        self.color_curves = Some(curves);

    }

    pub fn generate_wl_curve(&mut self, n: usize) {
        const LENGTH: usize = 1000;
        const LENGTH_F32: f32 = LENGTH as f32;

        let mut curve = [0.0; LENGTH];

        let mut sum = 0_f32;
        for amount in self.wl_repartition.iter() {
            sum += amount;
        }

        let mut amounts_vec: Vec<f32> = vec![];
        let mut f = 0_f32;

        for amount in self.wl_repartition.iter() {
            f += amount / sum;
            amounts_vec.push(f);
        }

        let mut counter: usize = 0;
        let mut value: f32;

        for i in 0..LENGTH {
            value = i as f32 / LENGTH_F32;
            while value > amounts_vec[counter] {
                counter += 1;
            }
            curve[i] = self.colors[counter].0;
        }

        // make the curve continuous
        let mut i2: usize;

        for j in (1..=n).rev() {
            for i in j..(LENGTH - j) {
                i2 = LENGTH - 1 - i;

                curve[i] = (curve[i - j] + curve[i + j]) / 2.0;
                curve[i2] = (curve[i2 - j] + curve[i2 + j]) / 2.0;
            }

            for to_correct in 0..j {
                curve[to_correct] = curve[j];
                curve[LENGTH - to_correct - 1] = curve[LENGTH - 1 - j];
            }

        }

        self.wl_curve = Some(curve);

    }

    pub fn get_wl(&self, value: f32) -> Option<f32> {
        if let Some(curve) = &self.wl_curve {
            let value = value.max(0.0).min(1.0);
        

            let i1: usize = (value * 999.0) as usize;
            let i2 = i1 + 1;
    
            if i2 >= 1000 {
                Some(curve[999])
            } else {
                Some((curve[i1] + curve[i2]) / 2.0)
            }
        } else {None}
        
        
    }    

    pub fn get_color_from_wavelength(&self, wavelength: f32) -> Option<(f32, f32, f32)> {
        if let Some(curves) = &self.color_curves {
            let mut color = (0_f32, 0_f32, 0_f32);

            let value = self.normalize(wavelength).max(0.0).min(1.0);
        

            let i1: usize = (value * 999.0) as usize;
            let i2 = i1 + 1;

            if i2 >= LENGTH {
                color.0 = curves[i1][0];
                color.1 = curves[i1][1];
                color.2 = curves[i1][2];
            } else {
                color.0 = (curves[i1][0] + curves[i2][0]) / 2.0;
                color.1 = (curves[i1][1] + curves[i2][1]) / 2.0;
                color.2 = (curves[i1][2] + curves[i2][2]) / 2.0;
            }
    
            Some(color)
        } else {None}

    }

    pub fn new_empty() -> LightSpectrum {LightSpectrum {colors: vec![], wl_repartition: vec![], wl_curve: None, color_curves: None}}

    pub fn add_color(&mut self, wavelength: f32, color: (f32, f32, f32), repartition: f32) {
        self.colors.push((wavelength, color));
        self.wl_repartition.push(repartition);
    }

    pub fn get_sum_of_colors(&self) -> (f32, f32, f32) {
        let mut r = 0.0_f32;
        let mut g = 0.0_f32;
        let mut b = 0.0_f32;

        for ((_, color), amount) in self.colors.iter().zip(self.wl_repartition.iter()) {
            r += color.0 * amount;
            g += color.1 * amount;
            b += color.2 * amount;
        };

        (r, g, b)
    }

    pub fn diffuse(&mut self, planet_radius: f32, atmosphere_radius: f32, angle: f32) -> LightSpectrum {
        let distance = compute_travelling_distance(planet_radius, atmosphere_radius, angle);
        let mut diffused_light_spectrum = LightSpectrum::new_empty();

        let mut diffusion_rate: f32;

        let mut emergent_color: (f32, f32, f32);
        let mut diffused_color: (f32, f32, f32);

        for (i, (wavelength, color)) in self.colors.iter_mut().enumerate() {
            diffusion_rate = compute_diffusion_rate(distance, *wavelength);
            println!("diffusion rate: {}", diffusion_rate);

            diffused_light_spectrum.add_color(*wavelength, *color, diffusion_rate);

            self.wl_repartition[i] = 1.0 - diffusion_rate;

        }

        diffused_light_spectrum
    }

}

pub fn compute_travelling_distance(planet_radius: f32, atmosphere_radius: f32, angle: f32) -> f32 {

    let sin_alpha = f32::sin(HALF_PI + angle);

    if sin_alpha == 0.0 {
        atmosphere_radius - planet_radius
    } else {
        (  atmosphere_radius * f32::sin(HALF_PI - angle - f32::asin( sin_alpha * planet_radius  / atmosphere_radius ) )  ) / sin_alpha
    }

}


pub fn compute_diffusion_rate(distance: f32, wavelength: f32) -> f32 {
    if distance < 0.0 {
        0.0
    } else {
        1.0 - 1.0 / (distance.powi(2) * wavelength.powi(4) + 1.0)
    }
}


pub fn generate_sky_colormap(
    colormap: &mut ColorMapArray, w: usize,
    planet_radius: f32, atmosphere_radius: f32,
    incident_spectrum: &mut LightSpectrum, angle: f32, sun_size: f32, ambient_sky_light: f32, ambient_col_out: &mut [f32;3], sun_col_out: &mut [f32;3])
{

    let mut diffused_spectrum = incident_spectrum.diffuse(planet_radius, atmosphere_radius, angle);
    let ambient_color = diffused_spectrum.get_sum_of_colors();

    let mut sun_color = incident_spectrum.get_sum_of_colors();
    {
        let mut current_max = 0_f32;
        if sun_color.0 > current_max {
            current_max = sun_color.0
        }
        if sun_color.1 > current_max {
            current_max = sun_color.1
        }
        if sun_color.2 > current_max {
            current_max = sun_color.2
        }
        if current_max > 1.0 {
            sun_color.0 /= current_max;
            sun_color.1 /= current_max;
            sun_color.2 /= current_max;
        }

    }



    println!("ambient_color: {}, {}, {}", ambient_color.0, ambient_color.1, ambient_color.2);
    println!("sun_color: {}, {}, {}", sun_color.0, sun_color.1, sun_color.2);
    ambient_col_out[0] = ambient_color.0;
    ambient_col_out[1] = ambient_color.1;
    ambient_col_out[2] = ambient_color.2;
    sun_col_out[0] = sun_color.0;
    sun_col_out[1] = sun_color.1;
    sun_col_out[2] = sun_color.2;


    {
        let mut file = File::create("cfg.data").expect("welp");
        file.write_all(format!("{}\n{}\n{}", sun_color.0, sun_color.1, sun_color.2).as_bytes()).expect("welp2");
    }

    let center = (w / 2) as i32;
    let squared_sphere_radius = center.pow(2);
    let mut squared_distance_from_center: i32;

    let mut distance_rate: f32;

    let mut pix_color: (f32, f32, f32);
    let mut wl: f32;

    diffused_spectrum.generate_wl_curve(30);
    diffused_spectrum.generate_color_curves(50);

    let mut local_radius: f32;
    let alpha = f32::abs(HALF_PI - angle);
    let mut beta: f32;
    let mut sv_distance: f32;

    let mut view_angle: f32;

    let mut travelling_distance: f32;

    let mut ratio: f32;

    let mut buf: usize;

    for x in 0..w {
        for y in 0..w {

            squared_distance_from_center = (x as i32 - center).pow(2) + (y as i32 - center).pow(2);
            if squared_distance_from_center <= squared_sphere_radius + squared_sphere_radius / 50 * 0 {
                if let Some(pixel) = colormap.get_mut_pixel(w - 1 - y, x) {
                    

                    local_radius = (squared_sphere_radius as f32 - (center - x as i32).pow(2) as f32).sqrt() / center as f32;
                    sv_distance = f32::abs((y as i32 - center) as f32 / center as f32);

                    if sv_distance < 0.1 && local_radius < 0.1 {
                        ratio = 1.0
                    } else {
                        ratio = sv_distance / local_radius
                    }

                    beta = (ratio).acos();
                    assert!(ratio <= 1.6, "sv_distance = {}, local_radius = {}", sv_distance, local_radius);

                    if (y as i32) < center {
                        view_angle = (beta - alpha).abs();
                    } else {
                        view_angle = (beta - alpha + 2.0 * (HALF_PI - beta)).abs();
                    }

                    wl = diffused_spectrum.get_wl(1.0 - view_angle / (PI / 2.0)).unwrap();


                    pix_color = diffused_spectrum.get_color_from_wavelength(wl).unwrap();

                    *pixel.0 = pix_color.0;
                    *pixel.1 = pix_color.1;
                    *pixel.2 = pix_color.2;

                    add_lighting_and_sun_effect(pixel, squared_distance_from_center, w, sun_size, sun_color, ambient_sky_light, view_angle / (PI / 2.0));

                }
            }



        }
    }
}

pub fn add_lighting_and_sun_effect(pixel: (&mut f32, &mut f32, &mut f32), squared_distance_from_center: i32, w: usize, sun_size: f32, sun_color: (f32, f32, f32), ambient_sky_light: f32,
    angle_rate: f32
) {
    let distance_rate = (squared_distance_from_center as f32).sqrt() / w as f32;
    //let sun_light = ((0.99 - (distance_rate - sun_size).max(0.0)).powi(164) / 1.0).min(1.0);
    let sun_light = if (distance_rate - sun_size) <= -50.0 {0.5} else {(1.0 - (distance_rate - sun_size).max(0.0)).powi(250)};
    let ambient_light = (1.0 - angle_rate).powf(1.0) / 8.0 * ambient_sky_light;

    *pixel.0 = (sun_light * (sun_color.0) + (1.0 - sun_light * 1.0).max(0.0) * *pixel.0) * (1.0 + (ambient_light).max(0.0) * 20.0) + sun_light * 2.0;
    *pixel.1 = (sun_light * (sun_color.1) + (1.0 - sun_light * 1.0).max(0.0) * *pixel.1) * (1.0 + (ambient_light).max(0.0) * 20.0) + sun_light * 2.0;
    *pixel.2 = (sun_light * (sun_color.2) + (1.0 - sun_light * 1.0).max(0.0) * *pixel.2) * (1.0 + (ambient_light).max(0.0) * 20.0) + sun_light * 2.0;

    // *pixel.0 = sun_light;
    // *pixel.1 = sun_light;
    // *pixel.2 = sun_light;
}



