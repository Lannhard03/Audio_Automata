// A read-only storage buffer that stores and array of unsigned 32bit integers
@group(0) @binding(0) var<storage, read_write> input: array<f32>;
// This storage buffer can be read from and written to
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<storage, read> prm: array<u32>;

@group(0) @binding(3)
var out_texture: texture_storage_2d<rgba8unorm, write>;


// Tells wgpu that this function is a valid compute pipeline entry_point
@compute
// Specifies the "dimension" of this work group
@workgroup_size(16, 16)
fn cp_main(
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

    let mid = ix + (width * iy);
    let right = (ix+1)%width;
    let left = (ix+width-1)%width; //We add width - 1 to avoid underflow
    let bot = width*((iy+1)%height);     
    let top = width*((iy-1+height)%height); 
      

    // a simple copy operation
    let neigh = input[left + top     ] +    input[ix + top] + input[right + top     ] +
                input[left + width*iy] + 10*input[mid     ] + input[right + width*iy] +
                input[left + bot     ] +    input[ix + bot] + input[right + bot     ];

    output[mid] = f32(neigh == 3) + f32(neigh == 12) + f32(neigh == 13);

    storageBarrier();
    //Convert buffer to texture
    let on =  f32(output[mid]);
    let color = vec4<f32>(on, on, on, 1.0);


    textureStore(out_texture, vec2<u32>(ix, iy), color);

}

