#version 450

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec2 vertex_uv;
layout(location = 3) in vec4 vertex_color;

layout(location = 0) out vec3 v_world_position;
layout(location = 1) out vec3 v_world_normal;
layout(location = 2) out vec4 v_color;

layout(set = 0, binding = 0) uniform Transform {
    mat4 model;
};

layout(set = 0, binding = 1) uniform Camera {
    mat4 view_proj;
};

void main() {
    v_world_position = (model * vec4(vertex_position, 1.0)).xyz;
    v_world_normal = normalize((model * vec4(vertex_normal, 0.0)).xyz);
    v_color = vertex_color;
    gl_Position = view_proj * model * vec4(vertex_position, 1.0);
}