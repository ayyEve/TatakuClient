use crate::prelude::*;

pub struct ReserveData<'a> {
    pub vtx: &'a mut [Vertex],
    pub idx: &'a mut [u32],
    pub idx_offset: u64,
    // pub scissor_index: u32,
}
impl<'a> ReserveData<'a> {
    pub fn copy_in(&mut self, vtx: &[Vertex], idx: &[u32]) {
        // std::mem::swap(vtx, self.vtx);
        // std::mem::swap(idx, self.idx);

        for i in 0..vtx.len() { self.vtx[i] = vtx[i] }
        for i in 0..idx.len() { self.idx[i] = idx[i] }
    }
}
