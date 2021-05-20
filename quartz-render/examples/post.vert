#version 450

const vec2 position[6] = vec2[6](
	vec2(-1.0, -1.0),
	vec2(1.0, -1.0),
	vec2(-1.0, 1.0),
	vec2(1.0, 1.0),
	vec2(-1.0, 1.0),
	vec2(1.0, -1.0)
);

layout(location = 0) out vec2 v_uv;

void main() {
    vec2 pos = position[gl_VertexIndex];
    gl_Position = vec4(pos, 0.0, 1.0);
	pos.y *= -1.0;
    v_uv = pos * 0.5 + 0.5;
}
