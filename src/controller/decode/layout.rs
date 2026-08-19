// Copyright 2026 Alexandre Mahdhaoui
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::types::pad::{DeviceKind, Transport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryLayout {
    pub level: usize,
    pub charging: usize,
    pub charging_mask: u8,
    pub full_mask: u8,
    pub max_when_charging: u32,
    pub max_when_discharging: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub sticks: usize,
    pub triggers: usize,
    pub faces: usize,
    pub shoulders: usize,
    pub misc: usize,
    pub frame_counter: Option<usize>,
    pub gyro: Option<usize>,
    pub accel: Option<usize>,
    pub touch: Option<usize>,
    pub battery: Option<BatteryLayout>,
    pub extended_buttons: bool,
    pub min_len: usize,
}

const DUALSENSE_BATTERY: BatteryLayout = BatteryLayout {
    level: 53,
    charging: 54,
    charging_mask: 0x08,
    full_mask: 0x20,
    max_when_charging: 8,
    max_when_discharging: 8,
};

const DUALSHOCK4_BATTERY: BatteryLayout = BatteryLayout {
    level: 30,
    charging: 30,
    charging_mask: 0x10,
    full_mask: 0x00,
    max_when_charging: 11,
    max_when_discharging: 8,
};

const DUALSENSE_FULL: Layout = Layout {
    sticks: 1,
    triggers: 5,
    faces: 8,
    shoulders: 9,
    misc: 10,
    frame_counter: Some(7),
    gyro: Some(16),
    accel: Some(22),
    touch: Some(33),
    battery: Some(DUALSENSE_BATTERY),
    extended_buttons: true,
    min_len: 64,
};

const DUALSENSE_BASIC: Layout = Layout {
    sticks: 1,
    triggers: 8,
    faces: 5,
    shoulders: 6,
    misc: 7,
    frame_counter: None,
    gyro: None,
    accel: None,
    touch: None,
    battery: None,
    extended_buttons: false,
    min_len: 10,
};

const DUALSHOCK4_FULL: Layout = Layout {
    sticks: 1,
    triggers: 8,
    faces: 5,
    shoulders: 6,
    misc: 7,
    frame_counter: None,
    gyro: Some(13),
    accel: Some(19),
    touch: Some(35),
    battery: Some(DUALSHOCK4_BATTERY),
    extended_buttons: false,
    min_len: 64,
};

const DUALSHOCK4_BASIC: Layout = Layout {
    sticks: 1,
    triggers: 8,
    faces: 5,
    shoulders: 6,
    misc: 7,
    frame_counter: None,
    gyro: None,
    accel: None,
    touch: None,
    battery: None,
    extended_buttons: false,
    min_len: 10,
};

impl Layout {
    pub fn shifted_by(mut self, offset: usize) -> Self {
        self.sticks += offset;
        self.triggers += offset;
        self.faces += offset;
        self.shoulders += offset;
        self.misc += offset;
        self.frame_counter = self.frame_counter.map(|at| at + offset);
        self.gyro = self.gyro.map(|at| at + offset);
        self.accel = self.accel.map(|at| at + offset);
        self.touch = self.touch.map(|at| at + offset);
        self.battery = self.battery.map(|mut battery| {
            battery.level += offset;
            battery.charging += offset;
            battery
        });
        self.min_len += offset;
        self
    }

    pub fn fits(&self, report: &[u8]) -> bool {
        report.len() >= self.min_len && report.len() > self.misc
    }
}

pub const BLUETOOTH_HEADER_LEN: usize = 1;

pub const USB_INPUT_REPORT_LEN: usize = 64;

pub const BLUETOOTH_FULL_REPORT_LEN: usize = 78;

pub fn transport_of(kind: DeviceKind, caps_report_len: usize, report: &[u8]) -> Option<Transport> {
    let id = *report.first()?;
    let over_usb = caps_report_len == USB_INPUT_REPORT_LEN;

    match (kind.is_dualsense(), id, over_usb) {
        (true, 0x01, true) => Some(Transport::Usb),
        (true, 0x01, false) => Some(Transport::BluetoothBasic),
        (true, 0x31, _) => Some(Transport::BluetoothFull),
        (false, 0x01, true) => Some(Transport::Usb),
        (false, 0x01, false) => Some(Transport::BluetoothBasic),
        (false, 0x11, _) => Some(Transport::BluetoothFull),
        _ => None,
    }
}

pub fn layout_for(kind: DeviceKind, transport: Transport) -> Layout {
    match (kind.is_dualsense(), transport) {
        (true, Transport::Usb) => DUALSENSE_FULL,
        (true, Transport::BluetoothBasic) => DUALSENSE_BASIC,
        (true, Transport::BluetoothFull) => DUALSENSE_FULL.shifted_by(BLUETOOTH_HEADER_LEN),
        (false, Transport::Usb) => DUALSHOCK4_FULL,
        (false, Transport::BluetoothBasic) => DUALSHOCK4_BASIC,
        (false, Transport::BluetoothFull) => DUALSHOCK4_FULL.shifted_by(BLUETOOTH_HEADER_LEN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usb_report_length_matches_ds4windows_dualsensedevice_cs_line_2251() {
        assert_eq!(USB_INPUT_REPORT_LEN, 64);
    }

    #[test]
    fn bluetooth_full_report_length_matches_ds4windows_dualsensedevice_cs_lines_449_450() {
        assert_eq!(BLUETOOTH_FULL_REPORT_LEN, 78);
    }

    #[test]
    fn dualsense_usb_matches_the_reference_offsets() {
        let layout = layout_for(DeviceKind::DualSense, Transport::Usb);

        assert_eq!(layout.sticks, 1);
        assert_eq!(layout.triggers, 5);
        assert_eq!(layout.faces, 8);
        assert_eq!(layout.shoulders, 9);
        assert_eq!(layout.misc, 10);
        assert_eq!(layout.gyro, Some(16));
        assert_eq!(layout.accel, Some(22));
        assert_eq!(layout.touch, Some(33));
    }

    #[test]
    fn dualshock4_puts_triggers_after_the_button_bytes_and_dualsense_before() {
        let ds4 = layout_for(DeviceKind::DualShock4V2, Transport::Usb);
        let ds5 = layout_for(DeviceKind::DualSense, Transport::Usb);

        assert_eq!(ds4.triggers, 8);
        assert_eq!(ds5.triggers, 5);
    }

    #[test]
    fn bluetooth_full_shifts_every_offset_by_the_header_length() {
        let usb = layout_for(DeviceKind::DualSense, Transport::Usb);
        let bt = layout_for(DeviceKind::DualSense, Transport::BluetoothFull);

        assert_eq!(bt.faces, usb.faces + BLUETOOTH_HEADER_LEN);
        assert_eq!(bt.gyro, usb.gyro.map(|at| at + BLUETOOTH_HEADER_LEN));
        assert_eq!(
            bt.battery.map(|b| b.level),
            usb.battery.map(|b| b.level + BLUETOOTH_HEADER_LEN)
        );
    }

    #[test]
    fn a_bluetooth_report_windows_padded_to_seventy_eight_bytes_is_not_mistaken_for_usb() {
        let padded = [0x01u8; 78];

        assert_eq!(
            transport_of(DeviceKind::DualSense, 78, &padded),
            Some(Transport::BluetoothBasic)
        );
    }

    #[test]
    fn only_a_sixty_four_byte_capability_means_usb() {
        let report = [0x01u8; 64];

        assert_eq!(
            transport_of(DeviceKind::DualSense, USB_INPUT_REPORT_LEN, &report),
            Some(Transport::Usb)
        );
        assert_eq!(
            transport_of(DeviceKind::DualSense, 78, &report),
            Some(Transport::BluetoothBasic)
        );
    }

    #[test]
    fn report_id_0x31_is_dualsense_only_and_0x11_is_dualshock4_only() {
        let report = [0x31u8; 78];
        assert_eq!(
            transport_of(DeviceKind::DualSense, 78, &report),
            Some(Transport::BluetoothFull)
        );
        assert_eq!(transport_of(DeviceKind::DualShock4V2, 78, &report), None);

        let ds4 = [0x11u8; 78];
        assert_eq!(
            transport_of(DeviceKind::DualShock4V2, 78, &ds4),
            Some(Transport::BluetoothFull)
        );
        assert_eq!(transport_of(DeviceKind::DualSense, 78, &ds4), None);
    }

    #[test]
    fn an_empty_report_has_no_transport() {
        assert_eq!(transport_of(DeviceKind::DualSense, 64, &[]), None);
    }
}
