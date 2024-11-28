struct Params {
    bias: f32,
    shift: f32,
    scale: f32,
    gamma: f32,
};

fn scaled_inv_srgb_eotf(value: f32, params: Params) -> f32 {
    return (pow(( (1.0 - value) * pow(params.shift, params.gamma) + value * pow((params.shift + params.scale), params.gamma)), (1/params.gamma)) - params.shift) / params.scale;
}


@vertex
fn vs_main(@builtin(vertex_index) ix: u32) -> @builtin(position) vec4<f32> {
    // Generate a full screen quad in normalized device coordinates
    var vertex = vec2(-1.0, 1.0);
    switch ix {
        case 1u: {
            vertex = vec2(-1.0, -1.0);
        }
        case 2u, 4u: {
            vertex = vec2(1.0, -1.0);
        }
        case 5u: {
            vertex = vec2(1.0, 1.0);
        }
        default: {}
    }
    return vec4(vertex, 0.0, 1.0);
}

// bind the input texture to the shader
@group(0) @binding(0)
var fine_output: texture_2d<f32>;

// bind the uniform buffer to the shader
@group(0) @binding(1)
var<uniform> params: Params;


@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let rgba_sep = textureLoad(fine_output, vec2<i32>(pos.xy), 0);

    let rgb_pm = vec3(rgba_sep.rgb * rgba_sep.a);

    // Convert the linear RGB to sRGB for every pixel
    let rgb = vec3(
        scaled_inv_srgb_eotf(rgb_pm.r, params),
        scaled_inv_srgb_eotf(rgb_pm.g, params),
        scaled_inv_srgb_eotf(rgb_pm.b, params)
    );

    return vec4(rgb, rgba_sep.a);
}