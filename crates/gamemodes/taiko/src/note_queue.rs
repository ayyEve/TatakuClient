use crate::prelude::*;

#[derive(Default)]
pub struct NoteQueue {
    pub notes: Vec<HitObject>,
    pub index: usize,
}
impl NoteQueue {
    pub fn done(&self) -> bool { self.index >= self.notes.len() }
    pub fn next(&mut self) { self.index += 1; }

    #[inline]
    pub fn current_note(&mut self) -> Option<&mut HitObject> {
        self.notes.get_mut(self.index)
    }
    #[inline]
    #[allow(clippy::borrowed_box)]
    pub fn previous_note(&self) -> Option<&HitObject> {
        self.notes.get(self.index - 1)
    }
}

impl Deref for NoteQueue {
    type Target = Vec<HitObject>;

    fn deref(&self) -> &Self::Target {
        &self.notes
    }
}
impl DerefMut for NoteQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.notes
    }
}
