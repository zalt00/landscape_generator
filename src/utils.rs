use std::convert::TryInto;

use image::{DynamicImage, GenericImageView};
use rand_core::RngCore;
use rand_pcg::Mcg128Xsl64;

use crate::settings::GenerationOptions;

pub const TWO_POW_32_MINUS_1: u32 = 4294967295;
pub const TWO_POW_32: u64 = 4294967296;
pub const TWO_POW_31: u32 = 2147483648;
pub const TWO_POW_31_F32: f32 = 2147483648.0;

pub const TWO_POW_15_F32: f32 = 32768.0;

pub const TWO_POW_16: u32 = 65536;
pub const TWO_POW_16_F32: f32 = 65536.0;

pub const TWO_POW_17_F32: f32 = 131072.0;

pub const PI: f32 = 3.141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067982148086513282306647;
pub const HALF_PI: f32 = PI / 2.0;
pub const TAU: f32 = PI * 2.0;

pub const TWO_POW_15: u32 = 32768;

#[derive(Clone, Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T
}

impl Vec2<f64> {
    pub fn normalize_ip(&mut self) -> Result<(), ()> {
        let length = self.get_length();

        if length == 0.0 {
            Err(())
        } else {
            self.x /= length;
            self.y /= length;
            Ok(())
        }

    }

    pub fn get_length(&self) -> f64 {
        f64::sqrt(self.x * self.x + self.y * self.y)
    }
}


#[derive(Clone, Copy)]
pub struct Rect<T> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T
}

impl Rect<usize> {
    pub fn get_width(&self) -> usize {
        self.right - self.left
    }

    pub fn get_height(&self) -> usize {
        self.top - self.bottom
    }
} 


pub struct ColorMapArray {
    r: Arr2d<f32>,
    g: Arr2d<f32>,
    b: Arr2d<f32>
}

impl ColorMapArray {
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<(&f32, &f32, &f32)> {
        if let Some(r_pix) = self.r.get(x, y) {
            if let Some(g_pix) = self.g.get(x, y) {
                if let Some(b_pix) = self.b.get(x, y) {
                    Some((r_pix, g_pix, b_pix))
                } else {None}
            } else {None}
        } else {None}
    }

    pub fn get_mut_pixel(&mut self, x: usize, y: usize) -> Option<(&mut f32, &mut f32, &mut f32)> {
        if let Some(r_pix) = self.r.get_mut(x, y) {
            if let Some(g_pix) = self.g.get_mut(x, y) {
                if let Some(b_pix) = self.b.get_mut(x, y) {
                    Some((r_pix, g_pix, b_pix))
                } else {None}
            } else {None}
        } else {None}
    }

    pub fn lighten_portion(&mut self, x: usize, y: usize, width: usize, wp: f32) -> Option<()> {

        let mut t1: f32;
        let mut t2: f32;

        let mut pix;
        let mut white_proportion;

        for i in x..(x + width) {
            for j in y..(y + width) {

                pix = self.get_mut_pixel(i, j)?;

                white_proportion = wp;
                *pix.0 = (1.0 - white_proportion) * *pix.0 + white_proportion;
                *pix.1 = (1.0 - white_proportion) * *pix.1 + white_proportion;
                *pix.2 = (1.0 - white_proportion) * *pix.2 + white_proportion;

            }
        }

        Some(())


    }

    pub fn new_empty(width: usize, height: usize) -> ColorMapArray {
        ColorMapArray {r: Arr2d::zeros(width, height), g: Arr2d::zeros(width, height), b: Arr2d::zeros(width, height)}
    }

    pub fn get_width(&self) -> usize {
        self.r.get_width()
    }
    pub fn get_height(&self) -> usize {
        self.r.get_height()
    }
}


#[derive(Clone)]
pub struct Arr2d<T> {
    vector: Vec<T>,
    width: usize, 
    height: usize
}


impl<T> Arr2d<T> {
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= self.width || y >= self.height {
            None
        }
        else {
            self.vector.get(y * self.width + x)
        }
    }

    pub fn geti(&self, x: i32, y: i32) -> Option<&T> {
        let x2: usize;
        let y2: usize;
        x2 = x.try_into().ok()?;
        y2 = y.try_into().ok()?;
        self.get(x2, y2)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.width || y >= self.height{
            None
        }
        else {
            self.vector.get_mut(y * self.width + x)
        }
    }

    pub fn get_vec(&self) -> &Vec<T> { &self.vector }
    pub fn get_mut_vec(&mut self) -> &mut Vec<T> {&mut self.vector}

    pub fn get_width(&self) -> usize {self.width}
    pub fn get_height(&self) -> usize {self.height}
    pub fn get_length(&self) -> usize {self.width * self.height}

    pub fn from_vec(v: Vec<T>, width: usize, height: usize) -> Arr2d<T> {
        assert_eq!(v.len(), width * height);

        Arr2d {vector: v, width, height}
    }

}


impl Arr2d<f32> {
    pub fn zeros(width: usize, height: usize) -> Arr2d<f32> {
        
        let v = vec![0.0; width * height];

        Arr2d { vector: v, width, height }
    }

    pub fn init_with_value(width: usize, height: usize, value: f32) -> Arr2d<f32> {
        let v = vec![value; width * height];

        Arr2d { vector: v, width, height }
    }

    pub fn from_dynamic_image(img: DynamicImage, max_height: f32, generation_settings: &GenerationOptions) -> Arr2d<f32> {

        let w = 2_usize.pow(generation_settings.template_power_of_two) + 1;

        let mut arr = Arr2d::zeros(w, w);
        let mut v: &mut f32;


        for x in 0..w {
            for y in 0..w {
                v = arr.get_mut(x as usize, y as usize).unwrap();
                *v = img.get_pixel(x as u32, y as u32).0[0] as f32 / 255.0 * max_height;
            }
        };
        arr
    }

}

impl Arr2d<f64> {
    pub fn zeros64(width: usize, height: usize) -> Arr2d<f64> {
        
        let v = vec![0.0; width * height];

        Arr2d { vector: v, width, height }
    }
}


pub struct ReducedArrayWrapper<'a, T> {
    array: &'a mut Arr2d<T>,
    n_array: u32, 
    reduced_n: u32
}

impl<T> ReducedArrayWrapper<'_, T> {
    pub fn new(array: &mut Arr2d<T>, n_array: u32, reduced_n: u32) -> ReducedArrayWrapper<T> {

        assert!(n_array >= reduced_n);
        assert_eq!(array.get_width(), array.get_height());
        assert_eq!(array.get_width(), 2_usize.pow(n_array) + 1);

        ReducedArrayWrapper { array, n_array, reduced_n }
    }

    pub fn geti(&self, x: i32, y: i32) -> Option<&T> {
        self.array.geti(
            x * 2_i32.pow(self.n_array - self.reduced_n),
            y * 2_i32.pow(self.n_array - self.reduced_n))
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.array.get(
            x * 2_usize.pow(self.n_array - self.reduced_n),
            y * 2_usize.pow(self.n_array - self.reduced_n))
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.array.get_mut(
            x * 2_usize.pow(self.n_array - self.reduced_n),
            y * 2_usize.pow(self.n_array - self.reduced_n))
    }

    pub fn get_reduced_n(&self) -> u32 {
        self.reduced_n
    }

    pub fn get_scaling(&self) -> usize {
        2_usize.pow(self.n_array - self.reduced_n)
    }

    pub fn convert(&self, x: usize) -> usize {
        x * 2_usize.pow(self.n_array - self.reduced_n)
    }

    pub fn get_reduced_width(&self) -> usize {
        2_usize.pow(self.reduced_n) + 1
    }

    pub fn is_position_valid(&self, x: i32, y: i32) -> bool {
        let width = self.get_reduced_width() as i32;
        0 <= x && x < width && 0 <= y && y < width
    }

}



pub fn linear_interpolation(t: f32, v0: f32, v1: f32) -> f32 {
    assert!(0.0 <= t && t <= 1.0);
    (v1 - v0) * t + v0
}

pub fn bilinear_interpolation(t1: f32, t2: f32, v00: f32, v01: f32, v10: f32, v11:f32) -> f32 {
    let v0 = linear_interpolation(t1, v00, v10);
    let v1 = linear_interpolation(t1, v01, v11);
    linear_interpolation(t2, v0, v1)
} 


pub fn generate_in_interval(max: u16, rng: &mut Mcg128Xsl64) -> u16 {
    let big_number: u64 = rng.next_u64();
    (big_number % max as u64) as u16  // not uniform unless max is a power of two
}

pub fn next_random_number(max: u64, rng: &mut Mcg128Xsl64) -> u32 {
    (rng.next_u32() as u64 * max / TWO_POW_32_MINUS_1 as u64) as u32
}



pub fn rand(rng: &mut Mcg128Xsl64) -> f32 {
    rng.next_u32() as f32 / (2_f32.powi(32))
}


