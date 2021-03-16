use std::collections::HashMap;

use crate::{DatumKey, DatumValue, InterpolationType, Time};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MappingType {
    ToggleSwitch {
        event_name: String,
        off_event_name: Option<String>,
        #[serde(default)]
        switch_on: bool,
    },
    NumSet {
        event_name: String,
        swap_event_name: Option<String>,
        multiply_by: Option<DatumValue>,
        add_by: Option<DatumValue>,
    },
    NumIncrement {
        up_event_name: String,
        down_event_name: String,
        increment_amount: DatumValue,
        #[serde(default)]
        pass_difference: bool,
    },
    NumDigitSet {
        inc_events: Vec<String>,
        dec_events: Vec<String>,
    },
    Var,
    // TODO: ProgramAction,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VarType {
    WithUnits {
        name: String,
        units: String,
        index: Option<usize>,
    },
    Named {
        name: String,
    },
    Calculator {
        get: String,
        set: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncPermissionState {
    pub server: bool,
    pub master: bool,
    pub init: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncPermission {
    Shared,
    Master,
    Server,
    Init,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConditionMessage {
    #[serde(default)]
    pub use_var: bool,
    pub equals: Option<DatumValue>,
    pub less_than: Option<DatumValue>,
    pub greater_than: Option<DatumValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InterpolateMessage {
    pub calculator: String,
    pub interpolate_type: InterpolationType,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DatumMessage {
    pub var: Option<VarType>,
    pub watch_event: Option<String>,       // Event name,
    pub watch_period: Option<WatchPeriod>, // Watch variable
    pub condition: Option<ConditionMessage>,
    pub interpolate: Option<InterpolateMessage>,
    pub mapping: Option<MappingType>,
    pub sync_permission: Option<SyncPermission>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChangedDatum {
    pub key: DatumKey,
    pub value: DatumValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Payloads {
    // Transmit to Sim
    SetDatums {
        datums: Vec<DatumMessage>,
    },

    WatchVariable {},
    WatchEvent {},
    MultiWatchVariable {},
    MultiWatchEvent {},
    ExecuteCalculator {},
    AddMapping {},
    SendIncomingValues {
        data: HashMap<DatumKey, DatumValue>,
        time: Time,
    },
    UpdateSyncPermission {
        new: SyncPermissionState,
    },

    ResetInterpolation,
    Ping,
    ResetAll,
    // Receive from Sim
    VariableChange {
        changed: Vec<ChangedDatum>,
    },
    EventTriggered {},
    Pong,
}

/// Period where a variable becomes "Changed".
#[derive(Serialize, Deserialize, Debug)]
pub enum WatchPeriod {
    Frame,
    Hz16,
    Second,
}

impl WatchPeriod {
    pub fn as_seconds_f64(&self) -> f64 {
        match self {
            WatchPeriod::Frame => 0.0,
            WatchPeriod::Hz16 => 0.26,
            WatchPeriod::Second => 1.0,
        }
    }
}
