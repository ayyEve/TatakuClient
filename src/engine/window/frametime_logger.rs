
pub struct FrameTimeLogger {
    #[cfg(feature="log_frametimes")]
    frametimes: Vec<f32>
}
impl FrameTimeLogger {
    pub fn new() -> Self { Self {
        #[cfg(feature="log_frametimes")]
        frametimes: Vec::new(),
    } }

    pub fn add(&mut self, _time: f32) {
        #[cfg(feature="log_frametimes")]
        self.frametimes.push(_time);
    }

    pub fn write(&self) {
        #[cfg(feature="log_frametimes")] {
            std::fs::write(
                "./frametimes.csv", 
                format!(
                    "count, time\n{}", 
                    self.frametimes
                    .iter()
                    .enumerate()
                    .map(|(i, n)|format!("{i},{n}"))
                    .collect::<Vec<String>>()
                    .join("\n")
                )
            ).unwrap();
            println!("wrote frametimes");
        }
    }
}