// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::os::raw::c_char;
use std::{slice, str};

use vmm_sys_util::errno::Error as IoError;

use libgpiod::{Error, Result};
use libgpiod_sys as bindings;

/// Sim Ctx
#[derive(Debug)]
struct SimCtx {
    ctx: *mut bindings::gpiosim_ctx,
}

unsafe impl Send for SimCtx {}
unsafe impl Sync for SimCtx {}

impl SimCtx {
    fn new() -> Result<Self> {
        let ctx = unsafe { bindings::gpiosim_ctx_new() };
        if ctx.is_null() {
            return Err(Error::OperationFailed("gpio-sim ctx new", IoError::last()));
        }

        Ok(Self { ctx })
    }

    fn ctx(&self) -> *mut bindings::gpiosim_ctx {
        self.ctx
    }
}

impl Drop for SimCtx {
    fn drop(&mut self) {
        unsafe { bindings::gpiosim_ctx_unref(self.ctx) }
    }
}

/// Sim Dev
#[derive(Debug)]
struct SimDev {
    dev: *mut bindings::gpiosim_dev,
}

unsafe impl Send for SimDev {}
unsafe impl Sync for SimDev {}

impl SimDev {
    fn new(ctx: &SimCtx) -> Result<Self> {
        let dev = unsafe { bindings::gpiosim_dev_new(ctx.ctx()) };
        if dev.is_null() {
            return Err(Error::OperationFailed("gpio-sim dev new", IoError::last()));
        }

        Ok(Self { dev })
    }

    fn dev(&self) -> *mut bindings::gpiosim_dev {
        self.dev
    }

    fn enable(&self) -> Result<()> {
        let ret = unsafe { bindings::gpiosim_dev_enable(self.dev) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim dev-enable",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    fn disable(&self) -> Result<()> {
        let ret = unsafe { bindings::gpiosim_dev_disable(self.dev) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim dev-disable",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for SimDev {
    fn drop(&mut self) {
        unsafe { bindings::gpiosim_dev_unref(self.dev) }
    }
}

/// Sim Bank
#[derive(Debug)]
struct SimBank {
    bank: *mut bindings::gpiosim_bank,
}

unsafe impl Send for SimBank {}
unsafe impl Sync for SimBank {}

impl SimBank {
    fn new(dev: &SimDev) -> Result<Self> {
        let bank = unsafe { bindings::gpiosim_bank_new(dev.dev()) };
        if bank.is_null() {
            return Err(Error::OperationFailed("gpio-sim Bank new", IoError::last()));
        }

        Ok(Self { bank })
    }

    fn chip_name(&self) -> Result<&str> {
        // SAFETY: The string returned by gpiosim is guaranteed to live as long
        // as the `struct SimBank`.
        let name = unsafe { bindings::gpiosim_bank_get_chip_name(self.bank) };

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(name as *const u8, bindings::strlen(name) as usize)
        })
        .map_err(Error::InvalidString)
    }

    fn dev_path(&self) -> Result<&str> {
        // SAFETY: The string returned by gpiosim is guaranteed to live as long
        // as the `struct SimBank`.
        let path = unsafe { bindings::gpiosim_bank_get_dev_path(self.bank) };

        // SAFETY: The string is guaranteed to be valid here.
        str::from_utf8(unsafe {
            slice::from_raw_parts(path as *const u8, bindings::strlen(path) as usize)
        })
        .map_err(Error::InvalidString)
    }

    fn val(&self, offset: u32) -> Result<u32> {
        let ret = unsafe { bindings::gpiosim_bank_get_value(self.bank, offset) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim get-value",
                IoError::last(),
            ))
        } else {
            Ok(ret as u32)
        }
    }

    fn set_label(&self, label: &str) -> Result<()> {
        // Null-terminate the string
        let label = label.to_owned() + "\0";

        let ret =
            unsafe { bindings::gpiosim_bank_set_label(self.bank, label.as_ptr() as *const c_char) };

        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim set-label",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    fn set_num_lines(&self, num: u64) -> Result<()> {
        let ret = unsafe { bindings::gpiosim_bank_set_num_lines(self.bank, num) };
        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim set-num-lines",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    fn set_line_name(&self, offset: u32, name: &str) -> Result<()> {
        // Null-terminate the string
        let name = name.to_owned() + "\0";

        let ret = unsafe {
            bindings::gpiosim_bank_set_line_name(self.bank, offset, name.as_ptr() as *const c_char)
        };

        if ret == -1 {
            Err(Error::OperationFailed(
                "gpio-sim set-line-name",
                IoError::last(),
            ))
        } else {
            Ok(())
        }
    }

    fn set_pull(&self, offset: u32, pull: i32) -> Result<()> {
        let ret = unsafe { bindings::gpiosim_bank_set_pull(self.bank, offset, pull) };

        if ret == -1 {
            Err(Error::OperationFailed("gpio-sim set-pull", IoError::last()))
        } else {
            Ok(())
        }
    }

    fn hog_line(&self, offset: u32, name: &str, dir: i32) -> Result<()> {
        // Null-terminate the string
        let name = name.to_owned() + "\0";

        let ret = unsafe {
            bindings::gpiosim_bank_hog_line(self.bank, offset, name.as_ptr() as *const c_char, dir)
        };

        if ret == -1 {
            Err(Error::OperationFailed("gpio-sim hog-line", IoError::last()))
        } else {
            Ok(())
        }
    }
}

impl Drop for SimBank {
    fn drop(&mut self) {
        unsafe { bindings::gpiosim_bank_unref(self.bank) }
    }
}

/// GPIO SIM
#[derive(Debug)]
pub(crate) struct Sim {
    ctx: SimCtx,
    dev: SimDev,
    bank: SimBank,
}

unsafe impl Send for Sim {}
unsafe impl Sync for Sim {}

impl Sim {
    pub(crate) fn new(ngpio: Option<u64>, label: Option<&str>, enable: bool) -> Result<Self> {
        let ctx = SimCtx::new()?;
        let dev = SimDev::new(&ctx)?;
        let bank = SimBank::new(&dev)?;

        if let Some(ngpio) = ngpio {
            bank.set_num_lines(ngpio)?;
        }

        if let Some(label) = label {
            bank.set_label(label)?;
        }

        if enable {
            dev.enable()?;
        }

        Ok(Self { ctx, dev, bank })
    }

    pub(crate) fn chip_name(&self) -> &str {
        self.bank.chip_name().unwrap()
    }

    pub fn dev_path(&self) -> &str {
        self.bank.dev_path().unwrap()
    }

    pub(crate) fn val(&self, offset: u32) -> Result<u32> {
        self.bank.val(offset)
    }

    pub(crate) fn set_label(&self, label: &str) -> Result<()> {
        self.bank.set_label(label)
    }

    pub(crate) fn set_num_lines(&self, num: u64) -> Result<()> {
        self.bank.set_num_lines(num)
    }

    pub(crate) fn set_line_name(&self, offset: u32, name: &str) -> Result<()> {
        self.bank.set_line_name(offset, name)
    }

    pub(crate) fn set_pull(&self, offset: u32, pull: i32) -> Result<()> {
        self.bank.set_pull(offset, pull)
    }

    pub(crate) fn hog_line(&self, offset: u32, name: &str, dir: i32) -> Result<()> {
        self.bank.hog_line(offset, name, dir)
    }

    pub(crate) fn enable(&self) -> Result<()> {
        self.dev.enable()
    }

    pub(crate) fn disable(&self) -> Result<()> {
        self.dev.disable()
    }
}
