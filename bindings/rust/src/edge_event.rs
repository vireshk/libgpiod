// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::sync::Arc;
use std::time::Duration;

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, EdgeEventBufferInternal, Error, LineEdgeEvent, Result};

/// Line edge events handling
///
/// An edge event object contains information about a single line edge event.
/// It contains the event type, timestamp and the offset of the line on which
/// the event occurred as well as two sequence numbers (global for all lines
/// in the associated request and local for this line only).
///
/// Edge events are stored into an edge-event buffer object to improve
/// performance and to limit the number of memory allocations when a large
/// number of events are being read.

pub struct EdgeEvent {
    ibuffer: Option<Arc<EdgeEventBufferInternal>>,
    event: *mut bindings::gpiod_edge_event,
}

impl EdgeEvent {
    /// Get an event stored in the buffer.
    pub(crate) fn new(
        ibuffer: &Arc<EdgeEventBufferInternal>,
        index: u64,
        copy: bool,
    ) -> Result<Self> {
        let event = unsafe { bindings::gpiod_edge_event_buffer_get_event(ibuffer.buffer(), index) };
        if event.is_null() {
            return Err(Error::OperationFailed(
                "Gpio EdgeEvent buffer-get-event",
                IoError::last(),
            ));
        }

        if copy {
            let event = unsafe { bindings::gpiod_edge_event_copy(event) };
            if event.is_null() {
                return Err(Error::OperationFailed(
                    "Gpio EdgeEvent copy",
                    IoError::last(),
                ));
            }

            Ok(Self {
                ibuffer: None,
                event,
            })
        } else {
            Ok(Self {
                ibuffer: Some(ibuffer.clone()),
                event,
            })
        }
    }

    /// Get the event type.
    pub fn get_event_type(&self) -> Result<LineEdgeEvent> {
        LineEdgeEvent::new(unsafe { bindings::gpiod_edge_event_get_event_type(self.event) } as u32)
    }

    /// Get the timestamp of the event.
    pub fn get_timestamp(&self) -> Duration {
        Duration::from_nanos(unsafe { bindings::gpiod_edge_event_get_timestamp_ns(self.event) })
    }

    /// Get the offset of the line on which the event was triggered.
    pub fn get_line_offset(&self) -> u32 {
        unsafe { bindings::gpiod_edge_event_get_line_offset(self.event) }
    }

    /// Get the global sequence number of the event.
    ///
    /// Returns sequence number of the event relative to all lines in the
    /// associated line request.
    pub fn get_global_seqno(&self) -> u64 {
        unsafe { bindings::gpiod_edge_event_get_global_seqno(self.event) }
    }

    /// Get the event sequence number specific to concerned line.
    ///
    /// Returns sequence number of the event relative to the line within the
    /// lifetime of the associated line request.
    pub fn get_line_seqno(&self) -> u64 {
        unsafe { bindings::gpiod_edge_event_get_line_seqno(self.event) }
    }
}

impl Drop for EdgeEvent {
    /// Free the edge event.
    fn drop(&mut self) {
        // Free the event only if a copy is made
        if self.ibuffer.is_none() {
            unsafe { bindings::gpiod_edge_event_free(self.event) };
        }
    }
}
