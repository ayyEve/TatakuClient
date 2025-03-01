
// TODO: do we want to add kiai here?
#[derive(Clone, Debug)]
pub enum IngameEvent {
    Break { start: f32, end: f32 }
}
