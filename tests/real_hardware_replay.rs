use opends_core::controller::decode;
use opends_core::types::pad::{DeviceKind, Transport};

fn load(name: &str) -> Vec<u8> {
    let text = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();

    text.split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).unwrap())
        .collect()
}

#[test]
fn a_real_bluetooth_dualsense_at_rest_decodes_to_no_buttons_pressed() {
    let report = load("dualsense-bt-basic-neutral.hex");

    let (transport, state) = decode::decode(DeviceKind::DualSense, 78, &report).unwrap();

    assert_eq!(transport, Transport::BluetoothBasic);
    assert_eq!(state.buttons, 0, "held: {:?}", state.held_names());
    assert_eq!(state.left_trigger, 0);
    assert_eq!(state.right_trigger, 0);
}

#[test]
fn the_same_real_report_read_as_usb_would_have_invented_presses() {
    let report = load("dualsense-bt-basic-neutral.hex");

    let (_, wrong) = decode::decode(DeviceKind::DualSense, 64, &report).unwrap();

    assert_ne!(
        wrong.buttons, 0,
        "this is the bug the caps length fix removed"
    );
}

#[test]
fn the_sticks_of_a_resting_pad_sit_near_the_centre() {
    let report = load("dualsense-bt-basic-neutral.hex");

    let (_, state) = decode::decode(DeviceKind::DualSense, 78, &report).unwrap();
    let (lx, ly) = state.left_stick.normalised();

    assert!(lx.abs() < 0.05, "left stick x drifted to {lx}");
    assert!(ly.abs() < 0.05, "left stick y drifted to {ly}");
}

#[test]
fn two_real_reports_with_different_counter_values_both_decode_to_nothing_held() {
    let first = load("dualsense-bt-basic-neutral.hex");
    let second = load("dualsense-bt-basic-counter-0x10.hex");

    assert_ne!(
        first[7], second[7],
        "the fixtures must differ in the counter"
    );

    for report in [first, second] {
        let (_, state) = decode::decode(DeviceKind::DualSense, 78, &report).unwrap();

        assert_eq!(
            state.buttons,
            0,
            "counter byte {:#04x} invented {:?}",
            report[7],
            state.held_names()
        );
    }
}

#[test]
fn the_counter_byte_would_have_read_as_a_button_before_the_fix() {
    let report = load("dualsense-bt-basic-counter-0x10.hex");

    assert_eq!(
        report[7] & (1 << 4),
        1 << 4,
        "bit 4 is what FnL used to read"
    );
}

#[test]
fn a_real_bluetooth_full_dualsense_at_rest_decodes_to_no_buttons_pressed() {
    for fixture in [
        "dualsense-bt-full-neutral-1.hex",
        "dualsense-bt-full-neutral-2.hex",
    ] {
        let report = load(fixture);

        let (transport, state) = decode::decode(DeviceKind::DualSense, 78, &report).unwrap();

        assert_eq!(transport, Transport::BluetoothFull);
        assert_eq!(
            state.buttons,
            0,
            "{fixture}: held {:?}, this is the header-length-off-by-one bug",
            state.held_names()
        );
    }
}

#[test]
fn a_real_bluetooth_full_dualsense_at_rest_has_sticks_near_centre_and_motion_near_zero() {
    for fixture in [
        "dualsense-bt-full-neutral-1.hex",
        "dualsense-bt-full-neutral-2.hex",
    ] {
        let report = load(fixture);

        let (_, state) = decode::decode(DeviceKind::DualSense, 78, &report).unwrap();
        let (lx, ly) = state.left_stick.normalised();
        let (rx, ry) = state.right_stick.normalised();

        assert!(lx.abs() < 0.05, "{fixture}: left stick x drifted to {lx}");
        assert!(ly.abs() < 0.05, "{fixture}: left stick y drifted to {ly}");
        assert!(rx.abs() < 0.05, "{fixture}: right stick x drifted to {rx}");
        assert!(
            ry.abs() < 0.05,
            "{fixture}: right stick y drifted to {ry}, this is the pinned-RY bug"
        );

        assert!(
            state.motion.gyro_yaw.abs() < 50,
            "{fixture}: gyro yaw at rest read {}",
            state.motion.gyro_yaw
        );
    }
}
