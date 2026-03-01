use wgpu::{util::{BufferInitDescriptor, DeviceExt}};
use crate::automata::automata_state::AutomataState;

pub struct AutomataRule {
    pipeline: wgpu::ComputePipeline,
    prm_bindgroup: wgpu::BindGroup,
    prm_bindgroup_layout: wgpu::BindGroupLayout,
    num_prm: usize,
}


impl AutomataRule {
    pub fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        return &self.pipeline;
    }
    pub fn get_prm_bindgroup(&self) -> &wgpu::BindGroup {
        return &self.prm_bindgroup;
    }
    pub fn get_prm_bindgroup_layout(&self) -> &wgpu::BindGroupLayout {
        return &self.prm_bindgroup_layout;
    }

    pub fn update_prm_bindgroup(&mut self, prm: Vec<u32>, device: &wgpu::Device) {
        if prm.len() != self.num_prm {
            print!("Incorrect number of parameters, cannot update!");
            return;
        }


        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("parameters"),
                contents: bytemuck::cast_slice(&prm),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE, //maybe don't need
                                                                                   //both these usages?
            });

        let prm_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.get_prm_bindgroup_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: prm_buffer.as_entire_binding(),
                },
            ],
        });

        self.prm_bindgroup = prm_bindgroup;
    }

    fn create_rule(shader_module: wgpu::ShaderModule, states: &Vec<AutomataState>, 
           rule_name: &'static str, prm: Vec<u32>, device: &wgpu::Device) -> Self {

        let prm_bindgroup_layout = 
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
                    ],
                    label: Some("rule_bind_group_layout"),
                });

        let prm_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("parameters"),
                contents: bytemuck::cast_slice(&prm),
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
            ],
        });

        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = Vec::new();
        bind_group_layouts.push(&prm_bindgroup_layout);
        bind_group_layouts.append(&mut states.iter().map(|stt| &stt.automata_bindgroup_layout).collect());

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

        return AutomataRule {
            pipeline, 
            prm_bindgroup, 
            prm_bindgroup_layout,
            num_prm: prm.len(),
        }


    }

}

//A rule for a single automata (state).
impl AutomataRule {
    pub fn conway_rule(states: &Vec<AutomataState>, state_index: usize, device: &wgpu::Device) -> AutomataRule {
        let state = &states[state_index];
        let prm = vec![state.width, state.height, 3, 12, 13]; 
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("conway_automata.wgsl"));
        
        return AutomataRule::create_rule(shader_module, states, "conway rule", prm, device);
    }
}
