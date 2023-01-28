#version 450

layout(location = 0) in vec3 inColor;
layout(location = 0) out vec4 outColor;
layout(push_constant) uniform PushConstantData {
    mat4 transform;
    vec3 color;
} push;
void main() {
    outColor = vec4(inColor + push.color, 1.0);
    //outColor = vec4(inColor, 1.0);
}