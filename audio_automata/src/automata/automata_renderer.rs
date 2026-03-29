use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use crate::automata::automata_state::AutomataState;
use crate::data::Texture;

pub struct AutomataTexturerShared {
    width: u32,
    height: u32,
    wg_size: u32,
    pub texture: Texture,
    pub compute_bindgroups_even: wgpu::BindGroup,
    pub compute_bindgroups_odd: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    //Temporary as even_frame is no longer global
    even_frame: bool,
}

pub trait AutomataTexturer {
    fn get_data(&mut self) -> &mut AutomataTexturerShared;

    fn update_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let data = self.get_data();

        let wg_size = data.wg_size;
        let num_dispatches_x = data.width.div_ceil(wg_size) as u32;
        let num_dispatches_y = data.height.div_ceil(wg_size) as u32;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
           label: Some("Automata Texture Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&Default::default());
            compute_pass.set_pipeline(&data.compute_pipeline);
            if data.even_frame {
                compute_pass.set_bind_group(0, &data.compute_bindgroups_even, &[]);
            } else {
                compute_pass.set_bind_group(0, &data.compute_bindgroups_odd, &[]);
            }
            data.even_frame = !data.even_frame;
            compute_pass.dispatch_workgroups(num_dispatches_x, num_dispatches_y, 1);
        }
        queue.submit([encoder.finish()]);
    }
}

pub struct BasicAutomataTexturer {
    data: AutomataTexturerShared,
}

impl AutomataTexturer for BasicAutomataTexturer {
    fn get_data(&mut self) -> &mut AutomataTexturerShared {
        return &mut self.data;
    }  
} 

impl BasicAutomataTexturer {
    //State should be a list of states,
    //but then I need to understand buffer offsets
    pub fn new(state: &AutomataState, width: u32, height: u32, 
               device: &wgpu::Device, queue: &wgpu::Queue) -> BasicAutomataTexturer {

        let texture = match Texture::new(device, queue, width, height) {
            Ok(tex) => tex,
            Err(_) => panic!("Could not create texture"),
        };

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
                    resource: state.even_buffer.as_entire_binding(),
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
                    resource: state.odd_buffer.as_entire_binding(),
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

        let data = AutomataTexturerShared {
            width: state.width,
            height: state.height,
            wg_size: state.work_group_size,
            texture,
            compute_bindgroups_even: compute_bindgroup_even,
            compute_bindgroups_odd: compute_bindgroup_odd,
            compute_pipeline: pipeline,
            even_frame: true,
        };

        return BasicAutomataTexturer { 
            data
        }
    }

}

pub struct AnnihilationAutomataTexturer {
    data: AutomataTexturerShared,
}

impl AutomataTexturer for AnnihilationAutomataTexturer {
    fn get_data(&mut self) -> &mut AutomataTexturerShared {
        return &mut self.data;
    }  
} 

impl AnnihilationAutomataTexturer {
    //State should be a list of states,
    //but then I need to understand buffer offsets
    pub fn new(states: &Vec<AutomataState>, width: u32, height: u32, 
               device: &wgpu::Device, queue: &wgpu::Queue) -> AnnihilationAutomataTexturer {

        let texture = match Texture::new(device, queue, width, height) {
            Ok(tex) => tex,
            Err(_) => panic!("Could not create texture"),
        };

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
                                ty: wgpu::BindingType::Buffer { 
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
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
                            wgpu::BindGroupLayoutEntry {
                                binding: 4,
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
                    resource: states[0].even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: states[1].even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: states[2].even_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
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
                    resource: states[0].odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: states[1].odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: states[2].odd_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: prm_buffer.as_entire_binding(),
                },
            ],
        });


        let texture_shader = device.create_shader_module(wgpu::include_wgsl!("annihilation_automata_texture.wgsl"));
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

        let data = AutomataTexturerShared {
            width: states[0].width,
            height: states[0].height,
            wg_size: states[0].work_group_size,
            texture,
            compute_bindgroups_even: compute_bindgroup_even,
            compute_bindgroups_odd: compute_bindgroup_odd,
            compute_pipeline: pipeline,
            even_frame: true,
        };

        return AnnihilationAutomataTexturer { 
            data
        }
    }

}
