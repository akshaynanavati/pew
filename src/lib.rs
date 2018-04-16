#![feature(asm)]

use std::mem;
use std::time::{Duration, Instant};

const DURATION_RUN_UNTIL: u64 = 1_000_000_000;

/// This is the benchmark state. At a high level, it allows one to pause the
/// timer and also access an argument for this run of the benchmark.
///
/// `T` will either be `u64` in the case of `RANGE`, or a user defined `T: Clone`
/// in the case of `GENRANGE`.
pub struct State<T> {
    instant: Instant,
    time_elapsed: Duration,
    is_paused: bool,
    input: T,
}

/// This method forces the compiler to not optimize the return statement of
/// a benchmark.
///
/// # Examples
///
/// ```
/// use pew::{self, State};
///
/// fn bm_simple(state: &mut State<u64>) {
///     pew::do_not_optimize(5 + 10);
/// }
/// ```
///
/// TODO: Conditionally compile this only on nightly so the rest of the
/// benchmark crate can be used in stable.
pub fn do_not_optimize<T>(val: T) {
    let mut _v = val;
    unsafe {
        asm!("" : "+r" (&_v) : : : "volatile");
    }
}

/// This method forces the compiler to not optimize writes to memory in
/// a benchmark.
///
/// # Examples
///
/// ```
/// use pew::{self, State};
/// use std::vec::Vec;
///
/// fn bm_simple(state: &mut State<u64>) {
///     let vec = Vec::new();
///     vec.push(1);
///     vec.push(2);
///     pew::clobber();
/// }
/// ```
///
/// TODO: Conditionally compile this only on nightly so the rest of the
/// benchmark crate can be used in stable.
pub fn clobber() {
    unsafe {
        asm!("" : : : "memory" : "volatile");
    }
}

impl<T> State<T> {
    fn new(input: T) -> State<T> {
        return State {
            instant: Instant::now(),
            time_elapsed: Duration::new(0, 0),
            is_paused: false,
            input: input,
        };
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
    ///     let vec = Vec::new();
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
        self.time_elapsed += self.instant.elapsed();
        if self.is_paused {
            panic!("Calling pause on an already paused benchmark");
        }
        self.is_paused = true;
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
    ///     let vec = Vec::new();
    ///     vec.push(1);
    ///     state.resume();
    ///     pew::do_not_optimize(vec.get(1));
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the state is already paused.
    pub fn resume(&mut self) {
        if !self.is_paused {
            panic!("Calling resume on an already running benchmark");
        }
        self.is_paused = false;
        self.instant = Instant::now();
    }

    fn finish(&self) -> Duration {
        let elapsed = self.instant.elapsed();
        if self.is_paused {
            panic!("Calling finish on a paused state");
        }
        return self.time_elapsed + elapsed;
    }
}

impl<T: Default> State<T> {
    /// Returns the input. Either `u64` (if using `RANGE`) or a user specified
    /// `T: Clone` if using `GENRANGE`.
    ///
    /// # Examples
    ///
    /// ## RANGE
    ///
    /// ```
    /// use pew::{self, State};
    /// use std::vec::Vec;
    ///
    /// fn bm_simple(state: &mut State<u64>) {
    ///     let n = state.get_input();
    ///     state.pause();
    ///     let vec = Vec::with_capacity(n);
    ///     state.resume();
    ///     for i in 0..n {
    ///     vec.push(i);
    ///     }
    /// }
    /// ```
    ///
    /// ## GENRANGE
    ///
    /// ```
    /// use pew::{self, State};
    /// use std::vec::Vec;
    ///
    /// fn bm_simple(state: &mut State<Vec<u64>>) {
    ///     let mut vec = state.get_input();
    ///     for i in 0..n {
    ///     vec.push(i);
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
        return input;
    }
}

fn duration_as_nano(duration: &Duration) -> u64 {
    return duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
}

/// This is not intended to be used by the user. It should just be called by
/// `pew_main!`.
pub fn run_benchmark_range(f: &Fn(&mut State<u64>), input: u64) -> (u64, u64) {
    let mut runs = 0;
    let mut total_duration = 0;
    while total_duration < DURATION_RUN_UNTIL {
        let mut state = State::new(input);
        f(&mut state);
        total_duration += duration_as_nano(&state.finish());
        runs += 1;
    }
    return (total_duration, runs);
}

/// This is not intended to be used by the user. It should just be called by
/// `pew_main!`.
pub fn run_benchmark_gen<T: Clone>(f: &Fn(&mut State<T>), input: T) -> (u64, u64) {
    let mut runs = 0;
    let mut total_duration = 0;
    while total_duration < DURATION_RUN_UNTIL {
        let input_clone = input.clone();
        let mut state = State::new(input_clone);
        f(&mut state);
        total_duration += duration_as_nano(&state.finish());
        runs += 1;
    }
    return (total_duration, runs);
}

/// Generates the main method that runs the actual benchmarks.
///
/// Accepts a comma separated list of either of the following:
///
/// ```
/// <func_ident> -> RANGE(<lower_bound_expr>, <upper_bound_expr>, <mul_expr>)
/// <func_ident> -> GENRANGE(<generator_func_ident>, <lower_bound_expr>, <upper_bound_expr>, <mul_expr>)
/// ```

/// where:
///
/// - `func_ident` is the name of a function in scope. If using `RANGE`,
///   `func_ident` should have type `Fn(&mut pew::State<u64>)`. If using
///   `GENRANGE`, `func_ident` should have type `Fn(&mut pew::State<T>)`
///   where `T` depends on the generator type (see below).
/// - `lower_bound_expr`, `upper_bound_expr`, and `mul_expr` are all numerical
///   types representing the lower, upper, and multiplcation value for the
///   benchmark. If using `RANGE`, `state.get_input()` will return all values
///   from `i = lower_bound; i <= lower_bound; i *= mul`. If using `GENRANGE`,
///   the generator function will receives the aforementioned values.
/// - `generator_func_ident` is the name of a function in scope. The function
///   type should be `Fn(n: usize) -> T` for some `T: Clone`. This function will
///   be called once for every `i` in the range (see above). It will be generated
///   once per benchmark and cloned every time if the benchmark is run multiple
///   times. Note that cloning is not counted in the benchmark time.
///
/// # Examples
///
/// ```
/// use std::vec::Vec;
///
/// #[macro_use]
/// extern crate pew;
///
/// fn get_vec(n: usize) -> Vec<u64> {
///     let mut vec = Vec::new();
///     for i in 0..n {
///         vec.push(i as u64);
///     }
///     return vec;
/// }
///
/// fn bm_vector_range(state: &mut pew::State<u64>) {
///     let input = state.get_input();
///     state.pause();
///     let mut vec = get_vec(input as usize);
///     state.resume();
///     for _ in 0..input {
///         pew::do_not_optimize(vec.pop());
///     }
/// }
///
/// fn bm_vector_gen(state: &mut pew::State<Vec<u64>>) {
///     let mut vec = state.get_input();
///     let n = vec.len() as u64;
///     for _ in 0..n {
///         pew::do_not_optimize(vec.pop());
///     }
/// }
///
/// pew_main!(
///     bm_vector_range -> RANGE(1<<10, 1 << 20, 4),
///     bm_vector_gen -> GENRANGE(get_vec, 1<<10, 1<<20, 4)
/// );
/// ```
#[macro_export]
macro_rules! pew_main {
    (@inner $f: ident -> RANGE$e: expr) => {
        {
            let bm_name = stringify!($f);
            let (lb, ub, mul) = $e;
            let mut i = lb;
            while i <= ub {
                let run_name = format!("{}/{}", bm_name, i);
                let (total_duration, runs) = pew::run_benchmark_range(&$f, i);
                println!("{},{}", run_name, total_duration / runs);
                i *= mul;
            }
        }
    };
    (@inner $f: ident -> GENRANGE$e: expr) => {
        {
            let bm_name = stringify!($f);
            let (gen, lb, ub, mul) = $e;
            let mut i = lb;
            while i <= ub {
                let run_name = format!("{}/{}", bm_name, i);
                let (total_duration, runs) = pew::run_benchmark_gen(&$f, gen(i));
                println!("{},{}", run_name, total_duration / runs);
                i *= mul;
            }
        }
    };
    ($($f: ident -> $id: ident$e: expr),+) => {
        fn main() {
            println!("Name,Time (ns)");
            $(
                pew_main!(@inner $f -> $id$e);
            )+;
        }
    };
}
