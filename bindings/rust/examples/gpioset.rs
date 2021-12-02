// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of the gpioset tool.

use std::env;

use libgpiod::{Chip, Direction, LineConfig, RequestConfig};

fn usage(name: &str) {
    println!("Usage: {} <chip> <line_offset0>=<value0> ...", name);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage(&args[0]);
        return;
    }

    let mut config = LineConfig::new().unwrap();
    let mut offsets = Vec::<u32>::new();
    let mut values = Vec::<i32>::new();

    for arg in &args[2..] {
        let pair: Vec<&str> = arg.split('=').collect();
        if pair.len() != 2 {
            usage(&args[0]);
            return;
        }

        let offset = pair[0].parse::<u32>().unwrap();
        let value = pair[1].parse::<u32>().unwrap();

        offsets.push(offset);
        values.push(value as i32);
    }

    config.set_direction_default(Direction::Output);
    config.set_output_values(&offsets, &values).unwrap();

    let path = format!("/dev/gpiochip{}", args[1]);
    let chip = Chip::open(&path).unwrap();

    let rconfig = RequestConfig::new().unwrap();
    rconfig.set_consumer(&args[0]);
    rconfig.set_offsets(&offsets);

    chip.request_lines(&rconfig, &config).unwrap();
}
