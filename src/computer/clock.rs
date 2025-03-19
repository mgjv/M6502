use std::time::{Duration, Instant};
use std::thread::sleep;
use std::fmt::{Debug, Display};

use log::debug;

use super::DEFAULT_CLOCK_SPEED;

// TODO make this into a type that limits its range, maybe ranged_integer crate once it's mature
pub type TickCount = u16;

pub trait ClockTrait: Debug + Default {
    fn tick(&mut self, tick_count: TickCount);
}

// TODO I don't like this enum implementation. Would rather have a trait, but
// haven't yet figured out how to properly do that up in Computer and Cpu, without
// needing to genericise both of them on Clock, which is silly.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Clock {
    Normal(NormalClock),
    Speedy(SpeedyClock),
}

impl Default for Clock {
    fn default() -> Self {
        Self::Normal(NormalClock::default())
    }
}

impl ClockTrait for Clock {
    fn tick(&mut self, tick_count: TickCount) {
        match self {
            Self::Normal(c) => c.tick(tick_count),
            Self::Speedy(c) => c.tick(tick_count),
        }
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(c) => Display::fmt(c, f),
            Self::Speedy(_) => write!(f, "Speedy")
        }
    }
}

/*
 * A normal clock that runs at a fixed speed
 *
 * This clock needs to be able to run, so don't hold it up.
 * It might go weird in a debugging session.
 */
// A clock that has a clock speed, and ticks accordingly
#[derive(Debug, Clone)]
pub struct NormalClock {
    #[allow(dead_code)]
    speed: u32,
    interval: Duration,
    reference: Instant,
    ticks_since_reference: u32,
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
            reference: Instant::now(),
            ticks_since_reference: 0,
        }
    }
}

impl ClockTrait for NormalClock {
    /*
     * wait for the given number of ticks to have expired
     * This requires that the amount of time elapsed outside of this
     * function isn't consistently larger than the clock_speed's interval
     */
    fn tick(&mut self, tick_count: TickCount) {
        let now = Instant::now();
        self.ticks_since_reference += tick_count as u32;
        let next_tick = self.reference + self.interval.saturating_mul(self.ticks_since_reference);
        if now < next_tick {
            sleep(next_tick - now);
        }
        // Just before we can possibly overflow ticks_since_reference, reset
        if self.ticks_since_reference >= u32::MAX - TickCount::MIN as u32 - 1 {
            self.reference = Instant::now();
            self.ticks_since_reference = 0;
        }
    }
}

impl Display for NormalClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clock speed: {} Hz, interval {:?} us", self.speed, self.interval.as_micros())
    }
}

// A clock that does nothing at all to slow things down
#[derive(Debug, Default, Clone)]
pub struct SpeedyClock {}

impl ClockTrait for SpeedyClock {
    fn tick(&mut self, _: TickCount) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;
    use log::debug;

    fn test_clock_speed(mut clock: Clock, ticks: TickCount, low_micros: u64, high_micros: u64) -> Result<(), String> {
        let start = Instant::now();

        clock.tick(ticks);

        let duration = start.elapsed();
        let reference_low = Duration::from_micros(low_micros);
        let reference_high = Duration::from_micros(high_micros);

        debug!("Duration: {:?}, Reference: {:?}-{:?}", duration, reference_low, reference_high);
        if duration > reference_high {
            return Err("Normal Clock ticks too fast".to_string())
        } else if duration < reference_low {
            return Err("Normal Clock ticks too slow".to_string())
        }
        Ok(())
    }

    #[test]
    fn test_normal_clock() {
        // Test that normal clock ticks at the right speed
        // TODO This is a flaky test. It depends on what else your computer is doing
        // at the moment. Not sure how to fix.
        let clock = NormalClock::new(1_000);
        if let Err(e) = test_clock_speed(Clock::Normal(clock), 5, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = NormalClock::new(10_000);
        if let Err(e) = test_clock_speed(Clock::Normal(clock), 50, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = NormalClock::new(100_000);
        if let Err(e) = test_clock_speed(Clock::Normal(clock), 500, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = NormalClock::new(1_000_000);
        if let Err(e) = test_clock_speed(Clock::Normal(clock), 5_000, 4500, 6500) {
            panic!("{}", e);
        }
    }

    #[test]
    fn test_speedy_clock() {
        // Test that speedy clock just runs as fast as it possibly can
        let clock = SpeedyClock::default();
        if let Err(e) = test_clock_speed(Clock::Speedy(clock.clone()), 1_000, 0, 1) {
            panic!("{}", e);
        }
        if let Err(e) = test_clock_speed(Clock::Speedy(clock.clone()), TickCount::MAX, 0, 1) {
            panic!("{}", e);
        }
    }
}