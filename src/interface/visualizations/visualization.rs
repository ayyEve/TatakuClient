use crate::prelude::*;

#[async_trait]
pub trait Visualization: Send + Sync {
    fn should_lerp(&self) -> bool { true }
    fn lerp_factor(&self) -> f32 { 20.0 }
    async fn draw(&mut self, pos_offset:Vector2, list: &mut RenderableCollection);
    async fn update(&mut self) {}
    fn reset(&mut self) {}

    fn data(&mut self) -> &mut Vec<FFTData>;
    fn timer(&mut self) -> &mut Instant;
    async fn update_data(&mut self) {
        let mut audio_data = match AudioManager::get_song().await {
            Some(stream) => stream.get_data(),
            None => return
        };

        let elapsed = self.timer().elapsed_and_reset() / 1000.0;

        let mult = AudioManager::amplitude_multiplier();
        let should_lerp = self.should_lerp();
        let factor = self.lerp_factor() * elapsed;
        let data = self.data();
        if should_lerp && data.len() > 0 {
            data.resize(audio_data.len(), FFTData::default());
            for i in 0..audio_data.len() {
                let v = audio_data[i].amplitude() * mult;
                audio_data[i].set_amplitude(lerp(data[i].amplitude(), v, factor));
            }
        }

        *self.data() = audio_data;
    }
}

fn lerp(current:f32, target:f32, factor:f32) -> f32 {
    current + (target - current) * factor
}
