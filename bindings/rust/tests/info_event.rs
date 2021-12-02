// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod info_event {
    use libc::EINVAL;
    use std::sync::Arc;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    use vmm_sys_util::errno::Error as IoError;

    use crate::common::*;
    use libgpiod::{Chip, Direction, Error as ChipError, Event, LineConfig, RequestConfig};

    fn request_reconfigure_line(chip: Arc<Chip>) {
        spawn(move || {
            sleep(Duration::from_millis(10));

            let lconfig1 = LineConfig::new().unwrap();
            let rconfig = RequestConfig::new().unwrap();
            rconfig.set_offsets(&[7]);

            let request = chip.request_lines(&rconfig, &lconfig1).unwrap();

            sleep(Duration::from_millis(10));

            let mut lconfig2 = LineConfig::new().unwrap();
            lconfig2.set_direction_default(Direction::Output);

            request.reconfigure_lines(&lconfig2).unwrap();

            sleep(Duration::from_millis(10));
        });
    }

    mod watch {
        use super::*;
        const NGPIO: u64 = 8;
        const GPIO: u32 = 7;

        #[test]
        fn failure() {
            let sim = Sim::new(Some(NGPIO), None, true).unwrap();
            let chip = Chip::open(sim.dev_path()).unwrap();

            assert_eq!(
                chip.watch_line_info(NGPIO as u32).unwrap_err(),
                ChipError::OperationFailed("Gpio LineInfo line-info", IoError::new(EINVAL))
            );

            chip.watch_line_info(3).unwrap();

            // No events available
            assert_eq!(
                chip.wait_info_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );
        }

        #[test]
        fn verify() {
            let sim = Sim::new(Some(NGPIO), None, true).unwrap();
            let chip = Chip::open(sim.dev_path()).unwrap();
            let info = chip.watch_line_info(GPIO).unwrap();

            assert_eq!(info.get_offset(), GPIO);
        }

        #[test]
        fn reconfigure() {
            let sim = Sim::new(Some(NGPIO), None, true).unwrap();
            let chip = Arc::new(Chip::open(sim.dev_path()).unwrap());
            let info = chip.watch_line_info(GPIO).unwrap();

            assert_eq!(info.get_direction().unwrap(), Direction::Input);

            // Generate events
            request_reconfigure_line(chip.clone());

            // Line requested event
            chip.wait_info_event(Duration::from_secs(1)).unwrap();
            let event = chip.read_info_event().unwrap();
            let ts_req = event.get_timestamp();

            assert_eq!(event.get_event_type().unwrap(), Event::LineRequested);
            assert_eq!(
                event.line_info().unwrap().get_direction().unwrap(),
                Direction::Input
            );

            // Line changed event
            chip.wait_info_event(Duration::from_secs(1)).unwrap();
            let event = chip.read_info_event().unwrap();
            let ts_rec = event.get_timestamp();

            assert_eq!(event.get_event_type().unwrap(), Event::LineConfigChanged);
            assert_eq!(
                event.line_info().unwrap().get_direction().unwrap(),
                Direction::Output
            );

            // Line released event
            chip.wait_info_event(Duration::from_secs(1)).unwrap();
            let event = chip.read_info_event().unwrap();
            let ts_rel = event.get_timestamp();

            assert_eq!(event.get_event_type().unwrap(), Event::LineReleased);

            // No events available
            assert_eq!(
                chip.wait_info_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );

            // Check timestamps are really monotonic.
            assert!(ts_rel > ts_rec);
            assert!(ts_rec > ts_req);
        }
    }
}
