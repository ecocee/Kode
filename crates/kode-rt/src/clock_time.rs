//! A steady nanosecond clock exposed to Kode programs through `time.now_ns()`.
//!
//! It is monotonic (not a wall clock), so two measurements a few milliseconds
//! apart are guaranteed to advance, and it has sub-millisecond resolution for
//! p50/p95/p99 latency timing.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

static START: OnceLock<Instant> = OnceLock::new();

fn elapsed() -> Duration {
    START.get_or_init(Instant::now).elapsed()
}

/// Nanoseconds since the first clock call in this process.
pub(crate) fn now_ns() -> i64 {
    elapsed().as_nanos() as i64
}

/// Milliseconds since the same starting point as `now_ns()`.
pub(crate) fn now_ms() -> i64 {
    elapsed().as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_over_jumps_larger_than_one_millisecond() {
        let a = now_ns();
        std::thread::sleep(Duration::from_millis(2));
        let b = now_ns();
        assert!(b >= a);
        assert!(b - a >= 2_000_000, "2 ms sleep advanced only {} ns", b - a);
    }

    #[test]
    fn ms_and_ns_share_one_origin() {
        let ns = now_ns();
        let ms = now_ms();
        assert!(ms >= ns / 1_000_000);
    }
}
