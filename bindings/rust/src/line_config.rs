// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use libc::EINVAL;
use std::os::raw::c_ulong;
use std::time::Duration;

use vmm_sys_util::errno::Error as IoError;

use super::{bindings, Bias, Config, Direction, Drive, Edge, Error, EventClock, Result};

/// Line configuration objects.
///
/// The line-config object contains the configuration for lines that can be
/// used in two cases:
///  - when making a line request
///  - when reconfiguring a set of already requested lines.
///
/// A new line-config object is instantiated with a set of sane defaults
/// for all supported configuration settings. Those defaults can be modified by
/// the caller. Default values can be overridden by applying different values
/// for specific lines. When making a request or reconfiguring an existing one,
/// the overridden settings for specific lines take precedence. For lines
/// without an override the requested default settings are used.
///
/// For every setting there are two mutators (one setting the default and one
/// for the per-line override), two getters (one for reading the global
/// default and one for retrieving the effective value for the line),
/// a function for testing if a setting is overridden for the line
/// and finally a function for clearing the overrides (per line).
///
/// The mutators don't return errors. If the set of options is too complex to
/// be translated into kernel uAPI structures then an error will be returned at
/// the time of the request or reconfiguration. If an invalid value was passed
/// to any of the mutators then the default value will be silently used instead.
///
/// Operating on lines in struct LineConfig has no immediate effect on real
/// GPIOs, it only manipulates the config object in memory.  Those changes are
/// only applied to the hardware at the time of the request or reconfiguration.
///
/// Overrides for lines that don't end up being requested are silently ignored
/// both in LineRequest::new() as well as in LineRequest::reconfigure_lines().
///
/// In cases where all requested lines are using the one configuration, the
/// line overrides can be entirely ignored when preparing the configuration.

pub struct LineConfig {
    config: *mut bindings::gpiod_line_config,
}

impl LineConfig {
    /// Create a new line config object.
    pub fn new() -> Result<Self> {
        let config = unsafe { bindings::gpiod_line_config_new() };

        if config.is_null() {
            return Err(Error::OperationFailed(
                "Gpio LineConfig new",
                IoError::last(),
            ));
        }

        Ok(Self { config })
    }

    /// Private helper, Returns gpiod_line_config
    pub(crate) fn config(&self) -> *mut bindings::gpiod_line_config {
        self.config
    }

    /// Resets the entire configuration stored in the object. This is useful if
    /// the user wants to reuse the object without reallocating it.
    pub fn reset(&mut self) {
        unsafe { bindings::gpiod_line_config_reset(self.config) }
    }

    /// Set the default line direction.
    pub fn set_direction_default(&mut self, direction: Direction) {
        unsafe {
            bindings::gpiod_line_config_set_direction_default(
                self.config,
                direction.gpiod_direction() as i32,
            )
        }
    }

    /// Set the direction for a line.
    pub fn set_direction_override(&mut self, direction: Direction, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_direction_override(
                self.config,
                direction.gpiod_direction() as i32,
                offset,
            )
        }
    }

    /// Clear the direction for a line.
    pub fn clear_direction_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_direction_override(self.config, offset) }
    }

    /// Check if the direction is overridden for a line.
    pub fn direction_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_direction_is_overridden(self.config, offset) }
    }

    /// Get the default direction setting.
    pub fn get_direction_default(&self) -> Result<Direction> {
        Direction::new(
            unsafe { bindings::gpiod_line_config_get_direction_default(self.config) } as u32,
        )
    }

    /// Get the direction of a given line.
    ///
    /// Direction setting for the line if the config object were used in a request.
    pub fn get_direction_offset(&self, offset: u32) -> Result<Direction> {
        Direction::new(unsafe {
            bindings::gpiod_line_config_get_direction_offset(self.config, offset)
        } as u32)
    }

    /// Set the default edge event detection setting.
    pub fn set_edge_detection_default(&mut self, edge: Edge) {
        unsafe {
            bindings::gpiod_line_config_set_edge_detection_default(
                self.config,
                edge.gpiod_edge() as i32,
            )
        }
    }

    /// Set the edge event detection for a single line.
    pub fn set_edge_detection_override(&mut self, edge: Edge, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_edge_detection_override(
                self.config,
                edge.gpiod_edge() as i32,
                offset,
            )
        }
    }

    /// Clear the edge event detection for a single line.
    pub fn clear_edge_detection_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_edge_detection_override(self.config, offset) }
    }

    /// Check if the edge event detection is overridden for a line.
    pub fn edge_detection_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_edge_detection_is_overridden(self.config, offset) }
    }

    /// Get the default edge event detection setting.
    pub fn get_edge_detection_default(&self) -> Result<Edge> {
        Edge::new(
            unsafe { bindings::gpiod_line_config_get_edge_detection_default(self.config) } as u32,
        )
    }

    /// Get the edge event detection setting for a given line.
    ///
    /// Edge event detection setting for the line if the config object were used in a request.
    pub fn get_edge_detection_offset(&self, offset: u32) -> Result<Edge> {
        Edge::new(unsafe {
            bindings::gpiod_line_config_get_edge_detection_offset(self.config, offset)
        } as u32)
    }

    /// Set the default bias setting.
    pub fn set_bias_default(&mut self, bias: Bias) {
        unsafe {
            bindings::gpiod_line_config_set_bias_default(self.config, bias.gpiod_bias() as i32)
        }
    }

    /// Set the bias for a single line.
    pub fn set_bias_override(&mut self, bias: Bias, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_bias_override(
                self.config,
                bias.gpiod_bias() as i32,
                offset,
            )
        }
    }

    /// Clear the bias for a single line.
    pub fn clear_bias_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_bias_override(self.config, offset) }
    }

    /// Check if the bias is overridden for a line.
    pub fn bias_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_bias_is_overridden(self.config, offset) }
    }

    /// Get the default bias setting.
    pub fn get_bias_default(&self) -> Result<Bias> {
        Bias::new(unsafe { bindings::gpiod_line_config_get_bias_default(self.config) } as u32)
    }

    /// Get the bias setting for a given line.
    ///
    /// Bias setting used for the line if the config object were used in a request.
    pub fn get_bias_offset(&self, offset: u32) -> Result<Bias> {
        Bias::new(
            unsafe { bindings::gpiod_line_config_get_bias_offset(self.config, offset) } as u32,
        )
    }

    /// Set the default drive setting.
    pub fn set_drive_default(&mut self, drive: Drive) {
        unsafe {
            bindings::gpiod_line_config_set_drive_default(self.config, drive.gpiod_drive() as i32)
        }
    }

    /// Set the drive for a single line.
    pub fn set_drive_override(&mut self, drive: Drive, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_drive_override(
                self.config,
                drive.gpiod_drive() as i32,
                offset,
            )
        }
    }

    /// clear the drive for a single line.
    pub fn clear_drive_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_drive_override(self.config, offset) }
    }

    /// Check if the drive is overridden for a line.
    pub fn drive_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_drive_is_overridden(self.config, offset) }
    }

    /// Get the default drive setting.
    pub fn get_drive_default(&self) -> Result<Drive> {
        Drive::new(unsafe { bindings::gpiod_line_config_get_drive_default(self.config) } as u32)
    }

    /// Get the drive setting for a given line.
    ///
    /// The offset of the line for which to read the drive setting. Drive setting for the line if
    /// the config object were used in a request.
    pub fn get_drive_offset(&self, offset: u32) -> Result<Drive> {
        Drive::new(
            unsafe { bindings::gpiod_line_config_get_drive_offset(self.config, offset) } as u32,
        )
    }

    /// Set default active-low setting.
    pub fn set_active_low_default(&mut self, active_low: bool) {
        unsafe { bindings::gpiod_line_config_set_active_low_default(self.config, active_low) }
    }

    /// Set active-low setting for a single line.
    pub fn set_active_low_override(&mut self, active_low: bool, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_active_low_override(self.config, active_low, offset)
        }
    }

    /// Clear a single line's active-low setting.
    pub fn clear_active_low_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_active_low_override(self.config, offset) }
    }

    /// Check if the active-low is overridden for a line.
    pub fn active_low_is_overridden(&mut self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_active_low_is_overridden(self.config, offset) }
    }

    /// Check the default active-low setting.
    pub fn get_active_low_default(&self) -> bool {
        unsafe { bindings::gpiod_line_config_get_active_low_default(self.config) }
    }

    /// Check the active-low setting of a line.
    ///
    /// Active-low setting for the line if the config object were used in a request.
    pub fn get_active_low_offset(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_get_active_low_offset(self.config, offset) }
    }

    /// Set the deafult debounce period setting.
    pub fn set_debounce_period_default(&mut self, period: Duration) {
        unsafe {
            bindings::gpiod_line_config_set_debounce_period_us_default(
                self.config,
                period.as_micros() as u64,
            )
        }
    }

    /// Set the debounce period for a single line.
    pub fn set_debounce_period_override(&mut self, period: Duration, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_debounce_period_us_override(
                self.config,
                period.as_micros() as u64,
                offset,
            )
        }
    }

    /// Clear the debounce period for a single line.
    pub fn clear_debounce_period_override(&mut self, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_clear_debounce_period_us_override(self.config, offset)
        }
    }

    /// Check if the debounce period setting is overridden.
    pub fn debounce_period_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_debounce_period_us_is_overridden(self.config, offset) }
    }

    /// Get the default debounce period.
    pub fn get_debounce_period_default(&self) -> Result<Duration> {
        Ok(Duration::from_micros(unsafe {
            bindings::gpiod_line_config_get_debounce_period_us_default(self.config)
        }))
    }

    /// Get the debounce period for a given line.
    ///
    /// Debounce period for the line if the config object were used in a request, 0 if debouncing
    /// is disabled.
    pub fn get_debounce_period_offset(&self, offset: u32) -> Result<Duration> {
        Ok(Duration::from_micros(unsafe {
            bindings::gpiod_line_config_get_debounce_period_us_offset(self.config, offset)
        }))
    }

    /// Set the default event clock setting.
    pub fn set_event_clock_default(&mut self, clock: EventClock) {
        unsafe {
            bindings::gpiod_line_config_set_event_clock_default(
                self.config,
                clock.gpiod_clock() as i32,
            )
        }
    }

    /// Set the event clock for a single line.
    pub fn set_event_clock_override(&mut self, clock: EventClock, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_event_clock_override(
                self.config,
                clock.gpiod_clock() as i32,
                offset,
            )
        }
    }

    /// Clear the event clock for a single line.
    pub fn clear_event_clock_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_event_clock_override(self.config, offset) }
    }

    /// Check if the event clock is overridden for a line.
    pub fn event_clock_is_overridden(&mut self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_event_clock_is_overridden(self.config, offset) }
    }

    /// Get the default event clock setting.
    pub fn get_event_clock_default(&self) -> Result<EventClock> {
        EventClock::new(
            unsafe { bindings::gpiod_line_config_get_event_clock_default(self.config) } as u32,
        )
    }

    /// Get the event clock setting for a given line.
    ///
    /// Event clock setting for the line if the config object were used in a request.
    pub fn get_event_clock_offset(&self, offset: u32) -> Result<EventClock> {
        EventClock::new(unsafe {
            bindings::gpiod_line_config_get_event_clock_offset(self.config, offset)
        } as u32)
    }

    /// Set the default output value setting.
    pub fn set_output_value_default(&mut self, value: u32) {
        unsafe { bindings::gpiod_line_config_set_output_value_default(self.config, value as i32) }
    }

    /// Set the output value for a line.
    pub fn set_output_value_override(&mut self, value: u32, offset: u32) {
        unsafe {
            bindings::gpiod_line_config_set_output_value_override(self.config, value as i32, offset)
        }
    }

    /// Set the output values for a set of lines.
    pub fn set_output_values(&mut self, offsets: &[u32], values: &[i32]) -> Result<()> {
        if offsets.len() != values.len() {
            return Err(Error::OperationFailed(
                "Gpio LineConfig array size mismatch",
                IoError::new(EINVAL),
            ));
        }

        unsafe {
            bindings::gpiod_line_config_set_output_values(
                self.config,
                values.len() as c_ulong,
                offsets.as_ptr(),
                values.as_ptr(),
            );
        }

        Ok(())
    }

    /// Clear the output value for a line.
    pub fn clear_output_value_override(&mut self, offset: u32) {
        unsafe { bindings::gpiod_line_config_clear_output_value_override(self.config, offset) }
    }

    /// Check if the output value is overridden for a line.
    pub fn output_value_is_overridden(&self, offset: u32) -> bool {
        unsafe { bindings::gpiod_line_config_output_value_is_overridden(self.config, offset) }
    }

    /// Get the default output value, 0 or 1.
    pub fn get_output_value_default(&self) -> Result<u32> {
        let value = unsafe { bindings::gpiod_line_config_get_output_value_default(self.config) };

        if value != 0 && value != 1 {
            Err(Error::OperationFailed(
                "Gpio LineConfig get-output-value",
                IoError::last(),
            ))
        } else {
            Ok(value as u32)
        }
    }

    /// Get the output value configured for a given line, 0 or 1.
    pub fn get_output_value_offset(&self, offset: u32) -> Result<u32> {
        let value =
            unsafe { bindings::gpiod_line_config_get_output_value_offset(self.config, offset) };

        if value != 0 && value != 1 {
            Err(Error::OperationFailed(
                "Gpio LineConfig get-output-value",
                IoError::last(),
            ))
        } else {
            Ok(value as u32)
        }
    }

    /// Get the list of overridden offsets and the corresponding types of overridden settings.
    pub fn get_overrides(&self) -> Result<Vec<(u32, Config)>> {
        let num = unsafe { bindings::gpiod_line_config_get_num_overrides(self.config) } as usize;
        if num == 0 {
            return Ok(Vec::new());
        }

        let mut overrides = Vec::new();
        let mut offset = vec![0_u32; num];
        let mut props = vec![0_i32; num];

        unsafe {
            bindings::gpiod_line_config_get_overrides(
                self.config,
                offset.as_mut_ptr(),
                props.as_mut_ptr(),
            )
        };

        for i in 0..num {
            overrides.push((offset[i], Config::new(props[i] as u32)?));
        }

        Ok(overrides)
    }
}

impl Drop for LineConfig {
    /// Free the line config object and release all associated resources.
    fn drop(&mut self) {
        unsafe { bindings::gpiod_line_config_free(self.config) }
    }
}
