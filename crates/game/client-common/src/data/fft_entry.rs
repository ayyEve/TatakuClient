
#[derive(Copy, Clone, Debug)]
pub enum FFTEntry {
    AmplitudeOnly(f32),
    AmplitudeAndFrequency(f32, f32)
}
impl FFTEntry {
    pub fn amplitude(self) -> f32 {
        match self {
            FFTEntry::AmplitudeOnly(a) => a,
            FFTEntry::AmplitudeAndFrequency(_, a) => a,
        }
    }
    pub fn set_amplitude(&mut self, na: f32) {
        match self {
            FFTEntry::AmplitudeOnly(a) => *a = na,
            FFTEntry::AmplitudeAndFrequency(_, a) => *a = na,
        }
    }
}
impl From<f32> for FFTEntry {
    fn from(a: f32) -> Self {
        Self::AmplitudeOnly(a)
    }
}
impl From<(f32, f32)> for FFTEntry {
    fn from((f, a): (f32, f32)) -> Self {
        Self::AmplitudeAndFrequency(f, a)
    }
}
impl Default for FFTEntry {
    fn default() -> Self {
        Self::AmplitudeAndFrequency(0.0, 0.0)
    }
}
