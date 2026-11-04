//! Rate limiting for the streams sent to the UI.

use std::time::{Duration, Instant};

/// Allows at most `rate` events per second. It drops events instead of queueing them, so the UI
/// always gets recent data and a slow consumer cannot build up latency.
#[derive(Debug, Clone)]
pub struct Throttle {
    interval: Duration,
    next: Option<Instant>,
}

impl Throttle {
    pub fn new(rate_hz: u32) -> Self {
        Self {
            interval: Self::interval(rate_hz),
            next: None,
        }
    }

    fn interval(rate_hz: u32) -> Duration {
        Duration::from_secs_f64(1.0 / f64::from(rate_hz.max(1)))
    }

    pub fn set_rate(&mut self, rate_hz: u32) {
        self.interval = Self::interval(rate_hz);
        self.next = None;
    }

    /// Returns whether an event arriving at `now` should pass.
    pub fn allow(&mut self, now: Instant) -> bool {
        match self.next {
            Some(next) if now < next => false,
            Some(next) => {
                // Stay on the original schedule so the output rate does not drift below the
                // target, but do not accumulate credit after a pause.
                let scheduled = next + self.interval;
                self.next = Some(if scheduled > now {
                    scheduled
                } else {
                    now + self.interval
                });
                true
            }
            None => {
                self.next = Some(now + self.interval);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counts how many of `input_hz` evenly spaced events pass in one second.
    fn passed(mut throttle: Throttle, input_hz: u32) -> u32 {
        let start = Instant::now();
        (0..input_hz)
            .filter(|i| {
                throttle.allow(start + Duration::from_secs_f64(f64::from(*i) / f64::from(input_hz)))
            })
            .count() as u32
    }

    #[test]
    fn first_event_always_passes() {
        assert!(Throttle::new(1).allow(Instant::now()));
    }

    #[test]
    fn limits_a_faster_input_close_to_the_rate() {
        let out = passed(Throttle::new(60), 120);
        assert!((58..=61).contains(&out), "{out}");
        let out = passed(Throttle::new(30), 100);
        assert!((28..=31).contains(&out), "{out}");
    }

    #[test]
    fn slower_input_passes_untouched() {
        assert_eq!(passed(Throttle::new(60), 30), 30);
    }

    #[test]
    fn a_pause_does_not_cause_a_burst() {
        let mut throttle = Throttle::new(10);
        let start = Instant::now();
        assert!(throttle.allow(start));
        let later = start + Duration::from_secs(10);
        assert!(throttle.allow(later));
        assert!(!throttle.allow(later + Duration::from_millis(1)));
    }

    #[test]
    fn rate_changes_apply_immediately() {
        let mut throttle = Throttle::new(1);
        let start = Instant::now();
        assert!(throttle.allow(start));
        assert!(!throttle.allow(start + Duration::from_millis(100)));
        throttle.set_rate(100);
        assert!(throttle.allow(start + Duration::from_millis(101)));
    }

    #[test]
    fn zero_rate_is_treated_as_one() {
        let mut throttle = Throttle::new(0);
        let start = Instant::now();
        assert!(throttle.allow(start));
        assert!(!throttle.allow(start + Duration::from_millis(500)));
    }
}
