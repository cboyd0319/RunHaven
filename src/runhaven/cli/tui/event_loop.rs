use std::time::{Duration, Instant};

pub(crate) const DEFAULT_TICK_RATE: Duration = Duration::from_millis(250);
const MIN_TICK_RATE: Duration = Duration::from_millis(16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Tick {
    pub elapsed: Duration,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Ticker {
    interval: Duration,
    last_tick: Instant,
    next_tick: Instant,
}

impl Ticker {
    pub(crate) fn new(now: Instant, interval: Duration) -> Self {
        let interval = interval.max(MIN_TICK_RATE);
        Self {
            interval,
            last_tick: now,
            next_tick: now + interval,
        }
    }

    pub(crate) fn timeout(self, now: Instant) -> Duration {
        self.next_tick.saturating_duration_since(now)
    }

    pub(crate) fn tick(&mut self, now: Instant) -> Option<Tick> {
        if now < self.next_tick {
            return None;
        }

        let tick = Tick {
            elapsed: now.saturating_duration_since(self.last_tick),
        };
        self.last_tick = now;
        while self.next_tick <= now {
            self.next_tick += self.interval;
        }
        Some(tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_counts_down_to_next_tick() {
        let start = Instant::now();
        let ticker = Ticker::new(start, Duration::from_millis(250));

        assert_eq!(ticker.timeout(start), Duration::from_millis(250));
        assert_eq!(
            ticker.timeout(start + Duration::from_millis(100)),
            Duration::from_millis(150)
        );
        assert_eq!(
            ticker.timeout(start + Duration::from_millis(300)),
            Duration::ZERO
        );
    }

    #[test]
    fn tick_reports_elapsed_and_advances_past_missed_frames() {
        let start = Instant::now();
        let mut ticker = Ticker::new(start, Duration::from_millis(100));

        assert_eq!(ticker.tick(start + Duration::from_millis(99)), None);
        assert_eq!(
            ticker.tick(start + Duration::from_millis(350)),
            Some(Tick {
                elapsed: Duration::from_millis(350)
            })
        );
        assert_eq!(
            ticker.timeout(start + Duration::from_millis(350)),
            Duration::from_millis(50)
        );
    }

    #[test]
    fn zero_tick_rate_is_clamped() {
        let start = Instant::now();
        let mut ticker = Ticker::new(start, Duration::ZERO);

        assert_eq!(ticker.timeout(start), MIN_TICK_RATE);
        assert_eq!(ticker.tick(start + Duration::from_millis(1)), None);
        assert!(ticker.tick(start + MIN_TICK_RATE).is_some());
    }
}
