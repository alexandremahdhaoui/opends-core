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
use std::collections::{BTreeMap, BTreeSet};

use crate::controller::output::TriggerEffect;
use crate::types::pad::{parse_button_name, ButtonMask, PadState, Touch, ALL_BUTTONS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Fourth,
    Fifth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedStep {
    pub binding: Binding,
    pub delay_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Binding {
    Key { code: u16 },
    Mouse { button: MouseButton },
    Macro { steps: Vec<Binding> },
    TimedMacro { steps: Vec<TimedStep> },
    Unbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Output {
    KeyDown(u16),
    KeyUp(u16),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseMove { dx: i32, dy: i32 },
}

fn default_turbo_interval_ms() -> u32 {
    100
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub bindings: BTreeMap<String, Binding>,
    pub gyro_mouse_sensitivity: Option<f32>,
    pub gyro_toggle_button: Option<ButtonMask>,
    pub touch_mouse_sensitivity: Option<f32>,
    pub stick_dead_zone: Option<f32>,
    #[serde(default)]
    pub left_trigger: TriggerEffect,
    #[serde(default)]
    pub right_trigger: TriggerEffect,
    #[serde(default)]
    pub turbo_buttons: BTreeSet<String>,
    #[serde(default = "default_turbo_interval_ms")]
    pub turbo_interval_ms: u32,
    #[serde(default)]
    pub shift_button: Option<ButtonMask>,
    #[serde(default)]
    pub shift_bindings: BTreeMap<String, Binding>,
}

impl Default for Profile {
    fn default() -> Self {
        Self::named("")
    }
}

impl Profile {
    pub fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bindings: BTreeMap::new(),
            gyro_mouse_sensitivity: None,
            gyro_toggle_button: None,
            touch_mouse_sensitivity: None,
            stick_dead_zone: None,
            left_trigger: TriggerEffect::Off,
            right_trigger: TriggerEffect::Off,
            turbo_buttons: BTreeSet::new(),
            turbo_interval_ms: default_turbo_interval_ms(),
            shift_button: None,
            shift_bindings: BTreeMap::new(),
        }
    }

    pub fn with_gyro_mouse(mut self, sensitivity: f32) -> Self {
        self.gyro_mouse_sensitivity = Some(sensitivity);
        self
    }

    pub fn with_gyro_toggle_button(mut self, button: ButtonMask) -> Self {
        self.gyro_toggle_button = Some(button);
        self
    }

    pub fn with_touch_mouse(mut self, sensitivity: f32) -> Self {
        self.touch_mouse_sensitivity = Some(sensitivity);
        self
    }

    pub fn with_stick_dead_zone(mut self, dead_zone: f32) -> Self {
        self.stick_dead_zone = Some(dead_zone);
        self
    }

    pub fn with_triggers(mut self, left: TriggerEffect, right: TriggerEffect) -> Self {
        self.left_trigger = left;
        self.right_trigger = right;
        self
    }

    pub fn with_turbo(mut self, button: &str) -> Self {
        self.turbo_buttons.insert(button.to_string());
        self
    }

    pub fn with_turbo_interval_ms(mut self, interval_ms: u32) -> Self {
        self.turbo_interval_ms = interval_ms;
        self
    }

    pub fn bind(mut self, button: &str, binding: Binding) -> Self {
        self.bindings.insert(button.to_string(), binding);
        self
    }

    pub fn with_shift_button(mut self, button: ButtonMask) -> Self {
        self.shift_button = Some(button);
        self
    }

    pub fn bind_shift(mut self, button: &str, binding: Binding) -> Self {
        self.shift_bindings.insert(button.to_string(), binding);
        self
    }

    pub fn shift_binding_for(&self, button: ButtonMask, shift_active: bool) -> Option<&Binding> {
        if shift_active {
            if let Some(name) = ALL_BUTTONS
                .iter()
                .find(|(bit, _)| *bit == button)
                .map(|(_, name)| *name)
            {
                if let Some(binding) = self.shift_bindings.get(name) {
                    return Some(binding);
                }
            }
        }

        self.binding_for(button)
    }

    pub fn binding_for(&self, button: ButtonMask) -> Option<&Binding> {
        ALL_BUTTONS
            .iter()
            .find(|(bit, _)| *bit == button)
            .and_then(|(_, name)| self.bindings.get(*name))
    }

    pub fn unknown_button_names(&self) -> Vec<&str> {
        self.bindings
            .keys()
            .filter(|name| parse_button_name(name).is_none())
            .map(String::as_str)
            .collect()
    }

    pub fn bound_buttons(&self) -> ButtonMask {
        self.bindings
            .keys()
            .filter_map(|name| parse_button_name(name))
            .fold(0, |mask, bit| mask | bit)
    }
}

pub fn outputs_for(profile: &Profile, pressed: ButtonMask, released: ButtonMask) -> Vec<Output> {
    let mut outputs = Vec::new();

    for (bit, _) in ALL_BUTTONS {
        if released & bit != 0 {
            push_release(profile, *bit, &mut outputs);
        }
    }

    for (bit, _) in ALL_BUTTONS {
        if pressed & bit != 0 {
            push_press(profile, *bit, &mut outputs);
        }
    }

    outputs
}

fn push_press(profile: &Profile, button: ButtonMask, outputs: &mut Vec<Output>) {
    if let Some(binding) = profile.binding_for(button) {
        push_press_binding(binding, outputs);
    }
}

pub fn push_press_binding(binding: &Binding, outputs: &mut Vec<Output>) {
    match binding {
        Binding::Key { code } => outputs.push(Output::KeyDown(*code)),
        Binding::Mouse { button } => outputs.push(Output::MouseDown(*button)),
        Binding::Macro { steps } => {
            for step in steps {
                if !matches!(step, Binding::Macro { .. } | Binding::TimedMacro { .. }) {
                    push_press_binding(step, outputs);
                }
            }
        }
        Binding::TimedMacro { .. } => {}
        Binding::Unbound => {}
    }
}

fn push_release(profile: &Profile, button: ButtonMask, outputs: &mut Vec<Output>) {
    if let Some(binding) = profile.binding_for(button) {
        push_release_binding(binding, outputs);
    }
}

pub fn push_release_binding(binding: &Binding, outputs: &mut Vec<Output>) {
    match binding {
        Binding::Key { code } => outputs.push(Output::KeyUp(*code)),
        Binding::Mouse { button } => outputs.push(Output::MouseUp(*button)),
        Binding::Macro { steps } => {
            for step in steps {
                if !matches!(step, Binding::Macro { .. } | Binding::TimedMacro { .. }) {
                    push_release_binding(step, outputs);
                }
            }
        }
        Binding::TimedMacro { .. } => {}
        Binding::Unbound => {}
    }
}

pub fn outputs_to_release(profile: &Profile, held: &PadState) -> Vec<Output> {
    let mut outputs = Vec::new();

    for (bit, _) in ALL_BUTTONS {
        if held.buttons & bit != 0 {
            push_release(profile, *bit, &mut outputs);
        }
    }

    outputs
}

pub fn gyro_mouse_move(profile: &Profile, state: &PadState) -> Option<Output> {
    let sensitivity = profile.gyro_mouse_sensitivity?;

    if let Some(toggle_button) = profile.gyro_toggle_button {
        if state.buttons & toggle_button == 0 {
            return None;
        }
    }

    let dx = (f32::from(state.motion.gyro_yaw) * sensitivity) as i32;
    let dy = (f32::from(state.motion.gyro_pitch) * sensitivity) as i32;

    if dx == 0 && dy == 0 {
        return None;
    }

    Some(Output::MouseMove { dx, dy })
}

pub fn touch_mouse_move(
    profile: &Profile,
    previous: Option<(u16, u16)>,
    touch: Touch,
) -> (Option<Output>, Option<(u16, u16)>) {
    let Some(sensitivity) = profile.touch_mouse_sensitivity else {
        return (None, None);
    };

    if !touch.active {
        return (None, None);
    }

    let current = (touch.x, touch.y);

    let Some((previous_x, previous_y)) = previous else {
        return (None, Some(current));
    };

    let dx = ((i32::from(touch.x) - i32::from(previous_x)) as f32 * sensitivity) as i32;
    let dy = ((i32::from(touch.y) - i32::from(previous_y)) as f32 * sensitivity) as i32;

    if dx == 0 && dy == 0 {
        return (None, Some(current));
    }

    (Some(Output::MouseMove { dx, dy }), Some(current))
}

pub fn shape_sticks(profile: &Profile, state: &PadState) -> PadState {
    let Some(dead_zone) = profile.stick_dead_zone else {
        return *state;
    };

    PadState {
        left_stick: state.left_stick.with_dead_zone(dead_zone),
        right_stick: state.right_stick.with_dead_zone(dead_zone),
        ..*state
    }
}

pub fn steps_due_by(steps: &[TimedStep], elapsed_ms: u64) -> usize {
    let mut cumulative = 0u64;
    let mut due = 0usize;

    for step in steps {
        cumulative += u64::from(step.delay_ms);

        if cumulative > elapsed_ms {
            break;
        }

        due += 1;
    }

    due
}

pub fn turbo_pressed_phase(elapsed_ms: u64, interval_ms: u32) -> bool {
    let half_period = u64::from(interval_ms.max(2)) / 2;

    (elapsed_ms / half_period).is_multiple_of(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::pad::{Motion, CIRCLE, CROSS, DPAD_UP, SQUARE, TRIANGLE};

    const ESCAPE: u16 = 0x1B;
    const SPACE: u16 = 0x20;

    fn escape_on_circle() -> Profile {
        Profile::named("test").bind("Circle", Binding::Key { code: ESCAPE })
    }

    #[test]
    fn a_bound_press_becomes_a_key_down() {
        let outputs = outputs_for(&escape_on_circle(), CIRCLE, 0);

        assert_eq!(outputs, vec![Output::KeyDown(ESCAPE)]);
    }

    #[test]
    fn a_bound_release_becomes_a_key_up() {
        let outputs = outputs_for(&escape_on_circle(), 0, CIRCLE);

        assert_eq!(outputs, vec![Output::KeyUp(ESCAPE)]);
    }

    #[test]
    fn an_unbound_button_produces_nothing_at_all() {
        let outputs = outputs_for(&escape_on_circle(), CROSS, 0);

        assert!(outputs.is_empty());
    }

    #[test]
    fn a_button_bound_to_unbound_is_deliberately_silent() {
        let profile = Profile::named("test").bind("Circle", Binding::Unbound);

        assert!(outputs_for(&profile, CIRCLE, 0).is_empty());
    }

    #[test]
    fn releases_are_emitted_before_presses_so_a_swap_never_leaves_a_key_stuck() {
        let profile = Profile::named("test")
            .bind("Circle", Binding::Key { code: ESCAPE })
            .bind("Cross", Binding::Key { code: SPACE });

        let outputs = outputs_for(&profile, CROSS, CIRCLE);

        assert_eq!(outputs, vec![Output::KeyUp(ESCAPE), Output::KeyDown(SPACE)]);
    }

    #[test]
    fn two_buttons_bound_to_the_same_key_each_emit_their_own_event() {
        let profile = Profile::named("test")
            .bind("Circle", Binding::Key { code: ESCAPE })
            .bind("Cross", Binding::Key { code: ESCAPE });

        let outputs = outputs_for(&profile, CIRCLE | CROSS, 0);

        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn a_mouse_binding_emits_mouse_events_and_not_key_events() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::Mouse {
                button: MouseButton::Left,
            },
        );

        assert_eq!(
            outputs_for(&profile, CIRCLE, 0),
            vec![Output::MouseDown(MouseButton::Left)]
        );
        assert_eq!(
            outputs_for(&profile, 0, CIRCLE),
            vec![Output::MouseUp(MouseButton::Left)]
        );
    }

    #[test]
    fn a_macro_press_fires_every_step_in_order() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::Macro {
                steps: vec![
                    Binding::Key { code: ESCAPE },
                    Binding::Mouse {
                        button: MouseButton::Left,
                    },
                    Binding::Key { code: SPACE },
                ],
            },
        );

        assert_eq!(
            outputs_for(&profile, CIRCLE, 0),
            vec![
                Output::KeyDown(ESCAPE),
                Output::MouseDown(MouseButton::Left),
                Output::KeyDown(SPACE),
            ]
        );
    }

    #[test]
    fn a_macro_release_releases_every_step_that_was_pressed() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::Macro {
                steps: vec![
                    Binding::Key { code: ESCAPE },
                    Binding::Mouse {
                        button: MouseButton::Left,
                    },
                ],
            },
        );

        assert_eq!(
            outputs_for(&profile, 0, CIRCLE),
            vec![Output::KeyUp(ESCAPE), Output::MouseUp(MouseButton::Left),]
        );
    }

    #[test]
    fn a_step_bound_to_unbound_inside_a_macro_is_silently_skipped() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::Macro {
                steps: vec![Binding::Key { code: ESCAPE }, Binding::Unbound],
            },
        );

        assert_eq!(
            outputs_for(&profile, CIRCLE, 0),
            vec![Output::KeyDown(ESCAPE)]
        );
    }

    #[test]
    fn a_macro_nested_inside_a_macro_step_is_ignored_rather_than_recursing_forever() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::Macro {
                steps: vec![
                    Binding::Key { code: ESCAPE },
                    Binding::Macro {
                        steps: vec![Binding::Key { code: SPACE }],
                    },
                ],
            },
        );

        assert_eq!(
            outputs_for(&profile, CIRCLE, 0),
            vec![Output::KeyDown(ESCAPE)]
        );
    }

    #[test]
    fn an_empty_macro_produces_nothing_rather_than_panicking() {
        let profile = Profile::named("test").bind("Circle", Binding::Macro { steps: vec![] });

        assert!(outputs_for(&profile, CIRCLE, 0).is_empty());
    }

    #[test]
    fn a_macro_binding_round_trips_through_json() {
        let profile = Profile::named("round trip").bind(
            "Circle",
            Binding::Macro {
                steps: vec![
                    Binding::Key { code: ESCAPE },
                    Binding::Mouse {
                        button: MouseButton::Right,
                    },
                ],
            },
        );

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn nothing_pressed_and_nothing_released_produces_nothing() {
        assert!(outputs_for(&escape_on_circle(), 0, 0).is_empty());
    }

    #[test]
    fn every_held_button_is_released_when_a_profile_is_torn_down() {
        let profile = Profile::named("test")
            .bind("Circle", Binding::Key { code: ESCAPE })
            .bind("Cross", Binding::Key { code: SPACE });

        let held = PadState {
            buttons: CIRCLE | CROSS,
            ..PadState::default()
        };

        let outputs = outputs_to_release(&profile, &held);

        assert!(outputs.contains(&Output::KeyUp(ESCAPE)));
        assert!(outputs.contains(&Output::KeyUp(SPACE)));
    }

    #[test]
    fn a_dpad_direction_binds_like_any_other_button() {
        let profile = Profile::named("test").bind("DpadUp", Binding::Key { code: SPACE });

        assert_eq!(
            outputs_for(&profile, DPAD_UP, 0),
            vec![Output::KeyDown(SPACE)]
        );
    }

    #[test]
    fn button_names_are_matched_without_regard_to_case() {
        let profile = Profile::named("test").bind("circle", Binding::Key { code: ESCAPE });

        assert!(profile.binding_for(CIRCLE).is_none());
        assert_eq!(profile.unknown_button_names(), Vec::<&str>::new());
    }

    #[test]
    fn a_typo_in_a_profile_is_reported_rather_than_silently_ignored() {
        let profile = Profile::named("test").bind("Triangel", Binding::Key { code: ESCAPE });

        assert_eq!(profile.unknown_button_names(), vec!["Triangel"]);
    }

    #[test]
    fn bound_buttons_reports_the_mask_a_profile_actually_covers() {
        let profile = Profile::named("test")
            .bind("Circle", Binding::Key { code: ESCAPE })
            .bind("Triangle", Binding::Key { code: SPACE });

        assert_eq!(profile.bound_buttons(), CIRCLE | TRIANGLE);
    }

    #[test]
    fn a_profile_survives_a_round_trip_through_json() {
        let profile = Profile::named("round trip")
            .bind("Circle", Binding::Key { code: ESCAPE })
            .bind(
                "Square",
                Binding::Mouse {
                    button: MouseButton::Right,
                },
            );

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
        assert!(back.binding_for(SQUARE).is_some());
    }

    #[test]
    fn an_empty_profile_binds_nothing_and_never_panics() {
        let profile = Profile::named("empty");

        assert_eq!(profile.bound_buttons(), 0);

        for (bit, _) in ALL_BUTTONS {
            assert!(outputs_for(&profile, *bit, 0).is_empty());
        }
    }

    fn state_with_motion(motion: Motion) -> PadState {
        PadState {
            motion,
            ..PadState::default()
        }
    }

    #[test]
    fn a_profile_with_no_gyro_sensitivity_never_moves_the_mouse() {
        let profile = Profile::named("no gyro");
        let state = state_with_motion(Motion {
            gyro_yaw: 1000,
            gyro_pitch: 1000,
            ..Motion::default()
        });

        assert_eq!(gyro_mouse_move(&profile, &state), None);
    }

    #[test]
    fn a_profile_with_gyro_sensitivity_but_no_motion_produces_nothing() {
        let profile = Profile::named("gyro").with_gyro_mouse(0.05);

        assert_eq!(
            gyro_mouse_move(&profile, &state_with_motion(Motion::default())),
            None
        );
    }

    #[test]
    fn tilting_the_pad_moves_the_mouse_scaled_by_sensitivity() {
        let profile = Profile::named("gyro").with_gyro_mouse(0.1);
        let state = state_with_motion(Motion {
            gyro_yaw: 200,
            gyro_pitch: 50,
            ..Motion::default()
        });

        assert_eq!(
            gyro_mouse_move(&profile, &state),
            Some(Output::MouseMove { dx: 20, dy: 5 })
        );
    }

    #[test]
    fn tilting_the_other_way_moves_the_mouse_the_other_way() {
        let profile = Profile::named("gyro").with_gyro_mouse(0.1);
        let state = state_with_motion(Motion {
            gyro_yaw: -200,
            gyro_pitch: -50,
            ..Motion::default()
        });

        assert_eq!(
            gyro_mouse_move(&profile, &state),
            Some(Output::MouseMove { dx: -20, dy: -5 })
        );
    }

    #[test]
    fn a_tiny_motion_that_rounds_to_zero_pixels_produces_nothing() {
        let profile = Profile::named("gyro").with_gyro_mouse(0.001);
        let state = state_with_motion(Motion {
            gyro_yaw: 5,
            gyro_pitch: 5,
            ..Motion::default()
        });

        assert_eq!(gyro_mouse_move(&profile, &state), None);
    }

    #[test]
    fn with_no_toggle_button_set_gyro_mouse_is_always_active() {
        let profile = Profile::named("gyro").with_gyro_mouse(0.1);
        let state = state_with_motion(Motion {
            gyro_yaw: 200,
            gyro_pitch: 0,
            ..Motion::default()
        });

        assert!(gyro_mouse_move(&profile, &state).is_some());
    }

    #[test]
    fn a_toggle_button_that_is_not_held_suppresses_gyro_mouse() {
        use crate::types::pad::L1;

        let profile = Profile::named("gyro")
            .with_gyro_mouse(0.1)
            .with_gyro_toggle_button(L1);

        let state = state_with_motion(Motion {
            gyro_yaw: 200,
            gyro_pitch: 0,
            ..Motion::default()
        });

        assert_eq!(gyro_mouse_move(&profile, &state), None);
    }

    #[test]
    fn holding_the_toggle_button_lets_gyro_mouse_through() {
        use crate::types::pad::L1;

        let profile = Profile::named("gyro")
            .with_gyro_mouse(0.1)
            .with_gyro_toggle_button(L1);

        let state = PadState {
            buttons: L1,
            ..state_with_motion(Motion {
                gyro_yaw: 200,
                gyro_pitch: 0,
                ..Motion::default()
            })
        };

        assert_eq!(
            gyro_mouse_move(&profile, &state),
            Some(Output::MouseMove { dx: 20, dy: 0 })
        );
    }

    #[test]
    fn releasing_the_toggle_button_stops_gyro_mouse_again() {
        use crate::types::pad::L1;

        let profile = Profile::named("gyro")
            .with_gyro_mouse(0.1)
            .with_gyro_toggle_button(L1);

        let held = PadState {
            buttons: L1,
            ..state_with_motion(Motion {
                gyro_yaw: 200,
                gyro_pitch: 0,
                ..Motion::default()
            })
        };
        assert!(gyro_mouse_move(&profile, &held).is_some());

        let released = PadState { buttons: 0, ..held };
        assert_eq!(gyro_mouse_move(&profile, &released), None);
    }

    #[test]
    fn a_profile_carrying_gyro_sensitivity_still_round_trips_through_json() {
        let profile = Profile::named("round trip").with_gyro_mouse(0.25);

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_carrying_a_gyro_toggle_button_round_trips_through_json() {
        use crate::types::pad::R1;

        let profile = Profile::named("round trip")
            .with_gyro_mouse(0.25)
            .with_gyro_toggle_button(R1);

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_json_written_before_gyro_toggle_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert_eq!(profile.gyro_toggle_button, None);
    }

    #[test]
    fn a_profile_json_written_before_gyro_mouse_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert_eq!(profile.gyro_mouse_sensitivity, None);
    }

    #[test]
    fn a_fresh_profile_has_no_adaptive_trigger_effect() {
        let profile = Profile::named("test");

        assert_eq!(profile.left_trigger, TriggerEffect::Off);
        assert_eq!(profile.right_trigger, TriggerEffect::Off);
    }

    #[test]
    fn with_triggers_sets_both_sides_independently() {
        let profile = Profile::named("test").with_triggers(
            TriggerEffect::Rigid { force: 200 },
            TriggerEffect::Pulse {
                start: 10,
                force: 90,
            },
        );

        assert_eq!(profile.left_trigger, TriggerEffect::Rigid { force: 200 });
        assert_eq!(
            profile.right_trigger,
            TriggerEffect::Pulse {
                start: 10,
                force: 90
            }
        );
    }

    #[test]
    fn a_profile_carrying_trigger_effects_round_trips_through_json() {
        let profile = Profile::named("round trip").with_triggers(
            TriggerEffect::Weapon {
                start: 20,
                end: 80,
                force: 255,
            },
            TriggerEffect::Off,
        );

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_json_written_before_trigger_effects_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert_eq!(profile.left_trigger, TriggerEffect::Off);
        assert_eq!(profile.right_trigger, TriggerEffect::Off);
    }

    fn touch_at(active: bool, x: u16, y: u16) -> Touch {
        Touch {
            active,
            id: 0,
            x,
            y,
        }
    }

    #[test]
    fn a_profile_with_no_touch_sensitivity_never_moves_the_mouse() {
        let profile = Profile::named("no touch mouse");

        let (output, next) = touch_mouse_move(&profile, None, touch_at(true, 500, 500));

        assert_eq!(output, None);
        assert_eq!(next, None);
    }

    #[test]
    fn a_lifted_finger_never_moves_the_mouse_even_with_sensitivity_set() {
        let profile = Profile::named("touch mouse").with_touch_mouse(1.0);

        let (output, next) = touch_mouse_move(&profile, Some((400, 400)), touch_at(false, 0, 0));

        assert_eq!(output, None);
        assert_eq!(next, None);
    }

    #[test]
    fn the_first_touch_after_landing_only_records_a_baseline_and_does_not_move_the_mouse() {
        let profile = Profile::named("touch mouse").with_touch_mouse(1.0);

        let (output, next) = touch_mouse_move(&profile, None, touch_at(true, 300, 200));

        assert_eq!(output, None);
        assert_eq!(next, Some((300, 200)));
    }

    #[test]
    fn dragging_the_finger_moves_the_mouse_by_the_position_delta_scaled_by_sensitivity() {
        let profile = Profile::named("touch mouse").with_touch_mouse(0.5);

        let (output, next) = touch_mouse_move(&profile, Some((300, 200)), touch_at(true, 340, 180));

        assert_eq!(output, Some(Output::MouseMove { dx: 20, dy: -10 }));
        assert_eq!(next, Some((340, 180)));
    }

    #[test]
    fn a_drag_too_small_to_round_to_a_pixel_reports_no_movement_but_still_updates_the_baseline() {
        let profile = Profile::named("touch mouse").with_touch_mouse(0.01);

        let (output, next) = touch_mouse_move(&profile, Some((300, 200)), touch_at(true, 301, 200));

        assert_eq!(output, None);
        assert_eq!(next, Some((301, 200)));
    }

    #[test]
    fn lifting_the_finger_and_landing_again_starts_a_fresh_baseline_not_a_jump() {
        let profile = Profile::named("touch mouse").with_touch_mouse(1.0);

        let (_, after_lift) = touch_mouse_move(&profile, Some((300, 200)), touch_at(false, 0, 0));
        assert_eq!(after_lift, None);

        let (output, next) = touch_mouse_move(&profile, after_lift, touch_at(true, 900, 900));

        assert_eq!(
            output, None,
            "a fresh landing must not be treated as a drag"
        );
        assert_eq!(next, Some((900, 900)));
    }

    #[test]
    fn a_profile_carrying_touch_mouse_sensitivity_round_trips_through_json() {
        let profile = Profile::named("round trip").with_touch_mouse(0.3);

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_json_written_before_touch_mouse_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert_eq!(profile.touch_mouse_sensitivity, None);
    }

    #[test]
    fn a_profile_with_no_dead_zone_leaves_sticks_untouched() {
        use crate::types::pad::Stick;

        let profile = Profile::named("no dead zone");
        let state = PadState {
            left_stick: Stick { x: 140, y: 128 },
            ..PadState::default()
        };

        let shaped = shape_sticks(&profile, &state);

        assert_eq!(shaped.left_stick, state.left_stick);
    }

    #[test]
    fn a_profile_with_a_dead_zone_shapes_both_sticks_independently() {
        use crate::types::pad::Stick;

        let profile = Profile::named("dead zone").with_stick_dead_zone(0.5);
        let state = PadState {
            left_stick: Stick { x: 130, y: 128 },
            right_stick: Stick { x: 255, y: 128 },
            ..PadState::default()
        };

        let shaped = shape_sticks(&profile, &state);

        assert_eq!(shaped.left_stick, Stick::centred());
        assert_eq!(shaped.right_stick, Stick { x: 255, y: 128 });
    }

    #[test]
    fn shaping_sticks_never_touches_buttons_or_triggers() {
        use crate::types::pad::CIRCLE;

        let profile = Profile::named("dead zone").with_stick_dead_zone(0.9);
        let state = PadState {
            buttons: CIRCLE,
            left_trigger: 200,
            right_trigger: 50,
            ..PadState::default()
        };

        let shaped = shape_sticks(&profile, &state);

        assert_eq!(shaped.buttons, CIRCLE);
        assert_eq!(shaped.left_trigger, 200);
        assert_eq!(shaped.right_trigger, 50);
    }

    fn timed_step(code: u16, delay_ms: u32) -> TimedStep {
        TimedStep {
            binding: Binding::Key { code },
            delay_ms,
        }
    }

    #[test]
    fn nothing_is_due_before_any_time_has_passed_if_the_first_step_has_a_delay() {
        let steps = vec![timed_step(1, 50), timed_step(2, 50)];

        assert_eq!(steps_due_by(&steps, 0), 0);
    }

    #[test]
    fn a_step_with_no_delay_is_due_immediately() {
        let steps = vec![timed_step(1, 0), timed_step(2, 50)];

        assert_eq!(steps_due_by(&steps, 0), 1);
    }

    #[test]
    fn each_step_becomes_due_only_once_its_own_cumulative_delay_has_passed() {
        let steps = vec![timed_step(1, 0), timed_step(2, 50), timed_step(3, 50)];

        assert_eq!(steps_due_by(&steps, 0), 1);
        assert_eq!(steps_due_by(&steps, 49), 1);
        assert_eq!(steps_due_by(&steps, 50), 2);
        assert_eq!(steps_due_by(&steps, 99), 2);
        assert_eq!(steps_due_by(&steps, 100), 3);
    }

    #[test]
    fn every_step_is_due_once_enough_time_has_passed() {
        let steps = vec![timed_step(1, 0), timed_step(2, 10), timed_step(3, 10)];

        assert_eq!(steps_due_by(&steps, 10_000), 3);
    }

    #[test]
    fn an_empty_step_list_is_never_due_for_anything() {
        assert_eq!(steps_due_by(&[], 10_000), 0);
    }

    #[test]
    fn a_timed_macro_binding_round_trips_through_json() {
        let profile = Profile::named("round trip").bind(
            "Circle",
            Binding::TimedMacro {
                steps: vec![timed_step(0x1B, 0), timed_step(0x20, 150)],
            },
        );

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_timed_macro_binding_produces_no_immediate_outputs_by_itself() {
        let profile = Profile::named("test").bind(
            "Circle",
            Binding::TimedMacro {
                steps: vec![timed_step(0x1B, 0)],
            },
        );

        assert!(outputs_for(&profile, CIRCLE, 0).is_empty());
        assert!(outputs_for(&profile, 0, CIRCLE).is_empty());
    }

    #[test]
    fn a_new_profile_has_no_turbo_buttons_and_a_sane_default_interval() {
        let profile = Profile::named("test");

        assert!(profile.turbo_buttons.is_empty());
        assert_eq!(profile.turbo_interval_ms, 100);
    }

    #[test]
    fn turbo_starts_in_the_pressed_phase() {
        assert!(turbo_pressed_phase(0, 100));
    }

    #[test]
    fn turbo_flips_to_released_at_half_the_interval() {
        assert!(!turbo_pressed_phase(50, 100));
    }

    #[test]
    fn turbo_flips_back_to_pressed_a_full_interval_in() {
        assert!(turbo_pressed_phase(100, 100));
    }

    #[test]
    fn turbo_keeps_alternating_every_half_interval() {
        assert!(!turbo_pressed_phase(150, 100));
        assert!(turbo_pressed_phase(200, 100));
    }

    #[test]
    fn turbo_never_divides_by_zero_on_a_zero_interval() {
        assert!(turbo_pressed_phase(0, 0));
        assert!(!turbo_pressed_phase(1, 0));
    }

    #[test]
    fn a_profile_carrying_turbo_buttons_round_trips_through_json() {
        let profile = Profile::named("round trip")
            .with_turbo("Cross")
            .with_turbo("Circle")
            .with_turbo_interval_ms(40);

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_json_written_before_turbo_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert!(profile.turbo_buttons.is_empty());
        assert_eq!(profile.turbo_interval_ms, 100);
    }

    #[test]
    fn a_new_profile_has_no_shift_button_and_no_shift_bindings() {
        let profile = Profile::named("test");

        assert_eq!(profile.shift_button, None);
        assert!(profile.shift_bindings.is_empty());
    }

    #[test]
    fn holding_the_shift_button_uses_the_shift_binding_when_one_exists() {
        use crate::types::pad::{CIRCLE, L1};

        let profile = Profile::named("test")
            .with_shift_button(L1)
            .bind("Circle", Binding::Key { code: 0x1B })
            .bind_shift("Circle", Binding::Key { code: 0x20 });

        assert_eq!(
            profile.shift_binding_for(CIRCLE, true),
            Some(&Binding::Key { code: 0x20 })
        );
    }

    #[test]
    fn not_holding_the_shift_button_uses_the_primary_binding() {
        use crate::types::pad::{CIRCLE, L1};

        let profile = Profile::named("test")
            .with_shift_button(L1)
            .bind("Circle", Binding::Key { code: 0x1B })
            .bind_shift("Circle", Binding::Key { code: 0x20 });

        assert_eq!(
            profile.shift_binding_for(CIRCLE, false),
            Some(&Binding::Key { code: 0x1B })
        );
    }

    #[test]
    fn a_button_with_no_shift_binding_falls_back_to_the_primary_one_even_while_shifted() {
        use crate::types::pad::{CROSS, L1};

        let profile = Profile::named("test")
            .with_shift_button(L1)
            .bind("Cross", Binding::Key { code: 0x20 });

        assert_eq!(
            profile.shift_binding_for(CROSS, true),
            Some(&Binding::Key { code: 0x20 })
        );
    }

    #[test]
    fn a_profile_carrying_shift_bindings_round_trips_through_json() {
        use crate::types::pad::L1;

        let profile = Profile::named("round trip")
            .with_shift_button(L1)
            .bind("Circle", Binding::Key { code: 0x1B })
            .bind_shift("Circle", Binding::Key { code: 0x20 });

        let text = serde_json::to_string(&profile).unwrap();
        let back: Profile = serde_json::from_str(&text).unwrap();

        assert_eq!(back, profile);
    }

    #[test]
    fn a_profile_json_written_before_shift_existed_still_parses() {
        let text = r#"{"name":"old","bindings":{}}"#;

        let profile: Profile = serde_json::from_str(text).unwrap();

        assert_eq!(profile.shift_button, None);
        assert!(profile.shift_bindings.is_empty());
    }
}
