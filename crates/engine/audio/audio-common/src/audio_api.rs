use crate::prelude::*;
use std::path::Path;

pub trait AudioApi: Send + Sync {
    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>>;
    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>>;

    fn load_sample_path(&self, path: &str) -> TatakuResult<Arc<dyn AudioInstance>> {
        let data = Io::read_file(path)?;
        self.load_sample_data(data)
    }
    fn load_stream_path(&self, path: &Path) -> TatakuResult<Arc<dyn AudioInstance>> {
        let data = Io::read_file(path)?;
        self.load_stream_data(data)
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance>;
    fn amplitude_multiplier(&self) -> f32 { 1.0 }
}
