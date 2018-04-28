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

//! Transposes the output of a benchmark.
//!
//! This assumes that you are running multiple benchmarks (`RANGE or
//! `GENRANGE`) with the same range. It will then transform the default
//! output:
//!
//! ```
//! Name,Time (ns)
//! bm_vector_range/1024,102541
//! bm_vector_range/4096,423289
//! bm_vector_gen/1024,102316
//! bm_vector_gen/4096,416523
//! ```
//!
//! into:
//!
//! ```
//! Size,bm_vector_range,bm_vector_gen
//! 1024,105974,106845
//! 4096,418835,409143
//! ```

#[macro_use]
extern crate lazy_static;
extern crate clap;

use clap::{App, Arg, ArgMatches};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::vec::Vec;

lazy_static! {
    static ref APP_FLAGS: ArgMatches<'static> = App::new("pew-benchmark")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("File to write out to. If ommitted, will write out to stdout")
                .takes_value(true),
        )
        .get_matches();
}

fn parse_line(line: &str) -> Option<(String, &str, &str)> {
    let split: Vec<&str> = line.split(',').collect();
    if split.len() != 2 {
        return None;
    }
    let name = split[0];
    let time = split[1];

    let split: Vec<&str> = name.split('/').collect();
    if split.len() != 3 {
        return None;
    }
    let global_name = split[0];
    let bench_name = split[1];
    let size = split[2];
    return Some((format!("{}/{}", global_name, bench_name), size, time));
}

fn main() {
    let stdin = io::stdin();
    let mut results: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut names = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if let Some(_) = APP_FLAGS.value_of("file") {
            println!("{}", line);
        }

        if let Some((name, size, time)) = parse_line(&line) {
            if !names.contains(&name) {
                names.push(name);
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

    let header = format!("{},{}\n", "Size", names.join(","));
    if let Some(fname) = APP_FLAGS.value_of("file") {
        match File::create(&Path::new(fname)) {
            Ok(mut f) => {
                f.write(header.as_bytes()).expect("File write failed");
                for (size, times) in results {
                    f.write(format!("{},{}\n", size, times.join(",")).as_bytes())
                        .expect("File write failed");
                }
                return;
            }
            Err(why) => {
                eprintln!("ERROR: Could not open {}: {}", fname, why.description());
                eprintln!("Displaying results below:");
            }
        }
    }

    print!("{}", header);
    for (size, times) in results {
        println!("{},{}", size, times.join(","));
    }
}
