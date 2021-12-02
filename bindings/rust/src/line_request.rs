// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use libc::EINVAL;
use std::os::raw::c_ulong;
use std::sync::Arc;
use std::time::Duration;

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, ChipInternal, EdgeEventBuffer, Error, LineConfig, RequestConfig, Result};

/// Line request operations
///
/// Allows interaction with a set of requested lines.
#[derive(Debug)]
pub struct LineRequest {
    request: *mut bindings::gpiod_line_request,
}

impl LineRequest {
    /// Request a set of lines for exclusive usage.
    pub(crate) fn new(
        ichip: &Arc<ChipInternal>,
        rconfig: &RequestConfig,
        lconfig: &LineConfig,
    ) -> Result<Self> {
        let request = unsafe {
            bindings::gpiod_chip_request_lines(ichip.chip(), rconfig.config(), lconfig.config())
        };

        if request.is_null() {
            return Err(Error::OperationFailed(
                "Gpio LineRequest request-lines",
                IoError::last(),
            ));
        }

        Ok(Self { request })
    }

    /// Get the number of lines in the request.
    pub fn get_num_lines(&self) -> u32 {
        unsafe { bindings::gpiod_line_request_get_num_lines(self.request) as u32 }
    }

    /// Get the offsets of lines in the request.
    pub fn get_offsets(&self) -> Vec<u32> {
        let mut offsets = vec![0; self.get_num_lines() as usize];

        unsafe { bindings::gpiod_line_request_get_offsets(self.request, offsets.as_mut_ptr()) };
        offsets
    }

    /// Get the value (0 or 1) of a single line associated with the request.
    pub fn get_value(&self, offset: u32) -> Result<u32> {
        let value = unsafe { bindings::gpiod_line_request_get_value(self.request, offset) };

        if value != 0 && value != 1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest get-value",
                IoError::last(),
            ))
        } else {
            Ok(value as u32)
        }
    }

    /// Get values of a subset of lines associated with the request.
    pub fn get_values_subset(&self, offsets: &[u32], values: &mut Vec<i32>) -> Result<()> {
        if offsets.len() != values.len() {
            return Err(Error::OperationFailed(
                "Gpio LineRequest array size mismatch",
                IoError::new(EINVAL),
            ));
        }

        let ret = unsafe {
            bindings::gpiod_line_request_get_values_subset(
                self.request,
                offsets.len() as c_ulong,
                offsets.as_ptr(),
                values.as_mut_ptr(),
            )
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest get-values-subset",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get values of all lines associated with the request.
    pub fn get_values(&self, values: &mut Vec<i32>) -> Result<()> {
        if values.len() != self.get_num_lines() as usize {
            return Err(Error::OperationFailed(
                "Gpio LineRequest array size mismatch",
                IoError::new(EINVAL),
            ));
        }

        let ret =
            unsafe { bindings::gpiod_line_request_get_values(self.request, values.as_mut_ptr()) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest get-values",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Set the value of a single line associated with the request.
    pub fn set_value(&self, offset: u32, value: i32) -> Result<()> {
        let ret = unsafe { bindings::gpiod_line_request_set_value(self.request, offset, !!value) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest set-value",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get values of a subset of lines associated with the request.
    pub fn set_values_subset(&self, offsets: &[u32], values: &[i32]) -> Result<()> {
        if offsets.len() != values.len() {
            return Err(Error::OperationFailed(
                "Gpio LineRequest array size mismatch",
                IoError::new(EINVAL),
            ));
        }

        let ret = unsafe {
            bindings::gpiod_line_request_set_values_subset(
                self.request,
                offsets.len() as c_ulong,
                offsets.as_ptr(),
                values.as_ptr(),
            )
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest set-values-subset",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get values of all lines associated with the request.
    pub fn set_values(&self, values: &[i32]) -> Result<()> {
        if values.len() != self.get_num_lines() as usize {
            return Err(Error::OperationFailed(
                "Gpio LineRequest array size mismatch",
                IoError::new(EINVAL),
            ));
        }

        let ret = unsafe { bindings::gpiod_line_request_set_values(self.request, values.as_ptr()) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest set-values",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Update the configuration of lines associated with the line request.
    pub fn reconfigure_lines(&self, lconfig: &LineConfig) -> Result<()> {
        let ret = unsafe {
            bindings::gpiod_line_request_reconfigure_lines(self.request, lconfig.config())
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest reconfigure-lines",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    /// Get the file descriptor associated with the line request.
    pub fn get_fd(&self) -> u32 {
        unsafe { bindings::gpiod_line_request_get_fd(self.request) as u32 }
    }

    /// Wait for edge events on any of the lines associated with the request.
    pub fn wait_edge_event(&self, timeout: Duration) -> Result<()> {
        let ret = unsafe {
            bindings::gpiod_line_request_wait_edge_event(self.request, timeout.as_nanos() as i64)
        };

        match ret {
            -1 => Err(Error::OperationFailed(
                "Gpio LineRequest edge-event-wait",
                IoError::last(),
            )),
            0 => Err(Error::OperationTimedOut),
            _ => Ok(()),
        }
    }

    /// Get a number of edge events from a line request.
    ///
    /// This function will block if no event was queued for the line.
    pub fn read_edge_event(&self, buffer: &EdgeEventBuffer, max_events: u32) -> Result<u32> {
        let ret = unsafe {
            bindings::gpiod_line_request_read_edge_event(
                self.request,
                buffer.buffer(),
                max_events as c_ulong,
            )
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio LineRequest edge-event-read",
                IoError::last(),
            ))
        } else {
            Ok(ret as u32)
        }
    }
}

impl Drop for LineRequest {
    /// Release the requested lines and free all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_line_request_release(self.request) }
    }
}
