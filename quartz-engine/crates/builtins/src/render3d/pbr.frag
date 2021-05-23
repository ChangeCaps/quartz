#version 450

layout(location = 0) in vec3 v_world_position;
layout(location = 1) in vec3 v_world_normal;

layout(location = 0) out vec4 out_color;

const int MAX_LIGHTS = 64;

layout(set = 0, binding = 2) uniform Lights {
    uint num_lights;
    vec3 lights[MAX_LIGHTS];
};

void main() {
    vec3 color = vec3(1.0);

    float light = 0.0;
    
    for (int i = 0; i < num_lights; i++) {
        vec3 direction = normalize(lights[i] - v_world_position);

        light += max(dot(v_world_normal, direction), 0.0);
    }

    color *= light;

    out_color = vec4(color, 1.0);
}