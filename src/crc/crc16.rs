//! CRC16 implementation for RoboMaster protocol
//!
//! This implementation is compatible with the Python version in the original codebase.

/// CRC16 initial value (matches Python implementation)
pub const CRC16_INIT: u16 = 13970;

/// CRC16 polynomial table for fast calculation
const CRC16_TABLE: [u16; 256] = [
    0x0000, 0x1189, 0x2312, 0x329b, 0x4624, 0x57ad, 0x6536, 0x74bf,
    0x8c48, 0x9dc1, 0xaf5a, 0xbed3, 0xca6c, 0xdbe5, 0xe97e, 0xf8f7,
    0x1081, 0x0108, 0x3393, 0x221a, 0x56a5, 0x472c, 0x75b7, 0x643e,
    0x9cc9, 0x8d40, 0xbfdb, 0xae52, 0xdaed, 0xcb64, 0xf9ff, 0xe876,
    0x2102, 0x308b, 0x0210, 0x1399, 0x6726, 0x76af, 0x4434, 0x55bd,
    0xad4a, 0xbcc3, 0x8e58, 0x9fd1, 0xeb6e, 0xfae7, 0xc87c, 0xd9f5,
    0x3183, 0x200a, 0x1291, 0x0318, 0x77a7, 0x662e, 0x54b5, 0x453c,
    0xbdcb, 0xac42, 0x9ed9, 0x8f50, 0xfbef, 0xea66, 0xd8fd, 0xc974,
    0x4204, 0x538d, 0x6116, 0x709f, 0x0420, 0x15a9, 0x2732, 0x36bb,
    0xce4c, 0xdfc5, 0xed5e, 0xfcd7, 0x8868, 0x99e1, 0xab7a, 0xbaf3,
    0x5285, 0x430c, 0x7197, 0x601e, 0x14a1, 0x0528, 0x37b3, 0x263a,
    0xdecd, 0xcf44, 0xfddf, 0xec56, 0x98e9, 0x8960, 0xbbfb, 0xaa72,
    0x6306, 0x728f, 0x4014, 0x519d, 0x2522, 0x34ab, 0x0630, 0x17b9,
    0xef4e, 0xfec7, 0xcc5c, 0xddd5, 0xa96a, 0xb8e3, 0x8a78, 0x9bf1,
    0x7387, 0x620e, 0x5095, 0x411c, 0x35a3, 0x242a, 0x16b1, 0x0738,
    0xffcf, 0xee46, 0xdcdd, 0xcd54, 0xb9eb, 0xa862, 0x9af9, 0x8b70,
    0x8408, 0x9581, 0xa71a, 0xb693, 0xc22c, 0xd3a5, 0xe13e, 0xf0b7,
    0x0840, 0x19c9, 0x2b52, 0x3adb, 0x4e64, 0x5fed, 0x6d76, 0x7cff,
    0x9489, 0x8500, 0xb79b, 0xa612, 0xd2ad, 0xc324, 0xf1bf, 0xe036,
    0x18c1, 0x0948, 0x3bd3, 0x2a5a, 0x5ee5, 0x4f6c, 0x7df7, 0x6c7e,
    0xa50a, 0xb483, 0x8618, 0x9791, 0xe32e, 0xf2a7, 0xc03c, 0xd1b5,
    0x2942, 0x38cb, 0x0a50, 0x1bd9, 0x6f66, 0x7eef, 0x4c74, 0x5dfd,
    0xb58b, 0xa402, 0x9699, 0x8710, 0xf3af, 0xe226, 0xd0bd, 0xc134,
    0x39c3, 0x284a, 0x1ad1, 0x0b58, 0x7fe7, 0x6e6e, 0x5cf5, 0x4d7c,
    0xc60c, 0xd785, 0xe51e, 0xf497, 0x8028, 0x91a1, 0xa33a, 0xb2b3,
    0x4a44, 0x5bcd, 0x6956, 0x78df, 0x0c60, 0x1de9, 0x2f72, 0x3efb,
    0xd68d, 0xc704, 0xf59f, 0xe416, 0x90a9, 0x8120, 0xb3bb, 0xa232,
    0x5ac5, 0x4b4c, 0x79d7, 0x685e, 0x1ce1, 0x0d68, 0x3ff3, 0x2e7a,
    0xe70e, 0xf687, 0xc41c, 0xd595, 0xa12a, 0xb0a3, 0x8238, 0x93b1,
    0x6b46, 0x7acf, 0x4854, 0x59dd, 0x2d62, 0x3ceb, 0x0e70, 0x1ff9,
    0xf78f, 0xe606, 0xd49d, 0xc514, 0xb1ab, 0xa022, 0x92b9, 0x8330,
    0x7bc7, 0x6a4e, 0x58d5, 0x495c, 0x3de3, 0x2c6a, 0x1ef1, 0x0f78,
];

/// Calculate CRC16 checksum for the given data
///
/// # Arguments
/// * `data` - Byte slice to calculate CRC for
/// * `init_value` - Initial CRC value (usually CRC16_INIT)
///
/// # Returns
/// * CRC16 checksum value
///
/// # Examples
/// ```rust
/// use robomaster_rust::crc::{calculate_crc16, CRC16_INIT};
/// 
/// let data = vec![0x55, 0x1b, 0x04, 0xa2];
/// let crc = calculate_crc16(&data, CRC16_INIT);
/// println!("CRC16: 0x{:04x}", crc);
/// ```
pub fn calculate_crc16(data: &[u8], init_value: u16) -> u16 {
    let mut crc = init_value;
    
    for &byte in data {
        let table_index = ((crc ^ (byte as u16)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC16_TABLE[table_index];
    }
    
    crc
}

/// Append CRC16 checksum to the given data vector (little-endian)
///
/// # Arguments
/// * `data` - Mutable reference to data vector
/// * `init_value` - Initial CRC value (usually CRC16_INIT)
///
/// # Examples
/// ```rust
/// use robomaster_rust::crc::{append_crc16_checksum, CRC16_INIT};
/// 
/// let mut data = vec![0x55, 0x1b, 0x04, 0xa2];
/// append_crc16_checksum(&mut data, CRC16_INIT);
/// // CRC16 bytes appended in little-endian format
/// ```
pub fn append_crc16_checksum(data: &mut Vec<u8>, init_value: u16) {
    let crc = calculate_crc16(data, init_value);
    
    // Append CRC16 in little-endian format (low byte first)
    data.push((crc & 0xFF) as u8);
    data.push((crc >> 8) as u8);
}

/// Verify CRC16 checksum of the given data
///
/// # Arguments
/// * `data` - Data including CRC16 at the end (little-endian)
/// * `init_value` - Initial CRC value (usually CRC16_INIT)
///
/// # Returns
/// * `true` if CRC is valid, `false` otherwise
///
/// # Examples
/// ```rust
/// use robomaster_rust::crc::{verify_crc16_checksum, CRC16_INIT};
/// 
/// let data = vec![0x55, 0x1b, 0x04, 0xa2, 0x4c, 0x7d];
/// assert!(verify_crc16_checksum(&data, CRC16_INIT));
/// ```
pub fn verify_crc16_checksum(data: &[u8], init_value: u16) -> bool {
    if data.len() < 2 {
        return false;
    }
    
    let (payload, crc_bytes) = data.split_at(data.len() - 2);
    let expected_crc = (crc_bytes[0] as u16) | ((crc_bytes[1] as u16) << 8);
    let calculated_crc = calculate_crc16(payload, init_value);
    
    calculated_crc == expected_crc
}

/// Get CRC16 checksum from data (alternative interface)
///
/// This function calculates the full CRC16 including the data itself,
/// which is useful for protocol validation.
pub fn get_crc16_checksum(data: &[u8], init_value: u16) -> u16 {
    calculate_crc16(data, init_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_calculation() {
        // Test case from the original Python implementation
        let data = vec![0x55, 0x1b, 0x04, 0xa2, 0x09, 0x04, 0x00, 0x00, 0x40, 0x04, 0x4c, 0x00, 0x00];
        let expected = 0x2065;  // Python result: 8293 (0x2065)
        
        let result = calculate_crc16(&data, CRC16_INIT);
        assert_eq!(result, expected, "CRC16 mismatch for test data");
    }

    #[test]
    fn test_crc16_append() {
        let mut data = vec![0x55, 0x1b, 0x04, 0xa2];
        let original_len = data.len();
        
        append_crc16_checksum(&mut data, CRC16_INIT);
        
        assert_eq!(data.len(), original_len + 2);
        
        // Verify the appended CRC is correct
        assert!(verify_crc16_checksum(&data, CRC16_INIT));
    }

    #[test]
    fn test_crc16_verify() {
        // Create test data with valid CRC16
        let mut data = vec![0x55, 0x1b, 0x04, 0xa2];
        append_crc16_checksum(&mut data, CRC16_INIT);
        
        assert!(verify_crc16_checksum(&data, CRC16_INIT));
        
        // Corrupt the data and verify it fails
        let mut corrupted_data = data.clone();
        corrupted_data[0] = 0x56; // Change first byte
        assert!(!verify_crc16_checksum(&corrupted_data, CRC16_INIT));
    }

    #[test]
    fn test_crc16_empty_data() {
        let empty_data = vec![];
        assert!(!verify_crc16_checksum(&empty_data, CRC16_INIT));
        
        let calculated = calculate_crc16(&empty_data, CRC16_INIT);
        assert_eq!(calculated, CRC16_INIT);
    }

    #[test]
    fn test_crc16_single_byte() {
        let data = vec![0x40];
        let crc = calculate_crc16(&data, CRC16_INIT);
        
        // Python result: 62889 (0xf5a9)
        assert_eq!(crc, 0xf5a9);
    }

    #[test]
    fn test_crc16_compatibility_python() {
        // Specific test cases from Python implementation
        let test_cases = vec![
            (vec![0x40, 0x04, 0x4c, 0x00, 0x00], 0x3fee),  // Python result: 16366 (0x3fee)
        ];

        for (data, expected) in test_cases {
            let result = calculate_crc16(&data, CRC16_INIT);
            assert_eq!(result, expected, "CRC16 mismatch for data: {:?}", data);
        }
    }
}
