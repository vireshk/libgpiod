// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod edge_event {
    use libc::EINVAL;
    use std::sync::Arc;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    use vmm_sys_util::errno::Error as IoError;

    use crate::common::*;
    use libgpiod::{Direction, Edge, EdgeEventBuffer, Error as ChipError, LineEdgeEvent};
    use libgpiod_sys::{GPIOSIM_PULL_DOWN, GPIOSIM_PULL_UP};

    const NGPIO: u64 = 8;

    mod buffer_settings {
        use super::*;

        #[test]
        fn default_capacity() {
            assert_eq!(EdgeEventBuffer::new(0).unwrap().get_capacity(), 64);
        }

        #[test]
        fn user_defined_capacity() {
            assert_eq!(EdgeEventBuffer::new(123).unwrap().get_capacity(), 123);
        }

        #[test]
        fn max_capacity() {
            assert_eq!(EdgeEventBuffer::new(1024 * 2).unwrap().get_capacity(), 1024);
        }
    }

    mod failure {
        use super::*;

        #[test]
        fn wait_timeout() {
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[0]));
            config.lconfig_edge(Some(Edge::Both));
            config.request_lines().unwrap();

            // No events available
            assert_eq!(
                config
                    .request()
                    .wait_edge_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );
        }

        #[test]
        fn dir_out_edge_failure() {
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[0]));
            config.lconfig(Some(Direction::Output), None, None, Some(Edge::Both), None);

            assert_eq!(
                config.request_lines().unwrap_err(),
                ChipError::OperationFailed("Gpio LineRequest request-lines", IoError::new(EINVAL))
            );
        }
    }

    mod verify {
        use super::*;

        // Helpers to generate events
        fn trigger_falling_and_rising_edge(sim: Arc<Sim>, offset: u32) {
            spawn(move || {
                sleep(Duration::from_millis(30));
                sim.set_pull(offset, GPIOSIM_PULL_UP as i32).unwrap();

                sleep(Duration::from_millis(30));
                sim.set_pull(offset, GPIOSIM_PULL_DOWN as i32).unwrap();
            });
        }

        fn trigger_rising_edge_events_on_two_offsets(sim: Arc<Sim>, offset: [u32; 2]) {
            spawn(move || {
                sleep(Duration::from_millis(30));
                sim.set_pull(offset[0], GPIOSIM_PULL_UP as i32).unwrap();

                sleep(Duration::from_millis(30));
                sim.set_pull(offset[1], GPIOSIM_PULL_UP as i32).unwrap();
            });
        }

        fn trigger_multiple_events(sim: Arc<Sim>, offset: u32) {
            sim.set_pull(offset, GPIOSIM_PULL_UP as i32).unwrap();
            sleep(Duration::from_millis(10));

            sim.set_pull(offset, GPIOSIM_PULL_DOWN as i32).unwrap();
            sleep(Duration::from_millis(10));

            sim.set_pull(offset, GPIOSIM_PULL_UP as i32).unwrap();
            sleep(Duration::from_millis(10));
        }

        #[test]
        fn both_edges() {
            const GPIO: u32 = 2;
            let buf = EdgeEventBuffer::new(0).unwrap();
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_edge(Some(Edge::Both));
            config.request_lines().unwrap();

            // Generate events
            trigger_falling_and_rising_edge(config.sim(), GPIO);

            // Rising event
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            let ts_rising = event.get_timestamp();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Rising);
            assert_eq!(event.get_line_offset(), GPIO);

            // Falling event
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            let ts_falling = event.get_timestamp();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Falling);
            assert_eq!(event.get_line_offset(), GPIO);

            // No events available
            assert_eq!(
                config
                    .request()
                    .wait_edge_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );

            assert!(ts_falling > ts_rising);
        }

        #[test]
        fn rising_edge() {
            const GPIO: u32 = 6;
            let buf = EdgeEventBuffer::new(0).unwrap();
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_edge(Some(Edge::Rising));
            config.request_lines().unwrap();

            // Generate events
            trigger_falling_and_rising_edge(config.sim(), GPIO);

            // Rising event
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Rising);
            assert_eq!(event.get_line_offset(), GPIO);

            // No events available
            assert_eq!(
                config
                    .request()
                    .wait_edge_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );
        }

        #[test]
        fn falling_edge() {
            const GPIO: u32 = 7;
            let buf = EdgeEventBuffer::new(0).unwrap();
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_edge(Some(Edge::Falling));
            config.request_lines().unwrap();

            // Generate events
            trigger_falling_and_rising_edge(config.sim(), GPIO);

            // Falling event
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Falling);
            assert_eq!(event.get_line_offset(), GPIO);

            // No events available
            assert_eq!(
                config
                    .request()
                    .wait_edge_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );
        }

        #[test]
        fn edge_sequence() {
            const GPIO: [u32; 2] = [0, 1];
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&GPIO));
            config.lconfig_edge(Some(Edge::Both));
            config.request_lines().unwrap();

            // Generate events
            trigger_rising_edge_events_on_two_offsets(config.sim(), GPIO);

            // Rising event GPIO 0
            let buf = EdgeEventBuffer::new(0).unwrap();
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Rising);
            assert_eq!(event.get_line_offset(), GPIO[0]);
            assert_eq!(event.get_global_seqno(), 1);
            assert_eq!(event.get_line_seqno(), 1);

            // Rising event GPIO 1
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                1
            );
            assert_eq!(buf.get_num_events(), 1);

            let event = buf.get_event(0).unwrap();
            assert_eq!(event.get_event_type().unwrap(), LineEdgeEvent::Rising);
            assert_eq!(event.get_line_offset(), GPIO[1]);
            assert_eq!(event.get_global_seqno(), 2);
            assert_eq!(event.get_line_seqno(), 1);

            // No events available
            assert_eq!(
                config
                    .request()
                    .wait_edge_event(Duration::from_millis(100))
                    .unwrap_err(),
                ChipError::OperationTimedOut
            );
        }

        #[test]
        fn multiple_events() {
            const GPIO: u32 = 1;
            let buf = EdgeEventBuffer::new(0).unwrap();
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_edge(Some(Edge::Both));
            config.request_lines().unwrap();

            // Generate events
            trigger_multiple_events(config.sim(), GPIO);

            // Read multiple events
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                3
            );
            assert_eq!(buf.get_num_events(), 3);

            let mut global_seqno = 1;
            let mut line_seqno = 1;

            // Verify sequence number of events
            for i in 0..3 {
                let event = buf.get_event(i).unwrap();
                assert_eq!(event.get_line_offset(), GPIO);
                assert_eq!(event.get_global_seqno(), global_seqno);
                assert_eq!(event.get_line_seqno(), line_seqno);

                global_seqno += 1;
                line_seqno += 1;
            }
        }

        #[test]
        fn over_capacity() {
            const GPIO: u32 = 2;
            let buf = EdgeEventBuffer::new(2).unwrap();
            let mut config = TestConfig::new(NGPIO).unwrap();
            config.rconfig(Some(&[GPIO]));
            config.lconfig_edge(Some(Edge::Both));
            config.request_lines().unwrap();

            // Generate events
            trigger_multiple_events(config.sim(), GPIO);

            // Read multiple events
            config
                .request()
                .wait_edge_event(Duration::from_secs(1))
                .unwrap();

            assert_eq!(
                config
                    .request()
                    .read_edge_event(&buf, buf.get_capacity())
                    .unwrap(),
                2
            );
            assert_eq!(buf.get_num_events(), 2);
        }
    }
}
