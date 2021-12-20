// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Rust wrappers for GPIOD APIs
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

//! libgpiod public API
//!
//! This is the complete documentation of the public Rust API made available to
//! users of libgpiod.
//!
//! The API is logically split into several parts such as: GPIO chip & line
//! operators, GPIO events handling etc.

mod chip;
mod chip_info;
mod edge_event;
mod event_buffer;
mod info_event;
mod line_config;
mod line_info;
mod line_request;
mod request_config;

use libgpiod_sys as bindings;

pub use crate::chip::*;
pub use crate::edge_event::*;
pub use crate::event_buffer::*;
pub use crate::info_event::*;
pub use crate::line_config::*;
pub use crate::line_info::*;
pub use crate::line_request::*;
pub use crate::request_config::*;

use std::os::raw::c_char;
use std::{slice, str};

use thiserror::Error as ThisError;
use vmm_sys_util::errno::Error as IoError;

/// Result of libgpiod operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error codes for libgpiod operations
#[derive(Copy, Clone, Debug, PartialEq, ThisError)]
pub enum Error {
    #[error("Failed to find {0}")]
    NameNotFound(&'static str),
    #[error("Invalid String: {0:?}")]
    InvalidString(str::Utf8Error),
    #[error("Invalid {0} value: {1}")]
    InvalidValue(&'static str, u32),
    #[error("Operation {0} Failed: {1}")]
    OperationFailed(&'static str, IoError),
    #[error("Operation Timed-out")]
    OperationTimedOut,
}

/// Direction settings.
pub enum Direction {
    /// Request the line(s), but don't change direction.
    AsIs,
    /// Direction is input - for reading the value of an externally driven GPIO line.
    Input,
    /// Direction is output - for driving the GPIO line.
    Output,
}

impl Direction {
    fn new(dir: u32) -> Result<Self> {
        match dir {
            bindings::GPIOD_LINE_DIRECTION_AS_IS => Ok(Direction::AsIs),
            bindings::GPIOD_LINE_DIRECTION_INPUT => Ok(Direction::Input),
            bindings::GPIOD_LINE_DIRECTION_OUTPUT => Ok(Direction::Output),
            _ => Err(Error::InvalidValue("direction", dir)),
        }
    }

    fn gpiod_direction(&self) -> u32 {
        match self {
            Direction::AsIs => bindings::GPIOD_LINE_DIRECTION_AS_IS,
            Direction::Input => bindings::GPIOD_LINE_DIRECTION_INPUT,
            Direction::Output => bindings::GPIOD_LINE_DIRECTION_OUTPUT,
        }
    }
}

/// Internal bias settings.
pub enum Bias {
    /// Don't change the bias setting when applying line config.
    AsIs,
    /// The internal bias state is unknown.
    Unknown,
    /// The internal bias is disabled.
    Disabled,
    /// The internal pull-up bias is enabled.
    PullUp,
    /// The internal pull-down bias is enabled.
    PullDown,
}

impl Bias {
    fn new(bias: u32) -> Result<Self> {
        match bias {
            bindings::GPIOD_LINE_BIAS_AS_IS => Ok(Bias::AsIs),
            bindings::GPIOD_LINE_BIAS_UNKNOWN => Ok(Bias::Unknown),
            bindings::GPIOD_LINE_BIAS_DISABLED => Ok(Bias::Disabled),
            bindings::GPIOD_LINE_BIAS_PULL_UP => Ok(Bias::PullUp),
            bindings::GPIOD_LINE_BIAS_PULL_DOWN => Ok(Bias::PullDown),
            _ => Err(Error::InvalidValue("bias", bias)),
        }
    }

    fn gpiod_bias(&self) -> u32 {
        match self {
            Bias::AsIs => bindings::GPIOD_LINE_BIAS_AS_IS,
            Bias::Unknown => bindings::GPIOD_LINE_BIAS_UNKNOWN,
            Bias::Disabled => bindings::GPIOD_LINE_BIAS_DISABLED,
            Bias::PullUp => bindings::GPIOD_LINE_BIAS_PULL_UP,
            Bias::PullDown => bindings::GPIOD_LINE_BIAS_PULL_DOWN,
        }
    }
}

/// Drive settings.
pub enum Drive {
    /// Drive setting is push-pull.
    PushPull,
    /// Line output is open-drain.
    OpenDrain,
    /// Line output is open-source.
    OpenSource,
}

impl Drive {
    fn new(drive: u32) -> Result<Self> {
        match drive {
            bindings::GPIOD_LINE_DRIVE_PUSH_PULL => Ok(Drive::PushPull),
            bindings::GPIOD_LINE_DRIVE_OPEN_DRAIN => Ok(Drive::OpenDrain),
            bindings::GPIOD_LINE_DRIVE_OPEN_SOURCE => Ok(Drive::OpenSource),
            _ => Err(Error::InvalidValue("drive", drive)),
        }
    }

    fn gpiod_drive(&self) -> u32 {
        match self {
            Drive::PushPull => bindings::GPIOD_LINE_DRIVE_PUSH_PULL,
            Drive::OpenDrain => bindings::GPIOD_LINE_DRIVE_OPEN_DRAIN,
            Drive::OpenSource => bindings::GPIOD_LINE_DRIVE_OPEN_SOURCE,
        }
    }
}

/// Edge detection settings.
pub enum Edge {
    /// Line edge detection is disabled.
    None,
    /// Line detects rising edge events.
    Rising,
    /// Line detects falling edge events.
    Falling,
    /// Line detects both rising and falling edge events.
    Both,
}

impl Edge {
    fn new(edge: u32) -> Result<Self> {
        match edge {
            bindings::GPIOD_LINE_EDGE_NONE => Ok(Edge::None),
            bindings::GPIOD_LINE_EDGE_RISING => Ok(Edge::Rising),
            bindings::GPIOD_LINE_EDGE_FALLING => Ok(Edge::Falling),
            bindings::GPIOD_LINE_EDGE_BOTH => Ok(Edge::Both),
            _ => Err(Error::InvalidValue("edge", edge)),
        }
    }

    fn gpiod_edge(&self) -> u32 {
        match self {
            Edge::None => bindings::GPIOD_LINE_EDGE_NONE,
            Edge::Rising => bindings::GPIOD_LINE_EDGE_RISING,
            Edge::Falling => bindings::GPIOD_LINE_EDGE_FALLING,
            Edge::Both => bindings::GPIOD_LINE_EDGE_BOTH,
        }
    }
}

/// Line config settings.
pub enum Config {
    /// Line direction.
    Direction,
    /// Edge detection.
    EdgeDetection,
    /// Bias.
    Bias,
    /// Drive.
    Drive,
    /// Active-low setting.
    ActiveLow,
    /// Debounce period.
    DebouncePeriodUs,
    /// Event clock type.
    EventClock,
    /// Output value.
    OutputValue,
}

impl Config {
    fn new(config: u32) -> Result<Self> {
        match config {
            bindings::GPIOD_LINE_CONFIG_PROP_DIRECTION => Ok(Config::Direction),
            bindings::GPIOD_LINE_CONFIG_PROP_EDGE_DETECTION => Ok(Config::EdgeDetection),
            bindings::GPIOD_LINE_CONFIG_PROP_BIAS => Ok(Config::Bias),
            bindings::GPIOD_LINE_CONFIG_PROP_DRIVE => Ok(Config::Drive),
            bindings::GPIOD_LINE_CONFIG_PROP_ACTIVE_LOW => Ok(Config::ActiveLow),
            bindings::GPIOD_LINE_CONFIG_PROP_DEBOUNCE_PERIOD_US => Ok(Config::DebouncePeriodUs),
            bindings::GPIOD_LINE_CONFIG_PROP_EVENT_CLOCK => Ok(Config::EventClock),
            bindings::GPIOD_LINE_CONFIG_PROP_OUTPUT_VALUE => Ok(Config::OutputValue),
            _ => Err(Error::InvalidValue("config", config)),
        }
    }
}

/// Event clock settings.
pub enum EventClock {
    /// Line uses the monotonic clock for edge event timestamps.
    Monotonic,
    /// Line uses the realtime clock for edge event timestamps.
    Realtime,
}

impl EventClock {
    fn new(clock: u32) -> Result<Self> {
        match clock {
            bindings::GPIOD_LINE_EVENT_CLOCK_MONOTONIC => Ok(EventClock::Monotonic),
            bindings::GPIOD_LINE_EVENT_CLOCK_REALTIME => Ok(EventClock::Realtime),
            _ => Err(Error::InvalidValue("event clock", clock)),
        }
    }

    fn gpiod_clock(&self) -> u32 {
        match self {
            EventClock::Monotonic => bindings::GPIOD_LINE_EVENT_CLOCK_MONOTONIC,
            EventClock::Realtime => bindings::GPIOD_LINE_EVENT_CLOCK_REALTIME,
        }
    }
}

/// Line status change event types.
pub enum Event {
    /// Line has been requested.
    LineRequested,
    /// Previously requested line has been released.
    LineReleased,
    /// Line configuration has changed.
    LineConfigChanged,
}

impl Event {
    fn new(event: u32) -> Result<Self> {
        match event {
            bindings::GPIOD_INFO_EVENT_LINE_REQUESTED => Ok(Event::LineRequested),
            bindings::GPIOD_INFO_EVENT_LINE_RELEASED => Ok(Event::LineReleased),
            bindings::GPIOD_INFO_EVENT_LINE_CONFIG_CHANGED => Ok(Event::LineConfigChanged),
            _ => Err(Error::InvalidValue("event", event)),
        }
    }
}

#[derive(Copy, Clone)]
/// Edge event types.
pub enum LineEdgeEvent {
    /// Rising edge event.
    Rising,
    /// Falling edge event.
    Falling,
}

impl LineEdgeEvent {
    fn new(event: u32) -> Result<Self> {
        match event {
            bindings::GPIOD_EDGE_EVENT_RISING_EDGE => Ok(LineEdgeEvent::Rising),
            bindings::GPIOD_EDGE_EVENT_FALLING_EDGE => Ok(LineEdgeEvent::Falling),
            _ => Err(Error::InvalidValue("edge event", event)),
        }
    }
}

/// Various libgpiod-related functions.

/// Check if the file pointed to by path is a GPIO chip character device.
///
/// Returns true if the file exists and is a GPIO chip character device or a
/// symbolic link to it.
pub fn gpiod_is_gpiochip_device(path: &str) -> bool {
    // Null-terminate the string
    let path = path.to_owned() + "\0";

    unsafe { bindings::gpiod_is_gpiochip_device(path.as_ptr() as *const c_char) }
}

/// Get the API version of the library as a human-readable string.
pub fn gpiod_version_string() -> Result<&'static str> {
    // SAFETY: The string returned by libgpiod is guaranteed to live forever.
    let version = unsafe { bindings::gpiod_version_string() };

    if version.is_null() {
        return Err(Error::NameNotFound("GPIO library version"));
    }

    // SAFETY: The string is guaranteed to be valid here.
    str::from_utf8(unsafe {
        slice::from_raw_parts(version as *const u8, bindings::strlen(version) as usize)
    })
    .map_err(Error::InvalidString)
}
