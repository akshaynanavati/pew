/*
 * Copyright 2018 Akshay Nanavati <akshay.nanavati1@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![feature(asm)]

extern crate clap;
#[macro_use]
extern crate lazy_static;

use clap::{App, Arg, ArgMatches};
use std::mem;
use std::sync::{Once, ONCE_INIT};
use std::time::{Duration, Instant};

const DEFAULT_DURATION_RUN_UNTIL: &str = "1000000000";
static HEADER: Once = ONCE_INIT;
lazy_static! {
    static ref CLI_CONFIG: ArgMatches<'static> = App::new("pew-benchmark")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Only run benchmarks that contain this string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("run_until")
                .short("r")
                .long("run_until")
                .value_name("RUN_UNTIL")
                .help("Run benchmarks till this time (in ns) and then output average")
                .takes_value(true)
                .default_value(DEFAULT_DURATION_RUN_UNTIL),
        )
        .get_matches();
}

fn get_duration_run_until() -> u64 {
    CLI_CONFIG
        .value_of("run_until")
        .unwrap()
        .parse::<u64>()
        .unwrap()
}

fn should_run_bm(bm_name: &String) -> bool {
    match CLI_CONFIG.value_of("filter") {
        None => true,
        Some(s) => bm_name.contains(s),
    }
}

fn duration_as_nano(duration: &Duration) -> u64 {
    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64
}

fn range_generator<T>(i: T) -> T {
    i
}

fn compose<T: 'static, U: 'static>(f: Box<Fn(u64) -> T>, g: fn(T) -> U) -> Box<Fn(u64) -> U> {
    Box::new(move |i: u64| g(f(i)))
}

/// The benchmark state
///
/// At a high level, it allows one to pause/resume the timer and also access an argument for this
/// run of the benchmark.
///
/// `T` will either be `u64` in the case a generator is not specified, or a user defined `T: Clone
/// + Default` if a generator(s) is defined (where `T` is the return type of the final specified
/// generator).
pub struct State<T> {
    instant: Instant,
    time_elapsed: Duration,
    is_paused: bool,
    input: T,
}

impl<T> State<T> {
    fn new(input: T) -> State<T> {
        State {
            instant: Instant::now(),
            time_elapsed: Duration::new(0, 0),
            is_paused: false,
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
        self.time_elapsed + elapsed
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

/// The main Benchmark struct
///
/// A benchmark consists of the following:
///
/// - `name: &'static str` - will be prefixed to the output (see `run`)
/// - One or more `bench: fn(&mut State<T>)` which are the actual benchmark. This is the actual
/// benchmark which is timed
/// - A range which consists of a `lower_bound`, an `upper_bound`, and a `mul` factor. Will run the
/// benchmark once for each `i in [lower_bound, upperbound]` such that `i` is initialized to
/// `lower_bound` and gets multiplied by `mul`. Defaults to:
///   - `lower_bound = 1`
///   - `upper_bound = 1 << 20`
///   - `mul = 2`
/// - `generator: fn(u64) -> T` (optional) -  rather than passing `i` from above as input to the
/// benchmark, T produced by this method is passed in instead. For each `i`, `gen(i)` is called
/// once for each `bench` in this `benchmark`. The result of this is then cloned and passed into
/// `bench` each time it is run.
///
/// # Examples
///
/// ```
/// #[macro_use]
/// extern crate pew;
/// use pew::Benchmark;
///
/// fn get_vec(n: u64) -> Vec<u64> {
///     let mut vec = Vec::new();
///     for i in 0..n {
///         vec.push(i as u64);
///     }
///     return vec;
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
/// fn main() {
///     Benchmark::with_name("range_bench")
///         .with_range(1 << 5, 1 << 5, 2)
///         .with_generator(get_vec)
///         .with_bench(pew_bench!(bm_vector_gen))
///         .run();
/// }
/// ```
///
/// This will output:
///
/// ```txt
/// Name,Time (ns)
/// gen_bench/f/1024,103674
/// gen_bench/f/4096,412499
/// gen_bench/f/16384,1634809
/// gen_bench/f/65536,7168879
/// gen_bench/f/262144,27419824
/// gen_bench/f/1048576,109010591
/// ```
///
/// When running the benchmark, you can pass the `--filter` flag. This will only run benchmarks
/// who's name contains the substring passed in to `--filter`.
///
/// See `examples/` for more examples.
pub struct Benchmark<T: 'static + Clone> {
    name: &'static str,
    fns: Vec<(&'static str, fn(&mut State<T>))>,
    range: (u64, u64, u64),
    generator: Box<Fn(u64) -> T>,
}

impl Benchmark<u64> {
    /// Generates a new benchmark with name
    pub fn with_name(name: &'static str) -> Self {
        Benchmark {
            name,
            fns: Vec::new(),
            range: (1, 1 << 20, 2),
            generator: Box::new(range_generator),
        }
    }
}

impl<T: Clone> Benchmark<T> {
    /// Sets the `lower_bound` for this benchmark
    pub fn with_lower_bound(mut self, lb: u64) -> Self {
        self.range.0 = lb;
        self
    }

    /// Sets the `upper_bound` for this benchmark
    pub fn with_upper_bound(mut self, ub: u64) -> Self {
        self.range.1 = ub;
        self
    }

    /// Sets the `mul` for this benchmark
    pub fn with_mul(mut self, mul: u64) -> Self {
        self.range.2 = mul;
        self
    }

    /// Sets the entire range for this benchmark
    pub fn with_range(mut self, lb: u64, ub: u64, mul: u64) -> Self {
        self.range = (lb, ub, mul);
        self
    }

    /// Sets a generator for this benchmark
    ///
    /// Multiple generators can be specified, each of which will be `fn(T) -> U`. These will be
    /// composed with the previous generators.
    ///
    /// Note that calling this function will wipe out all previously set `bench`es. Therefore, this
    /// function should be called before calling `with_bench`.
    pub fn with_generator<U: Clone>(self, gen: fn(T) -> U) -> Benchmark<U> {
        Benchmark {
            name: self.name,
            fns: Vec::new(),
            range: self.range,
            generator: compose(self.generator, gen),
        }
    }

    /// Specifies a benchmark method
    ///
    /// This must be called one or more times before calling `run`. All functions in this suite
    /// will have the same range and generator(s).
    ///
    /// This accepts a tuple with the benchmark name and the function. If the function is an ident
    /// and you want the benchmark name to match the function, use `pew_bench!`.
    pub fn with_bench(mut self, t: (&'static str, fn(&mut State<T>))) -> Self {
        self.fns.push(t);
        self
    }

    /// Runs the benchmark
    ///
    /// Prints the result as a csv with the following format:
    ///
    /// - Header which will be exactly `Name,Time(ns)` (this will be printed once for the whole
    /// program, not once per call to run).
    /// - Rows where
    ///   - `name` will be a slash separated concatenation of the benchmark name, the function
    ///   name, and i
    ///   - `time` will be the time in nanoseconds for running the benchmark
    ///
    /// # Panics
    ///
    /// Panics if no bench methods are specified.
    pub fn run(self) {
        if self.fns.len() == 0 {
            panic!("Cannot call run on an empty benchmark");
        }

        let (lb, ub, mul) = self.range;
        let mut i = lb;
        let gen = &self.generator;
        while i <= ub {
            let input = gen(i);
            for (name, f) in &self.fns {
                let bm_name = format!("{}/{}/{}", self.name, name, i);
                if should_run_bm(&bm_name) {
                    let mut runs = 0;
                    let mut total_duration = 0;
                    while total_duration < get_duration_run_until() {
                        let mut state = State::new(input.clone());
                        f(&mut state);
                        total_duration += duration_as_nano(&state.finish());
                        runs += 1;
                    }

                    HEADER.call_once(|| {
                        println!("Name,Time (ns)");
                    });

                    println!("{},{}", bm_name, total_duration / runs);
                }
            }
            i *= mul;
        }
    }
}

/// This method forces the compiler to not optimize the return statement of a benchmark.
///
/// # Examples
///
/// ``` use pew::{self, State};
///
/// fn bm_simple(state: &mut State<u64>) { pew::do_not_optimize(5 + 10); } ```
///
/// TODO: Conditionally compile this only on nightly so the rest of the benchmark crate can be used
/// in stable.
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
///     let mut vec = Vec::new();
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

/// A convenience macro for stringifying a benchmark function
///
/// Effectively turns an identifier `f` into `(stringify!(f), f)`.
///
/// The result of this should be passed into `Benchmark::<T>::with_bench`.
#[macro_export]
macro_rules! pew_bench {
    ($f:ident) => {
        (stringify!($f), $f)
    };
}
