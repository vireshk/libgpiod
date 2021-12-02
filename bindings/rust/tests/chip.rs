// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod chip {
    use libc::{ENODEV, ENOENT, ENOTTY};

    use vmm_sys_util::errno::Error as IoError;

    use crate::common::*;
    use libgpiod::{Chip, Error as ChipError};

    mod create {
        use super::*;

        #[test]
        fn nonexistent_file_failure() {
            assert_eq!(
                Chip::open("/dev/nonexistent").unwrap_err(),
                ChipError::OperationFailed("Gpio Chip open", IoError::new(ENOENT))
            );
        }

        #[test]
        fn no_dev_file_failure() {
            assert_eq!(
                Chip::open("/tmp").unwrap_err(),
                ChipError::OperationFailed("Gpio Chip open", IoError::new(ENOTTY))
            );
        }

        #[test]
        fn non_gpio_char_dev_file_failure() {
            assert_eq!(
                Chip::open("/dev/null").unwrap_err(),
                ChipError::OperationFailed("Gpio Chip open", IoError::new(ENODEV))
            );
        }

        #[test]
        fn existing() {
            let sim = Sim::new(None, None, true).unwrap();
            Chip::open(sim.dev_path()).unwrap();
        }
    }

    mod configure {
        use super::*;
        const NGPIO: u64 = 16;
        const LABEL: &str = "foobar";

        #[test]
        fn verify() {
            let sim = Sim::new(Some(NGPIO), Some(LABEL), true).unwrap();
            let chip = Chip::open(sim.dev_path()).unwrap();

            assert_eq!(chip.get_label().unwrap(), LABEL);
            assert_eq!(chip.get_name().unwrap(), sim.chip_name());
            assert_eq!(chip.get_path().unwrap(), sim.dev_path());
            assert_eq!(chip.get_num_lines(), NGPIO as u32);
            chip.get_fd().unwrap();
        }

        #[test]
        fn line_lookup() {
            let sim = Sim::new(Some(NGPIO), None, false).unwrap();
            sim.set_line_name(0, "zero").unwrap();
            sim.set_line_name(2, "two").unwrap();
            sim.set_line_name(3, "three").unwrap();
            sim.set_line_name(5, "five").unwrap();
            sim.set_line_name(10, "ten").unwrap();
            sim.set_line_name(11, "ten").unwrap();
            sim.enable().unwrap();

            let chip = Chip::open(sim.dev_path()).unwrap();

            // Success case
            assert_eq!(chip.find_line("zero").unwrap(), 0);
            assert_eq!(chip.find_line("two").unwrap(), 2);
            assert_eq!(chip.find_line("three").unwrap(), 3);
            assert_eq!(chip.find_line("five").unwrap(), 5);

            // Success with duplicate names, should return first entry
            assert_eq!(chip.find_line("ten").unwrap(), 10);

            // Failure
            assert_eq!(
                chip.find_line("nonexistent").unwrap_err(),
                ChipError::OperationFailed("Gpio Chip find-line", IoError::new(ENOENT))
            );
        }
    }
}
