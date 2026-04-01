// OpenSim Next - VR Pixel Shader
// High-performance pixel shader for VR rendering with foveated rendering support

struct PixelInput {
    float4 position : SV_POSITION;
    float3 world_position : WORLD_POSITION;
    float3 normal : NORMAL;
    float2 texcoord : TEXCOORD0;
    uint eye_index : BLENDINDICES;
};

struct PixelOutput {
    float4 color : SV_TARGET0;
    float depth : SV_DEPTH;
};

Texture2D diffuse_texture : register(t0);
Texture2D normal_texture : register(t1);
Texture2D foveated_mask : register(t2);
SamplerState texture_sampler : register(s0);

cbuffer MaterialConstants : register(b0) {
    float4 base_color;
    float metallic;
    float roughness;
    float2 foveated_center;
    float foveated_radius;
    float3 padding;
};

cbuffer LightingConstants : register(b1) {
    float3 light_direction;
    float light_intensity;
    float3 light_color;
    float ambient_intensity;
    float3 camera_position;
    float padding2;
};

float3 calculate_pbr_lighting(float3 albedo, float3 normal, float3 view_dir, float metallic, float roughness) {
    // Simplified PBR lighting calculation
    float3 light_dir = normalize(-light_direction);
    float ndotl = max(dot(normal, light_dir), 0.0);
    
    // Diffuse component
    float3 diffuse = albedo * ndotl * light_color * light_intensity;
    
    // Specular component (simplified)
    float3 half_vector = normalize(light_dir + view_dir);
    float ndoth = max(dot(normal, half_vector), 0.0);
    float spec_power = pow(2.0 / (roughness * roughness + 0.001), 2.0);
    float3 specular = pow(ndoth, spec_power) * light_color * light_intensity * metallic;
    
    // Ambient
    float3 ambient = albedo * ambient_intensity;
    
    return diffuse + specular + ambient;
}

float calculate_foveated_quality(float2 screen_pos) {
    // Calculate distance from foveated center
    float2 center = foveated_center;
    float distance = length(screen_pos - center);
    
    // Calculate quality based on distance from center
    float quality = 1.0 - smoothstep(0.0, foveated_radius, distance);
    return max(quality, 0.25); // Minimum 25% quality
}

PixelOutput main(PixelInput input) {
    PixelOutput output;
    
    // Sample textures
    float4 albedo = diffuse_texture.Sample(texture_sampler, input.texcoord) * base_color;
    float3 normal_map = normal_texture.Sample(texture_sampler, input.texcoord).xyz * 2.0 - 1.0;
    
    // Transform normal from tangent to world space (simplified)
    float3 world_normal = normalize(input.normal + normal_map * 0.1);
    
    // Calculate view direction
    float3 view_dir = normalize(camera_position - input.world_position);
    
    // Apply PBR lighting
    float3 lit_color = calculate_pbr_lighting(albedo.rgb, world_normal, view_dir, metallic, roughness);
    
    // Apply foveated rendering quality
    float2 screen_pos = input.position.xy / input.position.w;
    screen_pos = screen_pos * 0.5 + 0.5; // Convert to [0,1] range
    float quality = calculate_foveated_quality(screen_pos);
    
    // Reduce quality for peripheral vision
    if (quality < 1.0) {
        // Simple quality reduction - could be more sophisticated
        lit_color = lerp(lit_color * 0.5, lit_color, quality);
    }
    
    // Apply gamma correction
    lit_color = pow(lit_color, 1.0 / 2.2);
    
    output.color = float4(lit_color, albedo.a);
    output.depth = input.position.z;
    
    return output;
}