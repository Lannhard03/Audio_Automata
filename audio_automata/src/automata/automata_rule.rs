use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use winit::keyboard::KeyCode;
use crate::{app::UpdateInfo, automata::automata_state::AutomataState};

struct AutomataInteractionShared {
    //Use vec of indicies for now, maybe change to slotmap later?
    state_indicies: Vec<usize>,
    pipeline: wgpu::ComputePipeline,
    prm_bindgroup: wgpu::BindGroup,
    prm_bindgroup_layout: wgpu::BindGroupLayout,
    num_disp_x: u32,
    num_disp_y: u32,
}


pub trait AutomataInteraction {
    fn update_prm(&mut self, info: &UpdateInfo, queue: &wgpu::Queue);
    fn get_interaction_data(&self) -> &AutomataInteractionShared;

    fn apply_interaction(&self, states: &Vec<AutomataState>, compute_pass: &mut wgpu::ComputePass) {
        let data = self.get_interaction_data();

        compute_pass.set_pipeline(&data.pipeline);
        compute_pass.set_bind_group(0, &data.prm_bindgroup, &[]);
        for i in data.state_indicies.iter() {
            let state = &states[*i];
            if state.even_frame {
                compute_pass.set_bind_group((i+1) as u32, &state.automata_bindgroup_even, &[]);
            } else {
                compute_pass.set_bind_group((i+1) as u32, &state.automata_bindgroup_odd, &[]);
            }
        }
        compute_pass.dispatch_workgroups(data.num_disp_x, data.num_disp_y, 1);
    }
}



impl AutomataInteractionShared {
    fn create_convolution_rule(state_indicies: Vec<usize>, num_states: u32, shader_module: wgpu::ShaderModule, 
                               aut_bindgroup_layout: wgpu::BindGroupLayout, rule_name: &'static str, 
                               prm_buffer: &wgpu::Buffer, kernel_buffer: &wgpu::Buffer, 
                               disp_x: u32, disp_y: u32,
                               device: &wgpu::Device) -> Self {

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

        return AutomataInteractionShared {
            state_indicies,
            pipeline, 
            prm_bindgroup, 
            prm_bindgroup_layout,
            num_disp_x: disp_x,
            num_disp_y: disp_y,
        }
    }

}

pub struct ConwayInteraction {
    data: AutomataInteractionShared,
    prm: Vec<u32>,
    prm_buffer: wgpu::Buffer,
    kernel: Vec<f32>,
    kernel_buffer: wgpu::Buffer,
}

impl AutomataInteraction for ConwayInteraction {
    fn update_prm(&mut self, info: &UpdateInfo, queue: &wgpu::Queue) {
        for keypress in info.key_presses.iter() {
            if *keypress == KeyCode::KeyU {
                self.prm[2] = 30;
                self.prm[2] = 12;
                self.prm[2] = 13;

                queue.write_buffer(&self.prm_buffer, 0, bytemuck::cast_slice(&self.prm));
                //Do we nned to update bindgroup also?
                //self.ecosystem.get_interaction_ref()[0].update_prm_bindgroup(new_prm, &gpu.queue);
            }
        }
    }

    fn get_interaction_data(&self) -> &AutomataInteractionShared {
        return &self.data;
    }
}

impl ConwayInteraction {
    pub fn new(width: u32, height: u32, state_indicies: Vec<usize>,
               aut_bindgroup_layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("conway_automata.wgsl"));

        let prm = vec![width, height, 3, 12, 13];
        let kernel = vec![1.0, 1.0,  1.0, 
                          1.0, 10.0, 1.0, 
                          1.0, 1.0,  1.0];

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("parameters"),
                contents: bytemuck::cast_slice(&prm),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

        let kernel_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("kernel"),
                contents: bytemuck::cast_slice(&kernel),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

        let wg_size = 16;
        let disp_x = width.div_ceil(wg_size) as u32;
        let disp_y = height.div_ceil(wg_size) as u32;


        let data = AutomataInteractionShared::
                   create_convolution_rule(state_indicies, 1, shader_module, aut_bindgroup_layout,
                   "conway rule", &prm_buffer, &kernel_buffer, disp_x, disp_y, device);

        return ConwayInteraction { 
            data, 
            prm,
            prm_buffer,
            kernel,
            kernel_buffer,
        }
    }

}


pub struct RainInteraction {
    data: AutomataInteractionShared,
    prm: Vec<u32>,
    prm_buffer: wgpu::Buffer,
    kernel: Vec<f32>,
    kernel_buffer: wgpu::Buffer,
}

impl AutomataInteraction for RainInteraction {
    fn update_prm(&mut self, info: &UpdateInfo, queue: &wgpu::Queue) {
        self.prm[2] += 1; //Increase time by one
        
        for keypress in info.key_presses.iter() {
            if *keypress == KeyCode::KeyU {

                //Do we nned to update bindgroup also?
                //self.ecosystem.get_interaction_ref()[0].update_prm_bindgroup(new_prm, &gpu.queue);
            }
        }
        queue.write_buffer(&self.prm_buffer, 0, bytemuck::cast_slice(&self.prm));
    }

    fn get_interaction_data(&self) -> &AutomataInteractionShared {
        return &self.data;
    }
}

impl RainInteraction {
    pub fn new(width: u32, height: u32, state_indicies: Vec<usize>,
               aut_bindgroup_layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("spectral_rain.wgsl"));

        let prm = vec![width, height, 0];

        let kernel = vec![0.0, 0.1, 0.8, 0.1, 0.0,
                          0.0, 0.0, 0.0, 0.0, 0.0,
                          0.0, 0.0, 0.0, 0.0, 0.0,
                          0.0, 0.0, 0.0, 0.0, 0.0,
                          0.0, 0.0, 0.0, 0.0, 0.0];

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("parameters"),
                contents: bytemuck::cast_slice(&prm),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

        let kernel_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("kernel"),
                contents: bytemuck::cast_slice(&kernel),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            });

        let wg_size = 16;
        let disp_x = width.div_ceil(wg_size) as u32;
        let disp_y = height.div_ceil(wg_size) as u32;


        let data = AutomataInteractionShared::
                   create_convolution_rule(state_indicies, 1, shader_module, aut_bindgroup_layout,
                   "rain rule", &prm_buffer, &kernel_buffer, disp_x, disp_y, device);

        return RainInteraction { 
            data, 
            prm,
            prm_buffer,
            kernel,
            kernel_buffer,
        }
    }

}

//A rule for a single automata (state).
/*
impl AutomataInteraction {

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
*/
