// Copyright 2025 - Clipper2 Rust port
// Direct port of Timer.h by Angus Johnson
// License: https://www.boost.org/LICENSE_1_0.txt
//
// Purpose: Performance timing utility

use std::time::{Duration, Instant};

/// A simple stopwatch timer that supports pause and resume.
///
/// Direct port from C++ Timer.h. The timer starts immediately on construction
/// (unless `start_paused` is true). Use `pause()` and `resume()` any number
/// of times; `elapsed()` returns the total time spent unpaused.
///
/// # Examples
///
/// ```
/// use clipper2_rust::utils::timer::Timer;
/// let timer = Timer::new(false);
/// // ... do work ...
/// let elapsed = timer.elapsed();
/// println!("Took {}", Timer::format_duration(elapsed));
/// ```
pub struct Timer {
    time_started: Instant,
    duration: Duration,
    paused: bool,
}

impl Timer {
    /// Create a new timer. If `start_paused` is false, the timer starts immediately.
    pub fn new(start_paused: bool) -> Self {
        Self {
            time_started: Instant::now(),
            duration: Duration::ZERO,
            paused: start_paused,
        }
    }

    /// Restart the timer from zero.
    pub fn restart(&mut self) {
        self.paused = false;
        self.duration = Duration::ZERO;
        self.time_started = Instant::now();
    }

    /// Resume a paused timer. No-op if already running.
    pub fn resume(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        self.time_started = Instant::now();
    }

    /// Pause a running timer. No-op if already paused.
    pub fn pause(&mut self) {
        if self.paused {
            return;
        }
        self.duration += self.time_started.elapsed();
        self.paused = true;
    }

    /// Return the total elapsed duration (excluding paused intervals).
    ///
    /// If the timer is currently running, includes time since last resume.
    /// If the timer is paused, returns accumulated time only.
    pub fn elapsed(&self) -> Duration {
        if self.paused {
            self.duration
        } else {
            self.duration + self.time_started.elapsed()
        }
    }

    /// Return elapsed time in nanoseconds.
    /// Direct port from C++ `elapsed_nano()`.
    pub fn elapsed_nanos(&self) -> u128 {
        self.elapsed().as_nanos()
    }

    /// Format a duration as a human-readable string.
    /// Direct port from C++ `elapsed_str()`.
    ///
    /// Automatically selects appropriate units (microsecs, millisecs, secs).
    pub fn format_duration(dur: Duration) -> String {
        let nanos = dur.as_nanos() as f64;
        if nanos < 1.0 {
            return "0 microsecs".to_string();
        }
        let log10 = nanos.log10() as i32;
        if log10 < 6 {
            let precision = (2 - (log10 % 3)) as usize;
            format!("{:.prec$} microsecs", nanos * 1.0e-3, prec = precision)
        } else if log10 < 9 {
            let precision = (2 - (log10 % 3)) as usize;
            format!("{:.prec$} millisecs", nanos * 1.0e-6, prec = precision)
        } else {
            let precision = (2 - (log10 % 3)) as usize;
            format!("{:.prec$} secs", nanos * 1.0e-9, prec = precision)
        }
    }

    /// Return elapsed time as a human-readable string.
    pub fn elapsed_str(&self) -> String {
        Self::format_duration(self.elapsed())
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timer_starts_running() {
        let timer = Timer::new(false);
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn test_timer_starts_paused() {
        let timer = Timer::new(true);
        thread::sleep(Duration::from_millis(10));
        // Should have accumulated very little time since it was paused
        assert!(timer.elapsed() < Duration::from_millis(1));
    }

    #[test]
    fn test_timer_pause_resume() {
        let mut timer = Timer::new(false);
        thread::sleep(Duration::from_millis(20));
        timer.pause();
        let paused_elapsed = timer.elapsed();
        thread::sleep(Duration::from_millis(20));
        // Should not have changed while paused
        assert_eq!(timer.elapsed(), paused_elapsed);
        timer.resume();
        thread::sleep(Duration::from_millis(20));
        assert!(timer.elapsed() > paused_elapsed);
    }

    #[test]
    fn test_timer_restart() {
        let mut timer = Timer::new(false);
        thread::sleep(Duration::from_millis(20));
        timer.restart();
        // After restart, elapsed should be very small
        assert!(timer.elapsed() < Duration::from_millis(5));
    }

    #[test]
    fn test_format_duration_microsecs() {
        let s = Timer::format_duration(Duration::from_micros(500));
        assert!(s.contains("microsecs"));
    }

    #[test]
    fn test_format_duration_millisecs() {
        let s = Timer::format_duration(Duration::from_millis(50));
        assert!(s.contains("millisecs"));
    }

    #[test]
    fn test_format_duration_secs() {
        let s = Timer::format_duration(Duration::from_secs(2));
        assert!(s.contains("secs"));
    }

    #[test]
    fn test_format_duration_zero() {
        let s = Timer::format_duration(Duration::ZERO);
        assert!(s.contains("microsecs"));
    }

    #[test]
    fn test_default_timer_starts_running() {
        let timer = Timer::default();
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(5));
    }

    #[test]
    fn test_elapsed_nanos() {
        let timer = Timer::new(false);
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_nanos() > 1_000_000); // > 1ms in nanos
    }
}
