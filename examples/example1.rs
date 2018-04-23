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
use std::vec::Vec;

#[macro_use]
extern crate pew;

use pew::Benchmark;

fn get_vec(n: u64) -> Vec<u64> {
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i as u64);
    }
    return vec;
}

fn bm_vector_range(state: &mut pew::State<u64>) {
    let input = state.get_input();
    state.pause();
    let mut vec = get_vec(input);
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

fn main() {
    Benchmark::with_name("range_bench")
        .with_range(1 << 10, 1 << 20, 4)
        .with_bench(pew_bench!(bm_vector_range))
        .run();

    Benchmark::with_name("gen_bench")
        .with_range(1 << 10, 1 << 20, 4)
        .with_generator(get_vec)
        .with_bench(pew_bench!(bm_vector_gen))
        .run();
}
