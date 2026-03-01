// A read-only storage buffer that stores and array of unsigned 32bit integers
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1)
var out_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<storage, read> prm: array<u32>;

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

    if (ix >= width || iy >= height) {
        return;
    }

    let index = ix + (width * iy);

    let on =  input[index];
    let color = vec4<f32>(0.0*on, 0.2*on, 0.8*on, 1.0);

    textureStore(out_texture, vec2<u32>(ix, iy), color);
}
