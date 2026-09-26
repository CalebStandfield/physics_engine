//! Measuring the simulation's own output.
//!
//! The engine has to be able to answer "what period did you actually produce"
//! without anyone reading it off a graph, so the predicted and measured numbers
//! can be compared directly.

use crate::state::State;

/// Percent difference between two values of equal standing, relative to their
/// average: `|a - b| / ((|a| + |b|) / 2) * 100`.
///
/// Zero when both are zero.
pub fn percent_difference(a: f64, b: f64) -> f64 {
    let mean = (a.abs() + b.abs()) / 2.0;
    if mean == 0.0 {
        0.0
    } else {
        (a - b).abs() / mean * 100.0
    }
}

/// Percent error of a measurement against a value taken as correct:
/// `|measured - accepted| / |accepted| * 100`.
pub fn percent_error(measured: f64, accepted: f64) -> f64 {
    if accepted == 0.0 {
        0.0
    } else {
        (measured - accepted).abs() / accepted.abs() * 100.0
    }
}

/// Finds the period of an oscillation by timing repeated upward crossings of a
/// reference level.
///
/// Feed it states as they come. Each time the signal rises through `level`, the
/// crossing time is pinned down by straight-line interpolation between the two
/// samples that straddle it, which gets the timing well inside one step.
#[derive(Debug, Clone)]
pub struct PeriodDetector {
    level: f64,
    previous: Option<State>,
    crossings: Vec<f64>,
    max_crossings: usize,
}

impl PeriodDetector {
    /// Watch for crossings of `level`, keeping the most recent
    /// `max_crossings` of them.
    pub fn new(level: f64, max_crossings: usize) -> Self {
        Self {
            level,
            previous: None,
            crossings: Vec::new(),
            max_crossings: max_crossings.max(2),
        }
    }

    /// Watch for crossings of `level`, keeping enough history for 64 cycles.
    pub fn about(level: f64) -> Self {
        Self::new(level, 65)
    }

    pub fn level(&self) -> f64 {
        self.level
    }

    /// Times of the upward crossings seen so far, oldest first.
    pub fn crossings(&self) -> &[f64] {
        &self.crossings
    }

    /// Complete cycles measured.
    pub fn cycles(&self) -> usize {
        self.crossings.len().saturating_sub(1)
    }

    pub fn reset(&mut self) {
        self.previous = None;
        self.crossings.clear();
    }

    /// Offer the next state. Samples must arrive in time order.
    pub fn observe(&mut self, state: State) {
        if let Some(prev) = self.previous {
            let before = prev.x - self.level;
            let after = state.x - self.level;
            // Rising through the level, counting a landing exactly on it once.
            if before < 0.0 && after >= 0.0 {
                let span = after - before;
                let fraction = if span == 0.0 { 0.0 } else { -before / span };
                let crossing = prev.t + fraction * (state.t - prev.t);
                self.crossings.push(crossing);
                if self.crossings.len() > self.max_crossings {
                    self.crossings.remove(0);
                }
            }
        }
        self.previous = Some(state);
    }

    /// Mean time between crossings, seconds. `None` until two have been seen.
    ///
    /// Averaging over every cycle in the window is the same trick as timing ten
    /// swings with a stopwatch instead of one.
    pub fn period(&self) -> Option<f64> {
        let first = *self.crossings.first()?;
        let last = *self.crossings.last()?;
        let cycles = self.cycles();
        if cycles == 0 {
            None
        } else {
            Some((last - first) / cycles as f64)
        }
    }

    /// Cycles per second. `None` until a period is available.
    pub fn frequency(&self) -> Option<f64> {
        self.period().filter(|p| *p > 0.0).map(|p| 1.0 / p)
    }
}

/// Run a detector over a recorded run in one go.
pub fn period_of<'a, I>(samples: I, level: f64) -> Option<f64>
where
    I: IntoIterator<Item = &'a State>,
{
    let mut detector = PeriodDetector::new(level, usize::MAX);
    for s in samples {
        detector.observe(*s);
    }
    detector.period()
}

/// Smallest and largest position in a run, and the swing between them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Extent {
    pub min: f64,
    pub max: f64,
}

impl Extent {
    /// Half the peak-to-peak swing.
    pub fn amplitude(&self) -> f64 {
        (self.max - self.min) / 2.0
    }

    /// Midpoint of the swing.
    pub fn center(&self) -> f64 {
        (self.max + self.min) / 2.0
    }
}

/// Range of positions covered by a run. `None` for an empty run.
pub fn extent<'a, I>(samples: I) -> Option<Extent>
where
    I: IntoIterator<Item = &'a State>,
{
    let mut iter = samples.into_iter();
    let first = iter.next()?;
    let mut out = Extent {
        min: first.x,
        max: first.x,
    };
    for s in iter {
        out.min = out.min.min(s.x);
        out.max = out.max.max(s.x);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    #[test]
    fn percent_difference_is_symmetric() {
        assert!((percent_difference(1.0, 1.1) - percent_difference(1.1, 1.0)).abs() < 1e-12);
        assert_eq!(percent_difference(0.0, 0.0), 0.0);
        assert!((percent_difference(10.0, 10.0)).abs() < 1e-12);
        // 9 vs 11: difference 2 over an average of 10.
        assert!((percent_difference(9.0, 11.0) - 20.0).abs() < 1e-12);
    }

    #[test]
    fn percent_error_is_relative_to_the_accepted_value() {
        assert!((percent_error(1.05, 1.0) - 5.0).abs() < 1e-12);
        assert_eq!(percent_error(1.0, 0.0), 0.0);
    }

    /// A clean sine wave sampled far more finely than its period.
    fn sine_samples(period: f64, offset: f64, duration: f64, dt: f64) -> Vec<State> {
        let steps = (duration / dt) as usize;
        (0..steps)
            .map(|i| {
                let t = i as f64 * dt;
                State::new(t, offset + (TAU * t / period).sin(), 0.0)
            })
            .collect()
    }

    #[test]
    fn period_of_a_known_sine_comes_back() {
        let samples = sine_samples(0.7, 0.0, 7.0, 1e-4);
        let measured = period_of(&samples, 0.0).unwrap();
        assert!((measured - 0.7).abs() < 1e-5, "got {measured}");
    }

    #[test]
    fn interpolation_beats_the_sample_spacing() {
        // Deliberately coarse: only about 35 samples per cycle.
        let samples = sine_samples(0.7, 0.0, 7.0, 0.02);
        let measured = period_of(&samples, 0.0).unwrap();
        assert!((measured - 0.7).abs() < 0.001, "got {measured}");
    }

    #[test]
    fn crossings_are_measured_about_the_given_level() {
        let samples = sine_samples(0.5, 3.0, 5.0, 1e-4);
        assert!((period_of(&samples, 3.0).unwrap() - 0.5).abs() < 1e-5);
        // Nothing ever rises through a level the signal never reaches.
        assert!(period_of(&samples, 99.0).is_none());
    }

    #[test]
    fn detector_reports_cycles_and_frequency() {
        let mut d = PeriodDetector::about(0.0);
        for s in sine_samples(0.25, 0.0, 1.05, 1e-4) {
            d.observe(s);
        }
        assert_eq!(d.cycles(), 3);
        assert!((d.frequency().unwrap() - 4.0).abs() < 1e-3);
        d.reset();
        assert_eq!(d.cycles(), 0);
        assert!(d.period().is_none());
    }

    #[test]
    fn extent_measures_the_swing() {
        let samples = sine_samples(0.5, 2.0, 2.0, 1e-4);
        let e = extent(&samples).unwrap();
        assert!((e.amplitude() - 1.0).abs() < 1e-3);
        assert!((e.center() - 2.0).abs() < 1e-3);
        assert!(extent(&[]).is_none());
    }
}
