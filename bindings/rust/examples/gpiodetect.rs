// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of gpiodetect tool.

use std::env;
use std::fs;
use std::path::Path;

use libgpiod::{gpiod_is_gpiochip_device, Chip};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("Usage: {}", args[0]);
        return;
    }

    for entry in fs::read_dir(Path::new("/dev")).unwrap() {
        let pathbuf = entry.unwrap().path();
        let path = pathbuf.to_str().unwrap();

        if gpiod_is_gpiochip_device(path) {
            let chip = Chip::open(path).unwrap();
            let ngpio = chip.get_num_lines();

            println!(
                "{} [{}] ({})",
                chip.get_name().unwrap(),
                chip.get_label().unwrap(),
                ngpio
            );
        }
    }
}
