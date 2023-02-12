#version 450

layout(binding = 0) uniform UniformBufferObject {
  mat4 model;
  mat4 view;
  mat4 proj;
  mat4 normal;
}
ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 fragColor;

const vec3 DIRECTION_TO_LIGHT = normalize(vec3(1.0, -3.0, -1.0));
const float AMBIENT = 0.02;

void main() {
  gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);

  vec3 normalWorldSpace = normalize(mat3(ubo.normal) * normal);
  float lightIntensity =
      max(dot(normalWorldSpace, DIRECTION_TO_LIGHT), AMBIENT);

  fragColor = lightIntensity * color;
}