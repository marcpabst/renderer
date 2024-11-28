@compute

fn linear_to_srgb(is_piecewise: bool, scale: f32, bias: f32, gamma: f32, linear_rgb: vec3<f32>) -> vec3<f32> {
    // sRGB hardware can either use a piecewise function or a pure gamma function
    if is_piecewise {
        // the piecewise function is a linear function for small values and a power function for large values
        // L(P) = (P * 12.92) for P <= 0.0031308
        // L(P) = (1.055 * P^(1/2.4) - 0.055) for P > 0.0031308
        // where P is the linear RGB value and L is the sRGB value

        // NOT IMPLEMENTED YET
        return linear_rgb;
    } else {
        // the sRGB transfer function is luminance(value) = scale * pow(bias + value, gamma),
        // where value is the
    }
}

@workgroup_size(1)
