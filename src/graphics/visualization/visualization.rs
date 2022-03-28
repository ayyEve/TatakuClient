use crate::prelude::*;

pub use f32 as Amplitude;
pub use f32 as Frequency;

#[cfg(feature="bass_audio")]
pub type FFTEntry = f32; //(Frequency, Amplitude);
#[cfg(feature="neb_audio")]
pub type FFTEntry = (Frequency, Amplitude);

#[cfg(feature="bass_audio")]
const BASS_MULT:f32 = 1_000.0;


pub trait Visualization {
    fn should_lerp(&self) -> bool {true}
    fn lerp_factor(&self) -> f32 {20.0}
    fn draw(&mut self, args:piston::RenderArgs, pos_offset:Vector2, depth:f64, list:&mut Vec<Box<dyn Renderable>>);
    fn update(&mut self) {}
    fn reset(&mut self) {}

    fn data(&mut self) -> &mut Vec<FFTEntry>;
    fn timer(&mut self) -> &mut Instant;
    fn update_data(&mut self) {
        let mut audio_data;

        #[cfg(feature="bass_audio")] {
            let data = match Audio::get_song() {
                Some(stream) => stream.get_data(bass_rs::prelude::DataType::FFT2048, 1024i32),
                None => return
            }.unwrap_or(vec![0.0]);
            audio_data = data[0..data.len() / 2].to_vec();
        }

        #[cfg(feature="neb_audio")] {
            // get the audio being fed to the sound card
            let data = crate::game::audio::CURRENT_DATA.clone();
            let mut data = data.lock().clone();
            // trace!("{}", audio_data.len());

            let len = data.len();
            let size;

            if !cfg!(target_os = "linux") {
                let scale = (1024.0 / len as f32) * 8.0;
                for sample in data.iter_mut() {
                    *sample *= scale;
                }
                data.resize(1024, 0.0);
                size = FFT::F1024;
            } else {
                data.resize(8192, 0.0);
                size = FFT::F8192;
            }

            let mut data = fft(
                &mut data, 
                size
            );

            data.retain(|(freq, _amp)| *freq < 7_000.0);
            audio_data = data;
        }

        let time = self.timer();
        let elapsed = time.elapsed().as_secs_f32();
        *time = Instant::now();
        drop(time);



        let should_lerp = self.should_lerp();
        let factor = self.lerp_factor() * elapsed;
        let data = self.data();
        if should_lerp && data.len() > 0 {
            #[cfg(feature="bass_audio")] {
                data.resize(audio_data.len(), 0.0);
                for i in 0..audio_data.len() {
                    audio_data[i] = lerp(data[i], audio_data[i] * BASS_MULT, factor);
                }
            }
            #[cfg(feature="neb_audio")] {
                data.resize(audio_data.len(), (0.0, 0.0));
                for i in 0..audio_data.len() {
                    audio_data[i].1 = lerp(data[i].1, audio_data[i].1, factor);
                }
            }
        }

        *self.data() = audio_data;
    }
}

fn lerp(current:f32, target:f32, factor:f32) -> f32 {
    current + (target - current) * factor
}
