//! The output a hidden launcher shows on when asked: the focused output, else the first one the
//! compositor announced.

use std::sync::{Arc, Mutex};

use crownui::surfaces::{Output, OutputId};

use crate::util::lock::lock;

#[derive(Clone, Default)]
pub struct HostOutput(Arc<Mutex<Option<OutputId>>>);

impl HostOutput {
    pub fn follow(&self, outputs: &[Output]) {
        let host = focused_else_first(outputs, |output| {
            output.crown.as_ref().is_some_and(|crown| crown.focused)
        });
        *lock(&self.0) = host.map(|output| output.id);
    }

    pub fn get(&self) -> Option<OutputId> {
        *lock(&self.0)
    }
}

fn focused_else_first<Output>(
    outputs: &[Output],
    is_focused: impl Fn(&Output) -> bool,
) -> Option<&Output> {
    outputs
        .iter()
        .find(|output| is_focused(output))
        .or(outputs.first())
}

#[cfg(test)]
mod tests {
    use super::focused_else_first;

    #[test]
    fn the_focused_output_hosts_the_launcher() {
        assert_eq!(focused_else_first(&[1, 2, 3], |&id| id == 2), Some(&2));
    }

    #[test]
    fn without_a_focused_output_the_first_announced_hosts_it() {
        assert_eq!(focused_else_first(&[3, 1], |_| false), Some(&3));
        assert_eq!(focused_else_first::<u32>(&[], |_| true), None);
    }
}
