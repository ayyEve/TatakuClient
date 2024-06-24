use crate::prelude::*;

#[async_trait]
pub trait Visualization: Send + Sync {
    fn should_lerp(&self) -> bool { true }
    fn lerp_factor(&self) -> f32 { 20.0 }
    async fn draw(&mut self, bounds:Bounds, list: &mut RenderableCollection);
    async fn update(&mut self) {}
    async fn reload_skin(&mut self, _skin_manager: &mut SkinManager) {}
    fn reset(&mut self) {}

    fn song_changed(&mut self) {}

    fn data(&mut self) -> &mut Vec<FFTData>;
    fn timer(&mut self) -> &mut Instant;
    async fn update_data(&mut self) {
        // let Some(mut audio_data) = AudioManager::get_song().await.map(|f|f.get_data()) else { return };
        let mut audio_data = vec![FFTData::default(); 2048];

        let elapsed = self.timer().elapsed_and_reset() / 1000.0;

        let mult = AudioManager::amplitude_multiplier();
        let should_lerp = self.should_lerp();
        let lerp_factor = self.lerp_factor();
        let data = self.data();
        if should_lerp && data.len() > 0 {
            let factor = lerp_factor * elapsed;
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
