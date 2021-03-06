#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 vPosition;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in vec3 vTangent;
layout(location = 3) in vec2 vTexCoord;

layout(set = 0, binding = 0) uniform UniformBlock {
    mat4 projection_matrix;
    mat4 view_matrix;
    mat4 normal_matrix;
    vec3 light_position;
} uniform_block;

layout(location = 0) out vec3 fView;
layout(location = 1) out vec3 fLight;
layout(location = 2) out vec2 fTexCoord;

out gl_PerVertex {
  vec4 gl_Position;
};

void main()
{
    // Tangent space vectors give the columns of the eye-to-tangent transform.
    vec4 vPosition4 = vec4(vPosition, 1.0);
    vec4 vlight_position4 = vec4(uniform_block.light_position, 1.0);

    vec3 N = vec3(uniform_block.normal_matrix * vec4(vNormal, 0.0));
    vec3 T = vec3(uniform_block.normal_matrix * vec4(vTangent, 0.0));
    mat3 M = transpose(mat3(T, cross(N, T), N));

    // Compute the per-fragment attributes.

    fView     =  M * vec3(uniform_block.view_matrix * vPosition4);
    fLight    =  M * vec3(uniform_block.view_matrix * vlight_position4);
    fTexCoord =  vTexCoord;

    gl_Position = uniform_block.projection_matrix
        * uniform_block.view_matrix
        * vPosition4;
}