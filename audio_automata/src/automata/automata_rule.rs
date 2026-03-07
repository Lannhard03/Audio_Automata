use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use crate::automata::automata_state::AutomataState;

pub struct AutomataInteraction {
    num_states: u32,
    pipeline: wgpu::ComputePipeline,
    prm: Vec<u32>,
    prm_buffer: wgpu::Buffer,
    prm_bindgroup: wgpu::BindGroup,
    prm_bindgroup_layout: wgpu::BindGroupLayout,
}


impl AutomataInteraction {
    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        return &self.pipeline;
    }
    pub fn get_prm_bindgroup(&self) -> &wgpu::BindGroup {
        return &self.prm_bindgroup;
    }
    pub fn get_prm_bindgroup_layout(&self) -> &wgpu::BindGroupLayout {
        return &self.prm_bindgroup_layout;
    }

    pub fn update_prm_bindgroup(&mut self, new_prm: Vec<u32>, queue: &wgpu::Queue) {
        if new_prm.len() != self.prm.len() {
            print!("Incorrect number of parameters, cannot update!");
            return;
        }

        self.prm = new_prm;
        queue.write_buffer(&self.prm_buffer, 0, bytemuck::cast_slice(&self.prm));
    }

    fn create_convolution_rule(shader_module: wgpu::ShaderModule, aut_bindgroup_layout: wgpu::BindGroupLayout,
                   num_states: u32, rule_name: &'static str, 
                   prm: Vec<u32>, kernel: Vec<f32>, device: &wgpu::Device) -> Self {

        let prm_bindgroup_layout = 
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
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false, //Maybe should be true?
                                min_binding_size: None,
                            },
                            count: None,
                        }, 
                    ],
                    label: Some("rule_bind_group_layout"),
                });

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("parameters"),
                contents: bytemuck::cast_slice(&prm),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, //maybe don't need
                                                                                   //both these usages?
            });

        let kernel_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("kernel"),
                contents: bytemuck::cast_slice(&kernel),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, //maybe don't need
                                                                                   //both these usages?
            });
  
        let prm_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &prm_bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: prm_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = Vec::new();
        bind_group_layouts.push(&prm_bindgroup_layout);
        for _i in 0..num_states {
            bind_group_layouts.push(&aut_bindgroup_layout);
        }

        let prm_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline layout"),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[], //What does this one do?? Something with buffers?
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(rule_name),
            layout: Some(&prm_pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"), //Correct name?
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        return AutomataInteraction {
            num_states,
            pipeline, 
            prm_buffer,
            prm_bindgroup, 
            prm_bindgroup_layout,
            prm,
        }
    }

}

//A rule for a single automata (state).
impl AutomataInteraction {
    pub fn apply_rule(&self, even_frame: bool, states: &Vec<AutomataState>, compute_pass: &mut wgpu::ComputePass,
        num_disp_x: u32, num_disp_y: u32) {
        if states.len() as u32 != self.num_states {
            panic!("Incorrect number of states for this rule");
        }

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.prm_bindgroup, &[]);
        for (i, state) in states.iter().enumerate() {
            if even_frame {
                compute_pass.set_bind_group((i+1) as u32, &state.automata_bindgroup_even, &[]);
            } else {
                compute_pass.set_bind_group((i+1) as u32, &state.automata_bindgroup_odd, &[]);
            }
        }
        compute_pass.dispatch_workgroups(num_disp_x, num_disp_y, 1);
    }

    pub fn conway_rule(width: u32, height: u32, aut_bindgroup_layout: wgpu::BindGroupLayout,
                       device: &wgpu::Device) -> AutomataInteraction {
        let prm = vec![width, height, 3, 12, 13]; 
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("conway_automata.wgsl"));

        let kernel = vec![1.0, 1.0,  1.0, 
                          1.0, 10.0, 1.0, 
                          1.0, 1.0,  1.0];
        return AutomataInteraction::create_convolution_rule(shader_module, aut_bindgroup_layout, 1,
                                                "conway rule", prm, kernel, device);
    }


    pub fn spectral_rain_rule(width: u32, height: u32, aut_bindgroup_layout: wgpu::BindGroupLayout,
                       device: &wgpu::Device) -> AutomataInteraction {
        let prm = vec![width, height]; 

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("spectral_rain.wgsl"));
        
        let kernel = vec![0.1, 0.8, 0.1, 
                          0.0, 0.0, 0.0, 
                          0.0, 0.0, 0.0];

        return AutomataInteraction::create_convolution_rule(shader_module, aut_bindgroup_layout, 1,
                                                "spectral rain rule", prm, kernel, device);

    }
}
