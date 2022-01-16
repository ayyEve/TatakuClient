use std::time::Instant;

pub struct BenchmarkHelper {
    timer: Instant,
    name: String
}
impl BenchmarkHelper {
    pub fn new(name:&str) -> Self{
        println!("{} benchmarker created", name);
        Self {
            timer:Instant::now(),
            name: name.to_owned()
        }
    }

    /// ms since timer was set
    pub fn get_elapsed(&self) -> f64 {
        self.timer.elapsed().as_micros() as f64 / 1_000.0
    }

    pub fn log(&mut self, msg:&str, reset_timer:bool) {
        println!("[{}]: {} >> {}", self.name, self.get_elapsed(), msg);
        if reset_timer {self.timer = Instant::now()}
    }
}