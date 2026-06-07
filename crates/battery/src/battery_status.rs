use std::{path::Display, str::FromStr};

pub use battery_lib::State as ExternalBatteryState;
use battery_lib::{
    Manager,
    units::{
        electric_potential::volt, energy::watt_hour, power::watt, ratio::ratio,
        thermodynamic_temperature::degree_celsius, time::second,
    },
};
use chrono::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use status::{Last, Status as BaseStatus};
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct State(pub ExternalBatteryState);
impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state_str = match self.0 {
            ExternalBatteryState::Unknown => "Unknown",
            ExternalBatteryState::Charging => "Charging",
            ExternalBatteryState::Discharging => "Discharging",
            ExternalBatteryState::Empty => "Empty",
            ExternalBatteryState::Full => "Full",
            _ => "Unknown",
        };
        serializer.serialize_str(state_str)
    }
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state_str = String::deserialize(deserializer);
        let state =
            ExternalBatteryState::from_str(state_str.unwrap_or(String::from("Unknown")).as_str());
        Ok(State(match state {
            Ok(v) => v,
            _ => ExternalBatteryState::Unknown,
        }))
    }
}
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Identifier {
    // vendor
    pub vendor: Option<String>,
    // model
    pub model: Option<String>,
    // serial number
    pub serial_number: Option<String>,
}
impl Default for Identifier {
    fn default() -> Self {
        Self {
            vendor: None,
            serial_number: None,
            model: None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Status {
    // Whether battery info changed since last update
    pub state_changed: bool,
    // additional identifier info
    identifier: Identifier,
    // index
    index: u16,
    // timestamp
    pub timestamp: i64,
    // state
    pub state: State,
    // temperature
    pub temperature: Option<f32>,
    // cycle count
    pub cycle_count: Option<u32>,
    // percentage
    pub percentage: f32,
    // energy rate (charge/discharge watts)
    pub energy_rate: f32,
    // voltage
    pub voltage: f32,
    // state of health
    pub state_of_health: f32,
    // design capacity
    pub design_capacity: f32,
    // full charge capacity
    pub full_capacity: f32,
    // current capacity
    pub capacity: f32,
    // estimated seconds to empty
    pub time_to_empty_secs: u64,
    // estimated seconds to full
    pub time_to_full_secs: u64,
}
impl<'a> Default for Status {
    fn default() -> Self {
        Self {
            identifier: Identifier {
                serial_number: None,
                vendor: None,
                model: None,
            },
            index: 0,
            state_changed: false,
            timestamp: Utc::now().timestamp(),

            state: State(ExternalBatteryState::Unknown),
            temperature: None,
            cycle_count: None,
            percentage: 0.0,
            energy_rate: 0.0,
            voltage: 0.0,
            state_of_health: 0.0,
            design_capacity: 0.0,
            full_capacity: 0.0,
            capacity: 0.0,
            time_to_empty_secs: 0,
            time_to_full_secs: 0,
        }
    }
}
impl Status {
    fn refresh(&mut self) {
        let manager = Manager::new().unwrap();
        let batteries = manager.batteries();
        let o = batteries.unwrap().find(|x| {
            if let Ok(b) = x {
                b.vendor().map(|x| x.to_string()) == self.identifier.vendor
                    && b.model().map(|x| x.to_string()) == self.identifier.model
                    && b.serial_number().map(|x| x.to_string()) == self.identifier.serial_number
            } else {
                false
            }
        });
        if o.is_none() {
            return;
        }
        if let Ok(battery) = o.unwrap() {
            let status = self;

            let new_state = State(battery.state());
            status.state_changed =
                status.state.0 != ExternalBatteryState::Unknown && status.state != new_state;
            status.state = new_state;
            status.timestamp = Utc::now().timestamp();
            status.percentage = battery.state_of_charge().get::<ratio>();
            status.voltage = battery.voltage().get::<volt>();
            status.state_of_health = battery.state_of_health().get::<ratio>();
            status.energy_rate = match status.state {
                State(ExternalBatteryState::Empty) => -battery.energy_rate().get::<watt>(),
                State(ExternalBatteryState::Discharging) => -battery.energy_rate().get::<watt>(),
                State(ExternalBatteryState::Full) => 0.0,
                _ => battery.energy_rate().get::<watt>(),
            };
            status.design_capacity = battery.energy_full_design().get::<watt_hour>();
            status.full_capacity = battery.energy_full().get::<watt_hour>();
            status.capacity = battery.energy().get::<watt_hour>();
            status.time_to_empty_secs = match battery.time_to_empty() {
                Some(duration) => duration.get::<second>() as u64,
                None => 0,
            };
            status.time_to_full_secs = match battery.time_to_full() {
                Some(duration) => duration.get::<second>() as u64,
                None => 0,
            };
            status.temperature = battery.temperature().map(|x| x.get::<degree_celsius>());
            status.cycle_count = battery.cycle_count();
        }
    }
}
impl BaseStatus<Vec<Status>> for Status {
    fn build() -> Option<Vec<Status>> {
        let manager = Manager::new().unwrap();
        let batteries = manager.batteries();
        let mut statuses = Vec::<Status>::new();
        match batteries {
            Ok(rows) => {
                for (idx, row) in rows.enumerate() {
                    if let Ok(battery) = row {
                        let mut status = Status::default();
                        status.index = idx as u16;
                        status.identifier = Identifier {
                            serial_number: battery.serial_number().map(|x| x.to_string()),
                            vendor: battery.vendor().map(|x| x.to_string()),
                            model: battery.model().map(|x| x.to_string()),
                        };
                        status.refresh();
                        statuses.push(status);
                    }
                }
            }
            Err(_) => (),
        };
        if statuses.len() > 0 {
            Some(statuses)
        } else {
            None
        }
    }
}
impl Last for Status {
    fn last(&mut self) {
        self.refresh();
    }
}
