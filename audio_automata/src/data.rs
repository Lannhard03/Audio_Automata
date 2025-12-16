use wgpu::{SurfaceError, util::{BufferInitDescriptor, DeviceExt}};
use rand::Rng;
use anyhow::Result;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3], //x,y,z coords
    tex_coords: [f32; 2], //rgb color
}

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.8, -0.8, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.8, -0.8, 0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.8,  0.8, 0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.8,  0.8, 0.0], tex_coords: [0.0, 0.0] },
];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

pub const INDICES: &[u16] = &[
    0, 1, 2,
    0, 2, 3,
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeParams {
    height: u32,
    width: u32, //OBS this is assumed to have uniform type
    //Maybe need to pad data to a multiple of 16 bytes?
}

//Convenience wrapper for buffer to texture conversion
pub struct ComputeTexture {
    pub width: u32,
    pub height: u32,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_bind_group_even: wgpu::BindGroup,
    pub compute_bind_group_odd: wgpu::BindGroup,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: wgpu::Extent3d,
}

impl ComputeTexture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) -> Result<Self> {
        let mut rng = rand::rng();
        let grid_width = width;
        let grid_height = height;
        let compute_prm: &[u32] = &[width, height];
        let input_data: Vec<f32> = (0..(grid_height*grid_width))
                                   .map(|_x| rng.random_range(0..2) as f32).collect(); 
                                    //faster to use a uniform dist,
                                    //but doesnt matter
        println!("{:?}", input_data.len());
        
        let texture_size = wgpu::Extent3d {
            width: grid_width,
            height: grid_height,
            depth_or_array_layers: 1, //Textures are 3D, setting this to 1 givse a 2D image
        };
        
        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB, so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8Unorm,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::STORAGE_BINDING |
                       wgpu::TextureUsages::TEXTURE_BINDING | 
                       wgpu::TextureUsages::COPY_DST,

                label: Some("diffuse_texture"),
                view_formats: &[],
            }
        );

        let diffuse_bytes = include_bytes!("../dboi.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &diffuse_rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * grid_width),
                rows_per_image: Some(grid_height),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat, 
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
                    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                // This should match the filterable field of the
                                // corresponding Texture entry above.
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    });

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );



        let even_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("input"),
            contents: bytemuck::cast_slice(&input_data),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });

        let odd_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output"),
            size: even_buffer.size(),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("parameters"),
            contents: bytemuck::cast_slice(compute_prm),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, //maybe don't need
                                                                               //both these usages?
        });

        let compute_bind_group_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },                   
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 3,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::StorageTexture { 
                                    access: wgpu::StorageTextureAccess::WriteOnly, 
                                    format: wgpu::TextureFormat::Rgba8Unorm, 
                                    view_dimension: wgpu::TextureViewDimension::D2, 
                                },
                                count: None,
                            },
                        ],
                        label: Some("compute_bind_group_layout"),
                    });

        //Binding 0 = input, bindning 1 = output
        let compute_bind_group_even = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: prm_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
            ],
        });

        let compute_bind_group_odd = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: prm_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
            ],
        });


        return Ok(Self {
            width,
            height,
            texture_bind_group: diffuse_bind_group,
            texture_bind_group_layout,
            compute_bind_group_even,
            compute_bind_group_odd,
            compute_bind_group_layout,
            texture: diffuse_texture,
            view: diffuse_texture_view,
            sampler: diffuse_sampler,
            size: texture_size,
        });
    }
}
