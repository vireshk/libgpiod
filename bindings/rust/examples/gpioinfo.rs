// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of gpioinfo tool.

use std::env;
use std::fs;
use std::path::Path;

use libgpiod::{gpiod_is_gpiochip_device, Chip, Direction};

fn line_info(chip: &Chip, offset: u32) {
    let info = chip.line_info(offset).unwrap();
    let off = info.get_offset();

    let name = match info.get_name() {
        Ok(name) => name,
        _ => "unused",
    };

    let consumer = match info.get_consumer() {
        Ok(name) => name,
        _ => "unnamed",
    };

    let low = if info.is_active_low() {
        "active-low"
    } else {
        "active-high"
    };

    let dir = match info.get_direction().unwrap() {
        Direction::AsIs => "None",
        Direction::Input => "Input",
        Direction::Output => "Output",
    };

    println!(
        "\tline {:>3}\
              \t{:>10}\
              \t{:>10}\
              \t{:>6}\
              \t{:>14}",
        off, name, consumer, dir, low
    );
}

fn chip_info(path: &str) {
    if gpiod_is_gpiochip_device(path) {
        let chip = Chip::open(path).unwrap();
        let ngpio = chip.get_num_lines();

        println!("GPIO Chip name: {}", chip.get_name().unwrap());
        println!("\tlabel: {}", chip.get_label().unwrap());
        println!("\tpath: {}", chip.get_path().unwrap());
        println!("\tngpio: {}\n", ngpio);

        println!("\tLine information:");

        for offset in 0..ngpio {
            line_info(&chip, offset);
        }
        println!("\n");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: {}", args[0]);
        return;
    }

    if args.len() == 1 {
        for entry in fs::read_dir(Path::new("/dev")).unwrap() {
            let pathbuf = entry.unwrap().path();
            let path = pathbuf.to_str().unwrap();

            chip_info(path);
        }
    } else {
        let index = args[1].parse::<u32>().unwrap();
        let path = format!("/dev/gpiochip{}", index);

        chip_info(&path);
    }
}
