#version 450 

const vec2 position[3] = vec2[3](
	vec2(-1.0, -1.0),
	vec2(1.0, -1.0),
	vec2(-1.0, 1.0)
);

layout(set = 0, binding = 0) uniform Model {
    mat4 model;
};

layout(set = 0, binding = 1) uniform ViewProj {
    mat4 view_proj;
};

layout(set = 0, binding = 2) uniform Aspect {
    float aspect;
};

void main() {
    vec4 pos = view_proj * model * vec4(position[gl_VertexIndex], 0.0, 1.0);

    pos.x /= aspect;

    gl_Position = pos;
}