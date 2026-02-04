struct VertexInput {
    @location(0) pos: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos: vec2<f32>,
}

@vertex
fn vs_main(
    point: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(point.pos, 0.0, 1.0);

    out.pos = vec2<f32>(point.pos.x, point.pos.y);
    return out;
}

@group(0) @binding(0)
var t_jfa: texture_2d<f32>;
@group(0) @binding(1)
var s_jfa: sampler;

@group(1) @binding(0)
var t_material: texture_2d<f32>;
@group(1) @binding(1)
var s_material: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = vec2<f32>((in.pos.x + 1) / 2, (in.pos.y - 1) / -2);
    // return textureSample(t_jfa, s_jfa, pos);
    // return textureSample(t_material, s_material, pos);
    return textureSample(t_material, s_material, tex_coords(decode_coords(textureSample(t_jfa, s_jfa, pos))));
}

fn decode_coords(color: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(decode_data(color.rg), decode_data(color.ba));
}

fn decode_data(encoded: vec2<f32>) -> f32 {
    return encoded.x * 65025 + encoded.y * 255;
}

fn tex_coords(coords: vec2<f32>) -> vec2<f32> {
    return coords/2048; // hardcoded dimensions
}
