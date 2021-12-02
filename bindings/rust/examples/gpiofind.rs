// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of gpiofind tool.

use std::env;
use std::fs;
use std::path::Path;

use libgpiod::{gpiod_is_gpiochip_device, Chip};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <line-name>", args[0]);
        return;
    }

    for entry in fs::read_dir(Path::new("/dev")).unwrap() {
        let pathbuf = entry.unwrap().path();
        let path = pathbuf.to_str().unwrap();

        if gpiod_is_gpiochip_device(path) {
            let chip = Chip::open(path).unwrap();

            let offset = chip.find_line(&args[1]);
            if offset.is_ok() {
                println!(
                    "Line {} found: Chip: {}, offset: {}",
                    args[1],
                    chip.get_name().unwrap(),
                    offset.unwrap()
                );
                return;
            }
        }
    }

    println!("Failed to find line: {}", args[1]);
}
