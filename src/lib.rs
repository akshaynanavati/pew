#![feature(asm)]

use std::mem;
use std::time::{Duration, Instant};

const DURATION_RUN_UNTIL: u64 = 1_000_000_000;

pub struct State<T> {
    instant: Instant,
    time_elapsed: Duration,
    is_paused: bool,
    input: T,
}

pub fn do_not_optimize<T>(val: T) {
    let mut _v = val;
    unsafe {
        asm!("" : "+r" (&_v) : : : "volatile");
    }
}

pub fn clobber() {
    unsafe {
        asm!("" : : : "memory" : "volatile");
    }
}

impl<T> State<T> {
    pub fn new(input: T) -> State<T> {
        return State {
            instant: Instant::now(),
            time_elapsed: Duration::new(0, 0),
            is_paused: false,
            input: input,
        };
    }

    pub fn pause(&mut self) {
        self.time_elapsed += self.instant.elapsed();
        if self.is_paused {
            panic!("Calling pause on an already paused benchmark");
        }
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        if !self.is_paused {
            panic!("Calling resume on an already running benchmark");
        }
        self.is_paused = false;
        self.instant = Instant::now();
    }

    pub fn finish(&self) -> Duration {
        let elapsed = self.instant.elapsed();
        if self.is_paused {
            panic!("Calling finish on a paused state");
        }
        return self.time_elapsed + elapsed;
    }
}

impl<T: Default> State<T> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
