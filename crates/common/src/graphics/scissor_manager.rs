use crate::prelude::*;

#[derive(Default)]
pub struct ScissorManager {
    scissors: Vec<[f32; 4]>,
    current_scissor: Scissor,
}
impl ScissorManager {
    pub fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push(scissor);
        self.recalc_current_scissor();
    }
    pub fn pop_scissor(&mut self) {
        self.scissors.pop();
        self.recalc_current_scissor();
    }

    pub fn current_scissor(&self) -> Scissor {
        self.current_scissor
    }

    fn recalc_current_scissor(&mut self) {
        // TODO: this could be improved by comparing the current scissor to the last one in the list.
        // then we're only comparing two scissors instead of all scissors
        // would need to account for scissors getting removed though

        if self.scissors.is_empty() {
            self.current_scissor = None;
            return;
        }

        let s = self.scissors.iter().fold([f32::MIN, f32::MIN, f32::MAX, f32::MAX], |i, n| {
            [
                i[0].max(n[0]),
                i[1].max(n[1]),
                i[2].min(n[2]),
                i[3].min(n[3]),
            ]
        });
        
        self.current_scissor = Some(s);
    }
}
