use std::time::{Duration, Instant};
use std::thread::sleep;

// TODO make this into a type that limits its range
pub type TickCount = u8;

#[derive(Debug)]
pub struct Clock {
    speed: u32,
    interval: Duration,
    last_tick: Instant,
}

impl Clock {
    /* Clock speed in Hz, so 1 MHz is 1_000_000 */
    pub fn new(clock_speed: u32) -> Self {
        Self {
            speed: clock_speed,
            interval: Duration::from_nanos(1_000_000_000/clock_speed as u64),
            last_tick: Instant::now(),
        }
    }

    /* 
     * wait for the given number of ticks to have expired
     * This requires that the amount of time elapsed outside of this
     * function isn't consistently larger than the clock_speed's interval
     */
    pub fn tick(&mut self, tick_count: TickCount) {
        let now = Instant::now();
        let next_tick = self.last_tick + self.interval.saturating_mul(tick_count as u32);
        if now < next_tick {
            sleep(next_tick - now);
        }
        self.last_tick = next_tick;
    }
}