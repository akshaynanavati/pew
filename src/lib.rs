use std::time::{Duration, Instant};

const DURATION_RUN_UNTIL: u64 = 1_000_000_000;

pub struct State {
    instant: Instant,
    time_elapsed: Duration,
    is_paused: bool,
    input: u64,
}

impl State {
    pub fn new(input: u64) -> State {
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

    pub fn get_input(&self) -> u64 {
        return self.input;
    }

    pub fn finish(&self) -> Duration {
        let elapsed = self.instant.elapsed();
        if self.is_paused {
            panic!("Calling finish on a paused state");
        }
        return self.time_elapsed + elapsed;
    }
}

fn duration_as_nano(duration: &Duration) -> u64 {
    return duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
}

pub fn run_benchmark(f: &Fn(&mut State), input: u64) -> (u64, u64) {
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

#[macro_export]
macro_rules! pew_main {
    ($($f: ident -> $gen:expr),+) => {
        fn main() {
            println!("Name,Time (ns)");
            $({
                let bm_name = stringify!($f);
                let (lb, ub, mul) = $gen;
                let mut i = lb;
                while i <= ub {
                    let run_name = format!("{}/{}", bm_name, i);
                    let (total_duration, runs) = pew::run_benchmark(&$f, i);
                    println!("{},{}", run_name, total_duration / runs);
                    i *= mul;
                }
            })+
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
