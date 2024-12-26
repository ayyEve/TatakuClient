use crate::prelude::*;
// use tataku_client_common::math::Interpolation;

pub type FFTHook = RwLock<FFTData>;

// #[async_trait]
// pub trait Visualization: Send + Sync {
//     fn should_lerp(&self) -> bool { true }
//     fn lerp_factor(&self) -> f32 { 20.0 }
//     async fn draw(&mut self, bounds: Bounds, list: &mut RenderableCollection);
//     async fn update(&mut self) {}
//     #[cfg(feature="graphics")]
//     async fn reload_skin(&mut self, _skin_manager: &mut dyn SkinProvider) {}
//     fn reset(&mut self) {}

//     fn song_changed(&mut self) {}

//     fn data(&mut self) -> &mut Vec<FFTEntry>;
//     fn timer(&mut self) -> &mut Instant;
//     async fn update_data(&mut self) {
//         // let Some(mut audio_data) = AudioManager::get_song().await.map(|f|f.get_data()) else { return };
//         let mut audio_data = vec![FFTEntry::default(); 2048];

//         let elapsed = self.timer().elapsed_and_reset() / 1000.0;

//         let mult = AudioManager::amplitude_multiplier();
//         let should_lerp = self.should_lerp();
//         let lerp_factor = self.lerp_factor();
//         let data = self.data();
//         if should_lerp && !data.is_empty() {
//             let factor = lerp_factor * elapsed;
//             data.resize(audio_data.len(), FFTEntry::default());
//             for i in 0..audio_data.len() {
//                 let v = audio_data[i].amplitude() * mult;
//                 audio_data[i].set_amplitude(lerp(data[i].amplitude(), v, factor));
//             }
//         }

//         *self.data() = audio_data;
//     }
// }

// fn lerp(current:f32, target:f32, factor:f32) -> f32 {
//     current + (target - current) * factor
// }


#[derive(Default, Clone)]
pub struct FFTData {
    pub data: Vec<FFTEntry>,
    pub amplitude_multiplier: f32,
}

pub struct VisualizationData {
    pub config: VisualizationConfig,
    hook: Arc<FFTHook>,

    pub data: Vec<FFTEntry>,

    pub timer: Instant,
}
impl VisualizationData {
    pub fn new(config: VisualizationConfig) -> Self {
        Self {
            config,
            hook: Default::default(),
            data: Vec::new(),
            timer: Instant::now()
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
        // let Some(data) = self.hook.try_read() else { return };

        // // let Some(mut audio_data) = AudioManager::get_song().await.map(|f|f.get_data()) else { return };
        // let mut audio_data = vec![FFTEntry::default(); 2048];


        // let mult = AudioManager::amplitude_multiplier();
        // let should_lerp = self.should_lerp();
        // let lerp_factor = self.lerp_factor();
        // if should_lerp && !data.is_empty() {
        //     let factor = lerp_factor * elapsed;
        //     data.resize(audio_data.len(), FFTEntry::default());
        //     for i in 0..audio_data.len() {
        //         let v = audio_data[i].amplitude() * mult;
        //         audio_data[i].set_amplitude(lerp(data[i].amplitude(), v, factor));
        //     }
        // }

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


pub struct VisualizationConfig {
    pub should_lerp: bool,
    pub lerp_factor: f32,
    
}
impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            should_lerp: true,
            lerp_factor: 20.0
        }
    }
}
