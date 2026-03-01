@group(0) @binding(0) var<storage, read> prm: array<u32>;

@group(1) @binding(0) var<storage, read_write> cells: array<f32>;
@group(1) @binding(1) var<storage, read_write> next_cells: array<f32>;
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

    if (ix >= width || iy >= height) {
        return;
    }

    let mid = ix + (width * iy);
    let right = (ix+1)%width;
    let left = (ix+width-1)%width; //We add (width - 1) to avoid underflow
    let bot = width*((iy+1)%height);     
    let top = width*((iy-1+height)%height); 
      

    // a simple copy operation
    let neigh =  cells[left + top     ] +    cells[ix + top] + cells[right + top     ] +
                 cells[left + width*iy] + 10*cells[mid     ] + cells[right + width*iy] +
                 cells[left + bot     ] +    cells[ix + bot] + cells[right + bot     ];

    next_cells[mid] = f32(neigh == f32(prm[2])) 
                    + f32(neigh == f32(prm[3])) 
                    + f32(neigh == f32(prm[4]));
}
