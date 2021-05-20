#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec4 v_color;

layout(set = 0, binding = 0) uniform ScreenSize {
    vec2 screen_size;
};

void main() {
    v_uv = uv;
    v_color = color;
    gl_Position = vec4(2.0 * pos.x / screen_size.x - 1.0, 1.0 - 2.0 * pos.y / screen_size.y, 0.0, 1.0);
}