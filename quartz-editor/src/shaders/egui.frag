#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec4 v_color;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D tex;
layout(set = 1, binding = 1) uniform sampler tex_sampler;

void main() {
    vec4 color = v_color * texture(sampler2D(tex, tex_sampler), v_uv);
    out_color = color;
}