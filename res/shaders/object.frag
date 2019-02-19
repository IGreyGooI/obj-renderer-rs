#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBlock {
    vec4 ambient_light;
} uniform_block;


layout(binding = 1) uniform sampler2D NormalTexture;
layout(binding = 2) uniform sampler2D DiffuseTexture;
layout(binding = 3) uniform sampler2D SpecularTexture;

layout(location = 0) in vec3 fView;
layout(location = 1) in vec3 fLight;
layout(location = 2) in vec2 fTexCoord;

layout(location = 0) out vec4 fColor;

void main()
{
    // Sample the textures.

    vec4 tN = texture(NormalTexture,   fTexCoord);
    vec4 tD = texture(DiffuseTexture,  fTexCoord);
    vec4 tS = texture(SpecularTexture, fTexCoord);

    // Determine the per-fragment lighting vectors.

    vec3 N = normalize(2.0 * tN.xyz - 1.0);
    vec3 L = normalize(fLight);
    vec3 V = normalize(fView);
    vec3 R = reflect(L, N);

    // Compute the diffuse shading.

    float kd =     max(dot(L, N), 0.0);
    float ks = pow(max(dot(V, R), 0.0), 8.0);

    // Calculate the fragment color.

    fColor.rgb = vec3(uniform_block.ambient_light * tD + kd * tD + tS * ks);
    fColor.a   = tD.a;
}