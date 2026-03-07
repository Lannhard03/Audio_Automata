pub mod automata_rule;
pub mod automata_state;
pub mod automata_renderer;

use crate::gpu_state::GPUState;
use crate::automata::automata_state::AutomataState;
use crate::automata::automata_rule::AutomataInteraction;


pub struct Ecosystem {
    states: Vec<AutomataState>,
    rules: Vec<AutomataInteraction>,
}


//Automata update cycle:
//Automata owns a vector of automata states and rules, 
//aswell as a list of which automata interact for each rule
//When updating we go through the list and give the rule the corresponding
//states and run it.
//
//After these updates, the automata are rendered in some similair way.

impl Ecosystem {
    pub fn get_state_ref(&self) -> &Vec<AutomataState> {
        return &self.states;
    }
    pub fn get_rule_ref(&mut self) -> &mut Vec<AutomataInteraction> {
        return &mut self.rules;
    }


    pub fn update(&self, device: &wgpu::Device, queue: &wgpu::Queue, even_frame: bool) {
        for state in &self.states {
            let wg_size = state.work_group_size;
            let num_disp_x = state.width.div_ceil(wg_size) as u32;
            let num_disp_y = state.height.div_ceil(wg_size) as u32;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&Default::default());

                for rule in self.rules.iter() {
                    rule.apply_rule(even_frame, &self.states, &mut compute_pass, num_disp_x, num_disp_y);

                }
            }
            queue.submit([encoder.finish()]);
        }

    }

    fn std_bindgroup_layout(gpu: &GPUState) -> wgpu::BindGroupLayout {
        let compute_bind_group_layout = 
            gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ],
                        label: Some("compute_bind_group_layout"),
                    });
        return compute_bind_group_layout;
    }

    pub fn new_conway_automata(width: u32, height: u32, gpu: &GPUState) -> Self {
        let bind_group_layout = Self::std_bindgroup_layout(gpu);
        let state = AutomataState::new(&gpu.device, width, height);
        let states = vec![state];
        let mut rules: Vec<AutomataInteraction> = vec![];
        let conway_rule = AutomataInteraction::conway_rule(width, height,
                                               bind_group_layout, &gpu.device);

        rules.push(conway_rule);

        Ecosystem {
            states,
            rules,
        }
    }

    pub fn new_spectral_rain_aut(width: u32, height: u32, gpu: &GPUState) -> Self {
        let bind_group_layout = Self::std_bindgroup_layout(gpu);
        let state = AutomataState::new(&gpu.device, width, height);
        let states = vec![state];
        let mut rules: Vec<AutomataInteraction> = vec![];
        let rule = AutomataInteraction::spectral_rain_rule(width, height,
                                               bind_group_layout, &gpu.device);

        rules.push(rule);

        Ecosystem {
            states,
            rules,
        }
    }
}
