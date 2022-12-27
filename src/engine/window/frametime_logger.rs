#[derive(Default)]
pub struct FrameTimeLogger {
    frametimes: Vec<f32>
}
impl FrameTimeLogger {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, _time: f32) {
        #[cfg(feature="log_frametimes")]
        self.frametimes.push(_time);
    }

    pub fn write(&self) {
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