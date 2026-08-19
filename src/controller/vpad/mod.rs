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

use crate::controller::decode::mask_to_dpad;
use crate::types::pad::{
    ButtonMask, PadState, CIRCLE, CROSS, L1, L3, OPTIONS, PS, R1, R3, SHARE, SQUARE, TRIANGLE,
};

pub const XBOX_VENDOR: u16 = 0x045E;
pub const XBOX_ONE_PRODUCT: u16 = 0x02FF;

pub const XINPUT_HARDWARE_ID: &str = "HID\\VID_045E&PID_02FF&IG_00";

pub const REPORT_LEN: usize = 17;

pub const GAMEPAD_USAGE_PAGE: u16 = 0x01;
pub const GAMEPAD_USAGE: u16 = 0x05;

pub const REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, 0x09, 0x05, 0xA1, 0x01, 0xA1, 0x00, 0x09, 0x30, 0x09, 0x31, 0x15, 0x00, 0x27, 0xFF,
    0xFF, 0x00, 0x00, 0x95, 0x02, 0x75, 0x10, 0x81, 0x02, 0xC0, 0xA1, 0x00, 0x09, 0x33, 0x09, 0x34,
    0x15, 0x00, 0x27, 0xFF, 0xFF, 0x00, 0x00, 0x95, 0x02, 0x75, 0x10, 0x81, 0x02, 0xC0, 0x05, 0x01,
    0x09, 0x32, 0x15, 0x00, 0x26, 0xFF, 0x03, 0x95, 0x01, 0x75, 0x0A, 0x81, 0x02, 0x15, 0x00, 0x25,
    0x00, 0x75, 0x06, 0x95, 0x01, 0x81, 0x03, 0x05, 0x01, 0x09, 0x35, 0x15, 0x00, 0x26, 0xFF, 0x03,
    0x95, 0x01, 0x75, 0x0A, 0x81, 0x02, 0x15, 0x00, 0x25, 0x00, 0x75, 0x06, 0x95, 0x01, 0x81, 0x03,
    0x05, 0x09, 0x19, 0x01, 0x29, 0x0A, 0x95, 0x0A, 0x75, 0x01, 0x81, 0x02, 0x15, 0x00, 0x25, 0x00,
    0x75, 0x06, 0x95, 0x01, 0x81, 0x03, 0x05, 0x01, 0x09, 0x39, 0x15, 0x01, 0x25, 0x08, 0x35, 0x00,
    0x46, 0x3B, 0x01, 0x66, 0x14, 0x00, 0x75, 0x04, 0x95, 0x01, 0x81, 0x42, 0x75, 0x04, 0x95, 0x01,
    0x15, 0x00, 0x25, 0x00, 0x35, 0x00, 0x45, 0x00, 0x65, 0x00, 0x81, 0x03, 0xA1, 0x02, 0x05, 0x0F,
    0x09, 0x97, 0x15, 0x00, 0x25, 0x01, 0x75, 0x04, 0x95, 0x01, 0x91, 0x02, 0x15, 0x00, 0x25, 0x00,
    0x91, 0x03, 0x09, 0x70, 0x15, 0x00, 0x25, 0x64, 0x75, 0x08, 0x95, 0x04, 0x91, 0x02, 0x09, 0x50,
    0x66, 0x01, 0x10, 0x55, 0x0E, 0x26, 0xFF, 0x00, 0x95, 0x01, 0x91, 0x02, 0x09, 0xA7, 0x91, 0x02,
    0x65, 0x00, 0x55, 0x00, 0x09, 0x7C, 0x91, 0x02, 0xC0, 0x05, 0x01, 0x09, 0x80, 0xA1, 0x00, 0x09,
    0x85, 0x15, 0x00, 0x25, 0x01, 0x95, 0x01, 0x75, 0x01, 0x81, 0x02, 0x15, 0x00, 0x25, 0x00, 0x75,
    0x07, 0x95, 0x01, 0x81, 0x03, 0xC0, 0x05, 0x06, 0x09, 0x20, 0x15, 0x00, 0x26, 0xFF, 0x00, 0x75,
    0x08, 0x95, 0x01, 0x81, 0x02, 0xC0,
];

const A: u16 = 1 << 0;
const B: u16 = 1 << 1;
const X: u16 = 1 << 2;
const Y: u16 = 1 << 3;
const LB: u16 = 1 << 4;
const RB: u16 = 1 << 5;
const BACK: u16 = 1 << 6;
const MENU: u16 = 1 << 7;
const LS: u16 = 1 << 8;
const RS: u16 = 1 << 9;

const SONY_TO_XBOX: &[(ButtonMask, u16)] = &[
    (CROSS, A),
    (CIRCLE, B),
    (SQUARE, X),
    (TRIANGLE, Y),
    (L1, LB),
    (R1, RB),
    (SHARE, BACK),
    (OPTIONS, MENU),
    (L3, LS),
    (R3, RS),
];

pub fn xbox_buttons(sony: ButtonMask) -> u16 {
    SONY_TO_XBOX
        .iter()
        .filter(|(from, _)| sony & from != 0)
        .fold(0u16, |mask, (_, to)| mask | to)
}

pub fn widen_axis(byte: u8) -> u16 {
    u16::from(byte) * 257
}

pub fn widen_trigger(byte: u8) -> u16 {
    ((u32::from(byte) * 1023 / 255) as u16) & 0x03FF
}

pub fn widen_battery(percent: u8) -> u8 {
    (u16::from(percent) * 255 / 100).min(255) as u8
}

pub fn hat_from(buttons: ButtonMask) -> u8 {
    match mask_to_dpad(buttons) {
        8 => 0,
        clockwise => clockwise + 1,
    }
}

pub fn pack(state: &PadState) -> [u8; REPORT_LEN] {
    let mut report = [0u8; REPORT_LEN];

    report[0..2].copy_from_slice(&widen_axis(state.left_stick.x).to_le_bytes());
    report[2..4].copy_from_slice(&widen_axis(state.left_stick.y).to_le_bytes());
    report[4..6].copy_from_slice(&widen_axis(state.right_stick.x).to_le_bytes());
    report[6..8].copy_from_slice(&widen_axis(state.right_stick.y).to_le_bytes());

    report[8..10].copy_from_slice(&widen_trigger(state.left_trigger).to_le_bytes());
    report[10..12].copy_from_slice(&widen_trigger(state.right_trigger).to_le_bytes());

    let buttons = xbox_buttons(state.buttons);

    report[12] = (buttons & 0xFF) as u8;
    report[13] = ((buttons >> 8) & 0x03) as u8;
    report[14] = hat_from(state.buttons) & 0x0F;
    report[15] = match state.is_down(PS) {
        true => 1,
        false => 0,
    };

    report[16] = state.battery.map(|b| widen_battery(b.percent)).unwrap_or(0);

    report
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rumble {
    pub left_motor: u8,
    pub right_motor: u8,
    pub left_trigger_motor: u8,
    pub right_trigger_motor: u8,
}

pub fn parse_rumble(output_report: &[u8]) -> Option<Rumble> {
    if output_report.len() < 5 {
        return None;
    }

    Some(Rumble {
        left_trigger_motor: output_report[1],
        right_trigger_motor: output_report[2],
        left_motor: output_report[3],
        right_motor: output_report[4],
    })
}

pub fn descriptor_is_a_gamepad(descriptor: &[u8]) -> bool {
    let mut at = 0;

    while at + 3 < descriptor.len() {
        let page_is_generic_desktop = descriptor[at] == 0x05 && descriptor[at + 1] == 0x01;
        let usage_is_gamepad = descriptor[at + 2] == 0x09 && descriptor[at + 3] == 0x05;

        if page_is_generic_desktop && usage_is_gamepad {
            return true;
        }

        at += 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::pad::{Battery, Stick, DPAD_DOWN, DPAD_LEFT, DPAD_RIGHT, DPAD_UP, TOUCHPAD};

    fn state_with(buttons: ButtonMask) -> PadState {
        PadState {
            buttons,
            ..PadState::default()
        }
    }

    #[test]
    fn the_hardware_id_is_the_one_windows_binds_its_xinput_driver_to() {
        assert_eq!(XINPUT_HARDWARE_ID, "HID\\VID_045E&PID_02FF&IG_00");
        assert!(XINPUT_HARDWARE_ID.contains("IG_00"));
    }

    #[test]
    fn the_descriptor_declares_a_gamepad_so_the_driver_will_accept_it() {
        assert!(descriptor_is_a_gamepad(REPORT_DESCRIPTOR));
    }

    #[test]
    fn a_keyboard_descriptor_is_refused_because_that_is_what_guards_the_acl() {
        let keyboard = [0x05u8, 0x01, 0x09, 0x06, 0xA1, 0x01, 0xC0];

        assert!(!descriptor_is_a_gamepad(&keyboard));
    }

    #[test]
    fn a_mouse_descriptor_is_refused_too() {
        let mouse = [0x05u8, 0x01, 0x09, 0x02, 0xA1, 0x01, 0xC0];

        assert!(!descriptor_is_a_gamepad(&mouse));
    }

    #[test]
    fn an_empty_descriptor_is_refused_rather_than_panicking() {
        assert!(!descriptor_is_a_gamepad(&[]));
        assert!(!descriptor_is_a_gamepad(&[0x05]));
    }

    #[test]
    fn the_descriptor_opens_a_collection_and_closes_every_one_it_opens() {
        let opened = REPORT_DESCRIPTOR
            .windows(2)
            .filter(|pair| pair[0] == 0xA1)
            .count();
        let closed = REPORT_DESCRIPTOR
            .iter()
            .filter(|byte| **byte == 0xC0)
            .count();

        assert!(opened > 0);
        assert_eq!(opened, closed);
    }

    #[test]
    fn cross_becomes_a_and_circle_becomes_b() {
        assert_eq!(xbox_buttons(CROSS), A);
        assert_eq!(xbox_buttons(CIRCLE), B);
        assert_eq!(xbox_buttons(SQUARE), X);
        assert_eq!(xbox_buttons(TRIANGLE), Y);
    }

    #[test]
    fn a_button_the_xbox_pad_does_not_have_maps_to_nothing() {
        assert_eq!(xbox_buttons(TOUCHPAD), 0);
    }

    #[test]
    fn two_buttons_at_once_set_two_bits() {
        assert_eq!(xbox_buttons(CROSS | CIRCLE), A | B);
    }

    #[test]
    fn a_centred_stick_widens_to_the_middle_of_the_sixteen_bit_range() {
        assert_eq!(widen_axis(128), 32896);
        assert_eq!(widen_axis(0), 0);
        assert_eq!(widen_axis(255), 65535);
    }

    #[test]
    fn a_full_trigger_widens_to_the_top_of_the_ten_bit_range() {
        assert_eq!(widen_trigger(0), 0);
        assert_eq!(widen_trigger(255), 1023);
        assert!(widen_trigger(255) <= 0x03FF);
    }

    #[test]
    fn no_trigger_value_ever_overflows_ten_bits() {
        for raw in 0..=255u8 {
            assert!(widen_trigger(raw) <= 0x03FF);
        }
    }

    #[test]
    fn the_hat_is_one_based_with_zero_meaning_centred() {
        assert_eq!(hat_from(0), 0);
        assert_eq!(hat_from(DPAD_UP), 1);
        assert_eq!(hat_from(DPAD_RIGHT), 3);
        assert_eq!(hat_from(DPAD_DOWN), 5);
        assert_eq!(hat_from(DPAD_LEFT), 7);
    }

    #[test]
    fn a_diagonal_press_reads_as_a_diagonal_hat_position() {
        assert_eq!(hat_from(DPAD_UP | DPAD_RIGHT), 2);
        assert_eq!(hat_from(DPAD_DOWN | DPAD_LEFT), 6);
    }

    #[test]
    fn the_hat_never_leaves_the_range_the_descriptor_declares() {
        for buttons in [
            0,
            DPAD_UP,
            DPAD_DOWN,
            DPAD_LEFT,
            DPAD_RIGHT,
            DPAD_UP | DPAD_LEFT,
            DPAD_UP | DPAD_DOWN,
        ] {
            assert!(hat_from(buttons) <= 8);
        }
    }

    #[test]
    fn a_packed_report_is_exactly_the_length_the_descriptor_implies() {
        assert_eq!(pack(&PadState::default()).len(), REPORT_LEN);
    }

    #[test]
    fn a_neutral_pad_packs_to_centred_sticks_and_no_buttons() {
        let report = pack(&PadState::default());

        assert_eq!(u16::from_le_bytes([report[0], report[1]]), 32896);
        assert_eq!(u16::from_le_bytes([report[2], report[3]]), 32896);
        assert_eq!(report[12], 0);
        assert_eq!(report[13], 0);
        assert_eq!(report[14], 0);
    }

    #[test]
    fn holding_cross_sets_the_a_bit_in_the_packed_report() {
        let report = pack(&state_with(CROSS));

        assert_eq!(report[12] & 0x01, 0x01);
    }

    #[test]
    fn the_two_high_buttons_land_in_the_second_button_byte() {
        let report = pack(&state_with(L3 | R3));

        assert_eq!(report[12], 0);
        assert_eq!(report[13], 0x03);
    }

    #[test]
    fn the_ps_button_is_reported_as_the_system_menu_and_not_as_a_face_button() {
        let report = pack(&state_with(PS));

        assert_eq!(report[15], 1);
        assert_eq!(report[12], 0);
        assert_eq!(report[13], 0);
    }

    #[test]
    fn a_trigger_pull_survives_the_round_trip_into_the_packed_report() {
        let state = PadState {
            left_trigger: 255,
            right_trigger: 0,
            ..PadState::default()
        };

        let report = pack(&state);

        assert_eq!(u16::from_le_bytes([report[8], report[9]]), 1023);
        assert_eq!(u16::from_le_bytes([report[10], report[11]]), 0);
    }

    #[test]
    fn a_stick_pushed_fully_left_packs_to_zero_and_fully_right_packs_to_the_maximum() {
        let left = PadState {
            left_stick: Stick { x: 0, y: 128 },
            ..PadState::default()
        };
        let right = PadState {
            left_stick: Stick { x: 255, y: 128 },
            ..PadState::default()
        };

        assert_eq!(u16::from_le_bytes([pack(&left)[0], pack(&left)[1]]), 0);
        assert_eq!(
            u16::from_le_bytes([pack(&right)[0], pack(&right)[1]]),
            65535
        );
    }

    #[test]
    fn packing_never_panics_for_any_button_combination() {
        for bit in 0..23u32 {
            let _ = pack(&state_with(1 << bit));
        }
    }

    #[test]
    fn the_packed_report_is_exactly_as_long_as_the_descriptor_declares() {
        let x_and_y: usize = 16 + 16;
        let rx_and_ry: usize = 16 + 16;
        let z_with_padding: usize = 10 + 6;
        let rz_with_padding: usize = 10 + 6;
        let buttons_with_padding: usize = 10 + 6;
        let hat_with_padding: usize = 4 + 4;
        let sys_menu_with_padding: usize = 1 + 7;
        let battery: usize = 8;

        let bits = x_and_y
            + rx_and_ry
            + z_with_padding
            + rz_with_padding
            + buttons_with_padding
            + hat_with_padding
            + sys_menu_with_padding
            + battery;

        assert_eq!(bits.div_ceil(8), REPORT_LEN);
        assert_eq!(REPORT_LEN, 17);
    }

    #[test]
    fn a_pad_with_no_battery_reading_packs_the_battery_byte_to_zero() {
        let report = pack(&PadState::default());

        assert_eq!(report[16], 0);
    }

    #[test]
    fn a_full_battery_widens_to_the_top_of_the_eight_bit_range() {
        assert_eq!(widen_battery(100), 255);
        assert_eq!(widen_battery(0), 0);
        assert_eq!(widen_battery(50), 127);
    }

    #[test]
    fn a_known_battery_reading_reaches_the_last_byte_of_the_packed_report() {
        let state = PadState {
            battery: Some(Battery {
                percent: 100,
                charging: false,
                full: true,
            }),
            ..PadState::default()
        };

        assert_eq!(pack(&state)[16], 255);
    }

    #[test]
    fn rumble_is_read_out_of_the_output_report_the_game_sends() {
        let output = [0x00u8, 10, 20, 30, 40];

        let rumble = parse_rumble(&output).unwrap();

        assert_eq!(rumble.left_trigger_motor, 10);
        assert_eq!(rumble.right_trigger_motor, 20);
        assert_eq!(rumble.left_motor, 30);
        assert_eq!(rumble.right_motor, 40);
    }

    #[test]
    fn a_truncated_output_report_yields_no_rumble_rather_than_panicking() {
        assert!(parse_rumble(&[0x00, 1, 2]).is_none());
        assert!(parse_rumble(&[]).is_none());
    }
}
