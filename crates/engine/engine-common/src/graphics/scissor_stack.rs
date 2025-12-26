pub type Scissor = Option<[f32; 4]>;

const BASE: [f32; 4] = [f32::MIN, f32::MIN, f32::MAX, f32::MAX];

#[derive(Default)]
pub struct ScissorStack {
    scissors: Vec<[f32; 4]>,
    current_scissor: Scissor,
}
impl ScissorStack {
    pub fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push(scissor);
        self.recalc_current_scissor(true);
    }
    pub fn pop_scissor(&mut self) {
        self.scissors.pop();
        self.recalc_current_scissor(false);
    }

    pub fn current_scissor(&self) -> Scissor {
        self.current_scissor
    }


    // TODO: rename? this gives the impression that there will always be an intersection, but if there isnt the result will be [x,y,0,0]
    fn intersection(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
        [
            a[0].max(b[0]),
            a[1].max(b[1]),
            a[2].min(b[2]),
            a[3].min(b[3]),
        ]
    }

    fn recalc_current_scissor(&mut self, full_recalc: bool) {
        if self.scissors.is_empty() {
            self.current_scissor = None;
            return;
        }

        // only compare the last scissor added
        if !full_recalc {
            let current = self.current_scissor.unwrap_or(BASE);
            let last = self.scissors.last().unwrap(); // unwrap is ok because if its empty we would return earlier
            self.current_scissor = Some(Self::intersection(current, *last));
            return
        }
    
        // full recalc goes through all scissors and intersects them all
        let s = self.scissors.iter()
            .copied()
            .fold(BASE, Self::intersection);
        self.current_scissor = Some(s);
    }
}
