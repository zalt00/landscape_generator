use std::convert::TryInto;

use image::{DynamicImage, GenericImageView};

pub const TWO_POW_32_MINUS_1: u32 = 4294967295;
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

    pub fn from_dynamic_image(img: DynamicImage, max_height: f32) -> Arr2d<f32> {

        let mut arr = Arr2d::zeros(129, 129);
        let mut v: &mut f32;


        for x in 0..65_u32 {
            for y in 0..65_u32 {
                v = arr.get_mut(x as usize, y as usize).unwrap();
                *v = img.get_pixel(x, y).0[0] as f32 / 255.0 * max_height;
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
