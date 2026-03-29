// A read-only storage buffer that stores and array of unsigned 32bit integers
@group(0) @binding(0) var<storage, read> cells1: array<f32>;
@group(0) @binding(1) var<storage, read> cells2: array<f32>;
@group(0) @binding(2) var<storage, read> cells3: array<f32>;
@group(0) @binding(3)
var out_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var<storage, read> prm: array<u32>;

//Convert computed automata state into a texture


// Tells wgpu that this function is a valid compute pipeline entry_point
@compute
// Specifies the "dimension" of this work group
@workgroup_size(16, 16)
fn main(
    // global_invocation_id specifies our position in the invocation grid
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let ix = global_id.x;
    let iy = global_id.y;
    let width = prm[0];
    let height = prm[1];

    let cell1_color = vec4<f32>(0.0, 0.2, 0.8, 1.0);
    let cell2_color = vec4<f32>(0.8, 0.2, 0.0, 1.0);
    let cell3_color = vec4<f32>(1, 1, 1, 1.0);


    if (ix >= width || iy >= height) {
        return;
    }

    let index = ix + (width * iy);

    let color = cell1_color*cells1[index] + cell2_color*cells2[index] + cell3_color*cells3[index];

    textureStore(out_texture, vec2<u32>(ix, iy), color);
}
