#version 450

layout(location = 0) in vec2 v_pos;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec4 v_color;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D tex;
layout(set = 1, binding = 1) uniform sampler tex_sampler;

layout(set = 0, binding = 1) uniform ClipRect {
    vec4 clip_rect;
};

void main() {
    vec4 color = v_color * texture(sampler2D(tex, tex_sampler), v_uv);

    if (any(lessThan(v_pos, clip_rect.xy)) || any(greaterThan(v_pos, clip_rect.zw))) {
        color = vec4(0.0);
    }

    out_color = color;
}