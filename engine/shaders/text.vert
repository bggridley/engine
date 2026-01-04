#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(push_constant) uniform PushConstant {
    mat4 projection;
    mat4 transform;
    vec3 colorModulation;
} pc;

layout(location = 0) out vec2 frag_uv;
layout(location = 1) out vec3 frag_color;

void main() {
    vec4 pos = pc.projection * pc.transform * vec4(position, 0.0, 1.0);
    gl_Position = pos;
    frag_uv = uv;
    frag_color = pc.colorModulation;
}
