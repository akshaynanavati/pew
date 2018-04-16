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

/// Transposes the output of a benchmark.
///
/// This assumes that you are running multiple benchmarks (`RANGE or
/// `GENRANGE`) with the same range. It will then transform the default
/// output:
///
/// ```
/// Name,Time (ns)
/// bm_vector_range/1024,102541
/// bm_vector_range/4096,423289
/// bm_vector_gen/1024,102316
/// bm_vector_gen/4096,416523
/// ```
///
/// into:
///
/// ```
/// Size,bm_vector_range,bm_vector_gen
/// 1024,105974,106845
/// 4096,418835,409143
/// ```
use std::collections::BTreeMap;
use std::io::{self, BufRead};
use std::vec::Vec;

fn parse_line(line: &str) -> Option<(&str, &str, &str)> {
    let split: Vec<&str> = line.split(',').collect();
    if split.len() != 2 {
        return None;
    }
    let name = split[0];
    let time = split[1];

    let split: Vec<&str> = name.split('/').collect();
    if split.len() != 2 {
        return None;
    }
    let name = split[0];
    let size = split[1];
    return Some((name, size, time));
}

fn main() {
    let stdin = io::stdin();
    let mut results: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut names = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if let Some((name, size, time)) = parse_line(&line) {
            let name_str = name.to_string();
            if !names.contains(&name_str) {
                names.push(name_str);
            }

            let size = size.to_string().parse::<usize>().unwrap();
            if let None = results.get(&size) {
                results.insert(size, vec![time.to_string()]);
            } else {
                let vec = results.get_mut(&size).unwrap();
                vec.push(time.to_string());
            }
        }
    }

    // Header
    println!("{},{}", "Size", names.join(","));
    for (size, times) in results {
        println!("{},{}", size, times.join(","));
    }
}
