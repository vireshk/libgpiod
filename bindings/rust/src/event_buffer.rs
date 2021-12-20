// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::os::raw::c_ulong;
use std::sync::Arc;

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, EdgeEvent, Error, Result};

/// Line edge events buffer
pub(crate) struct EdgeEventBufferInternal {
    buffer: *mut bindings::gpiod_edge_event_buffer,
}

impl EdgeEventBufferInternal {
    /// Create a new edge event buffer.
    ///
    /// If capacity equals 0, it will be set to a default value of 64. If
    /// capacity is larger than 1024, it will be limited to 1024.
    pub fn new(capacity: u32) -> Result<Self> {
        let buffer = unsafe { bindings::gpiod_edge_event_buffer_new(capacity as c_ulong) };
        if buffer.is_null() {
            return Err(Error::OperationFailed(
                "Gpio EdgeEventBuffer new",
                IoError::last(),
            ));
        }

        Ok(Self { buffer })
    }

    /// Private helper, Returns gpiod_edge_event_buffer
    pub(crate) fn buffer(&self) -> *mut bindings::gpiod_edge_event_buffer {
        self.buffer
    }
}

impl Drop for EdgeEventBufferInternal {
    /// Free the edge event buffer and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_edge_event_buffer_free(self.buffer) };
    }
}

/// Line edge events buffer
pub struct EdgeEventBuffer {
    ibuffer: Arc<EdgeEventBufferInternal>,
}

impl EdgeEventBuffer {
    /// Create a new edge event buffer.
    ///
    /// If capacity equals 0, it will be set to a default value of 64. If
    /// capacity is larger than 1024, it will be limited to 1024.
    pub fn new(capacity: u32) -> Result<Self> {
        Ok(Self {
            ibuffer: Arc::new(EdgeEventBufferInternal::new(capacity)?),
        })
    }

    /// Private helper, Returns gpiod_edge_event_buffer
    pub(crate) fn buffer(&self) -> *mut bindings::gpiod_edge_event_buffer {
        self.ibuffer.buffer()
    }

    /// Get the capacity of the event buffer.
    pub fn get_capacity(&self) -> u32 {
        unsafe { bindings::gpiod_edge_event_buffer_get_capacity(self.buffer()) as u32 }
    }

    /// Read an event stored in the buffer.
    pub fn get_event(&self, index: u64) -> Result<EdgeEvent> {
        EdgeEvent::new(&self.ibuffer, index, false)
    }

    /// Make copy of an edge event stored in the buffer.
    pub fn get_event_copy(&self, index: u64) -> Result<EdgeEvent> {
        EdgeEvent::new(&self.ibuffer, index, true)
    }

    /// Get the number of events the buffers stores.
    pub fn get_num_events(&self) -> u32 {
        unsafe { bindings::gpiod_edge_event_buffer_get_num_events(self.buffer()) as u32 }
    }
}
