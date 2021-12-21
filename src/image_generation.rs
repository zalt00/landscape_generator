use crate::utils::{Arr2d, ColorMapArray};




pub fn generate_colormap_image(cm: &ColorMapArray) -> Vec<u8> {
    let mut output: Vec<u8> = vec![];
    let mut v: u16;

    for x in 0..cm.get_width() {
        for y in 0..cm.get_height() {
            if let Some((r, g, b)) = cm.get_pixel(x, y) {

                v = (r * 65535.0) as u16;
                output.push((v % 256) as u8);
                output.push((v >> 8) as u8);

                v = (g * 65535.0) as u16;
                output.push((v % 256) as u8);
                output.push((v >> 8) as u8);

                v = (b * 65535.0) as u16;
                output.push((v % 256) as u8);
                output.push((v >> 8) as u8);

            } else {
                println!("error - invalid pixel position: x={} y={}", x, y);
                output.push(0_u8);
                output.push(0_u8);
                output.push(0_u8);
                output.push(0_u8);
                output.push(0_u8);
                output.push(0_u8);
            }
        }
    }

    output
}

pub fn generate_heightmap_image(v: &Arr2d<f32>, flattens: bool) -> Vec<u8> {

    println!("converting to image buffer...");

    let mut min_value = f32::INFINITY;
    let mut max_value = -f32::INFINITY;

    for value in v.get_vec().iter() {
        min_value = f32::min(min_value, *value);
        max_value = f32::max(max_value, *value);
    }

    let mut normalized_value: f32;

    let mut output: Vec<u8> = vec![];

    println!("begin iterations.");

    for n in v.get_vec().iter() {
        normalized_value = (n - min_value) / (max_value - min_value);

        assert!(0.0 <= normalized_value && normalized_value <= 1.0, "invalid value: {}", normalized_value);
        
        if flattens {
            normalized_value = normalized_value.sqrt();
        }

        output.push((normalized_value * 255.0) as u8);
        output.push((normalized_value * 255.0) as u8);
        output.push((normalized_value * 255.0) as u8);
    };

    println!("conversion done.");

    output

}


pub fn generate_test_image() -> Vec<u8> {
    let mut output = Vec::new();

    for i in 0..=255 {
        output.push(i);
        output.push(12);
        output.push(i);
        output.push(12);
        output.push(i);
        output.push(12);

    }
    for i in 0..=255 {
        output.push(i);
        output.push(13);
        output.push(i);
        output.push(13);
        output.push(i);
        output.push(13);

    }

    output
}






