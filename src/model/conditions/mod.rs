pub mod condition_definition;
use log::*;

use crate::logging_utils::device_error;
use crate::model::Device;
use crate::state::DeviceState;
use condition_definition::{ConditionDefinition, ConditionKind};

pub struct Condition<'a> {
    pub name: String,
    pub id: String,
    pub kind: ConditionKind,
    pub device: &'a mut Device,
    pub state: DeviceState,
    pub margin_above: f64,
    pub margin_below: f64,
}

impl<'a> Condition<'a> {
    pub fn from_definition(def: ConditionDefinition, device: &'a mut Device) -> Self {
        return Self {
            name: def.name,
            id: def.id,
            kind: def.kind,
            device,
            state: def.state,
            margin_above: def.margin_above,
            margin_below: def.margin_below,
        };
    }

    pub async fn evaluate(&mut self) -> bool {
        if let Err(e) = self.device.update().await {
            device_error!(
                self.device,
                &format!(
                    "error updating device when evaluating condition `{}`: {e}",
                    self.name
                )
            );
        }

        let result = match self.kind {
            ConditionKind::RelayStateIs => self.evaluate_relay_state_is(),
            ConditionKind::PVIsAtLeast => self.evaluate_pv_is_at_least(),
            ConditionKind::PVIsAround => self.evaluate_pv_is_around(),
            ConditionKind::PVReachesSV => todo!(),
        };

        return result;
    }

    /// Returns false if the relay from the condition definition is none.
    /// Note that this shouldn't be possible because the DeviceState struct provides a default
    /// value for relay_state. This is just an extra check.
    fn ensure_relay_state(&self) -> bool {
        if self.state.relay_state.is_none() {
            error!("Error when evaluating condition `{}`. There was no `relay_state` provided to match against. This shouldn't be possible", self.name);
            return false;
        }
        return true;
    }

    /// Returns false if the pv from the condition definition is none.
    /// Note that this shouldn't be possible because the DeviceState struct provides a default
    /// value for pv. This is just an extra check.
    fn ensure_pv(&self) -> bool {
        if self.state.pv.is_none() {
            error!("Error when evaluating condition `{}`. There was no `pv` state provided to match against. This shouldn't be possible", self.name);
            return false;
        }
        return true;
    }

    /// Returns false if the sv from the condition definition is none.
    /// Note that this shouldn't be possible because the DeviceState struct provides a default
    /// value for sv. This is just an extra check.
    #[allow(unused)]
    fn ensure_sv(&self) -> bool {
        if self.state.sv.is_none() {
            error!("Error when evaluating condition `{}`. There was no `sv` state provided to match against. This shouldn't be possible", self.name);
            return false;
        }
        return true;
    }

    fn evaluate_relay_state_is(&mut self) -> bool {
        if self.device.state.relay_state.is_none() {
            device_error!(self.device, "device relay state was None. How?");
            return false;
        }

        if !self.ensure_relay_state() {
            return false;
        }

        return self.device.state.relay_state.unwrap() == self.state.relay_state.unwrap();
    }

    fn evaluate_pv_is_at_least(&self) -> bool {
        if !self.ensure_pv() {
            return false;
        }

        if self.device.state.pv.is_none() {
            device_error!(self.device, "device pv was None. How?");
            return false;
        }

        return self.device.state.pv.unwrap() >= self.state.pv.unwrap();
    }

    fn evaluate_pv_is_around(&self) -> bool {
        if !self.ensure_pv() {
            return false;
        }

        if self.device.state.pv.is_none() {
            device_error!(self.device, "device pv was None. How?");
            return false;
        }

        let actual = self.device.state.pv.unwrap();
        let target = self.state.pv.unwrap();

        let lower_bound = target - self.margin_below;
        let upper_bound = target + self.margin_above;

        return actual >= lower_bound && actual <= upper_bound;
    }
}

