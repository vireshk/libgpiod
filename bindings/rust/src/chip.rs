// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Duration;
use std::{slice, str};

use vmm_sys_util::errno::Error as IoError;

use super::{
    bindings, chip_info::ChipInfo, Error, InfoEvent, LineConfig, LineInfo, LineRequest,
    RequestConfig, Result,
};

/// GPIO chip
///
/// A GPIO chip object is associated with an open file descriptor to the GPIO
/// character device. It exposes basic information about the chip and allows
/// callers to retrieve information about each line, watch lines for state
/// changes and make line requests.
#[derive(Debug)]
pub(crate) struct ChipInternal {
    chip: *mut bindings::gpiod_chip,
}

impl ChipInternal {
    /// Find a chip by path.
    pub(crate) fn open(path: &str) -> Result<Self> {
        // Null-terminate the string
        let path = path.to_owned() + "\0";

        let chip = unsafe { bindings::gpiod_chip_open(path.as_ptr() as *const c_char) };
        if chip.is_null() {
            return Err(Error::OperationFailed("Gpio Chip open", IoError::last()));
        }

        Ok(Self { chip })
    }

    /// Private helper, Returns gpiod_chip
    pub(crate) fn chip(&self) -> *mut bindings::gpiod_chip {
        self.chip
    }
}

impl Drop for ChipInternal {
    /// Close the chip and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_chip_close(self.chip) }
    }
}

#[derive(Debug)]
pub struct Chip {
    ichip: Arc<ChipInternal>,
    info: ChipInfo,
}

unsafe impl Send for Chip {}
unsafe impl Sync for Chip {}

impl Chip {
    /// Find a chip by path.
    pub fn open(path: &str) -> Result<Self> {
        let ichip = Arc::new(ChipInternal::open(path)?);
        let info = ChipInfo::new(ichip.clone())?;

        Ok(Self { ichip, info })
    }

    /// Get the chip name as represented in the kernel.
    pub fn get_name(&self) -> Result<&str> {
        self.info.name()
    }

    /// Get the chip label as represented in the kernel.
    pub fn get_label(&self) -> Result<&str> {
        self.info.label()
    }

    /// Get the number of GPIO lines exposed by the chip.
    pub fn get_num_lines(&self) -> u32 {
        self.info.num_lines()
    }

    /// Get the path used to find the chip.
    pub fn get_path(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct Chip`.
        let path = unsafe { bindings::gpiod_chip_get_path(self.ichip.chip()) };

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(path as *const u8, bindings::strlen(path) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Get information about the chip.
    pub fn info(&self) -> Result<ChipInfo> {
        ChipInfo::new(self.ichip.clone())
    }

    /// Get a snapshot of information about the line.
    pub fn line_info(&self, offset: u32) -> Result<LineInfo> {
        LineInfo::new(self.ichip.clone(), offset, false)
    }

    /// Get the current snapshot of information about the line at given offset
    /// and optionally start watching it for future changes.
    pub fn watch_line_info(&self, offset: u32) -> Result<LineInfo> {
        LineInfo::new(self.ichip.clone(), offset, true)
    }

    /// Get the file descriptor associated with the chip.
    ///
    /// The returned file descriptor must not be closed by the caller, else other methods for the
    /// `struct Chip` may fail.
    pub fn get_fd(&self) -> Result<u32> {
        let fd = unsafe { bindings::gpiod_chip_get_fd(self.ichip.chip()) };

        if fd < 0 {
            Err(Error::OperationFailed("Gpio Chip get-fd", IoError::last()))
        } else {
            Ok(fd as u32)
        }
    }

    /// Wait for line status events on any of the watched lines on the chip.
    pub fn wait_info_event(&self, timeout: Duration) -> Result<()> {
        let ret = unsafe {
            bindings::gpiod_chip_wait_info_event(self.ichip.chip(), timeout.as_nanos() as i64)
        };

        match ret {
            -1 => Err(Error::OperationFailed(
                "Gpio Chip info-event-wait",
                IoError::last(),
            )),
            0 => Err(Error::OperationTimedOut),
            _ => Ok(()),
        }
    }

    /// Read a single line status change event from the chip. If no events are
    /// pending, this function will block.
    pub fn read_info_event(&self) -> Result<InfoEvent> {
        InfoEvent::new(&self.ichip)
    }

    /// Map a GPIO line's name to its offset within the chip.
    pub fn find_line(&self, name: &str) -> Result<u32> {
        // Null-terminate the string
        let name = name.to_owned() + "\0";

        let ret = unsafe {
            bindings::gpiod_chip_get_line_offset_from_name(
                self.ichip.chip(),
                name.as_ptr() as *const c_char,
            )
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "Gpio Chip find-line",
                IoError::last(),
            ))
        } else {
            Ok(ret as u32)
        }
    }

    /// Request a set of lines for exclusive usage.
    pub fn request_lines(
        &self,
        rconfig: &RequestConfig,
        lconfig: &LineConfig,
    ) -> Result<LineRequest> {
        LineRequest::new(&self.ichip, rconfig, lconfig)
    }
}
