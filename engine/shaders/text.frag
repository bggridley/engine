#version 450

layout(location = 0) in vec2 frag_uv;
layout(location = 1) in vec3 frag_color;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D fontTexture;
layout(set = 0, binding = 1) uniform sampler fontSampler;

void main() {
    // Simple clean text rendering without gamma correction or effects
    float alpha = texture(sampler2D(fontTexture, fontSampler), frag_uv).r;
    
    out_color = vec4(frag_color, alpha);
}