use std::time::{Duration, Instant};
use std::thread::sleep;
use std::fmt::Debug;

use super::DEFAULT_CLOCK_SPEED;

// TODO make this into a type that limits its range, maybe ranged_integer crate once it's mature
pub type TickCount = u8;


pub trait Clock: Debug {
    fn tick(&mut self, tick_count: TickCount);
}

// A clock that has a clock speed, and ticks accordingly
#[derive(Debug)]
pub struct NormalClock {
    #[allow(dead_code)]
    speed: u32,
    interval: Duration,
    last_tick: Instant,
}

impl Default for NormalClock {
    fn default() -> Self {
        Self::new(DEFAULT_CLOCK_SPEED)
    }
}

impl NormalClock {
    /* Clock speed in Hz, so 1 MHz is 1_000_000 */
    pub fn new(clock_speed: u32) -> Self {
        Self {
            speed: clock_speed,
            interval: Duration::from_nanos(1_000_000_000/clock_speed as u64),
            last_tick: Instant::now(),
        }
    }
}

impl Clock for NormalClock {
    /*
     * wait for the given number of ticks to have expired
     * This requires that the amount of time elapsed outside of this
     * function isn't consistently larger than the clock_speed's interval
     *
     * FIXME: This will go weird if tick() isn't called on average at least
     * once per interval. De ide what to do.
     */
    fn tick(&mut self, tick_count: TickCount) {
        let now = Instant::now();
        let next_tick = self.last_tick + self.interval.saturating_mul(tick_count as u32);
        if now < next_tick {
            sleep(next_tick - now);
        }
        self.last_tick = next_tick;
    }
}

// A clock that does nothing at all to slow things down
#[derive(Debug)]
pub struct SpeedyClock {}

impl Clock for SpeedyClock {
    fn tick(&mut self, _: TickCount) {}
}