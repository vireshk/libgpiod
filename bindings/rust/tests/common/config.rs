// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause
//
// Copyright 2022 Linaro Ltd. All Rights Reserved.
//     Viresh Kumar <viresh.kumar@linaro.org>

use std::sync::Arc;

use crate::common::*;

use libgpiod::{Bias, Chip, Direction, Edge, LineConfig, LineRequest, RequestConfig, Result};

//#[derive(Debug)]
pub(crate) struct TestConfig {
    sim: Arc<Sim>,
    chip: Option<Chip>,
    request: Option<LineRequest>,
    rconfig: RequestConfig,
    lconfig: LineConfig,
}

impl TestConfig {
    pub(crate) fn new(ngpio: u64) -> Result<Self> {
        Ok(Self {
            sim: Arc::new(Sim::new(Some(ngpio), None, true)?),
            chip: None,
            request: None,
            rconfig: RequestConfig::new().unwrap(),
            lconfig: LineConfig::new().unwrap(),
        })
    }

    pub(crate) fn set_pull(&self, offsets: &[u32], pulls: &[u32]) {
        for i in 0..pulls.len() {
            self.sim.set_pull(offsets[i], pulls[i] as i32).unwrap();
        }
    }

    pub(crate) fn rconfig_consumer(&self, offsets: Option<&[u32]>, consumer: Option<&str>) {
        if let Some(offsets) = offsets {
            self.rconfig.set_offsets(offsets);
        }

        if let Some(consumer) = consumer {
            self.rconfig.set_consumer(consumer);
        }
    }

    pub(crate) fn rconfig(&self, offsets: Option<&[u32]>) {
        self.rconfig_consumer(offsets, None);
    }

    pub(crate) fn lconfig(
        &mut self,
        dir: Option<Direction>,
        val: Option<u32>,
        val_override: Option<(u32, u32)>,
        edge: Option<Edge>,
        bias: Option<Bias>,
    ) {
        if let Some(bias) = bias {
            self.lconfig.set_bias_default(bias);
        }

        if let Some(edge) = edge {
            self.lconfig.set_edge_detection_default(edge);
        }

        if let Some(dir) = dir {
            self.lconfig.set_direction_default(dir);
        }

        if let Some(val) = val {
            self.lconfig.set_output_value_default(val);
        }

        if let Some((offset, val)) = val_override {
            self.lconfig.set_output_value_override(val, offset);
        }
    }

    pub(crate) fn lconfig_raw(&mut self) {
        self.lconfig(None, None, None, None, None);
    }

    pub(crate) fn lconfig_edge(&mut self, edge: Option<Edge>) {
        self.lconfig(None, None, None, edge, None);
    }

    pub(crate) fn request_lines(&mut self) -> Result<()> {
        let chip = Chip::open(self.sim.dev_path())?;

        self.request = Some(chip.request_lines(&self.rconfig, &self.lconfig)?);
        self.chip = Some(chip);

        Ok(())
    }

    pub(crate) fn sim(&self) -> Arc<Sim> {
        self.sim.clone()
    }

    pub(crate) fn chip(&self) -> &Chip {
        &self.chip.as_ref().unwrap()
    }

    pub(crate) fn request(&self) -> &LineRequest {
        &self.request.as_ref().unwrap()
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        // Explicit freeing is important to make sure "request" get freed
        // before "sim" and "chip".
        self.request = None;
    }
}
