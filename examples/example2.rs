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

#[macro_use]
extern crate pew;
extern crate rand;

use pew::Benchmark;
use rand::{thread_rng, Rng};
use std::vec::Vec;

fn get_vec(n: u64) -> Vec<u64> {
    let mut rng = thread_rng();
    let mut vec = Vec::new();
    for _ in 0..n {
        vec.push(rng.gen::<u64>());
    }
    return vec;
}

fn bm_vector_iterate(state: &mut pew::State<Vec<u64>>) {
    let vec = state.get_input();
    let n = vec.len() as u64;
    for i in 0..n {
        pew::do_not_optimize(i);
    }
}

fn bm_vector_delete(state: &mut pew::State<Vec<u64>>) {
    let mut vec = state.get_input();
    let n = vec.len() as u64;
    for _ in 0..n {
        pew::do_not_optimize(vec.pop());
    }
}

fn main() {
    Benchmark::with_name("example2")
        .with_range(1 << 10, 1 << 20, 4)
        .with_generator(get_vec)
        .with_bench(pew_bench!(bm_vector_iterate))
        .with_bench(pew_bench!(bm_vector_delete))
        .run();
}
