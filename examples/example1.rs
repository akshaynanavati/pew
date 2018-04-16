use std::vec::Vec;

#[macro_use]
extern crate pew;

/// Builds a vector.
///
/// This is not part of the benchmark, but just creates it so
/// we can benchmark the pop performance.
///
/// It matches the signature for a `GENRANGE` where each `n` will be
/// one in the range.
fn get_vec(n: usize) -> Vec<u64> {
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i as u64);
    }
    return vec;
}

/// `RANGE` benchmark.
///
/// Here, we pause the benchmark and build the vector each time. While the
/// benchmark time itself will not be impacted, the overall benchmark may
/// take longer expecially if it is cheaper to clone the vec than build it
/// (which may not be the case in this simple example).
///
/// The state input will be a `u64` per the `RANGE` spec.
fn bm_vector_range(state: &mut pew::State<u64>) {
    let input = state.get_input();
    state.pause();
    let mut vec = get_vec(input as usize);
    state.resume();
    for _ in 0..input {
        pew::do_not_optimize(vec.pop());
    }
}

/// `GENRANGE` benchmark.
///
/// Here, we no longer need to build the vector each time. It is built once
/// per benchmark (or `n` in `RANGE`) and then cloned. The benchmark times
/// themselves should be the same; but this should run slightly faster.
///
/// The state input will be a `Vec<u64>` per the `GENRANGE` return type.
fn bm_vector_gen(state: &mut pew::State<Vec<u64>>) {
    let mut vec = state.get_input();
    let n = vec.len() as u64;
    for _ in 0..n {
        pew::do_not_optimize(vec.pop());
    }
}

/// We can pass in any number of `RANGE` or `GENRANGE` "arguments" to this
/// main macro and they will all be run.
pew_main!(
    bm_vector_range -> RANGE(1<<10, 1 << 20, 4),
    bm_vector_gen -> GENRANGE(get_vec, 1<<10, 1<<20, 4)
);
