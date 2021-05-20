#version 450

layout(location = 0) in vec2 vertex_position;
 
layout(set = 0, binding = 0) uniform CameraProj {
	mat4 proj;
};

layout(set = 0, binding = 1) uniform Transform {
	mat4 model;
};

void main() {
	gl_Position = proj * model * vec4(vertex_position, 0.0, 1.0);
}
