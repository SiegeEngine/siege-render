#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) out vec4 outFragColor;

void main() {
  // 0.21404 goes to sRGB 50% (web page #808080)
  // This "srgb-linear" space we are in is luminance-linear, not
  //   'brightness' linear.
  outFragColor = vec4(0.21404, 0.21404, 0.21404, 1.0);
}
