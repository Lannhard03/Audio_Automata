@group(0) @binding(0) var<storage, read_write> prm: array<u32>;
@group(0) @binding(1) var<storage> kernel: array<f32>;

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
    let iwidth = i32(prm[0]);
    let iheight = i32(prm[1]);
    let time = prm[2];

    if (ix >= width || iy >= height) {
        return;
    }
    let mid = iy*width + ix;

    var sum: f32 = 0;
    for (var dx: i32 = -2; dx <= 2; dx++) {
        for (var dy: i32 = -2; dy <= 2; dy++) {
            let nx = (i32(ix) + dx + iwidth) % iwidth;
            let ny = clamp(i32(iy) + dy, 0, iheight-1);
            sum += kernel[(dy+2)*5 + (dx + 2)]*cells[iwidth*ny + nx];
        }
    }

    next_cells[mid] = sum;
    if (iy == 0) {
        if ((ix + time) % (width/10) == 0){
            next_cells[mid] = 1;
        } else {
            next_cells[mid] = 0;
        }
    } 

}
     
