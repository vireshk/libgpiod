// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

mod common;

mod line_config {
    use std::time::Duration;

    use libgpiod::{Bias, Direction, Drive, Edge, EventClock, LineConfig};

    mod default {
        use super::*;

        #[test]
        fn verify() {
            let lconfig = LineConfig::new().unwrap();

            assert_eq!(lconfig.get_direction_default().unwrap(), Direction::AsIs);
            assert_eq!(lconfig.get_edge_detection_default().unwrap(), Edge::None);
            assert_eq!(lconfig.get_bias_default().unwrap(), Bias::AsIs);
            assert_eq!(lconfig.get_drive_default().unwrap(), Drive::PushPull);
            assert_eq!(lconfig.get_active_low_default(), false);
            assert_eq!(
                lconfig.get_debounce_period_default().unwrap(),
                Duration::from_millis(0)
            );
            assert_eq!(
                lconfig.get_event_clock_default().unwrap(),
                EventClock::Monotonic
            );
            assert_eq!(lconfig.get_output_value_default().unwrap(), 0);
            assert_eq!(lconfig.get_overrides().unwrap().len(), 0);
        }
    }

    mod overrides {
        use super::*;

        #[test]
        fn direction() {
            const GPIO: u32 = 0;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_direction_default(Direction::AsIs);
            lconfig.set_direction_override(Direction::Input, GPIO);

            assert_eq!(lconfig.direction_is_overridden(GPIO), true);
            assert_eq!(
                lconfig.get_direction_offset(GPIO).unwrap(),
                Direction::Input
            );

            lconfig.clear_direction_override(GPIO);
            assert_eq!(lconfig.direction_is_overridden(GPIO), false);
            assert_eq!(lconfig.get_direction_offset(GPIO).unwrap(), Direction::AsIs);
        }

        #[test]
        fn edge_detection() {
            const GPIO: u32 = 1;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_edge_detection_default(Edge::None);
            lconfig.set_edge_detection_override(Edge::Both, GPIO);

            assert_eq!(lconfig.edge_detection_is_overridden(GPIO), true);
            assert_eq!(lconfig.get_edge_detection_offset(GPIO).unwrap(), Edge::Both);

            lconfig.clear_edge_detection_override(GPIO);
            assert_eq!(lconfig.edge_detection_is_overridden(GPIO), false);
            assert_eq!(lconfig.get_edge_detection_offset(GPIO).unwrap(), Edge::None);
        }

        #[test]
        fn bias() {
            const GPIO: u32 = 2;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_bias_default(Bias::AsIs);
            lconfig.set_bias_override(Bias::PullDown, GPIO);

            assert_eq!(lconfig.bias_is_overridden(GPIO), true);
            assert_eq!(lconfig.get_bias_offset(GPIO).unwrap(), Bias::PullDown);

            lconfig.clear_bias_override(GPIO);
            assert_eq!(lconfig.bias_is_overridden(GPIO), false);
            assert_eq!(lconfig.get_bias_offset(GPIO).unwrap(), Bias::AsIs);
        }

        #[test]
        fn drive() {
            const GPIO: u32 = 3;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_drive_default(Drive::PushPull);
            lconfig.set_drive_override(Drive::OpenDrain, GPIO);

            assert_eq!(lconfig.drive_is_overridden(GPIO), true);
            assert_eq!(lconfig.get_drive_offset(GPIO).unwrap(), Drive::OpenDrain);

            lconfig.clear_drive_override(GPIO);
            assert_eq!(lconfig.drive_is_overridden(GPIO), false);
            assert_eq!(lconfig.get_drive_offset(GPIO).unwrap(), Drive::PushPull);
        }

        #[test]
        fn active_low() {
            const GPIO: u32 = 4;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_active_low_default(false);
            lconfig.set_active_low_override(true, GPIO);

            assert_eq!(lconfig.active_low_is_overridden(GPIO), true);
            assert_eq!(lconfig.get_active_low_offset(GPIO), true);

            lconfig.clear_active_low_override(GPIO);
            assert_eq!(lconfig.active_low_is_overridden(GPIO), false);
            assert_eq!(lconfig.get_active_low_offset(GPIO), false);
        }

        #[test]
        fn debounce_period() {
            const GPIO: u32 = 5;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_debounce_period_default(Duration::from_millis(5));
            lconfig.set_debounce_period_override(Duration::from_millis(3), GPIO);

            assert_eq!(lconfig.debounce_period_is_overridden(GPIO), true);
            assert_eq!(
                lconfig.get_debounce_period_offset(GPIO).unwrap(),
                Duration::from_millis(3)
            );

            lconfig.clear_debounce_period_override(GPIO);
            assert_eq!(lconfig.debounce_period_is_overridden(GPIO), false);
            assert_eq!(
                lconfig.get_debounce_period_offset(GPIO).unwrap(),
                Duration::from_millis(5)
            );
        }

        #[test]
        fn event_clock() {
            const GPIO: u32 = 6;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_event_clock_default(EventClock::Monotonic);
            lconfig.set_event_clock_override(EventClock::Realtime, GPIO);

            assert_eq!(lconfig.event_clock_is_overridden(GPIO), true);
            assert_eq!(
                lconfig.get_event_clock_offset(GPIO).unwrap(),
                EventClock::Realtime
            );

            lconfig.clear_event_clock_override(GPIO);
            assert_eq!(lconfig.event_clock_is_overridden(GPIO), false);
            assert_eq!(
                lconfig.get_event_clock_offset(GPIO).unwrap(),
                EventClock::Monotonic
            );
        }

        #[test]
        fn output_value() {
            const GPIO: u32 = 0;
            let mut lconfig = LineConfig::new().unwrap();

            lconfig.set_output_value_default(0);
            lconfig.set_output_value_override(1, GPIO);
            lconfig.set_output_values(&[1, 2, 8], &[1, 1, 1]).unwrap();

            for line in [0, 1, 2, 8] {
                assert_eq!(lconfig.output_value_is_overridden(line), true);
                assert_eq!(lconfig.get_output_value_offset(line).unwrap(), 1);

                lconfig.clear_output_value_override(line);
                assert_eq!(lconfig.output_value_is_overridden(line), false);
                assert_eq!(lconfig.get_output_value_offset(line).unwrap(), 0);
            }
        }
    }
}
