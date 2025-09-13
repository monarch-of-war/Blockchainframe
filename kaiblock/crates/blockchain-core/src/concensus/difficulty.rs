/// Consensus difficulty utilities (Bitcoin-style compact encoding).
/// This is industry-standard and allows smooth PoW difficulty adjustment.
use num_bigint::BigUint;
use num_traits::{Zero, One};
use std::cmp;

/// Encodes a target (BigUint) into compact `nBits` format.
pub fn target_to_compact(target: &BigUint) -> u32 {
    if target.is_zero() {
        return 0;
    }

    let mut size = (target.bits() + 7) / 8; // number of bytes
    let mut compact: u32;

    let mut target_bytes = target.to_bytes_be();

    // Normalize length to at least size bytes
    while target_bytes.len() < size as usize {
        target_bytes.insert(0, 0u8);
    }

    if target_bytes[0] > 0x7f {
        size += 1;
        compact = ((size as u32) << 24) | ((target_bytes[0] as u32) << 16);
    } else {
        compact = ((size as u32) << 24)
            | ((target_bytes[0] as u32) << 16)
            | ((target_bytes.get(1).cloned().unwrap_or(0) as u32) << 8)
            | (target_bytes.get(2).cloned().unwrap_or(0) as u32);
    }

    compact
}

/// Decodes compact `nBits` into target (BigUint).
pub fn compact_to_target(compact: u32) -> BigUint {
    let size = (compact >> 24) as u32;
    let mut word = compact & 0x007fffff;

    let mut result = if size <= 3 {
        BigUint::from(word) >> (8 * (3 - size))
    } else {
        BigUint::from(word) << (8 * (size - 3))
    };

    result
}

/// Adjusts difficulty using Bitcoin's retarget algorithm.
/// old_target = previous difficulty
/// actual_time = time taken for last interval
/// target_time = expected time for last interval
pub fn retarget(old_target: &BigUint, actual_time: u64, target_time: u64) -> BigUint {
    // Clamp adjustment factor between 0.25x and 4x (Bitcoin rule)
    let mut adjustment = actual_time as f64 / target_time as f64;
    if adjustment < 0.25 {
        adjustment = 0.25;
    }
    if adjustment > 4.0 {
        adjustment = 4.0;
    }

    let new_target = old_target * BigUint::from_f64(adjustment).unwrap();
    new_target
}
