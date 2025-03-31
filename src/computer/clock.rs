use std::time::{Duration, Instant};
use std::thread;
use std::fmt::{Debug, Display};

use super::DEFAULT_CLOCK_SPEED;

// TODO make this into a type that limits its range, maybe ranged_integer crate once it's mature
pub type TickCount = u16;

#[derive(Debug, Clone)]
pub enum ClockMode {
    Normal,
    Speedy,
}

#[derive(Debug, Clone)]
pub struct Clock {
    mode: ClockMode,
    speed: u32,
    interval: Duration,
    reference: Instant,
    ticks_since_reference: u32,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            mode: ClockMode::Normal,
            speed: DEFAULT_CLOCK_SPEED,
            interval: Duration::from_nanos(1_000_000_000/DEFAULT_CLOCK_SPEED as u64),
            reference: Instant::now(),
            ticks_since_reference: 0,
        }
    }
}

impl Clock {
    pub fn new(mode: ClockMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    pub fn with_clock_speed(mut self, speed: u32) -> Self {
        self.speed = speed;
        self.interval = Duration::from_nanos(1_000_000_000/speed as u64);
        self
    }
}

impl Clock {

    fn wait_for_normal_tick(&mut self, tick_count: TickCount) {

        let now = Instant::now();
        self.ticks_since_reference += tick_count as u32;
        let next_tick = self.reference + self.interval.saturating_mul(self.ticks_since_reference);

        // If we're within a tick, sleep
        if now < next_tick {
            thread::sleep(next_tick - now);
        }

        // If we missed the tick (delay, debug, stopped computer)
        // or just before we can possibly overflow ticks_since_reference, reset
        if now >= next_tick ||
                (self.ticks_since_reference >= u32::MAX - TickCount::MAX as u32 - 1) {
            self.reference = Instant::now();
            self.ticks_since_reference = 0;
        }
    }

    pub fn wait_for_tick(&mut self, tick_count: TickCount) {
        match self.mode {
            ClockMode::Normal => self.wait_for_normal_tick(tick_count),
            ClockMode::Speedy => {} // Do nothing at all
        }
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clock speed: {} Hz, interval {:?} us, mode {:?}", self.speed, self.interval.as_micros(), self.mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use log::debug;

    // TODO These tests are somewhat flaky. Not sure what to do about it.
    #[test_case(10, 2, 200_000, 210_000; "normal clock @ 10")]
    #[test_case(1_000, 5, 4_500, 6_500; "normal clock @ 1_000")]
    #[test_case(1_000_000, 5_000, 4_500, 6_500; "normal clock @ 1_000_000")]
    #[test_case(0, 1000, 0, 1; "speedy clock 1")]
    #[test_case(0, TickCount::MAX, 0, 1; "speedy clock 2")]
    fn test_clock_speed(clock_speed: u32, ticks: TickCount, low_micros: u64, high_micros: u64) {
        let _ = env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .format_target(false)
            .try_init();

        let mut clock = if clock_speed > 0 {
            Clock::new(ClockMode::Normal).with_clock_speed(clock_speed)
        } else {
            Clock::new(ClockMode::Speedy)
        };

        let start = Instant::now();
        clock.wait_for_tick(ticks);

        let duration = start.elapsed();
        let reference_low = Duration::from_micros(low_micros);
        let reference_high = Duration::from_micros(high_micros);

        debug!("Duration: {:?}, Reference: {:?}-{:?}", duration, reference_low, reference_high);
        if duration > reference_high {
            panic!("Normal Clock@{clock_speed} ticks too fast")
        } else if duration < reference_low {
            panic!("Normal Clock@{clock_speed} ticks too slow")
        }
    }
}