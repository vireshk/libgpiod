// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod request_config {
    use vmm_sys_util::errno::Error as IoError;

    use libgpiod::{Error as ChipError, RequestConfig};

    mod verify {
        use super::*;

        #[test]
        fn default() {
            let rconfig = RequestConfig::new().unwrap();

            assert_eq!(rconfig.get_offsets().len(), 0);
            assert_eq!(rconfig.get_event_buffer_size(), 0);
            assert_eq!(
                rconfig.get_consumer().unwrap_err(),
                ChipError::OperationFailed("Gpio RequestConfig get-consumer", IoError::new(0))
            );
        }

        #[test]
        fn initialized() {
            let offsets = [0, 1, 2, 3];
            const CONSUMER: &str = "foobar";
            let rconfig = RequestConfig::new().unwrap();
            rconfig.set_consumer(CONSUMER);
            rconfig.set_offsets(&offsets);
            rconfig.set_event_buffer_size(64);

            assert_eq!(rconfig.get_offsets(), offsets);
            assert_eq!(rconfig.get_event_buffer_size(), 64);
            assert_eq!(rconfig.get_consumer().unwrap(), CONSUMER);
        }
    }
}
