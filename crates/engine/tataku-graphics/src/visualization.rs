use crate::prelude::*;
pub type FFTHook = RwLock<FFTData>;

#[derive(Default, Clone)]
pub struct FFTData {
    pub data: Vec<FFTEntry>,
    pub amplitude_multiplier: f32,
}

pub struct VisualizationData {
    pub config: VisualizationConfig,
    hook: Arc<FFTHook>,

    pub data: Vec<FFTEntry>,

    pub timer: TatakuInstant,
}
impl VisualizationData {
    pub fn new(config: VisualizationConfig) -> Self {
        Self {
            config,
            hook: Arc::default(),
            data: Vec::new(),
            timer: TatakuInstant::now()
        }
    }
    pub fn reset(&mut self) {
        self.data.clear();
        self.hook.write().data.clear();
    }

    pub fn get_hook(&self) -> Weak<FFTHook> {
        Arc::downgrade(&self.hook)
    }

    pub fn update(&mut self) {
        let data = self.hook.read();

        let elapsed = self.timer.elapsed_and_reset() / 1000.0;
        let mut audio_data = data.data.clone();

        if self.config.should_lerp && !audio_data.is_empty() {
            let factor = self.config.lerp_factor * elapsed;
            self.data.resize(audio_data.len(), FFTEntry::default());

            for (i, entry) in audio_data.iter_mut().enumerate() {
                entry.set_amplitude(f32::lerp(
                    self.data[i].amplitude(), 
                    entry.amplitude() * data.amplitude_multiplier, 
                    factor
                ));
            }

            self.data = audio_data;
        } else {
            self.data = audio_data
                .iter()
                .copied()
                .map(|mut i| { i.set_amplitude(i.amplitude() * data.amplitude_multiplier); i })
                .collect();
        }

    }
}

#[derive(Default2)]
pub struct VisualizationConfig {
    #[default(true)] pub should_lerp: bool,
    #[default(20.0)] pub lerp_factor: f32,   
}
