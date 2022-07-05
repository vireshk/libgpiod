// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;
use std::{slice, str};

use vmm_sys_util::errno::Error as IoError;

use super::{
    bindings, Bias, ChipInternal, Direction, Drive, Edge, Error, EventClock, InfoEvent, Result,
};

/// Line info
///
/// Exposes functions for retrieving kernel information about both requested and
/// free lines.  Line info object contains an immutable snapshot of a line's status.
///
/// The line info contains all the publicly available information about a
/// line, which does not include the line value.  The line must be requested
/// to access the line value.

#[derive(Debug)]
pub struct LineInfo {
    info: *mut bindings::gpiod_line_info,
    ichip: Option<Arc<ChipInternal>>,
    free: bool,
}

impl LineInfo {
    /// Get a snapshot of information about the line and optionally start watching it for changes.
    pub(crate) fn new(ichip: Arc<ChipInternal>, offset: u32, watch: bool) -> Result<Self> {
        let info = if watch {
            unsafe { bindings::gpiod_chip_watch_line_info(ichip.chip(), offset) }
        } else {
            unsafe { bindings::gpiod_chip_get_line_info(ichip.chip(), offset) }
        };

        if info.is_null() {
            return Err(Error::OperationFailed(
                "Gpio LineInfo line-info",
                IoError::last(),
            ));
        }

        Ok(Self {
            info,
            ichip: if watch { Some(ichip) } else { None },
            free: watch,
        })
    }

    /// Stop watching the line
    pub fn unwatch(&mut self) {
        if let Some(ichip) = &self.ichip {
            unsafe {
                bindings::gpiod_chip_unwatch_line_info(ichip.chip(), self.get_offset());
            }
            self.ichip = None;
        }
    }

    /// Get the offset of the line within the GPIO chip.
    ///
    /// The offset uniquely identifies the line on the chip. The combination of the chip and offset
    /// uniquely identifies the line within the system.

    pub fn get_offset(&self) -> u32 {
        unsafe { bindings::gpiod_line_info_get_offset(self.info) }
    }

    /// Get GPIO line's name.
    pub fn get_name(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct LineInfo`.
        let name = unsafe { bindings::gpiod_line_info_get_name(self.info) };
        if name.is_null() {
            return Err(Error::NameNotFound("GPIO line's name"));
        }

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(name as *const u8, bindings::strlen(name) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Returns True if the line is in use, false otherwise.
    ///
    /// The user space can't know exactly why a line is busy. It may have been
    /// requested by another process or hogged by the kernel. It only matters that
    /// the line is used and we can't request it.
    pub fn is_used(&self) -> bool {
        unsafe { bindings::gpiod_line_info_is_used(self.info) }
    }

    /// Get the GPIO line's consumer name.
    pub fn get_consumer(&self) -> Result<&str> {
        // SAFETY: The string returned by libgpiod is guaranteed to live as long
        // as the `struct LineInfo`.
        let name = unsafe { bindings::gpiod_line_info_get_consumer(self.info) };
        if name.is_null() {
            return Err(Error::NameNotFound("GPIO line's consumer name"));
        }

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(name as *const u8, bindings::strlen(name) as usize)
        })
        .map_err(Error::InvalidString)
    }

    /// Get the GPIO line's direction.
    pub fn get_direction(&self) -> Result<Direction> {
        Direction::new(unsafe { bindings::gpiod_line_info_get_direction(self.info) } as u32)
    }

    /// Returns true if the line is "active-low", false otherwise.
    pub fn is_active_low(&self) -> bool {
        unsafe { bindings::gpiod_line_info_is_active_low(self.info) }
    }

    /// Get the GPIO line's bias setting.
    pub fn get_bias(&self) -> Result<Bias> {
        Bias::new(unsafe { bindings::gpiod_line_info_get_bias(self.info) } as u32)
    }

    /// Get the GPIO line's drive setting.
    pub fn get_drive(&self) -> Result<Drive> {
        Drive::new(unsafe { bindings::gpiod_line_info_get_drive(self.info) } as u32)
    }

    /// Get the current edge detection setting of the line.
    pub fn get_edge_detection(&self) -> Result<Edge> {
        Edge::new(unsafe { bindings::gpiod_line_info_get_edge_detection(self.info) } as u32)
    }

    /// Get the current event clock setting used for edge event timestamps.
    pub fn get_event_clock(&self) -> Result<EventClock> {
        EventClock::new(unsafe { bindings::gpiod_line_info_get_event_clock(self.info) } as u32)
    }

    /// Returns true if the line is debounced (either by hardware or by the
    /// kernel software debouncer), false otherwise.
    pub fn is_debounced(&self) -> bool {
        unsafe { bindings::gpiod_line_info_is_debounced(self.info) }
    }

    /// Get the debounce period of the line.
    pub fn get_debounce_period(&self) -> Duration {
        Duration::from_micros(unsafe {
            bindings::gpiod_line_info_get_debounce_period_us(self.info)
        })
    }
}

impl TryFrom<&InfoEvent> for LineInfo {
    type Error = Error;

    /// Get the Line info object associated with a event.
    fn try_from(event: &InfoEvent) -> Result<Self> {
        let info = unsafe { bindings::gpiod_info_event_get_line_info(event.event()) };
        if info.is_null() {
            return Err(Error::OperationFailed(
                "Gpio LineInfo try-from",
                IoError::last(),
            ));
        }

        Ok(Self {
            info,
            ichip: None,
            free: false,
        })
    }
}

impl Drop for LineInfo {
    fn drop(&mut self) {
        // We must not free the Line info object created from `struct InfoEvent` by calling
        // libgpiod API.
        if self.free {
            self.unwatch();
            unsafe { bindings::gpiod_line_info_free(self.info) }
        }
    }
}
