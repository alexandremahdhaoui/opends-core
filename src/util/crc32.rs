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

const POLYNOMIAL: u32 = 0xEDB8_8320;

pub const OUTPUT_REPORT_SEED: u8 = 0xA2;

pub fn checksum(bytes: &[u8]) -> u32 {
    update(0xFFFF_FFFF, bytes) ^ 0xFFFF_FFFF
}

pub fn checksum_with_seed(seed: u8, bytes: &[u8]) -> u32 {
    let started = update(0xFFFF_FFFF, &[seed]);

    update(started, bytes) ^ 0xFFFF_FFFF
}

fn update(mut crc: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        crc ^= u32::from(*byte);

        for _ in 0..8 {
            let carry = crc & 1;
            crc >>= 1;

            if carry != 0 {
                crc ^= POLYNOMIAL;
            }
        }
    }

    crc
}

pub fn append_output_checksum(report: &mut Vec<u8>) {
    let crc = checksum_with_seed(OUTPUT_REPORT_SEED, report);

    report.extend_from_slice(&crc.to_le_bytes());
}

pub fn input_checksum_matches(report: &[u8]) -> bool {
    if report.len() < 5 {
        return false;
    }

    let split = report.len() - 4;
    let (payload, trailer) = report.split_at(split);
    let claimed = u32::from_le_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);

    checksum(payload) == claimed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bluetooth_output_seed_matches_ds4windows_dualsensedevice_cs_line_494() {
        assert_eq!(OUTPUT_REPORT_SEED, 0xA2);
    }

    #[test]
    fn matches_the_published_crc32_check_value() {
        assert_eq!(checksum(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn an_empty_input_checksums_to_zero() {
        assert_eq!(checksum(&[]), 0);
    }

    #[test]
    fn the_seed_changes_the_result_so_a_seeded_report_is_not_a_bare_one() {
        let payload = [0x31u8, 0x02, 0x00];

        assert_ne!(
            checksum(&payload),
            checksum_with_seed(OUTPUT_REPORT_SEED, &payload)
        );
    }

    #[test]
    fn seeding_is_the_same_as_prepending_the_seed_byte() {
        let payload = [0x31u8, 0x02, 0x00, 0xFF];

        let mut prepended = vec![OUTPUT_REPORT_SEED];
        prepended.extend_from_slice(&payload);

        assert_eq!(
            checksum_with_seed(OUTPUT_REPORT_SEED, &payload),
            checksum(&prepended)
        );
    }

    #[test]
    fn an_appended_checksum_is_four_bytes_little_endian() {
        let mut report = vec![0x31u8, 0x02];
        let expected = checksum_with_seed(OUTPUT_REPORT_SEED, &report);

        append_output_checksum(&mut report);

        assert_eq!(report.len(), 6);
        assert_eq!(&report[2..], &expected.to_le_bytes());
    }

    #[test]
    fn an_input_report_validates_against_its_own_trailer() {
        let payload = [0x31u8, 0x01, 0x02, 0x03];
        let mut report = payload.to_vec();
        report.extend_from_slice(&checksum(&payload).to_le_bytes());

        assert!(input_checksum_matches(&report));
    }

    #[test]
    fn one_flipped_bit_fails_the_input_check() {
        let payload = [0x31u8, 0x01, 0x02, 0x03];
        let mut report = payload.to_vec();
        report.extend_from_slice(&checksum(&payload).to_le_bytes());

        report[1] ^= 0x01;

        assert!(!input_checksum_matches(&report));
    }

    #[test]
    fn a_report_too_short_to_hold_a_trailer_fails_rather_than_panicking() {
        assert!(!input_checksum_matches(&[0x31, 0x00]));
        assert!(!input_checksum_matches(&[]));
    }
}
