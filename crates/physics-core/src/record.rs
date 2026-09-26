//! Time-series capture, for the position-vs-time and velocity-vs-time plots.

use crate::state::State;

/// Fixed-capacity ring buffer of samples.
///
/// Fixed capacity so a long run cannot grow without bound in the browser: once
/// full, the oldest sample is dropped. `stride` thins the recording, e.g.
/// `stride = 10` keeps every tenth step.
#[derive(Debug, Clone)]
pub struct Recorder {
    samples: Vec<State>,
    capacity: usize,
    stride: usize,
    /// Steps seen since the last kept sample.
    pending: usize,
    /// Index of the oldest sample once the buffer has wrapped.
    head: usize,
}

impl Recorder {
    /// `capacity` samples kept, one sample taken every `stride` steps. Both are
    /// forced to at least 1.
    pub fn new(capacity: usize, stride: usize) -> Self {
        Self {
            samples: Vec::new(),
            capacity: capacity.max(1),
            stride: stride.max(1),
            pending: 0,
            head: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn set_stride(&mut self, stride: usize) {
        self.stride = stride.max(1);
        self.pending = 0;
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
        self.pending = 0;
        self.head = 0;
    }

    /// Offer a sample. Kept only on every `stride`-th call.
    pub fn observe(&mut self, state: State) {
        if self.pending == 0 {
            self.push(state);
        }
        self.pending = (self.pending + 1) % self.stride;
    }

    /// Keep a sample regardless of stride. Used for the initial state, so a
    /// fresh recording always starts at t = 0.
    pub fn push(&mut self, state: State) {
        if self.samples.len() < self.capacity {
            self.samples.push(state);
        } else {
            self.samples[self.head] = state;
            self.head = (self.head + 1) % self.capacity;
        }
    }

    /// Samples oldest first.
    pub fn iter(&self) -> impl Iterator<Item = &State> + '_ {
        let (front, back) = self.samples.split_at(self.head.min(self.samples.len()));
        back.iter().chain(front.iter())
    }

    /// Samples oldest first, as an owned vector.
    pub fn to_vec(&self) -> Vec<State> {
        self.iter().copied().collect()
    }

    /// Flattened `[t, x, v, t, x, v, ...]`, oldest first. This is the shape the
    /// bindings hand to JS as one typed array.
    pub fn to_flat(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.samples.len() * 3);
        for s in self.iter() {
            out.extend_from_slice(&[s.t, s.x, s.v]);
        }
        out
    }

    /// Most recent sample, if any.
    pub fn last(&self) -> Option<State> {
        self.iter().last().copied()
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new(4096, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stride_thins_the_recording() {
        let mut r = Recorder::new(100, 3);
        for i in 0..9 {
            r.observe(State::new(i as f64, 0.0, 0.0));
        }
        let times: Vec<_> = r.iter().map(|s| s.t).collect();
        assert_eq!(times, vec![0.0, 3.0, 6.0]);
    }

    #[test]
    fn wrapping_keeps_the_newest_in_order() {
        let mut r = Recorder::new(3, 1);
        for i in 0..5 {
            r.observe(State::new(i as f64, 0.0, 0.0));
        }
        let times: Vec<_> = r.iter().map(|s| s.t).collect();
        assert_eq!(times, vec![2.0, 3.0, 4.0]);
        assert_eq!(r.len(), 3);
        assert_eq!(r.last().unwrap().t, 4.0);
    }

    #[test]
    fn flat_layout_is_triples() {
        let mut r = Recorder::new(4, 1);
        r.observe(State::new(1.0, 2.0, 3.0));
        assert_eq!(r.to_flat(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn clearing_resets_the_wrap() {
        let mut r = Recorder::new(2, 1);
        for i in 0..5 {
            r.observe(State::new(i as f64, 0.0, 0.0));
        }
        r.clear();
        r.observe(State::new(9.0, 0.0, 0.0));
        assert_eq!(r.to_vec().len(), 1);
        assert_eq!(r.last().unwrap().t, 9.0);
    }
}
