#version 450

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 o_target;

layout(set = 0, binding = 0) uniform texture2D test_tex;
layout(set = 0, binding = 1) uniform sampler test_sampler;

layout(set = 1, binding = 0) uniform Color {
    vec4 color;
};

void main() {
    o_target = texture(sampler2D(test_tex, test_sampler), v_uv) * color;
}