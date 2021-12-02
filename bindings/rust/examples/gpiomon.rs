// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>
//
// Simplified Rust implementation of the gpiomon tool.

use std::env;
use std::time::Duration;

use libgpiod::{Chip, Edge, EdgeEventBuffer, Error, LineConfig, LineEdgeEvent, RequestConfig};

fn usage(name: &str) {
    println!("Usage: {} <chip> <offset0> ...", name);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        usage(&args[0]);
        return;
    }

    let mut config = LineConfig::new().unwrap();
    let mut offsets = Vec::<u32>::new();

    for arg in &args[2..] {
        let offset = arg.parse::<u32>().unwrap();

        offsets.push(offset);
    }

    config.set_edge_detection_default(Edge::Both);

    let path = format!("/dev/gpiochip{}", args[1]);
    let chip = Chip::open(&path).unwrap();

    let rconfig = RequestConfig::new().unwrap();
    rconfig.set_offsets(&offsets);

    let buffer = EdgeEventBuffer::new(1).unwrap();
    let request = chip.request_lines(&rconfig, &config).unwrap();

    loop {
        match request.wait_edge_event(Duration::new(1, 0)) {
            Err(Error::OperationTimedOut) => continue,
            Err(x) => {
                println!("{:?}", x);
                return;
            }
            Ok(()) => (),
        }

        let count = request.read_edge_event(&buffer, 1).unwrap();
        if count == 1 {
            let event = buffer.get_event(0).unwrap();
            println!(
                "line: {} type: {}, time: {:?}",
                event.get_line_offset(),
                match event.get_event_type().unwrap() {
                    LineEdgeEvent::Rising => "Rising",
                    LineEdgeEvent::Falling => "Falling",
                },
                event.get_timestamp()
            );
        }
    }
}
