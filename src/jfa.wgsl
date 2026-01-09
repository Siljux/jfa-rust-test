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

    out.pos = point.pos;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> step: f32;

@fragment
fn fs_main(@location(0) in: vec2<f32>) -> @location(0) vec4<f32> {
    let pos = vec2<f32>((in.x + 1) / 2, (in.y - 1) / -2);

    let data = textureSample(t_diffuse, s_diffuse, pos);
    var closest = decode_coords(data);

    // if (distance(closest, pos) < 0.1) {
    //     return vec4<f32>(.0, .0, .0, 1.);
    // }

    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(-step, -step));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(0, -step));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(step, -step));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(-step, 0));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(step, 0));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(-step, step));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(0, step));
    closest = compare_point_with_offset(pos, closest, pos + vec2<f32>(step, step));

    return vec4<f32>(encode_data(closest.x), encode_data(closest.y));
}

fn compare_point_with_offset(pos: vec2<f32>, current: vec2<f32>, new_pos: vec2<f32>) -> vec2<f32> {
    let other = decode_coords(textureSample(t_diffuse, s_diffuse, new_pos));

    if distance(pos, other) < distance(pos, current) {
        return other;
    }
    return current;
}

fn decode_coords(color: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(decode_data(color.rg), decode_data(color.ba));
}

fn encode_data(data: f32) -> vec2<f32> {
    return vec2<f32>(floor(data / 255.), data % 255.);
}

fn decode_data(encoded: vec2<f32>) -> f32 {
    return encoded.x * 255. + encoded.y;
}
