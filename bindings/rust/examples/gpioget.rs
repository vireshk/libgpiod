// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of gpioget tool.

use std::env;

use libgpiod::{Chip, Direction, LineConfig, RequestConfig};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <chip> <line_offset0> ...", args[0]);
        return;
    }

    let mut config = LineConfig::new().unwrap();
    let mut offsets = Vec::<u32>::new();

    for arg in &args[2..] {
        let offset = arg.parse::<u32>().unwrap();

        offsets.push(offset);
        config.set_direction_override(Direction::Input, offset);
    }

    let path = format!("/dev/gpiochip{}", args[1]);
    let chip = Chip::open(&path).unwrap();

    let rconfig = RequestConfig::new().unwrap();
    rconfig.set_consumer(&args[0]);
    rconfig.set_offsets(&offsets);

    let request = chip.request_lines(&rconfig, &config).unwrap();

    let mut values: Vec<i32> = vec![0; offsets.len()];
    request.get_values(&mut values).unwrap();

    println!("{:?}", values);
}
