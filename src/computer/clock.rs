use std::time::{Duration, Instant};
use std::thread::sleep;

#[derive(Debug)]
pub struct Clock {
    speed: u32,
    interval: Duration,
    next_tick: Instant,
}

impl Clock {
    /* Clock speed in Hz, so 1 MHz is 1_000_000 */
    pub fn new(clock_speed: u32) -> Self {
        let now = Instant::now();
        let interval = Duration::from_nanos(1_000_000_000/clock_speed as u64);
        Self {
            speed: clock_speed,
            interval: interval,
            next_tick: now + interval,
        }
    }

    /* 
     * wait for the next tick
     * This requires that the amount of time elapsed outside of this
     * function isn't consistently larger than the clock_speed's interval
     */
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now < self.next_tick {
            sleep(self.next_tick - now);
        }
        self.next_tick += self.interval;
    }
}