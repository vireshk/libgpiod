// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, ChipInternal, Error, Event, LineInfo, Result};

/// Line status watch events
///
/// Accessors for the info event objects allowing to monitor changes in GPIO
/// line state.
///
/// Callers can be notified about changes in line's state using the interfaces
/// exposed by GPIO chips. Each info event contains information about the event
/// itself (timestamp, type) as well as a snapshot of line's state in the form
/// of a line-info object.

pub struct InfoEvent {
    event: *mut bindings::gpiod_info_event,
}

impl InfoEvent {
    /// Get a single chip's line's status change event.
    pub(crate) fn new(ichip: &Arc<ChipInternal>) -> Result<Self> {
        let event = unsafe { bindings::gpiod_chip_read_info_event(ichip.chip()) };
        if event.is_null() {
            return Err(Error::OperationFailed(
                "Gpio InfoEvent event-read",
                IoError::last(),
            ));
        }

        Ok(Self { event })
    }

    /// Private helper, Returns gpiod_info_event
    pub(crate) fn event(&self) -> *mut bindings::gpiod_info_event {
        self.event
    }

    /// Get the event type of the status change event.
    pub fn get_event_type(&self) -> Result<Event> {
        Event::new(unsafe { bindings::gpiod_info_event_get_event_type(self.event) } as u32)
    }

    /// Get the timestamp of the event, read from the monotonic clock.
    pub fn get_timestamp(&self) -> Duration {
        Duration::from_nanos(unsafe { bindings::gpiod_info_event_get_timestamp_ns(self.event) })
    }

    /// Get the line-info object associated with the event.
    pub fn line_info(&self) -> Result<LineInfo> {
        LineInfo::try_from(self)
    }
}

impl Drop for InfoEvent {
    /// Free the info event object and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_info_event_free(self.event) }
    }
}
