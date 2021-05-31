#version 450

layout(location = 0) in vec3 v_world_position;
layout(location = 1) in vec3 v_world_normal;
layout(location = 2) in vec4 v_color;

layout(location = 0) out vec4 out_color;

const int MAX_LIGHTS = 64;
const int MAX_DIR_LIGHTS = 8;

struct AmbientLightRaw {
    vec4 color;
    float intensity;
};

struct PointLight {
    vec4 color;
    vec4 data;
};

struct DirectionalLight {
    vec4 color;
    vec3 pos;
    vec4 data;
    mat4 view_proj;
    bool shadows;
};

layout(set = 0, binding = 2) uniform PointLights {
    uint num_point_lights;
    PointLight point_lights[MAX_LIGHTS];
};

layout(set = 0, binding = 3) uniform DirectionalLights {
    uint num_directional_lights;
    DirectionalLight directional_lights[MAX_DIR_LIGHTS];
};

layout(set = 0, binding = 4) uniform AmbientLight {
    AmbientLightRaw ambient;
};

layout(set = 1, binding = 0) uniform texture2DArray DirectionalShadowMaps;
layout(set = 1, binding = 1) uniform sampler ShadowSampler;

void main() {
    vec3 color = v_color.rgb;

    vec3 ambient = ambient.color.rgb * ambient.intensity;

    float sky_diffuse = 0.5 + 0.5 * max(dot(normalize(v_world_normal), vec3(0.0, 1.0, 0.0)), 0.0);

    vec3 light = ambient * sky_diffuse;
    
    for (int i = 0; i < num_point_lights; i++) {
        PointLight point_light = point_lights[i];
        vec3 delta = point_light.data.xyz - v_world_position;
        vec3 direction = normalize(delta);

        float dist = length(delta);
        float falloff = 1.0 / (dist * dist);
        float intensity = falloff * point_light.data.w;

        light += point_light.color.rgb * max(dot(normalize(v_world_normal), direction), 0.0) * intensity;
    }

    for (int i = 0; i < num_directional_lights; i++) {
        DirectionalLight dlight = directional_lights[i];

        vec2 uv = (dlight.view_proj * vec4(v_world_position, 1.0)).xy;
        uv /= 2.0;
        uv.y *= -1.0;
        uv += 0.5;

        const int BLUR = 1;
        float shadow = 0.0;
        vec2 texel_size = 1.0 / vec2(textureSize(sampler2DArray(DirectionalShadowMaps, ShadowSampler), 0));

        if (dlight.shadows) {
            for (int x = -BLUR; x <= BLUR; x++) {
                for (int y = -BLUR; y <= BLUR; y++) {
                    vec2 offset = vec2(x, y) * texel_size / BLUR;

                    float depth = texture(
                        sampler2DArray(DirectionalShadowMaps, ShadowSampler), 
                        vec3(uv + offset, i)
                    ).r;

                    float dist = distance(dlight.pos, v_world_position) / 1000.0;

                    if (any(lessThanEqual(uv + offset, vec2(0.0))) || any(greaterThanEqual(uv + offset, vec2(1.0)))) {
                        shadow += 1.0;
                    } else if (dist < depth + 0.001) {
                        shadow += 1.0;
                    }
                }
            }

            shadow /= pow(BLUR * 2 + 1, 2);
        } else {
            shadow = 1.0;
        }


        float diffuse = max(dot(normalize(v_world_normal), -dlight.data.xyz), 0.0);

        light += dlight.color.rgb * dlight.data.w * shadow * diffuse;
    }

    color *= light;

    out_color = vec4(color, 1.0);
}