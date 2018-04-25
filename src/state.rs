use clock::Clock;
use std::mem;

/// The benchmark state
///
/// At a high level, it allows one to pause/resume the timer and also access an argument for this
/// run of the benchmark.
///
/// `T` will either be `u64` in the case a generator is not specified, or a user defined `T: Clone
/// + Default` if a generator(s) is defined (where `T` is the return type of the final specified
/// generator).
pub struct State<T> {
    clock: Clock,
    input: T,
}

impl<T> State<T> {
    pub fn new(input: T) -> State<T> {
        State {
            clock: Clock::new(),
            input: input,
        }
    }

    /// Pauses the benchmark timer. Useful to do any initialization work, etc.
    /// The state begins in a running (unpaused) state.
    ///
    /// # Examples
    ///
    /// ```
    /// use pew::{self, State};
    /// use std::vec::Vec;
    ///
    /// fn bm_simple(state: &mut State<u64>) {
    ///     state.pause();
    ///     let mut vec = Vec::new();
    ///     vec.push(1);
    ///     state.resume();
    ///     pew::do_not_optimize(vec.get(1));
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the state is already paused.
    pub fn pause(&mut self) {
        self.clock.pause();
    }

    /// Resumes the benchmark timer. Useful after any initialization work, etc.
    /// The state begins in a running (unpaused) state.
    ///
    /// #Examples
    ///
    /// ```
    /// use pew::{self, State};
    /// use std::vec::Vec;
    ///
    /// fn bm_simple(state: &mut State<u64>) {
    ///     state.pause();
    ///     let mut vec = Vec::new();
    ///     vec.push(1);
    ///     state.resume();
    ///     pew::do_not_optimize(vec.get(1));
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the state is paused.
    pub fn resume(&mut self) {
        self.clock.resume();
    }

    pub fn finish(self) -> u64 {
        self.clock.stop()
    }
}

impl<T: Default> State<T> {
    /// Returns the input. Either `u64` (if no generator is specified) or a user specified
    /// `T: Clone + Default` where `T` is the return type of the last generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use pew::{self, State};
    /// use std::vec::Vec;
    ///
    /// fn bm_simple(state: &mut State<Vec<u64>>) {
    ///     let mut vec = state.get_input();
    ///     let n = vec.len();
    ///     for i in 0..n {
    ///         vec.push(i as u64);
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the state is paused.
    pub fn get_input(&mut self) -> T {
        self.pause();
        let input = mem::replace(&mut self.input, T::default());
        self.resume();
        input
    }
}
