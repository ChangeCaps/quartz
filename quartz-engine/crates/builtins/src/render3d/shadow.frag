#version 450

layout(location = 0) in vec3 v_world_position;

layout(set = 0, binding = 2) uniform CameraPos {
    vec3 camera_pos;
};

void main() {
    gl_FragDepth = distance(camera_pos, v_world_position) / 1000.0;
}