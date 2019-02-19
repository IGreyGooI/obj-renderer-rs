#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBlock {
    mat4 projection_matrix;
    mat4 model_view_matrix;
    mat3 normal_matrix;
    vec4 light_position;
} uniform_block;


layout(location = 0) in vec4 vPosition;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in vec3 vTangent;
layout(location = 3) in vec2 vTexCoord;

layout(location = 0) out vec3 fView;
layout(location = 1) out vec3 fLight;
layout(location = 2) out vec2 fTexCoord;

void main()
{
    // Tangent space vectors give the columns of the eye-to-tangent transform.

    vec3 N = uniform_block.normal_matrix * vNormal;
    vec3 T = uniform_block.normal_matrix * vTangent;
    mat3 M = transpose(mat3(T, cross(N, T), N));

    // Compute the per-fragment attributes.

    fView     =  M * vec3(uniform_block.model_view_matrix * vPosition);
    fLight    =  M * vec3(uniform_block.model_view_matrix * uniform_block.light_position);
    fTexCoord =  vTexCoord;

    gl_Position = uniform_block.projection_matrix
        * uniform_block.model_view_matrix
        * vPosition;
}