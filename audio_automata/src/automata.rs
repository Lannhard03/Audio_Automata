pub mod automata_rule;
pub mod automata_state;
pub mod automata_renderer;

use crate::automata::automata_state::AutomataState;
use crate::automata::automata_rule::AutomataRule;



pub struct Automata {
    pub states: Vec<AutomataState>,
    pub rules: Vec<AutomataRule>,
}

impl Automata {
    pub fn update(&self, device: &wgpu::Device, queue: &wgpu::Queue, even_frame: bool) {
        for state in &self.states {
            let wg_size = state.work_group_size;
            let num_dispatches_x = state.width.div_ceil(wg_size) as u32;
            let num_dispatches_y = state.height.div_ceil(wg_size) as u32;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&Default::default());

                for rule in self.rules.iter() {
                    compute_pass.set_pipeline(rule.get_pipeline());
                    compute_pass.set_bind_group(0, rule.get_prm_bindgroup(), &[]);
                    if even_frame {
                        compute_pass.set_bind_group(1, &state.automata_bindgroup_even, &[]);
                    } else {
                        compute_pass.set_bind_group(1, &state.automata_bindgroup_odd, &[]);
                    }
                    compute_pass.dispatch_workgroups(num_dispatches_x, num_dispatches_y, 1);
                }
            }
            queue.submit([encoder.finish()]);
        }
    }
}
