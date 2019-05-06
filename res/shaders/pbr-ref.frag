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


// This fragment shader defines a reference implementation for Physically Based Shading of
// a microfacet surface material defined by a glTF model.
//
// References:
// [1] Real Shading in Unreal Engine 4
//     http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
// [2] Physically Based Shading at Disney
//     http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
// [3] README.md - Environment Maps
//     https://github.com/KhronosGroup/glTF-Sample-Viewer/#environment-maps
// [4] "An Inexpensive BRDF Model for Physically based Rendering" by Christophe Schlick
//     https://www.cs.virginia.edu/~jdl/bib/appearance/analytic%20models/schlick94b.pdf
#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_shader_texture_lod: enable
#extension GL_OES_standard_derivatives : enable

#define USE_IBL;
#define HAS_BASECOLORMAP;
#define HAS_NORMALMAP;
#define HAS_EMISSIVEMAP;
#define HAS_METALROUGHNESSMAP;
#define HAS_OCCLUSIONMAP;
#define HAS_NORMALS;
#define HAS_TANGENTS;

precision highp float;

layout(set = 1, binding = 0) uniform FragmentConstants {
    vec3 light_direction;
    vec3 light_color;
    vec2 metallic_roughness_values;
    vec4 base_color_factor;
    vec3 camera;
// debugging flags used for shader output of intermediate PBR variables
    vec4 scale_diff_base_mr;
    vec4 scale_fgd_spec;
    vec4 scale_ibl_ambient;
} U_FRAGMENT_CONSTANTS;

#ifdef USE_IBL
layout(set = 2, binding = 0) uniform samplerCube U_DIFFUSE_ENV_SAMPLER;
layout(set = 2, binding = 1) uniform samplerCube U_SPECULAR_ENV_SAMPLER;
layout(set = 2, binding = 2) uniform sampler2D U_BRDF_LUT;
#endif

#ifdef HAS_BASECOLORMAP
layout(set = 3, binding = 0) uniform sampler2D U_BASE_COLOR_SAMPLER;
#endif

#ifdef HAS_NORMALMAP
layout(set = 4, binding = 0) uniform sampler2D U_NORMAL_SAMPLER;
layout(set = 4, binding = 1) uniform NormalScale {
    float normal_scale;
} U_NORMAL_SCALE;
#endif

#ifdef HAS_EMISSIVEMAP
layout(set = 5, binding = 0) uniform sampler2D U_EMISSIVE_SAMPLER;
layout(set = 5, binding = 1) uniform EmissiveFactor {
    vec3 emissive_factor;
} U_EMISSIVE_FACTOR;
#endif

#ifdef HAS_METALROUGHNESSMAP
layout(set = 6, binding = 0) uniform sampler2D U_METALLIC_ROUGHNESS_SAMPLER;
#endif

#ifdef HAS_OCCLUSIONMAP
layout(set = 7, binding = 0) uniform sampler2D U_OCCLUSION_SAMPLER;
layout(set = 7, binding = 1) uniform OcclusionStrength {
    float occlusion_strength;
} U_OCCLUSION_STRENGTH;
#endif


layout(location = 0) in vec3 i_position;
layout(location = 1) in vec2 i_uv;

#ifdef HAS_NORMALS
#ifdef HAS_TANGENTS
layout(location = 2) in mat3 i_ibn;
#else
layout(location = 3) in vec3 i_normal;
#endif
#endif

layout(location = 0) out vec4 fColor;

// Encapsulate the various inputs used by the various functions in the shading equation
// We store values in this struct to simplify the integration of alternative implementations
// of the shading terms, outlined in the Readme.MD Appendix.
struct PBRInfo
{
    float NdotL;// cos angle between normal and light direction
    float NdotV;// cos angle between normal and view direction
    float NdotH;// cos angle between normal and half vector
    float LdotH;// cos angle between light direction and half vector
    float VdotH;// cos angle between view direction and half vector
    float perceptualRoughness;// roughness value, as authored by the models creator (input to shader)
    float metalness;// metallic value at the surface
    vec3 reflectance0;// full reflectance color (normal incidence angle)
    vec3 reflectance90;// reflectance color at grazing angle
    float alphaRoughness;// roughness mapped to a more linear change in the roughness (proposed by [2])
    vec3 diffuseColor;// color contribution from diffuse lighting
    vec3 specularColor;// color contribution from specular lighting
};

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;

vec4 SRGBtoLINEAR(vec4 srgbIn)
{
    #ifdef MANUAL_SRGB
    #ifdef SRGB_FAST_APPROXIMATION
    vec3 linOut = pow(srgbIn.xyz, vec3(2.2));
    #else//SRGB_FAST_APPROXIMATION
    vec3 bLess = step(vec3(0.04045), srgbIn.xyz);
    vec3 linOut = mix(srgbIn.xyz/vec3(12.92), pow((srgbIn.xyz+vec3(0.055))/vec3(1.055), vec3(2.4)), bLess);
    #endif//SRGB_FAST_APPROXIMATION
    return vec4(linOut, srgbIn.w);
    #else//MANUAL_SRGB
    return srgbIn;
    #endif//MANUAL_SRGB
}

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
vec3 getNormal()
{
    // Retrieve the tangent space matrix
    #ifndef HAS_TANGENTS
    vec3 pos_dx = dFdx(i_position);
    vec3 pos_dy = dFdy(i_position);
    vec3 tex_dx = dFdx(vec3(i_uv, 0.0));
    vec3 tex_dy = dFdy(vec3(i_uv, 0.0));
    vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);

    #ifdef HAS_NORMALS
    vec3 ng = normalize(i_normal);
    #else
    vec3 ng = cross(pos_dx, pos_dy);
    #endif

    t = normalize(t - ng * dot(ng, t));
    vec3 b = normalize(cross(ng, t));
    mat3 tbn = mat3(t, b, ng);
    #else// HAS_TANGENTS
    mat3 tbn = i_ibn;
    #endif

    #ifdef HAS_NORMALMAP
    vec3 n = texture(U_NORMAL_SAMPLER, i_uv).rgb;
    n = normalize(tbn * ((2.0 * n - 1.0) * vec3(
    U_NORMAL_SCALE.normal_scale,
    U_NORMAL_SCALE.normal_scale,
    1.0)));
    #else
    // The tbn matrix is linearly interpolated, so we need to re-normalize
    vec3 n = normalize(tbn[2].xyz);
    #endif

    return n;
}

    // Calculation of the lighting contribution from an optional Image Based Light source.
    // Precomputed Environment Maps are required uniform inputs and are computed as outlined in [1].
    // See our README.md on Environment Maps [3] for additional discussion.
    #ifdef USE_IBL
vec3 getIBLContribution(PBRInfo pbrInputs, vec3 n, vec3 reflection)
{
    float mipCount = 9.0;// resolution of 512x512
    float lod = (pbrInputs.perceptualRoughness * mipCount);
    // retrieve a scale and bias to F0. See [1], Figure 3
    vec3 brdf = SRGBtoLINEAR(texture(U_BRDF_LUT, vec2(pbrInputs.NdotV, 1.0 - pbrInputs.perceptualRoughness))).rgb;
    vec3 diffuseLight = SRGBtoLINEAR(texture(U_DIFFUSE_ENV_SAMPLER, n)).rgb;

    #ifdef USE_TEX_LOD
    vec3 specularLight = SRGBtoLINEAR(textureCubeLodEXT(U_SPECULAR_ENV_SAMPLER, reflection, lod)).rgb;
    #else
    vec3 specularLight = SRGBtoLINEAR(texture(U_SPECULAR_ENV_SAMPLER, reflection)).rgb;
    #endif

    vec3 diffuse = diffuseLight * pbrInputs.diffuseColor;
    vec3 specular = specularLight * (pbrInputs.specularColor * brdf.x + brdf.y);

    // For presentation, this allows us to disable IBL terms
    diffuse *= U_FRAGMENT_CONSTANTS.scale_ibl_ambient.x;
    specular *= U_FRAGMENT_CONSTANTS.scale_ibl_ambient.y;

    return diffuse + specular;
}
    #endif

// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
vec3 diffuse(PBRInfo pbrInputs)
{
    return pbrInputs.diffuseColor / M_PI;
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
vec3 specularReflection(PBRInfo pbrInputs)
{
    return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
}

// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
float geometricOcclusion(PBRInfo pbrInputs)
{
    float NdotL = pbrInputs.NdotL;
    float NdotV = pbrInputs.NdotV;
    float r = pbrInputs.alphaRoughness;

    float attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));
    float attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
    return attenuationL * attenuationV;
}

// The following equation(s) models the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
float microfacetDistribution(PBRInfo pbrInputs)
{
    float roughnessSq = pbrInputs.alphaRoughness * pbrInputs.alphaRoughness;
    float f = (pbrInputs.NdotH * roughnessSq - pbrInputs.NdotH) * pbrInputs.NdotH + 1.0;
    return roughnessSq / (M_PI * f * f);
}

void main()
{
    // Metallic and Roughness material properties are packed together
    // In glTF, these factors can be specified by fixed scalar values
    // or from a metallic-roughness map
    float perceptual_roughness = U_FRAGMENT_CONSTANTS.metallic_roughness_values.y;
    float metallic = U_FRAGMENT_CONSTANTS.metallic_roughness_values.x;

    #ifdef HAS_METALROUGHNESSMAP
    // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
    // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
    vec4 mrSample = texture(U_METALLIC_ROUGHNESS_SAMPLER, i_uv);
    perceptual_roughness = mrSample.g * perceptual_roughness;
    metallic = mrSample.b * metallic;
    #endif

    perceptual_roughness = clamp(perceptual_roughness, c_MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);
    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    float alphaRoughness = perceptual_roughness * perceptual_roughness;

    // The albedo may be defined from a base texture or a flat color
    #ifdef HAS_BASECOLORMAP
    vec4 baseColor = SRGBtoLINEAR(texture(U_BASE_COLOR_SAMPLER, i_uv)) * U_FRAGMENT_CONSTANTS.base_color_factor;
    #else
    vec4 baseColor = U_FRAGMENT_CONSTANTS.base_color_factor;
    #endif

    vec3 f0 = vec3(0.04);
    vec3 diffuseColor = baseColor.rgb * (vec3(1.0) - f0);
    diffuseColor *= 1.0 - metallic;
    vec3 specularColor = mix(f0, baseColor.rgb, metallic);

    // Compute reflectance.
    float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    vec3 specularEnvironmentR0 = specularColor.rgb;
    vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    vec3 n = getNormal();// normal at surface point
    vec3 v = normalize(U_FRAGMENT_CONSTANTS.camera - i_position);// Vector from surface point to camera
    vec3 l = normalize(U_FRAGMENT_CONSTANTS.light_direction);// Vector from surface point to light
    vec3 h = normalize(l+v);// Half vector between both l and v
    vec3 reflection = -normalize(reflect(v, n));

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
    float NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float LdotH = clamp(dot(l, h), 0.0, 1.0);
    float VdotH = clamp(dot(v, h), 0.0, 1.0);

    PBRInfo pbrInputs = PBRInfo(
    NdotL,
    NdotV,
    NdotH,
    LdotH,
    VdotH,
    perceptual_roughness,
    metallic,
    specularEnvironmentR0,
    specularEnvironmentR90,
    alphaRoughness,
    diffuseColor,
    specularColor
    );

    // Calculate the shading terms for the microfacet specular shading models
    vec3 F = specularReflection(pbrInputs);
    float G = geometricOcclusion(pbrInputs);
    float D = microfacetDistribution(pbrInputs);

    // Calculation of analytical lighting contribution
    vec3 diffuseContrib = (1.0 - F) * diffuse(pbrInputs);
    vec3 specContrib = F * G * D / (4.0 * NdotL * NdotV);
    // Obtain final intensity as reflectance (BRDF) scaled by the energy of the light (cosine law)
    vec3 color = NdotL * U_FRAGMENT_CONSTANTS.light_color * (diffuseContrib + specContrib);

    // Calculate lighting contribution from image based lighting source (IBL)
    #ifdef USE_IBL
    color += getIBLContribution(pbrInputs, n, reflection);
    #endif

    // Apply optional PBR terms for additional (optional) shading
    #ifdef HAS_OCCLUSIONMAP
    float ao = texture(U_OCCLUSION_SAMPLER, i_uv).r;
    color = mix(color, color * ao, U_OCCLUSION_STRENGTH.occlusion_strength);
    #endif

    #ifdef HAS_EMISSIVEMAP
    vec3 emissive = SRGBtoLINEAR(texture(U_EMISSIVE_SAMPLER, i_uv)).rgb * U_EMISSIVE_FACTOR.emissive_factor;
    color += emissive;
    #endif

    // This section uses mix to override final color for reference app visualization
    // of various parameters in the lighting equation.
    color = mix(color, F, U_FRAGMENT_CONSTANTS.scale_fgd_spec.x);
    color = mix(color, vec3(G), U_FRAGMENT_CONSTANTS.scale_fgd_spec.y);
    color = mix(color, vec3(D), U_FRAGMENT_CONSTANTS.scale_fgd_spec.z);
    color = mix(color, specContrib, U_FRAGMENT_CONSTANTS.scale_fgd_spec.w);

    color = mix(color, diffuseContrib, U_FRAGMENT_CONSTANTS.scale_diff_base_mr.x);
    color = mix(color, baseColor.rgb, U_FRAGMENT_CONSTANTS.scale_diff_base_mr.y);
    color = mix(color, vec3(metallic), U_FRAGMENT_CONSTANTS.scale_diff_base_mr.z);
    color = mix(color, vec3(perceptual_roughness), U_FRAGMENT_CONSTANTS.scale_diff_base_mr.w);

    fColor = vec4(pow(color, vec3(1.0/2.2)), baseColor.a);
}