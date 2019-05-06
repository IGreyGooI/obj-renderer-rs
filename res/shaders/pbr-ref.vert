// The MIT License

// Copyright (c) 2019 Kikokushi <s468zhan@edu.uwaterloo.ca>
// Copyright (c) 2016-2017 Mohamad Moneimne and Contributors

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software
// and associated documentation files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
// BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#version 450
#extension GL_ARB_separate_shader_objects : enable

#define HAS_NORMALS;
#define HAS_TANGENTS;
#define HAS_UV;

layout(location = 0) in vec4 i_Position;
#ifdef HAS_NORMALS
layout(location = 1) in vec4 i_Normal;
#endif
#ifdef HAS_TANGENTS
layout(location = 2) in vec4 i_Tangent;
#endif
#ifdef HAS_UV
layout(location = 3) in vec2 i_UV;
#endif

layout(set = 0, binding = 0) uniform ViewMatrix {
    mat4 mvp;
    mat4 model;
    mat4 normal;
} U_VIEW_MATRIX;

layout(location = 0) out vec3 o_position;
layout(location = 1) out vec2 o_uv;

#ifdef HAS_NORMALS
#ifdef HAS_TANGENTS
layout(location = 2) out mat3 o_tbn;
#else
layout(location = 3) out vec3 o_normal;
#endif
#endif


void main()
{
    vec4 pos = U_VIEW_MATRIX.model * i_Position;
    o_position = vec3(pos.xyz) / pos.w;

    #ifdef HAS_NORMALS
    #ifdef HAS_TANGENTS
    vec3 normalW = normalize(vec3(U_VIEW_MATRIX.normal * vec4(i_Normal.xyz, 0.0)));
    vec3 tangentW = normalize(vec3(U_VIEW_MATRIX.model * vec4(i_Tangent.xyz, 0.0)));
    vec3 bitangentW = cross(normalW, tangentW) * i_Tangent.w;
    o_tbn = mat3(tangentW, bitangentW, normalW);
    #else// HAS_TANGENTS != 1
    o_normal = normalize(vec3(U_VIEW_MATRIX.model * vec4(i_Normal.xyz, 0.0)));
    #endif
    #endif

    #ifdef HAS_UV
    o_uv = i_UV;
    #else
    o_uv = vec2(0., 0.);
    #endif

    gl_Position = U_VIEW_MATRIX.mvp * i_Position;// needs w for proper perspective correction
}