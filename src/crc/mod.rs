//! CRC calculation utilities for RoboMaster protocol
//!
//! This module provides CRC8 and CRC16 implementations that are compatible
//! with the original Python implementation.

pub mod crc8;
pub mod crc16;

pub use crc8::{calculate_crc8, append_crc8_checksum, verify_crc8_checksum};
pub use crc16::{calculate_crc16, append_crc16_checksum, verify_crc16_checksum, CRC16_INIT};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc8_compatibility() {
        // Test data from Python implementation
        let data = vec![0x55, 0x0f, 0x04];
        let expected_crc = 0xa2; // Expected CRC8 from Python
        
        let calculated = calculate_crc8(&data);
        assert_eq!(calculated, expected_crc, "CRC8 calculation mismatch");
    }

    #[test]
    fn test_crc16_compatibility() {
        // Test data from Python implementation
        let data = vec![0x40, 0x04, 0x4c, 0x00, 0x00];
        let expected_crc = 0x3fee; // Expected CRC16 from Python: 16366 (0x3fee)
        
        let calculated = calculate_crc16(&data, CRC16_INIT);
        assert_eq!(calculated, expected_crc, "CRC16 calculation mismatch");
    }
}
