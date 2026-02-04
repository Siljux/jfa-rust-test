@group(1) @binding(0)
var<uniform> dimensions: vec2<f32>;

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

struct MouseUniform {
    pos: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> mouse: MouseUniform;

@fragment
fn fs_main(@location(0) in: vec2<f32>, @builtin(position) coords: vec4<f32>) -> @location(0) vec4<f32> {
    let pos = vec2<f32>((in.x + 1) / 2, (in.y - 1) / -2);
    let a = vec2<f32>(.1, .1);
    let b = vec2<f32>(.2, .5);
    let c = vec2<f32>(.9, .7);
    let dist = 0.02;
    if distance(coords.xy, mouse.pos) < 40 {
        return vec4f(0.4, 0.7, 0, 1);
    } else if distance(pos, a) < dist {
        return vec4f(0, 1, 0, 1);
    } else if distance(pos, b) < dist {
        return vec4f(0, 0, 1, 1);
    } else if distance(pos, c) < dist {
        return vec4f(1, 0, 0, 1);
    } else {
        return vec4<f32>(1., 1., 1., 1.);
    }
}
