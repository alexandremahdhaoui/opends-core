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

pub const SONY_VENDOR: u16 = 0x054C;

pub const DUALSENSE: u16 = 0x0CE6;
pub const DUALSENSE_EDGE: u16 = 0x0DF2;
pub const DUALSHOCK4_V1: u16 = 0x05C4;
pub const DUALSHOCK4_V2: u16 = 0x09CC;
pub const DUALSHOCK4_DONGLE: u16 = 0x0BA0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceKind {
    DualSense,
    DualSenseEdge,
    DualShock4V1,
    DualShock4V2,
    DualShock4Dongle,
}

impl DeviceKind {
    pub fn from_ids(vendor: u16, product: u16) -> Option<Self> {
        if vendor != SONY_VENDOR {
            return None;
        }

        match product {
            DUALSENSE => Some(Self::DualSense),
            DUALSENSE_EDGE => Some(Self::DualSenseEdge),
            DUALSHOCK4_V1 => Some(Self::DualShock4V1),
            DUALSHOCK4_V2 => Some(Self::DualShock4V2),
            DUALSHOCK4_DONGLE => Some(Self::DualShock4Dongle),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::DualSense => "DualSense",
            Self::DualSenseEdge => "DualSense Edge",
            Self::DualShock4V1 => "DualShock 4",
            Self::DualShock4V2 => "DualShock 4 v2",
            Self::DualShock4Dongle => "DualShock 4 wireless adapter",
        }
    }

    pub fn is_dualsense(self) -> bool {
        matches!(self, Self::DualSense | Self::DualSenseEdge)
    }

    pub fn has_mute_button(self) -> bool {
        self.is_dualsense()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Usb,
    BluetoothBasic,
    BluetoothFull,
}

impl Transport {
    pub fn needs_crc(self) -> bool {
        !matches!(self, Self::Usb)
    }
}

pub type ButtonMask = u32;

pub const CROSS: ButtonMask = 1 << 0;
pub const CIRCLE: ButtonMask = 1 << 1;
pub const SQUARE: ButtonMask = 1 << 2;
pub const TRIANGLE: ButtonMask = 1 << 3;
pub const L1: ButtonMask = 1 << 4;
pub const R1: ButtonMask = 1 << 5;
pub const L2: ButtonMask = 1 << 6;
pub const R2: ButtonMask = 1 << 7;
pub const SHARE: ButtonMask = 1 << 8;
pub const OPTIONS: ButtonMask = 1 << 9;
pub const L3: ButtonMask = 1 << 10;
pub const R3: ButtonMask = 1 << 11;
pub const PS: ButtonMask = 1 << 12;
pub const TOUCHPAD: ButtonMask = 1 << 13;
pub const MUTE: ButtonMask = 1 << 14;
pub const DPAD_UP: ButtonMask = 1 << 15;
pub const DPAD_DOWN: ButtonMask = 1 << 16;
pub const DPAD_LEFT: ButtonMask = 1 << 17;
pub const DPAD_RIGHT: ButtonMask = 1 << 18;
pub const FN_LEFT: ButtonMask = 1 << 19;
pub const FN_RIGHT: ButtonMask = 1 << 20;
pub const PADDLE_LEFT: ButtonMask = 1 << 21;
pub const PADDLE_RIGHT: ButtonMask = 1 << 22;

pub const ALL_BUTTONS: &[(ButtonMask, &str)] = &[
    (CROSS, "Cross"),
    (CIRCLE, "Circle"),
    (SQUARE, "Square"),
    (TRIANGLE, "Triangle"),
    (L1, "L1"),
    (R1, "R1"),
    (L2, "L2"),
    (R2, "R2"),
    (SHARE, "Share"),
    (OPTIONS, "Options"),
    (L3, "L3"),
    (R3, "R3"),
    (PS, "PS"),
    (TOUCHPAD, "Touchpad"),
    (MUTE, "Mute"),
    (DPAD_UP, "DpadUp"),
    (DPAD_DOWN, "DpadDown"),
    (DPAD_LEFT, "DpadLeft"),
    (DPAD_RIGHT, "DpadRight"),
    (FN_LEFT, "FnL"),
    (FN_RIGHT, "FnR"),
    (PADDLE_LEFT, "PaddleL"),
    (PADDLE_RIGHT, "PaddleR"),
];

pub fn button_name(mask: ButtonMask) -> Option<&'static str> {
    ALL_BUTTONS
        .iter()
        .find(|(bit, _)| *bit == mask)
        .map(|(_, name)| *name)
}

pub fn parse_button_name(name: &str) -> Option<ButtonMask> {
    ALL_BUTTONS
        .iter()
        .find(|(_, known)| known.eq_ignore_ascii_case(name))
        .map(|(bit, _)| *bit)
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Stick {
    pub x: u8,
    pub y: u8,
}

impl Stick {
    pub const CENTRE: u8 = 128;

    pub fn centred() -> Self {
        Self {
            x: Self::CENTRE,
            y: Self::CENTRE,
        }
    }

    pub fn normalised(self) -> (f32, f32) {
        (Self::axis_to_unit(self.x), Self::axis_to_unit(self.y))
    }

    fn axis_to_unit(value: u8) -> f32 {
        let centred = f32::from(value) - f32::from(Self::CENTRE);
        (centred / 127.0).clamp(-1.0, 1.0)
    }

    fn unit_to_axis(value: f32) -> u8 {
        (f32::from(Self::CENTRE) + value * 127.0)
            .round()
            .clamp(0.0, 255.0) as u8
    }

    pub fn with_dead_zone(self, dead_zone: f32) -> Self {
        if dead_zone >= 1.0 {
            return Self::centred();
        }

        let (nx, ny) = self.normalised();
        let magnitude = nx.hypot(ny);

        if magnitude <= dead_zone.max(0.0) {
            return Self::centred();
        }

        let rescaled = ((magnitude - dead_zone) / (1.0 - dead_zone)).clamp(0.0, 1.0);
        let scale = rescaled / magnitude;

        Self {
            x: Self::unit_to_axis(nx * scale),
            y: Self::unit_to_axis(ny * scale),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Touch {
    pub active: bool,
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TouchPad {
    pub first: Touch,
    pub second: Touch,
    pub packet_counter: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Motion {
    pub gyro_pitch: i16,
    pub gyro_yaw: i16,
    pub gyro_roll: i16,
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
    pub full: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PadState {
    pub buttons: ButtonMask,
    pub left_stick: Stick,
    pub right_stick: Stick,
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub touch: TouchPad,
    pub motion: Motion,
    pub battery: Option<Battery>,
    pub frame_counter: u8,
}

impl Default for PadState {
    fn default() -> Self {
        Self {
            buttons: 0,
            left_stick: Stick::centred(),
            right_stick: Stick::centred(),
            left_trigger: 0,
            right_trigger: 0,
            touch: TouchPad::default(),
            motion: Motion::default(),
            battery: None,
            frame_counter: 0,
        }
    }
}

impl PadState {
    pub fn is_down(&self, button: ButtonMask) -> bool {
        self.buttons & button != 0
    }

    pub fn pressed_since(&self, previous: &PadState) -> ButtonMask {
        self.buttons & !previous.buttons
    }

    pub fn released_since(&self, previous: &PadState) -> ButtonMask {
        previous.buttons & !self.buttons
    }

    pub fn held_names(&self) -> Vec<&'static str> {
        ALL_BUTTONS
            .iter()
            .filter(|(bit, _)| self.buttons & bit != 0)
            .map(|(_, name)| *name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sony_vendor_id_matches_ds4windows_ds4devices_cs_line_129() {
        assert_eq!(SONY_VENDOR, 0x054C);
    }

    #[test]
    fn dualshock4_v1_product_id_matches_ds4windows_ds4devices_cs_line_145() {
        assert_eq!(DUALSHOCK4_V1, 0x05C4);
    }

    #[test]
    fn dualshock4_v2_product_id_matches_ds4windows_ds4devices_cs_line_146() {
        assert_eq!(DUALSHOCK4_V2, 0x09CC);
    }

    #[test]
    fn dualsense_product_id_matches_ds4windows_ds4devices_cs_line_147() {
        assert_eq!(DUALSENSE, 0x0CE6);
    }

    #[test]
    fn dualsense_edge_product_id_matches_ds4windows_ds4devices_cs_line_148() {
        assert_eq!(DUALSENSE_EDGE, 0x0DF2);
    }

    #[test]
    fn dualshock4_dongle_product_id_matches_ds4windows_ds4devices_cs_line_144() {
        assert_eq!(DUALSHOCK4_DONGLE, 0x0BA0);
    }

    #[test]
    fn every_sony_product_id_resolves_and_a_stranger_does_not() {
        for (product, expected) in [
            (DUALSENSE, DeviceKind::DualSense),
            (DUALSENSE_EDGE, DeviceKind::DualSenseEdge),
            (DUALSHOCK4_V1, DeviceKind::DualShock4V1),
            (DUALSHOCK4_V2, DeviceKind::DualShock4V2),
            (DUALSHOCK4_DONGLE, DeviceKind::DualShock4Dongle),
        ] {
            assert_eq!(DeviceKind::from_ids(SONY_VENDOR, product), Some(expected));
        }

        assert_eq!(DeviceKind::from_ids(SONY_VENDOR, 0x9999), None);
        assert_eq!(DeviceKind::from_ids(0x045E, DUALSENSE), None);
    }

    #[test]
    fn every_device_kind_has_a_name_a_person_would_recognise() {
        for kind in [
            DeviceKind::DualSense,
            DeviceKind::DualSenseEdge,
            DeviceKind::DualShock4V1,
            DeviceKind::DualShock4V2,
            DeviceKind::DualShock4Dongle,
        ] {
            assert!(!kind.display_name().is_empty());
        }

        assert!(DeviceKind::DualSense.has_mute_button());
        assert!(!DeviceKind::DualShock4V2.has_mute_button());
    }

    #[test]
    fn every_bluetooth_transport_carries_a_crc_and_usb_never_does() {
        assert!(Transport::BluetoothFull.needs_crc());
        assert!(Transport::BluetoothBasic.needs_crc());
        assert!(!Transport::Usb.needs_crc());
    }

    #[test]
    fn every_button_name_round_trips_through_the_parser() {
        for (mask, name) in ALL_BUTTONS {
            assert_eq!(button_name(*mask), Some(*name));
            assert_eq!(parse_button_name(name), Some(*mask));
        }
    }

    #[test]
    fn a_button_name_is_matched_without_regard_to_case() {
        assert_eq!(parse_button_name("circle"), Some(CIRCLE));
        assert_eq!(parse_button_name("CIRCLE"), Some(CIRCLE));
        assert_eq!(parse_button_name("Trianlge"), None);
    }

    #[test]
    fn an_unknown_mask_has_no_name() {
        assert_eq!(button_name(1 << 30), None);
    }

    #[test]
    fn a_centred_stick_normalises_to_zero_and_the_ends_reach_one() {
        let (x, y) = Stick::centred().normalised();

        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);

        assert_eq!(Stick { x: 255, y: 128 }.normalised().0, 1.0);
        assert_eq!(Stick { x: 0, y: 128 }.normalised().0, -1.0);
    }

    #[test]
    fn no_stick_value_ever_normalises_outside_minus_one_to_one() {
        for raw in 0..=255u8 {
            let (x, _) = Stick { x: raw, y: 128 }.normalised();

            assert!((-1.0..=1.0).contains(&x), "{raw} gave {x}");
        }
    }

    #[test]
    fn an_already_centred_stick_is_unaffected_by_a_dead_zone() {
        let shaped = Stick::centred().with_dead_zone(0.2);

        assert_eq!(shaped, Stick::centred());
    }

    #[test]
    fn a_push_inside_the_dead_zone_reads_as_perfectly_centred() {
        let barely_pushed = Stick { x: 138, y: 128 };

        assert_eq!(barely_pushed.with_dead_zone(0.2), Stick::centred());
    }

    #[test]
    fn a_full_push_still_reaches_the_edge_after_a_dead_zone_is_applied() {
        let full = Stick { x: 255, y: 128 };

        let shaped = full.with_dead_zone(0.2);

        assert_eq!(shaped.x, 255);
        assert_eq!(shaped.y, 128);
    }

    #[test]
    fn a_push_just_past_the_dead_zone_starts_from_zero_not_from_a_jump() {
        let (nx, _) = Stick { x: 155, y: 128 }.normalised();
        assert!(nx > 0.2, "test fixture must push past the 0.2 dead zone");

        let shaped = Stick { x: 155, y: 128 }.with_dead_zone(0.2);
        let (shaped_nx, _) = shaped.normalised();

        assert!(
            shaped_nx < 0.15,
            "a push just past the dead zone should rescale close to zero, got {shaped_nx}"
        );
        assert!(shaped_nx > 0.0);
    }

    #[test]
    fn a_dead_zone_of_zero_leaves_a_single_axis_push_within_a_rounding_step() {
        for raw in 0..=255u8 {
            let stick = Stick {
                x: raw,
                y: Stick::CENTRE,
            };
            let shaped = stick.with_dead_zone(0.0);

            assert!(shaped.x.abs_diff(raw) <= 1, "raw {raw} became {}", shaped.x);
            assert_eq!(shaped.y, Stick::CENTRE);
        }
    }

    #[test]
    fn a_diagonal_corner_is_clamped_to_the_unit_circle_even_with_no_dead_zone() {
        let corner = Stick { x: 0, y: 0 };

        let shaped = corner.with_dead_zone(0.0);
        let (nx, ny) = shaped.normalised();

        assert!(
            nx.hypot(ny) <= 1.01,
            "a real thumbstick cannot reach a square corner, shaping must clamp it"
        );
    }

    #[test]
    fn a_dead_zone_of_one_or_more_always_centres_the_stick() {
        assert_eq!(
            Stick { x: 255, y: 255 }.with_dead_zone(1.0),
            Stick::centred()
        );
        assert_eq!(Stick { x: 0, y: 0 }.with_dead_zone(1.5), Stick::centred());
    }

    #[test]
    fn a_diagonal_push_keeps_its_direction_after_a_dead_zone_is_applied() {
        let diagonal = Stick { x: 255, y: 0 };

        let (nx_before, ny_before) = diagonal.normalised();
        let (nx_after, ny_after) = diagonal.with_dead_zone(0.1).normalised();

        let angle_before = ny_before.atan2(nx_before);
        let angle_after = ny_after.atan2(nx_after);

        assert!((angle_before - angle_after).abs() < 0.05);
    }

    #[test]
    fn held_names_lists_every_button_that_is_down_and_nothing_else() {
        let state = PadState {
            buttons: CIRCLE | DPAD_UP,
            ..PadState::default()
        };

        let held = state.held_names();

        assert!(held.contains(&"Circle"));
        assert!(held.contains(&"DpadUp"));
        assert_eq!(held.len(), 2);
    }

    #[test]
    fn a_default_pad_reports_no_battery_rather_than_an_empty_one() {
        assert!(PadState::default().battery.is_none());
    }

    #[test]
    fn nothing_is_down_on_a_default_pad() {
        let state = PadState::default();

        for (mask, name) in ALL_BUTTONS {
            assert!(!state.is_down(*mask), "{name} was down");
        }
    }
}
