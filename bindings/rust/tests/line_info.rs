// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod line_info {
    use libc::EINVAL;
    use std::time::Duration;

    use vmm_sys_util::errno::Error as IoError;

    use crate::common::*;
    use libgpiod::{Bias, Chip, Direction, Drive, Edge, Error as ChipError, EventClock};
    use libgpiod_sys::{GPIOSIM_HOG_DIR_OUTPUT_HIGH, GPIOSIM_HOG_DIR_OUTPUT_LOW};

    const NGPIO: u64 = 8;

    mod basic {
        use super::*;

        #[test]
        fn verify() {
            const GPIO: u32 = 0;
            const LABEL: &str = "foobar";
            let sim = Sim::new(Some(NGPIO), None, false).unwrap();
            sim.set_line_name(GPIO, LABEL).unwrap();
            sim.hog_line(GPIO, "hog", GPIOSIM_HOG_DIR_OUTPUT_HIGH as i32)
                .unwrap();
            sim.enable().unwrap();

            let chip = Chip::open(sim.dev_path()).unwrap();
            let info = chip.line_info(GPIO).unwrap();

            assert_eq!(info.get_offset(), GPIO);
            assert_eq!(info.get_name().unwrap(), LABEL);
            assert_eq!(info.is_used(), true);
            assert_eq!(info.get_consumer().unwrap(), "hog");
            assert_eq!(info.get_direction().unwrap(), Direction::Output);
            assert_eq!(info.is_active_low(), false);
            assert_eq!(info.get_bias().unwrap(), Bias::Unknown);
            assert_eq!(info.get_drive().unwrap(), Drive::PushPull);
            assert_eq!(info.get_edge_detection().unwrap(), Edge::None);
            assert_eq!(info.get_event_clock().unwrap(), EventClock::Monotonic);
            assert_eq!(info.is_debounced(), false);
            assert_eq!(info.get_debounce_period(), Duration::from_millis(0));

            assert_eq!(
                chip.line_info(NGPIO as u32).unwrap_err(),
                ChipError::OperationFailed("Gpio LineInfo line-info", IoError::new(EINVAL))
            );
        }
    }

    mod properties {
        use super::*;

        #[test]
        fn verify() {
            let sim = Sim::new(Some(NGPIO), None, false).unwrap();
            sim.set_line_name(1, "one").unwrap();
            sim.set_line_name(2, "two").unwrap();
            sim.set_line_name(4, "four").unwrap();
            sim.set_line_name(5, "five").unwrap();
            sim.hog_line(3, "hog3", GPIOSIM_HOG_DIR_OUTPUT_HIGH as i32)
                .unwrap();
            sim.hog_line(4, "hog4", GPIOSIM_HOG_DIR_OUTPUT_LOW as i32)
                .unwrap();
            sim.enable().unwrap();

            let chip = Chip::open(sim.dev_path()).unwrap();
            chip.line_info(6).unwrap();

            let info4 = chip.line_info(4).unwrap();
            assert_eq!(info4.get_offset(), 4);
            assert_eq!(info4.get_name().unwrap(), "four");
            assert_eq!(info4.is_used(), true);
            assert_eq!(info4.get_consumer().unwrap(), "hog4");
            assert_eq!(info4.get_direction().unwrap(), Direction::Output);
            assert_eq!(info4.is_active_low(), false);
            assert_eq!(info4.get_bias().unwrap(), Bias::Unknown);
            assert_eq!(info4.get_drive().unwrap(), Drive::PushPull);
            assert_eq!(info4.get_edge_detection().unwrap(), Edge::None);
            assert_eq!(info4.get_event_clock().unwrap(), EventClock::Monotonic);
            assert_eq!(info4.is_debounced(), false);
            assert_eq!(info4.get_debounce_period(), Duration::from_millis(0));
        }
    }
}
