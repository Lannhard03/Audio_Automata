@group(0) @binding(0) var<storage, read_write> prm: array<u32>;

@group(1) @binding(0) var<storage, read_write> cells1: array<f32>;
@group(1) @binding(1) var<storage, read_write> next_cells1: array<f32>;

@group(2) @binding(0) var<storage, read_write> cells2: array<f32>;
@group(2) @binding(1) var<storage, read_write> next_cells2: array<f32>;

@group(3) @binding(0) var<storage, read_write> cells3: array<f32>;
@group(3) @binding(1) var<storage, read_write> next_cells3: array<f32>;

//@group(1) @binding(2) var<storage, read_write> neigh: array<f32>;


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

    let threshold = 0.00001;

    if (ix >= width || iy >= height) {
        return;
    }
    let mid = iy*width + ix;
    
    let mask = select(0.0, 1.0, cells1[mid]*cells2[mid] >= threshold);

    //Cells 1 and 2 annhilate eachother
    next_cells1[mid] = (1 - mask) * cells1[mid];
    next_cells2[mid] = (1 - mask) * cells2[mid];

    //Creating cells of type 3.
    next_cells3[mid] = clamp(cells3[mid] + mask, 0, 1);






}
     
