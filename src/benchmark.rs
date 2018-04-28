use config::Config;
use state::State;
use std::sync::{Once, ONCE_INIT};

static HEADER: Once = ONCE_INIT;

fn should_run_bm(bm_name: &String) -> bool {
    let filter = &Config::get().filter;

    filter.is_match(bm_name)
}

fn range_generator<T>(i: T) -> T {
    i
}

fn compose<T: 'static, U: 'static>(f: Box<Fn(u64) -> T>, g: fn(T) -> U) -> Box<Fn(u64) -> U> {
    Box::new(move |i: u64| g(f(i)))
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
                    while runs < Config::get().min_runs as u64
                        || total_duration < Config::get().min_duration
                    {
                        let mut state = State::new(input.clone());
                        f(&mut state);
                        total_duration += state.finish();
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
