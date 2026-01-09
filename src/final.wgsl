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
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = vec2<f32>((in.pos.x + 1) / 2, (in.pos.y - 1) / -2);
    return textureSample(t_diffuse, s_diffuse, pos);
}
