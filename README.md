# Pew

Pew is a benchmarking library written in Rust based of
[Google's C++ Benchmarking library](https://github.com/google/benchmark). It
is currently in very alpha stages (I'd consider it an MVP). It was built with
to be able to do the following (which you cannot do in the rust benchmarking
library):

1) Pause and unpause the benchmark timer
2) Run multiple benchmarks by specifying a range of arguments
3) Creating some initial state that gets passed to all runs of the benchmark

Currently, it runs the benchmark enough times such that it runs for at least 1
second and then averages all those runs.

TODO: How do we make this more statistically significant? Run the benchmark
till the variance settles?

## Installation

This is available on [crates.io](crates.io). You can link it as a library,
or `cargo install` to also get the transpose script (see [output](#output)).

## Usage

The main exported macro is `pew_main!`. It accepts a comma separated list of
one of the following

```
<func_ident> -> RANGE(<lower_bound_expr>, <upper_bound_expr>, <mul_expr>)
<func_ident> -> GENRANGE(<generator_func_ident>, <lower_bound_expr>, <upper_bound_expr>, <mul_expr>)
```

where:

- `func_ident` is the name of a function in scope. If using `RANGE`,
  `func_ident` should have type `Fn(&mut pew::State<u64>)`. If using
  `GENRANGE`, `func_ident` should have type `Fn(&mut pew::State<T>)`
  where `T` depends on the generator type (see below).
- `lower_bound_expr`, `upper_bound_expr`, and `mul_expr` are all numerical
  types representing the lower, upper, and multiplcation value for the
  benchmark. If using `RANGE`, `state.get_input()` will return all values
  from `i = lower_bound; i <= lower_bound; i *= mul`. If using `GENRANGE`,
  the generator function will receives the aforementioned values.
- `generator_func_ident` is the name of a function in scope. The function
  type should be `Fn(n: usize) -> T` for some `T: Clone`. This function will
  be called once for every `i` in the range (see above). It will be generated
  once per benchmark and cloned every time if the benchmark is run multiple
  times. Note that cloning is not counted in the benchmark time.

## Example

There are examples in the `examples/` directory of how to use this. Basically,
you just want to define your benchmark functions and call `pew_main!` as
described above.

TODO: Add more examples

## Output

The output is a comma separated list of benchmark results (this is what
`cargo cargo run --example example1` will output):

```
Name,Time (ns)
bm_vector_range/1024,102541
bm_vector_range/4096,423289
bm_vector_range/16384,1627492
bm_vector_range/65536,6692188
bm_vector_range/262144,26717609
bm_vector_range/1048576,106552932
bm_vector_gen/1024,102316
bm_vector_gen/4096,416523
bm_vector_gen/16384,1657982
bm_vector_gen/65536,6566634
bm_vector_gen/262144,26780184
bm_vector_gen/1048576,105760350
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

TODO: Add more output types (e.g. pretty printed, JSON, etc).

