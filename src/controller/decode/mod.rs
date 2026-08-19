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

pub mod layout;

use crate::types::pad::{
    Battery, ButtonMask, DeviceKind, Motion, PadState, Stick, Touch, TouchPad, Transport, CIRCLE,
    CROSS, DPAD_DOWN, DPAD_LEFT, DPAD_RIGHT, DPAD_UP, FN_LEFT, FN_RIGHT, L1, L2, L3, MUTE, OPTIONS,
    PADDLE_LEFT, PADDLE_RIGHT, PS, R1, R2, R3, SHARE, SQUARE, TOUCHPAD, TRIANGLE,
};
use layout::{transport_of, BatteryLayout, Layout};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("report is empty")]
    Empty,

    #[error("report id {0:#04x} is not one this device sends")]
    UnknownReportId(u8),

    #[error("report is {got} bytes but this layout needs at least {need}")]
    TooShort { got: usize, need: usize },
}

pub fn decode(
    kind: DeviceKind,
    caps_report_len: usize,
    report: &[u8],
) -> Result<(Transport, PadState), DecodeError> {
    if report.is_empty() {
        return Err(DecodeError::Empty);
    }

    let transport = transport_of(kind, caps_report_len, report)
        .ok_or(DecodeError::UnknownReportId(report[0]))?;

    let state = decode_as(kind, transport, report)?;

    Ok((transport, state))
}

pub fn decode_as(
    kind: DeviceKind,
    transport: Transport,
    report: &[u8],
) -> Result<PadState, DecodeError> {
    let layout = layout::layout_for(kind, transport);

    if !layout.fits(report) {
        return Err(DecodeError::TooShort {
            got: report.len(),
            need: layout.min_len,
        });
    }

    Ok(PadState {
        buttons: buttons(kind, &layout, report),
        left_stick: Stick {
            x: report[layout.sticks],
            y: report[layout.sticks + 1],
        },
        right_stick: Stick {
            x: report[layout.sticks + 2],
            y: report[layout.sticks + 3],
        },
        left_trigger: report[layout.triggers],
        right_trigger: report[layout.triggers + 1],
        touch: touch(&layout, report),
        motion: motion(&layout, report),
        battery: battery(&layout, report),
        frame_counter: layout.frame_counter.map(|at| report[at] % 128).unwrap_or(0),
    })
}

fn buttons(kind: DeviceKind, layout: &Layout, report: &[u8]) -> ButtonMask {
    let faces = report[layout.faces];
    let shoulders = report[layout.shoulders];
    let misc = report[layout.misc];

    let mut mask = dpad_to_mask(faces & 0x0F);

    for (bit, button) in [(4, SQUARE), (5, CROSS), (6, CIRCLE), (7, TRIANGLE)] {
        if faces & (1 << bit) != 0 {
            mask |= button;
        }
    }

    for (bit, button) in [
        (0, L1),
        (1, R1),
        (2, L2),
        (3, R2),
        (4, SHARE),
        (5, OPTIONS),
        (6, L3),
        (7, R3),
    ] {
        if shoulders & (1 << bit) != 0 {
            mask |= button;
        }
    }

    if misc & (1 << 0) != 0 {
        mask |= PS;
    }

    if misc & (1 << 1) != 0 {
        mask |= TOUCHPAD;
    }

    if layout.extended_buttons && kind.has_mute_button() {
        for (bit, button) in [
            (2, MUTE),
            (4, FN_LEFT),
            (5, FN_RIGHT),
            (6, PADDLE_LEFT),
            (7, PADDLE_RIGHT),
        ] {
            if misc & (1 << bit) != 0 {
                mask |= button;
            }
        }
    }

    mask
}

pub fn dpad_to_mask(dpad: u8) -> ButtonMask {
    match dpad {
        0 => DPAD_UP,
        1 => DPAD_UP | DPAD_RIGHT,
        2 => DPAD_RIGHT,
        3 => DPAD_DOWN | DPAD_RIGHT,
        4 => DPAD_DOWN,
        5 => DPAD_DOWN | DPAD_LEFT,
        6 => DPAD_LEFT,
        7 => DPAD_UP | DPAD_LEFT,
        _ => 0,
    }
}

pub fn mask_to_dpad(mask: ButtonMask) -> u8 {
    let directions = mask & (DPAD_UP | DPAD_DOWN | DPAD_LEFT | DPAD_RIGHT);

    (0..8u8)
        .find(|dpad| dpad_to_mask(*dpad) == directions)
        .unwrap_or(8)
}

fn motion(layout: &Layout, report: &[u8]) -> Motion {
    let (Some(gyro), Some(accel)) = (layout.gyro, layout.accel) else {
        return Motion::default();
    };

    if report.len() < accel + 6 {
        return Motion::default();
    }

    Motion {
        gyro_pitch: le_i16(report, gyro),
        gyro_yaw: le_i16(report, gyro + 2),
        gyro_roll: le_i16(report, gyro + 4),
        accel_x: le_i16(report, accel),
        accel_y: le_i16(report, accel + 2),
        accel_z: le_i16(report, accel + 4),
    }
}

fn le_i16(report: &[u8], at: usize) -> i16 {
    i16::from_le_bytes([report[at], report[at + 1]])
}

fn touch(layout: &Layout, report: &[u8]) -> TouchPad {
    let Some(at) = layout.touch else {
        return TouchPad::default();
    };

    if report.len() < at + 9 {
        return TouchPad::default();
    }

    TouchPad {
        packet_counter: report[at],
        first: finger(report, at + 1),
        second: finger(report, at + 5),
    }
}

fn finger(report: &[u8], at: usize) -> Touch {
    Touch {
        active: report[at] & 0x80 == 0,
        id: report[at] & 0x7F,
        x: u16::from(report[at + 1]) | (u16::from(report[at + 2] & 0x0F) << 8),
        y: (u16::from(report[at + 2]) >> 4) | (u16::from(report[at + 3]) << 4),
    }
}

fn battery(layout: &Layout, report: &[u8]) -> Option<Battery> {
    let spec = layout.battery?;

    if report.len() <= spec.level.max(spec.charging) {
        return None;
    }

    Some(read_battery(
        &spec,
        report[spec.level],
        report[spec.charging],
    ))
}

fn read_battery(spec: &BatteryLayout, level_byte: u8, charging_byte: u8) -> Battery {
    let charging = charging_byte & spec.charging_mask != 0;
    let full = spec.full_mask != 0 && level_byte & spec.full_mask != 0;

    let max = match charging {
        true => spec.max_when_charging,
        false => spec.max_when_discharging,
    };

    let percent = match full {
        true => 100,
        false => (u32::from(level_byte & 0x0F) * 100 / max).min(100) as u8,
    };

    Battery {
        percent,
        charging,
        full,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dualsense_usb_report() -> Vec<u8> {
        let mut report = vec![0u8; 64];
        report[0] = 0x01;
        report[1] = 128;
        report[2] = 128;
        report[3] = 128;
        report[4] = 128;
        report[8] = 0x08;
        report
    }

    #[test]
    fn a_neutral_report_decodes_to_no_buttons_and_centred_sticks() {
        let report = dualsense_usb_report();

        let (transport, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert_eq!(transport, Transport::Usb);
        assert_eq!(state.buttons, 0);
        assert_eq!(state.left_stick, Stick::centred());
        assert_eq!(state.right_stick, Stick::centred());
    }

    #[test]
    fn circle_sets_only_the_circle_bit() {
        let mut report = dualsense_usb_report();
        report[8] = 0x08 | (1 << 6);

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert!(state.is_down(CIRCLE));
        assert!(!state.is_down(CROSS));
        assert_eq!(state.held_names(), vec!["Circle"]);
    }

    #[test]
    fn the_dpad_nibble_is_a_clock_and_eight_means_centred() {
        let mut report = dualsense_usb_report();

        report[8] = 0;
        let (_, up) = decode(DeviceKind::DualSense, 64, &report).unwrap();
        assert!(up.is_down(DPAD_UP));

        report[8] = 3;
        let (_, down_right) = decode(DeviceKind::DualSense, 64, &report).unwrap();
        assert!(down_right.is_down(DPAD_DOWN));
        assert!(down_right.is_down(DPAD_RIGHT));

        report[8] = 8;
        let (_, centred) = decode(DeviceKind::DualSense, 64, &report).unwrap();
        assert_eq!(centred.buttons, 0);
    }

    #[test]
    fn every_dpad_direction_survives_a_round_trip_through_the_mask() {
        for dpad in 0..=8u8 {
            assert_eq!(mask_to_dpad(dpad_to_mask(dpad)), dpad);
        }
    }

    #[test]
    fn dualsense_triggers_are_read_before_the_button_bytes() {
        let mut report = dualsense_usb_report();
        report[5] = 200;
        report[6] = 55;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert_eq!(state.left_trigger, 200);
        assert_eq!(state.right_trigger, 55);
    }

    #[test]
    fn dualshock4_triggers_are_read_after_the_button_bytes() {
        let mut report = vec![0u8; 64];
        report[0] = 0x01;
        report[5] = 0x08;
        report[8] = 200;
        report[9] = 55;

        let (_, state) = decode(DeviceKind::DualShock4V2, 64, &report).unwrap();

        assert_eq!(state.left_trigger, 200);
        assert_eq!(state.right_trigger, 55);
    }

    #[test]
    fn a_dualshock4_never_reports_mute_because_it_has_no_mute_button() {
        let mut report = vec![0u8; 64];
        report[0] = 0x01;
        report[5] = 0x08;
        report[7] = 0xFF;

        let (_, state) = decode(DeviceKind::DualShock4V2, 64, &report).unwrap();

        assert!(!state.is_down(MUTE));
        assert!(state.is_down(PS));
        assert!(state.is_down(TOUCHPAD));
    }

    #[test]
    fn a_bluetooth_basic_dualsense_never_reports_mute_because_that_byte_is_a_counter() {
        let mut report = vec![0u8; 78];
        report[0] = 0x01;
        report[5] = 0x08;
        report[7] = 0xFC;

        let (transport, state) = decode(DeviceKind::DualSense, 78, &report).unwrap();

        assert_eq!(transport, Transport::BluetoothBasic);
        assert!(!state.is_down(MUTE));
        assert!(!state.is_down(FN_LEFT));
        assert!(!state.is_down(FN_RIGHT));
        assert!(!state.is_down(PADDLE_LEFT));
        assert!(!state.is_down(PADDLE_RIGHT));
    }

    #[test]
    fn the_packet_counter_climbing_never_invents_a_button_press() {
        let mut previous = 0u32;

        for counter in 0..64u8 {
            let mut report = vec![0u8; 78];
            report[0] = 0x01;
            report[5] = 0x08;
            report[7] = counter << 2;

            let (_, state) = decode(DeviceKind::DualSense, 78, &report).unwrap();

            assert_eq!(state.buttons, 0, "counter {counter} invented a press");
            previous |= state.buttons;
        }

        assert_eq!(previous, 0);
    }

    #[test]
    fn a_dualsense_does_report_mute_from_the_same_bit() {
        let mut report = dualsense_usb_report();
        report[10] = 1 << 2;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert!(state.is_down(MUTE));
    }

    #[test]
    fn a_full_battery_reads_one_hundred_rather_than_the_raw_nibble() {
        let mut report = dualsense_usb_report();
        report[53] = 0x20 | 0x04;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert_eq!(state.battery.unwrap().percent, 100);
        assert!(state.battery.unwrap().full);
    }

    #[test]
    fn a_charging_dualsense_is_reported_as_charging() {
        let mut report = dualsense_usb_report();
        report[53] = 0x04;
        report[54] = 0x08;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert!(state.battery.unwrap().charging);
        assert_eq!(state.battery.unwrap().percent, 50);
    }

    #[test]
    fn a_finger_is_active_when_the_top_bit_is_clear() {
        let mut report = dualsense_usb_report();
        report[34] = 0x05;
        report[35] = 0x20;
        report[36] = 0x31;
        report[37] = 0x04;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert!(state.touch.first.active);
        assert_eq!(state.touch.first.id, 5);
        assert_eq!(state.touch.first.x, 0x120);
        assert_eq!(state.touch.first.y, 0x43);
    }

    #[test]
    fn a_finger_lifted_off_has_the_top_bit_set_and_reads_inactive() {
        let mut report = dualsense_usb_report();
        report[34] = 0x80;

        let (_, state) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert!(!state.touch.first.active);
    }

    #[test]
    fn a_bluetooth_pad_padded_to_seventy_eight_bytes_decodes_with_the_basic_layout() {
        let mut report = vec![0u8; 78];
        report[0] = 0x01;
        report[5] = 0x08 | (1 << 6);
        report[8] = 200;

        let (transport, state) = decode(DeviceKind::DualSense, 78, &report).unwrap();

        assert_eq!(transport, Transport::BluetoothBasic);
        assert!(state.is_down(CIRCLE));
        assert_eq!(state.left_trigger, 200);
    }

    #[test]
    fn the_same_bytes_read_as_usb_would_decode_to_something_different() {
        let mut report = vec![0u8; 78];
        report[0] = 0x01;
        report[5] = 0x08 | (1 << 6);
        report[8] = 200;

        let (_, as_bluetooth) = decode(DeviceKind::DualSense, 78, &report).unwrap();
        let (_, as_usb) = decode(DeviceKind::DualSense, 64, &report).unwrap();

        assert_ne!(as_bluetooth.buttons, as_usb.buttons);
    }

    #[test]
    fn a_bluetooth_full_report_decodes_the_same_press_as_usb() {
        let mut usb = dualsense_usb_report();
        usb[8] = 0x08 | (1 << 5);

        let mut bt = vec![0u8; 78];
        bt[0] = 0x31;
        bt[2] = 128;
        bt[3] = 128;
        bt[4] = 128;
        bt[5] = 128;
        bt[9] = 0x08 | (1 << 5);

        let (_, from_usb) = decode(DeviceKind::DualSense, 64, &usb).unwrap();
        let (transport, from_bt) = decode(DeviceKind::DualSense, 64, &bt).unwrap();

        assert_eq!(transport, Transport::BluetoothFull);
        assert!(from_usb.is_down(CROSS));
        assert_eq!(from_bt.buttons, from_usb.buttons);
        assert_eq!(from_bt.left_stick, from_usb.left_stick);
    }

    #[test]
    fn a_short_report_is_an_error_and_not_a_panic() {
        let report = [0x01u8, 0x00, 0x00];

        let error = decode(DeviceKind::DualSense, 64, &report).unwrap_err();

        assert!(matches!(error, DecodeError::TooShort { .. }));
    }

    #[test]
    fn an_empty_report_is_an_error_and_not_a_panic() {
        assert_eq!(
            decode(DeviceKind::DualSense, 64, &[]).unwrap_err(),
            DecodeError::Empty
        );
    }

    #[test]
    fn an_unknown_report_id_names_the_id_it_saw() {
        let report = [0x99u8; 64];

        assert_eq!(
            decode(DeviceKind::DualSense, 64, &report).unwrap_err(),
            DecodeError::UnknownReportId(0x99)
        );
    }

    #[test]
    fn decoding_never_panics_on_any_length_up_to_a_full_report() {
        for kind in [DeviceKind::DualSense, DeviceKind::DualShock4V2] {
            for id in [0x01u8, 0x11, 0x31] {
                for len in 0..80usize {
                    let mut report = vec![0u8; len];
                    if len > 0 {
                        report[0] = id;
                    }
                    let _ = decode(kind, 64, &report);
                }
            }
        }
    }

    #[test]
    fn pressed_since_reports_only_the_newly_held_buttons() {
        let previous = PadState {
            buttons: CROSS,
            ..PadState::default()
        };

        let current = PadState {
            buttons: CROSS | CIRCLE,
            ..PadState::default()
        };

        assert_eq!(current.pressed_since(&previous), CIRCLE);
        assert_eq!(current.released_since(&previous), 0);
        assert_eq!(previous.released_since(&current), CIRCLE);
    }
}
