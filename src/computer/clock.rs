use std::time::{Duration, Instant};
use std::thread::sleep;
use std::fmt::{Debug, Display};

use super::DEFAULT_CLOCK_SPEED;

// TODO make this into a type that limits its range, maybe ranged_integer crate once it's mature
pub type TickCount = u16;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ClockMode {
    Normal,
    Speedy,
    Debug,
}

// TODO I don't like this enum implementation. Would rather have a trait, but
// haven't yet figured out how to properly do that up in Computer and Cpu, without
// needing to genericise both of them on Clock, which is silly.
#[derive(Debug, Clone)]

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        if now < next_tick {
            sleep(next_tick - now);
        }
        // Just before we can possibly overflow ticks_since_reference, reset
        if self.ticks_since_reference >= u32::MAX - TickCount::MIN as u32 - 1 {
            self.reference = Instant::now();
            self.ticks_since_reference = 0;
        }
    }

    pub fn wait_for_tick(&mut self, tick_count: TickCount) {
        match self.mode {
            ClockMode::Normal => self.wait_for_normal_tick(tick_count),
            ClockMode::Debug => todo!(),
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
    use test_log::test;
    use log::debug;

    fn test_clock_speed(mut clock: Clock, ticks: TickCount, low_micros: u64, high_micros: u64) -> Result<(), String> {
        let start = Instant::now();

        clock.wait_for_tick(ticks);

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
        let clock = Clock::new(ClockMode::Normal).with_clock_speed(1_000);
        if let Err(e) = test_clock_speed(clock, 5, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = Clock::new(ClockMode::Normal).with_clock_speed(10_000);
        if let Err(e) = test_clock_speed(clock, 50, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = Clock::new(ClockMode::Normal).with_clock_speed(100_000);
        if let Err(e) = test_clock_speed(clock, 500, 4500, 6500) {
            panic!("{}", e);
        }
        let clock = Clock::new(ClockMode::Normal).with_clock_speed(1_000_000);
        if let Err(e) = test_clock_speed(clock, 5_000, 4500, 6500) {
            panic!("{}", e);
        }
    }

    #[test]
    fn test_speedy_clock() {
        // Test that speedy clock just runs as fast as it possibly can
        let clock = Clock::new(ClockMode::Speedy);
        if let Err(e) = test_clock_speed(clock.clone(), 1_000, 0, 1) {
            panic!("{}", e);
        }
        if let Err(e) = test_clock_speed(clock.clone(), TickCount::MAX, 0, 1) {
            panic!("{}", e);
        }
    }
}