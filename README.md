# Pew

Pew is a benchmarking library written in Rust based of
[Google's C++ Benchmarking library](https://github.com/google/benchmark). It
is currently in very alpha stages (I'd consider it an MVP). It was built to be
able to do the following (which you cannot do in the rust benchmarking library):

1) Pause and unpause the benchmark timer
2) Run multiple benchmarks by specifying a range of arguments
3) Creating some initial state that gets passed to all runs of the benchmark

Currently, it runs the benchmark enough times such that it runs for at least 1
second and then averages all those runs.

More comprehensive docs can be found [here](https://docs.rs/pew/0.1.0/pew/).

[TODO](https://github.com/akshaynanavati/pew/issues/1): How do we make this
more statistically significant? Run the benchmark till the variance settles?

## Installation

This is available on [crates.io](https://crates.io/crates/pew). You can link it
as a library, or `cargo install` to also get the transpose script (see [output](#output)).

First, add this to your `Cargo.toml`:

```
[dependencies]
pew = "0.1"
```

Next, add this to your crate root:

```
#[macro_use]
extern crate pew;
```

I usually create benchmarks in the `bin/` directory and run them with
`cargo run --bin <benchmark-name> --release`.

## Usage

View the docs [here](https://docs.rs/pew/0.1.0/pew/).

## Example

This can be used as follows:

```
fn main() {
    Benchmark::with_name("range_bench")
        .with_range(1 << 10, 1 << 20, 4)
        .with_generator(generator)
        .with_bench(pew_bench!(bm_vector1))
        .with_bench(pew_bench!(bm_vector2))
        .with_bench(pew_bench!(bm_vector3))
        .run();
}
```

There are more complete examples in the `examples/` directory of how to use this.

## Output

The output is a comma separated list of benchmark results (this is what
`cargo cargo run --example example1` will output):

```
Name,Time (ns)
range_bench/bm_vector_range/1024,104715
range_bench/bm_vector_range/4096,554838
range_bench/bm_vector_range/16384,2068971
range_bench/bm_vector_range/65536,7739376
range_bench/bm_vector_range/262144,31389948
range_bench/bm_vector_range/1048576,114633815
gen_bench/bm_vector_gen/1024,123643
gen_bench/bm_vector_gen/4096,545581
gen_bench/bm_vector_gen/16384,2590869
gen_bench/bm_vector_gen/65536,7799209
gen_bench/bm_vector_gen/262144,29498657
gen_bench/bm_vector_gen/1048576,113458415
```

You can also pass a `--filter` flag to the benchmark which would only run
benchmarks who's name contains the filter string. For example, running
`cargo cargo run --example example1 -- --filter gen` will output:

```
gen_bench/bm_vector_gen/1024,123643
gen_bench/bm_vector_gen/4096,545581
gen_bench/bm_vector_gen/16384,2590869
gen_bench/bm_vector_gen/65536,7799209
gen_bench/bm_vector_gen/262144,29498657
gen_bench/bm_vector_gen/1048576,113458415
```

while running `cargo cargo run --example example1 -- --filter 1024` will output:

```
range_bench/bm_vector_range/1024,104715
gen_bench/bm_vector_gen/1024,123643
```

Oftentimes I run multiple benchmarks on the same range and plot the results.
I find it is easier to plot the results by transposing the output. There is
a binary `pew_transpose` that does just that. It can be run by piping the
output of the benchmark code:

```
cargo run --example example1 | pew_transpose
```

The transposed output of the
above would be:

```
Size,bm_vector_range,bm_vector_gen
1024,105974,106845
4096,418835,409143
16384,1646391,1655855
65536,6566094,6668369
262144,27025772,26948818
1048576,108144239,107596626
```

[TODO](https://github.com/akshaynanavati/pew/issues/1): Add more output types
(e.g. pretty printed, JSON, etc).

## Cli

The Cli offers convenience flags and options while running benchmarks:

```
pew-benchmark 0.2.0
Akshay Nanavati <akshay.nanavati1@gmail.com>
A benchmarking library for Rust based on google/benchmark

USAGE:
    example1 [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter <FILTER>          Only run benchmarks that contain this string
    -r, --run_until <RUN_UNTIL>    Run benchmarks till this time (in ns) and then output average [default: 1000000000]
```

These can be passed to the main binary that is running your benchmark.

## License

This code is licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).

## Contributing

This library is in very early stages and I'd love any contributions I can get. This is my
first time writing a becnhmarking library (and a library in Rust) and would love input from
those who are more experienced.

A good starting point would be the issues (which are also linked here as TODOs).
