// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod line_request {
    use libc::{EBUSY, EINVAL};

    use vmm_sys_util::errno::Error as IoError;

    use crate::common::*;
    use libgpiod::{Bias, Direction, Error as ChipError, LineConfig};
    use libgpiod_sys::{
        GPIOSIM_PULL_DOWN, GPIOSIM_PULL_UP, GPIOSIM_VALUE_ACTIVE, GPIOSIM_VALUE_INACTIVE,
    };

    const NGPIO: u64 = 8;

    mod invalid_arguments {
        use super::*;

        #[test]
        fn no_offsets() {
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(None);
            config.lconfig_raw();

            assert_eq!(
                config.request_lines().unwrap_err(),
                ChipError::OperationFailed("Gpio LineRequest request-lines", IoError::new(EINVAL))
            );
        }

        #[test]
        fn duplicate_offsets() {
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[2, 0, 0, 4]));
            config.lconfig_raw();

            assert_eq!(
                config.request_lines().unwrap_err(),
                ChipError::OperationFailed("Gpio LineRequest request-lines", IoError::new(EBUSY))
            );
        }

        #[test]
        fn out_of_bound_offsets() {
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[2, 0, 8, 4]));
            config.lconfig_raw();

            assert_eq!(
                config.request_lines().unwrap_err(),
                ChipError::OperationFailed("Gpio LineRequest request-lines", IoError::new(EINVAL))
            );
        }
    }

    mod verify {
        use super::*;

        #[test]
        fn custom_consumer() {
            const GPIO: u32 = 2;
            const CONSUMER: &str = "foobar";
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig_consumer(Some(&[GPIO]), Some(CONSUMER));
            config.lconfig_raw();
            config.request_lines().unwrap();

            let info = config.chip().line_info(GPIO).unwrap();

            assert_eq!(info.is_used(), true);
            assert_eq!(info.get_consumer().unwrap(), CONSUMER);
        }

        #[test]
        fn empty_consumer() {
            const GPIO: u32 = 2;
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_raw();
            config.request_lines().unwrap();

            let info = config.chip().line_info(GPIO).unwrap();

            assert_eq!(info.is_used(), true);
            assert_eq!(info.get_consumer().unwrap(), "?");
        }

        #[test]
        fn read_values() {
            let offsets = [7, 1, 0, 6, 2];
            let pulls = [
                GPIOSIM_PULL_UP,
                GPIOSIM_PULL_UP,
                GPIOSIM_PULL_DOWN,
                GPIOSIM_PULL_UP,
                GPIOSIM_PULL_DOWN,
            ];
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.set_pull(&offsets, &pulls);
            config.rconfig(Some(&offsets));
            config.lconfig(Some(Direction::Input), None, None, None, None);
            config.request_lines().unwrap();

            let request = config.request();

            // Buffer is smaller
            let mut values: Vec<i32> = vec![0; 4];
            assert_eq!(
                request.get_values(&mut values).unwrap_err(),
                ChipError::OperationFailed(
                    "Gpio LineRequest array size mismatch",
                    IoError::new(EINVAL),
                )
            );

            // Buffer is larger
            let mut values: Vec<i32> = vec![0; 6];
            assert_eq!(
                request.get_values(&mut values).unwrap_err(),
                ChipError::OperationFailed(
                    "Gpio LineRequest array size mismatch",
                    IoError::new(EINVAL),
                )
            );

            // Single values read properly
            assert_eq!(request.get_value(7).unwrap(), 1);

            // Values read properly
            let mut values: Vec<i32> = vec![0; 5];
            request.get_values(&mut values).unwrap();
            for i in 0..values.len() {
                assert_eq!(
                    values[i],
                    match pulls[i] {
                        GPIOSIM_PULL_DOWN => 0,
                        _ => 1,
                    }
                );
            }

            // Subset of values read properly
            let mut values: Vec<i32> = vec![0; 3];
            request.get_values_subset(&[2, 0, 6], &mut values).unwrap();
            assert_eq!(values[0], 0);
            assert_eq!(values[1], 0);
            assert_eq!(values[2], 1);

            // Value read properly after reconfigure
            let mut lconfig = LineConfig::new().unwrap();
            lconfig.set_active_low_default(true);
            request.reconfigure_lines(&lconfig).unwrap();
            assert_eq!(request.get_value(7).unwrap(), 0);
        }

        #[test]
        fn set_output_values() {
            let offsets = [0, 1, 3, 4];
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&offsets));
            config.lconfig(Some(Direction::Output), Some(1), Some((4, 0)), None, None);
            config.request_lines().unwrap();

            assert_eq!(config.sim().val(0).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_ACTIVE);

            // Overriden
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_INACTIVE);

            // Default
            assert_eq!(config.sim().val(2).unwrap(), GPIOSIM_VALUE_INACTIVE);
        }

        #[test]
        fn reconfigure_output_values() {
            let offsets = [0, 1, 3, 4];
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&offsets));
            config.lconfig(Some(Direction::Output), Some(0), None, None, None);
            config.request_lines().unwrap();
            let request = config.request();

            // Set single value
            request.set_value(1, 1).unwrap();
            assert_eq!(config.sim().val(0).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_INACTIVE);
            request.set_value(1, 0).unwrap();
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_INACTIVE);

            // Set values of subset
            request.set_values_subset(&[4, 3], &[1, 1]).unwrap();
            assert_eq!(config.sim().val(0).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_ACTIVE);
            request.set_values_subset(&[4, 3], &[0, 0]).unwrap();
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_INACTIVE);

            // Set all values
            request.set_values(&[1, 0, 1, 0]).unwrap();
            assert_eq!(config.sim().val(0).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_ACTIVE);
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_INACTIVE);
            request.set_values(&[0, 0, 0, 0]).unwrap();
            assert_eq!(config.sim().val(0).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(1).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_INACTIVE);
            assert_eq!(config.sim().val(4).unwrap(), GPIOSIM_VALUE_INACTIVE);
        }

        #[test]
        fn set_bias() {
            let offsets = [3];
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&offsets));
            config.lconfig(Some(Direction::Input), None, None, None, Some(Bias::PullUp));
            config.request_lines().unwrap();
            config.request();

            // Set single value
            assert_eq!(config.sim().val(3).unwrap(), GPIOSIM_VALUE_ACTIVE);
        }
    }
}
