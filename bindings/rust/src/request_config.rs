// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::os::raw::{c_char, c_ulong};
use std::{slice, str};

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, Error, Result};

/// Request configuration objects
///
/// Request config objects are used to pass a set of options to the kernel at
/// the time of the line request. Similarly to the line-config - the mutators
/// don't return error values. If the values are invalid, in general they are
/// silently adjusted to acceptable ranges.

pub struct RequestConfig {
    config: *mut bindings::gpiod_request_config,
}

impl RequestConfig {
    /// Create a new request config object.
    pub fn new() -> Result<Self> {
        let config = unsafe { bindings::gpiod_request_config_new() };
        if config.is_null() {
            return Err(Error::OperationFailed(
                "Gpio RequestConfig new",
                IoError::last(),
            ));
        }

        Ok(Self { config })
    }

    /// Private helper, Returns gpiod_request_config
    pub(crate) fn config(&self) -> *mut bindings::gpiod_request_config {
        self.config
    }

    /// Set the consumer name for the request.
    ///
    /// If the consumer string is too long, it will be truncated to the max
    /// accepted length.
    pub fn set_consumer(&self, consumer: &str) {
        // Null-terminate the string
        let consumer = consumer.to_owned() + "\0";

        unsafe {
            bindings::gpiod_request_config_set_consumer(
                self.config,
                consumer.as_ptr() as *const c_char,
            )
        }
    }

    /// Get the consumer name configured in the request config.
    pub fn get_consumer(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct RequestConfig`.
        let consumer = unsafe { bindings::gpiod_request_config_get_consumer(self.config) };
        if consumer.is_null() {
            return Err(Error::OperationFailed(
                "Gpio RequestConfig get-consumer",
                IoError::last(),
            ));
        }

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(consumer as *const u8, bindings::strlen(consumer) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Set the offsets of the lines to be requested.
    ///
    /// If too many offsets were specified, the offsets above the limit accepted
    /// by the kernel (64 lines) are silently dropped.
    pub fn set_offsets(&self, offsets: &[u32]) {
        unsafe {
            bindings::gpiod_request_config_set_offsets(
                self.config,
                offsets.len() as c_ulong,
                offsets.as_ptr(),
            )
        }
    }

    /// Get the offsets of lines in the request config.
    pub fn get_offsets(&self) -> Vec<u32> {
        let num = unsafe { bindings::gpiod_request_config_get_num_offsets(self.config) };
        let mut offsets = vec![0; num as usize];

        unsafe { bindings::gpiod_request_config_get_offsets(self.config, offsets.as_mut_ptr()) };
        offsets
    }

    /// Set the size of the kernel event buffer for the request.
    ///
    /// The kernel may adjust the value if it's too high. If set to 0, the
    /// default value will be used.
    pub fn set_event_buffer_size(&self, size: u32) {
        unsafe {
            bindings::gpiod_request_config_set_event_buffer_size(self.config, size as c_ulong)
        }
    }

    /// Get the edge event buffer size for the request config.
    pub fn get_event_buffer_size(&self) -> u32 {
        unsafe { bindings::gpiod_request_config_get_event_buffer_size(self.config) as u32 }
    }
}

impl Drop for RequestConfig {
    /// Free the request config object and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_request_config_free(self.config) }
    }
}
