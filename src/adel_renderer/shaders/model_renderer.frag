#version 450

layout(location = 0) in vec3 frag_color;
layout(location = 1) in vec3 frag_pos_world;
layout(location = 2) in vec3 frag_normal_world;
layout(location = 0) out vec4 out_color;

struct PointLight {
  vec4 position; // ignore w
  vec4 color;    // w is intensity
};

layout(set = 0, binding = 0) uniform GlobalUbo {
  mat4 projection;
  mat4 view;
  vec4 ambient_light_color; // w is intensity
  PointLight point_lights[10];
  int num_lights;
}
ubo;

layout(push_constant) uniform PushConstantData {
  mat4 model_matrix;
  mat4 normal_matrix;
}
push;

void main() {
  vec3 diffuse_light = ubo.ambient_light_color.xyz * ubo.ambient_light_color.w;
  vec3 surface_normal = normalize(frag_normal_world);

  for (int i = 0; i < ubo.num_lights; i++) {
    PointLight light = ubo.point_lights[i];
    vec3 direction_to_light = light.position.xyz - frag_pos_world;
    float attenuation = 1.0 / dot(direction_to_light, direction_to_light);
    float cos_angle_incidence =
        max(dot(surface_normal, normalize(direction_to_light)), 0);
    vec3 intensity = light.color.xyz * light.color.w * attenuation;
    diffuse_light += intensity * cos_angle_incidence;
  }
  out_color = vec4(diffuse_light * frag_color, 1.0);
}