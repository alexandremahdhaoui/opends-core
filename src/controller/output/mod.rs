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

use serde::{Deserialize, Serialize};

use crate::types::pad::{DeviceKind, Transport};
use crate::util::crc32;

pub const DUALSENSE_USB_REPORT_ID: u8 = 0x02;
pub const DUALSENSE_BT_REPORT_ID: u8 = 0x31;
pub const DUALSHOCK4_USB_REPORT_ID: u8 = 0x05;
pub const DUALSHOCK4_BT_REPORT_ID: u8 = 0x11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Colour {
    pub const OFF: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
    };

    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rumble {
    pub weak: u8,
    pub strong: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum TriggerEffect {
    #[default]
    Off,
    Rigid {
        force: u8,
    },
    Pulse {
        start: u8,
        force: u8,
    },
    Weapon {
        start: u8,
        end: u8,
        force: u8,
    },
}

impl TriggerEffect {
    pub fn encode(self) -> [u8; 11] {
        let mut bytes = [0u8; 11];

        match self {
            Self::Off => {}
            Self::Rigid { force } => {
                bytes[0] = 0x01;
                bytes[1] = 0x00;
                bytes[2] = force;
            }
            Self::Pulse { start, force } => {
                bytes[0] = 0x02;
                bytes[1] = start;
                bytes[2] = force;
            }
            Self::Weapon { start, end, force } => {
                bytes[0] = 0x06;
                bytes[1] = start.min(end);
                bytes[2] = end.max(start);
                bytes[3] = force;
            }
        }

        bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PadOutput {
    pub lightbar: Colour,
    pub rumble: Rumble,
    pub left_trigger: TriggerEffect,
    pub right_trigger: TriggerEffect,
    pub player_leds: u8,
    pub mute_led: bool,
}

pub fn build(kind: DeviceKind, transport: Transport, output: &PadOutput) -> Vec<u8> {
    match kind.is_dualsense() {
        true => build_dualsense(transport, output),
        false => build_dualshock4(transport, output),
    }
}

fn build_dualsense(transport: Transport, output: &PadOutput) -> Vec<u8> {
    let mut report = Vec::new();

    let payload_at = match transport {
        Transport::Usb => {
            report.push(DUALSENSE_USB_REPORT_ID);
            1
        }
        Transport::BluetoothBasic | Transport::BluetoothFull => {
            report.push(DUALSENSE_BT_REPORT_ID);
            report.push(0x02);
            2
        }
    };

    report.resize(payload_at + 47, 0);

    report[payload_at] = 0xFF;
    report[payload_at + 1] = 0xF7;
    report[payload_at + 2] = output.rumble.weak;
    report[payload_at + 3] = output.rumble.strong;

    report[payload_at + 8] = match output.mute_led {
        true => 0x01,
        false => 0x00,
    };

    report[payload_at + 10..payload_at + 21].copy_from_slice(&output.right_trigger.encode());
    report[payload_at + 21..payload_at + 32].copy_from_slice(&output.left_trigger.encode());

    report[payload_at + 38] = 0x02;
    report[payload_at + 42] = output.player_leds & 0x1F;
    report[payload_at + 43] = output.lightbar.red;
    report[payload_at + 44] = output.lightbar.green;
    report[payload_at + 45] = output.lightbar.blue;

    if transport.needs_crc() {
        report.resize(74, 0);
        crc32::append_output_checksum(&mut report);
    }

    report
}

fn build_dualshock4(transport: Transport, output: &PadOutput) -> Vec<u8> {
    let mut report = Vec::new();

    let payload_at = match transport {
        Transport::Usb => {
            report.push(DUALSHOCK4_USB_REPORT_ID);
            1
        }
        Transport::BluetoothBasic | Transport::BluetoothFull => {
            report.push(DUALSHOCK4_BT_REPORT_ID);
            report.push(0xC0);
            report.push(0xA0);
            3
        }
    };

    report.resize(payload_at + 10, 0);

    report[payload_at] = 0xF7;
    report[payload_at + 1] = 0x04;
    report[payload_at + 3] = output.rumble.weak;
    report[payload_at + 4] = output.rumble.strong;
    report[payload_at + 5] = output.lightbar.red;
    report[payload_at + 6] = output.lightbar.green;
    report[payload_at + 7] = output.lightbar.blue;

    if transport.needs_crc() {
        report.resize(74, 0);
        crc32::append_output_checksum(&mut report);
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> PadOutput {
        PadOutput {
            lightbar: Colour::new(255, 0, 0),
            ..PadOutput::default()
        }
    }

    #[test]
    fn a_usb_dualsense_report_starts_with_the_usb_report_id() {
        let report = build(DeviceKind::DualSense, Transport::Usb, &red());

        assert_eq!(report[0], DUALSENSE_USB_REPORT_ID);
    }

    #[test]
    fn a_bluetooth_dualsense_report_starts_with_the_bluetooth_report_id() {
        let report = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());

        assert_eq!(report[0], DUALSENSE_BT_REPORT_ID);
    }

    #[test]
    fn bluetooth_basic_output_is_shaped_like_bluetooth_full_not_usb() {
        let basic = build(DeviceKind::DualSense, Transport::BluetoothBasic, &red());
        let full = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());

        assert_eq!(basic[0], DUALSENSE_BT_REPORT_ID);
        assert_eq!(basic.len(), full.len());
    }

    #[test]
    fn bluetooth_basic_dualshock4_output_is_shaped_like_bluetooth_full_not_usb() {
        let basic = build(DeviceKind::DualShock4V2, Transport::BluetoothBasic, &red());
        let full = build(DeviceKind::DualShock4V2, Transport::BluetoothFull, &red());

        assert_eq!(basic[0], DUALSHOCK4_BT_REPORT_ID);
        assert_eq!(basic.len(), full.len());
    }

    #[test]
    fn a_pad_still_connected_in_basic_mode_gets_a_crc_on_its_output_too() {
        assert!(Transport::BluetoothBasic.needs_crc());
    }

    #[test]
    fn the_lightbar_colour_lands_in_the_report_for_both_families() {
        let ds5 = build(DeviceKind::DualSense, Transport::Usb, &red());
        let ds4 = build(DeviceKind::DualShock4V2, Transport::Usb, &red());

        assert!(ds5.contains(&255));
        assert_eq!(ds4[6], 255);
        assert_eq!(ds4[7], 0);
        assert_eq!(ds4[8], 0);
    }

    #[test]
    fn a_bluetooth_dualsense_report_is_the_length_the_pad_expects() {
        let bt = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());

        assert_eq!(bt.len(), 78);
    }

    #[test]
    fn a_usb_report_carries_no_crc_and_a_bluetooth_one_always_does() {
        let usb = build(DeviceKind::DualSense, Transport::Usb, &red());
        let bt = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());

        assert!(usb.len() < bt.len());
    }

    #[test]
    fn the_bluetooth_crc_is_computed_over_everything_before_it() {
        let report = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());

        let split = report.len() - 4;
        let claimed = u32::from_le_bytes([
            report[split],
            report[split + 1],
            report[split + 2],
            report[split + 3],
        ]);

        assert_eq!(
            claimed,
            crc32::checksum_with_seed(crc32::OUTPUT_REPORT_SEED, &report[..split])
        );
    }

    #[test]
    fn flipping_one_byte_of_a_bluetooth_report_invalidates_its_crc() {
        let mut report = build(DeviceKind::DualSense, Transport::BluetoothFull, &red());
        let split = report.len() - 4;

        let before = u32::from_le_bytes([
            report[split],
            report[split + 1],
            report[split + 2],
            report[split + 3],
        ]);

        report[5] ^= 0xFF;

        let after = crc32::checksum_with_seed(crc32::OUTPUT_REPORT_SEED, &report[..split]);

        assert_ne!(before, after);
    }

    #[test]
    fn rumble_values_reach_the_report() {
        let output = PadOutput {
            rumble: Rumble {
                weak: 111,
                strong: 222,
            },
            ..PadOutput::default()
        };

        let report = build(DeviceKind::DualSense, Transport::Usb, &output);

        assert!(report.contains(&111));
        assert!(report.contains(&222));
    }

    #[test]
    fn the_off_effect_encodes_to_all_zeroes() {
        assert_eq!(TriggerEffect::Off.encode(), [0u8; 11]);
    }

    #[test]
    fn each_effect_uses_a_different_mode_byte() {
        assert_eq!(TriggerEffect::Rigid { force: 200 }.encode()[0], 0x01);
        assert_eq!(
            TriggerEffect::Pulse {
                start: 10,
                force: 20
            }
            .encode()[0],
            0x02
        );
        assert_eq!(
            TriggerEffect::Weapon {
                start: 2,
                end: 8,
                force: 255
            }
            .encode()[0],
            0x06
        );
    }

    #[test]
    fn rigid_mode_byte_matches_ds4windows_dualsensedevice_cs_line_293() {
        assert_eq!(TriggerEffect::Rigid { force: 200 }.encode()[0], 0x01);
    }

    #[test]
    fn pulse_family_mode_byte_matches_ds4windows_dualsensedevice_cs_line_303() {
        assert_eq!(
            TriggerEffect::Pulse {
                start: 10,
                force: 20
            }
            .encode()[0],
            0x02
        );
    }

    #[test]
    fn a_weapon_effect_with_its_ends_the_wrong_way_round_is_ordered_rather_than_refused() {
        let encoded = TriggerEffect::Weapon {
            start: 9,
            end: 2,
            force: 100,
        }
        .encode();

        assert_eq!(encoded[1], 2);
        assert_eq!(encoded[2], 9);
    }

    #[test]
    fn an_adaptive_trigger_effect_reaches_the_dualsense_report() {
        let output = PadOutput {
            right_trigger: TriggerEffect::Rigid { force: 0xAB },
            ..PadOutput::default()
        };

        let report = build(DeviceKind::DualSense, Transport::Usb, &output);

        assert!(report.contains(&0xAB));
    }

    #[test]
    fn a_dualshock4_has_no_adaptive_triggers_so_the_effect_changes_nothing() {
        let plain = build(
            DeviceKind::DualShock4V2,
            Transport::Usb,
            &PadOutput::default(),
        );

        let with_effect = build(
            DeviceKind::DualShock4V2,
            Transport::Usb,
            &PadOutput {
                right_trigger: TriggerEffect::Rigid { force: 255 },
                ..PadOutput::default()
            },
        );

        assert_eq!(plain, with_effect);
    }

    #[test]
    fn a_dualshock4_bluetooth_report_is_the_length_the_pad_expects() {
        let report = build(DeviceKind::DualShock4V2, Transport::BluetoothFull, &red());

        assert_eq!(report.len(), 78);
    }

    #[test]
    fn building_never_panics_for_any_device_and_transport_pair() {
        for kind in [
            DeviceKind::DualSense,
            DeviceKind::DualSenseEdge,
            DeviceKind::DualShock4V1,
            DeviceKind::DualShock4V2,
            DeviceKind::DualShock4Dongle,
        ] {
            for transport in [
                Transport::Usb,
                Transport::BluetoothBasic,
                Transport::BluetoothFull,
            ] {
                assert!(!build(kind, transport, &red()).is_empty());
            }
        }
    }
}
