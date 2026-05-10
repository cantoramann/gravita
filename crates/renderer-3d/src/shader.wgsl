// Vertex + fragment shader for unlit per-vertex colored meshes, with a simple
// directional Lambert term so geometry reads as 3D instead of flat-shaded.

struct Globals {
    view_proj: mat4x4<f32>,
    light_dir: vec3<f32>,
    _padding0: f32,
    ambient: vec3<f32>,
    _padding1: f32,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct InstanceIn {
    @location(3) row0: vec4<f32>,
    @location(4) row1: vec4<f32>,
    @location(5) row2: vec4<f32>,
    @location(6) row3: vec4<f32>,
    @location(7) tint: vec4<f32>,
};

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexIn, inst: InstanceIn) -> VertexOut {
    let model = mat4x4<f32>(inst.row0, inst.row1, inst.row2, inst.row3);
    let world_pos = model * vec4<f32>(in.position, 1.0);
    // Normal transform: assume uniform scale; for non-uniform scale a proper
    // inverse-transpose would be required.
    let world_normal = normalize((model * vec4<f32>(in.normal, 0.0)).xyz);

    var out: VertexOut;
    out.clip_position = globals.view_proj * world_pos;
    out.color = in.color * inst.tint.rgb;
    out.world_normal = world_normal;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let lambert = max(dot(normalize(in.world_normal), normalize(globals.light_dir)), 0.0);
    let lit = in.color * (globals.ambient + vec3<f32>(lambert));
    return vec4<f32>(lit, 1.0);
}
