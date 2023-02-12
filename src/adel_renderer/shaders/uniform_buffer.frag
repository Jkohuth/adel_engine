#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstantData {
  mat4 model_matrix;
  mat4 normal_matrix;
}
push;

void main() { outColor = vec4(fragColor, 1.0); }