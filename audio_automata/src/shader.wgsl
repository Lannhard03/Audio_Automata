struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

//Vertex shader: Determine geometry
//The shader below calculates verticies of a triangle (for vertex_indices 1, 2, 3)
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

//Fragment shader: Determine how to rasterize (?) the geometry
//Below we store the output to location(0) which has something to do with color?
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(1.0, 0.0, 1.0, 1.0);
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
//   let tex_size = textureDimensions(t_diffuse);
//   let coord = vec2<i32>(
//           i32(in.tex_coords.x * f32(tex_size.x)),
//           i32(in.tex_coords.y * f32(tex_size.y))
//           );

//   return textureLoad(t_diffuse, coord, 0);
}


 

 
