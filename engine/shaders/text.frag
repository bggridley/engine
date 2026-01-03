#version 450

layout(location = 0) in vec2 frag_uv;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D fontTexture;
layout(set = 0, binding = 1) uniform sampler fontSampler;

void main() {
    float alpha = texture(sampler2D(fontTexture, fontSampler), frag_uv).r;
    
    // Sample neighboring pixels for subtle shadow/outline effect
    vec2 texelSize = 1.0 / vec2(textureSize(sampler2D(fontTexture, fontSampler), 0));
    float shadow = 0.0;
    const float shadowOffset = 1.2;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(-shadowOffset, -shadowOffset) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(shadowOffset, -shadowOffset) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(-shadowOffset, shadowOffset) * texelSize).r;
    shadow += texture(sampler2D(fontTexture, fontSampler), frag_uv + vec2(shadowOffset, shadowOffset) * texelSize).r;
    shadow *= 0.15; // Subtle shadow strength
    
    // Gamma correction for better contrast (sRGB to linear)
    alpha = pow(alpha, 2.2);
    shadow = pow(shadow, 2.2);
    
    // Combine text with subtle shadow/glow
    float finalAlpha = clamp(alpha + shadow * (1.0 - alpha), 0.0, 1.0);
    
    // Convert back to sRGB for display
    finalAlpha = pow(finalAlpha, 1.0/2.2);
    
    out_color = vec4(1.0, 1.0, 1.0, finalAlpha);
}