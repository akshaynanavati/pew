use std::vec::Vec;

#[macro_use]
extern crate pew;

fn get_vec(n: usize) -> Vec<u64> {
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i as u64);
    }
    return vec;
}

fn bm_vector_range(state: &mut pew::State<u64>) {
    let input = state.get_input();
    state.pause();
    let mut vec = get_vec(input as usize);
    state.resume();
    for _ in 0..input {
        pew::do_not_optimize(vec.pop());
    }
}

fn bm_vector_gen(state: &mut pew::State<Vec<u64>>) {
    let mut vec = state.get_input();
    let n = vec.len() as u64;
    for _ in 0..n {
        pew::do_not_optimize(vec.pop());
    }
}

pew_main!(
    bm_vector_range -> RANGE(1<<10, 1 << 20, 4),
    bm_vector_gen -> GENRANGE(get_vec, 1<<10, 1<<20, 4)
);
