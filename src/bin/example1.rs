use std::vec::Vec;

#[macro_use]
extern crate pew;

fn bm_vector_1(state: &mut pew::State) {
    state.pause();
    let input = state.get_input();
    let mut vec = Vec::new();
    for i in 0..input {
        vec.push(i);
    }
    state.resume();
    for i in 0..input {
        assert_eq!(vec.pop(), Some(input - i - 1));
    }
}

fn bm_vector_2(state: &mut pew::State) {
    bm_vector_1(state);
}

pew_main!(
    bm_vector_1 -> (16, 1024, 2),
    bm_vector_2 -> (16, 1024, 2)
);
