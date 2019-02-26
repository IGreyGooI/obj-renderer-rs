#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 1, binding = 0) uniform texture2D NormalTexture;
layout(set = 1, binding = 1) uniform sampler NormalTextureSampler;
layout(set = 2, binding = 0) uniform texture2D DiffuseTexture;
layout(set = 2, binding = 1) uniform sampler DiffuseTextureSampler;
layout(set = 3, binding = 0) uniform texture2D SpecularTexture;
layout(set = 3, binding = 1) uniform sampler SpecularTextureSampler;

layout(location = 0) in vec3 fView;
layout(location = 1) in vec3 fLight;
layout(location = 2) in vec2 fTexCoord;

layout(location = 0) out vec4 fColor;

void main()
{
    // Sample the textures.
    vec4 normal = texture(sampler2D(NormalTexture, NormalTextureSampler),   fTexCoord);
    vec4 diffuse = texture(sampler2D(DiffuseTexture, DiffuseTextureSampler),  fTexCoord);
    vec4 specular = texture(sampler2D(SpecularTexture, SpecularTextureSampler), fTexCoord);

    // Determine the per-fragment lighting vectors.

    vec3 N = normalize(2.0 * normal.xyz - 1.0);
    vec3 L = normalize(fLight);
    vec3 V = normalize(fView);
    vec3 R = reflect(L, N);

    // Compute the diffuse shading.

    float kd =     max(dot(L, N), 0.0);
    float ks = pow(max(dot(V, R), 0.0), 8.0);

    // Calculate the fragment color.

    fColor.rgb = vec3(kd * diffuse + specular * ks);
    fColor.a   = diffuse.a;
}