#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 fragColor;

layout(push_constant) uniform PushConstants {
    mat4 projection;
    mat4 transform;
    vec3 colorModulation;
} push;

void main() {
    gl_Position = push.projection * push.transform * vec4(position, 0.0, 1.0);
    fragColor = color * push.colorModulation;  // Apply color modulation for hover effects
}
