use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use crate::automata::automata_state::AutomataState;
use crate::data::Texture;

pub struct AutomataTexturer {
    //automata: Vec<AutomataState>,
    //Render rules? Colors etc
    width: u32,
    height: u32,
    wg_size: u32,
    //pub texture_bind_group: wgpu::BindGroup,
    pub texture: Texture,
    //pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_bindgroup_even: wgpu::BindGroup,
    pub compute_bindgroup_odd: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline,

    //Temporary as even_frame is no longer global
    even_frame: bool,
}

impl AutomataTexturer {
    //State should be a list of states,
    //but then I need to understand buffer offsets
    pub fn new(state: &Vec<AutomataState>, width: u32, height: u32, device: &wgpu::Device, queue: &wgpu::Queue) -> AutomataTexturer {

        let texture = match Texture::new(device, queue, width, height) {
            Ok(tex) => tex,
            Err(_) => panic!("Could not create texture"),
        };

        /*
        let texture_size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1, //Textures are 3D, setting this to 1 gives a 2D image
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

        let diffuse_bytes = include_bytes!("../../dboi.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        //This will get overwritten after one frame, but good for debugging
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
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
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



        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: texture_bindgroup_layout,
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
        */

        let rule_prm = &[width, height]; 

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("parameters"),
            contents: bytemuck::cast_slice(rule_prm),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, //maybe don't need
                                                                               //both these usages?
        });

        let compute_bindgroup_layout = 
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false, //Maybe should be true?
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::COMPUTE,
                                ty: wgpu::BindingType::StorageTexture { 
                                    access: wgpu::StorageTextureAccess::WriteOnly, 
                                    format: wgpu::TextureFormat::Rgba8Unorm, 
                                    view_dimension: wgpu::TextureViewDimension::D2, 
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
                        ],
                        label: Some("compute_bind_group_layout"),
        });


        let compute_bindgroup_even = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state[0].even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: prm_buffer.as_entire_binding(),
                },
            ],
        });


        let compute_bindgroup_odd = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state[0].odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: prm_buffer.as_entire_binding(),
                },
            ],
        });


        let texture_shader = device.create_shader_module(wgpu::include_wgsl!("automata_texture.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline layout"),
            bind_group_layouts: &[
                &compute_bindgroup_layout,
            ],
            push_constant_ranges: &[], //What does this one do?? Something with buffers?
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Automata texture Pipeline"),
            layout: Some(&pipeline_layout),
            module: &texture_shader,
            entry_point: Some("main"), //Correct name?
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        return  AutomataTexturer { 
            //texture_bind_group: diffuse_bind_group,
            //texture_bind_group_layout,
            texture,
            compute_bindgroup_even,
            compute_bindgroup_odd,
            compute_pipeline: pipeline,
            width: state[0].width,
            height: state[0].height,
            wg_size: state[0].work_group_size,
            even_frame: true,
        }
    }

    pub fn update_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let wg_size = self.wg_size;
        let num_dispatches_x = self.width.div_ceil(wg_size) as u32;
        let num_dispatches_y = self.height.div_ceil(wg_size) as u32;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
           label: Some("Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&self.compute_pipeline);
            if self.even_frame {
                compute_pass.set_bind_group(0, &self.compute_bindgroup_even, &[]);
            } else {
                compute_pass.set_bind_group(0, &self.compute_bindgroup_odd, &[]);
            }
            self.even_frame = !self.even_frame;
            compute_pass.dispatch_workgroups(num_dispatches_x, num_dispatches_y, 1);
        }
        queue.submit([encoder.finish()]);
    }
}
