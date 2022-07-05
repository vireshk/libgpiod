// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::sync::Arc;
use std::{slice, str};

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, ChipInternal, Error, Result};

/// GPIO chip Information
#[derive(Debug)]
pub struct ChipInfo {
    info: *mut bindings::gpiod_chip_info,
}

impl ChipInfo {
    /// Find a GPIO chip by path.
    pub(crate) fn new(chip: Arc<ChipInternal>) -> Result<Self> {
        let info = unsafe { bindings::gpiod_chip_get_info(chip.chip()) };
        if info.is_null() {
            return Err(Error::OperationFailed(
                "Gpio Chip get info",
                IoError::last(),
            ));
        }

        Ok(Self { info })
    }

    /// Get the GPIO chip name as represented in the kernel.
    pub(crate) fn name(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct Chip`.
        let name = unsafe { bindings::gpiod_chip_info_get_name(self.info) };

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(name as *const u8, bindings::strlen(name) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Get the GPIO chip label as represented in the kernel.
    pub(crate) fn label(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct Chip`.
        let label = unsafe { bindings::gpiod_chip_info_get_label(self.info) };

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(label as *const u8, bindings::strlen(label) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Get the number of GPIO lines exposed by the chip.
    pub(crate) fn num_lines(&self) -> u32 {
        unsafe { bindings::gpiod_chip_info_get_num_lines(self.info) as u32 }
    }
}

impl Drop for ChipInfo {
    /// Close the GPIO chip info and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_chip_info_free(self.info) }
    }
}
