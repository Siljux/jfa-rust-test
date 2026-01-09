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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let mouse_pos = vec2<f32>(mouse.pos.x/dimensions.x * 2 - 1, mouse.pos.y/dimensions.y * -2.0 + 1);
    let a = vec2<f32>(.1, .1);
    let b = vec2<f32>(-.6, .3);
    let c = vec2<f32>(.6, -.4);
    let dist = 0.04;
    if distance(in.pos, mouse_pos) < dist {
        return seed(mouse_pos);
    } else if distance(in.pos, a) < dist {
        return seed(a);
    } else if distance(in.pos, b) < dist {
        return seed(b);
    } else if distance(in.pos, c) < dist {
        return seed(c);
    } else {
        return vec4<f32>(1., 1., 1., 1.);
    }
}

fn seed(pos: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(encode_data((pos.x + 1) / 2), encode_data((pos.y - 1) / -2));
}

fn encode_data(data: f32) -> vec2<f32> {
    return vec2<f32>(floor(data / 255.), data % 255.);
}
