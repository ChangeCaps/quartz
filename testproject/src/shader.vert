#version 450 

layout(location = 0) in vec3 vertex_position;

layout(set = 0, binding = 0) uniform Model {
    mat4 model;
};

layout(set = 0, binding = 1) uniform ViewProj {
    mat4 view_proj;
};

void main() {
    vec4 pos = view_proj * model * vec4(vertex_position, 1.0);
    
    gl_Position = pos;
}