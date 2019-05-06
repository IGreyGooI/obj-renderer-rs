#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 1) uniform UniformBlock {
    vec4 ambient_light;
} uniform_block;

layout(set = 1, binding = 0) uniform sampler2D normal_texture;
layout(set = 2, binding = 0) uniform sampler2D diffuse_texture;
layout(set = 3, binding = 0) uniform sampler2D specular_texture;

layout(location = 0) in vec3 view;
layout(location = 1) in vec3 light;
layout(location = 2) in vec2 texture_coord;

layout(location = 0) out vec4 fColor;

void main()
{
    // Sample the textures.
    vec4 normal = texture(normal_texture, texture_coord);
    vec4 diffuse = texture(diffuse_texture, texture_coord);
    vec4 specular = texture(specular_texture, texture_coord);

    // Determine the per-fragment lighting vectors.

    vec3 N = normalize(2.0 * normal.xyz - 1.0);
    vec3 L = normalize(light);
    vec3 V = normalize(view);
    vec3 R = reflect(L, N);

    // Compute the diffuse shading.

    float kd = max(dot(L, N), 0.0);
    float ks = pow(max(dot(V, R), 0.0), 8.0);

    // Calculate the fragment color.

    fColor.rgb = vec3(uniform_block.ambient_light * diffuse + kd * diffuse + specular * ks);
    fColor.a   = diffuse.a;

    //fColor = diffuse;
}