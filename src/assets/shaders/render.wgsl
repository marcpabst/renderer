// A simple shader to render a Texture2D to the screen (filling the entire screen)
struct VertexOutput {
    @builtin(position) position: vec4<f32>;
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;


// Vertex shader
@vertex
fn vertex_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // draw a single triangle that covers the screen
    let x = f32(i32(in_vertex_index & 1) * 4 - 1);
    let y = f32(i32(in_vertex_index & 2) * 2 - 1);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0));
}

// Fragment shader
@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler, input.position.xy);
}