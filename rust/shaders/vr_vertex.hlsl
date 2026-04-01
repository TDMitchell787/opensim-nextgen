// OpenSim Next - VR Vertex Shader
// High-performance vertex shader for VR rendering with stereo instancing support

struct VertexInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 texcoord : TEXCOORD0;
    uint instance_id : SV_InstanceID;
};

struct VertexOutput {
    float4 position : SV_POSITION;
    float3 world_position : WORLD_POSITION;
    float3 normal : NORMAL;
    float2 texcoord : TEXCOORD0;
    uint eye_index : BLENDINDICES;
};

cbuffer CameraConstants : register(b0) {
    float4x4 view_matrix_left;
    float4x4 projection_matrix_left;
    float4x4 view_matrix_right;
    float4x4 projection_matrix_right;
    float4x4 world_matrix;
    float3 camera_position;
    float padding;
};

cbuffer InstanceConstants : register(b1) {
    float4x4 instance_transforms[1000];
};

VertexOutput main(VertexInput input) {
    VertexOutput output;
    
    // Determine which eye we're rendering for (0 = left, 1 = right)
    uint eye_index = input.instance_id % 2;
    uint object_instance = input.instance_id / 2;
    
    // Apply instance transform
    float4x4 instance_transform = instance_transforms[object_instance];
    float4x4 world_transform = mul(world_matrix, instance_transform);
    
    // Transform to world space
    float4 world_pos = mul(world_transform, float4(input.position, 1.0));
    output.world_position = world_pos.xyz;
    
    // Transform normal to world space
    output.normal = normalize(mul((float3x3)world_transform, input.normal));
    
    // Apply view and projection based on eye
    float4x4 view_matrix = (eye_index == 0) ? view_matrix_left : view_matrix_right;
    float4x4 projection_matrix = (eye_index == 0) ? projection_matrix_left : projection_matrix_right;
    
    float4 view_pos = mul(view_matrix, world_pos);
    output.position = mul(projection_matrix, view_pos);
    
    // Pass through texture coordinates
    output.texcoord = input.texcoord;
    output.eye_index = eye_index;
    
    return output;
}