use std::time::Instant;
pub struct Benchmark {
    last_time : Option<Instant>
}

impl Benchmark{
    pub fn start(&mut self){
        self.last_time = Some(Instant::now())
    }
    pub fn stop(&self){
        println!(

        )
    }
}
pub fn new() -> Benchmark{
    Benchmark { last_time: None }
}
