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
#![doc(issue_tracker_base_url = "https://github.com/akshaynanavati/pew/issues/")]

//! # Pew is a benchmarking library
//!
//! Pew is inspired by [Google's C++ Benchmarking library](https://github.com/google/benchmark). It
//! is currently in very alpha stages (I'd consider it an MVP). It was built to be able to do the
//! following (which you cannot do in the rust benchmarking library):
//!
//! 1) Pause and unpause the benchmark timer
//! 2) Run multiple benchmarks by specifying a range of arguments
//! 3) Creating some initial state that gets passed to all runs of the benchmark
//!
//! The benchmark will run for at least 1 second (or the user specified
//! `--min_duration`) and at least 8 runs (or the user specified `--min_runs`).
//! The average of these runs is output as the `Time (ns)` column.
//!
//! The following flags are available when running the benchmark binary:
//!
//! ```txt
//! Akshay Nanavati <akshay.nanavati1@gmail.com>
//! A benchmarking library for Rust based on google/benchmark
//!
//! USAGE:
//!     example1 [OPTIONS]
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!     -f, --filter <FILTER>             Only run benchmarks that contain this string
//!     -d, --min_duration <RUN_UNTIL>    Run benchmarks till this time (in s) and then output average [default: 1]
//!     -r, --min_runs <MIN_RUNS>         Run benchmarks for at least this many runs [default: 8]
//!  ```
//!
//!  Use `-h` to get the most up to date flags.

extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate regex;

mod clock;
mod benchmark;
mod state;
mod config;

pub use benchmark::Benchmark;
pub use state::State;

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
