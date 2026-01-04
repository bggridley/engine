#version 450

layout(location = 0) in vec2 frag_uv;
layout(location = 1) in vec3 frag_color;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D fontTexture;
layout(set = 0, binding = 1) uniform sampler fontSampler;

void main() {
    // Sample the main text alpha
    float alpha = texture(sampler2D(fontTexture, fontSampler), frag_uv).r;
    
    // Add very subtle shadow for contrast
    vec2 texelSize = 1.0 / vec2(textureSize(sampler2D(fontTexture, fontSampler), 0));
    float shadow = 0.0;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(-1.0, -1.0) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(1.0, -1.0) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(-1.0, 1.0) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(1.0, 1.0) * texelSize).r;
    shadow *= 0.08; // Very subtle
    
    // Combine
    float finalAlpha = clamp(alpha + shadow * (1.0 - alpha), 0.0, 1.0);
    
    out_color = vec4(frag_color, finalAlpha);
}