use anyhow::Result;
use crate::error::{RoboMasterError, CanError};
use socketcan::{CanSocket, CanFrame, Socket, EmbeddedFrame, StandardId};
use std::time::Duration;
use tokio::time::timeout;

/// CAN arbitration ID used for RoboMaster communication
pub const ROBOMASTER_CAN_ID: u16 = 0x201;

/// Default timeout for CAN operations
pub const DEFAULT_CAN_TIMEOUT: Duration = Duration::from_millis(200);

/// Maximum CAN frame data length
pub const CAN_MAX_DATA_LEN: usize = 8;

/// CAN interface abstraction for RoboMaster communication
pub struct CanInterface {
    socket: CanSocket,
    interface_name: String,
}

impl CanInterface {
    /// Create a new CAN interface
    pub fn new(interface_name: &str) -> Result<Self, RoboMasterError> {
        println!("----------------------can open----------------------");
        
        let socket = CanSocket::open(interface_name)
            .map_err(|e| RoboMasterError::CanInterface(CanError::OpenFailed {
                interface: interface_name.to_string(),
                source: e,
            }))?;

        println!("generated can bus");
        
        Ok(Self {
            socket,
            interface_name: interface_name.to_string(),
        })
    }

    /// Send a single CAN message
    pub fn send_message(&self, data: &[u8]) -> Result<(), RoboMasterError> {
        if data.len() > CAN_MAX_DATA_LEN {
            return Err(RoboMasterError::CanInterface(CanError::InvalidDataLength {
                length: data.len(),
                max_length: CAN_MAX_DATA_LEN,
            }));
        }

        let standard_id = StandardId::new(ROBOMASTER_CAN_ID)
            .ok_or_else(|| RoboMasterError::CanInterface(CanError::InvalidMessage {
                reason: "Invalid CAN ID".to_string(),
            }))?;
            
        let frame = CanFrame::new(standard_id, data)
            .ok_or_else(|| RoboMasterError::CanInterface(CanError::FrameCreation(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to create CAN frame")
            )))?;

        self.socket.write_frame(&frame)
            .map_err(|e| RoboMasterError::CanInterface(CanError::SendFailed(e)))?;

        Ok(())
    }

    /// Send multiple CAN messages
    pub fn send_messages(&self, messages: &[Vec<u8>]) -> Result<(), RoboMasterError> {
        for msg in messages {
            self.send_message(msg)?;
        }
        Ok(())
    }

    /// Receive a CAN message with timeout
    pub async fn receive_message(&self, timeout_duration: Duration) -> Result<Option<CanFrame>, RoboMasterError> {
        let recv_future = async {
            self.socket.read_frame()
                .map_err(|e| RoboMasterError::CanInterface(CanError::ReceiveFailed(e)))
        };

        match timeout(timeout_duration, recv_future).await {
            Ok(Ok(frame)) => Ok(Some(frame)),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                println!("Time out");
                Ok(None)
            }
        }
    }

    /// Receive and process messages to extract command counters
    pub async fn receive_and_process(&self, cmd_counters: &mut CommandCounters) -> Result<(), RoboMasterError> {
        if let Some(frame) = self.receive_message(DEFAULT_CAN_TIMEOUT).await? {
            let frame_id = match frame.id() {
                socketcan::Id::Standard(std_id) => std_id.as_raw(),
                socketcan::Id::Extended(_) => return Ok(()), // Skip extended frames
            };
            
            if frame_id == ROBOMASTER_CAN_ID {
                let data = frame.data();
                if data.len() >= 8 && data[0..6] == [0x55, 0x1b, 0x04, 0x75, 0x09, 0xc3] {
                    let counter = (data[6] as u16) | ((data[7] as u16) << 8);
                    cmd_counters.joy = counter + 1;
                }
            }
        }
        Ok(())
    }

    /// Close the CAN interface
    pub fn shutdown(&self) {
        println!("----------------------shutdown----------------------");
        // The socket will be automatically closed when dropped
    }

    /// Get the interface name
    pub fn interface_name(&self) -> &str {
        &self.interface_name
    }
}

impl Drop for CanInterface {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Command counters for different command types
#[derive(Debug, Clone)]
pub struct CommandCounters {
    pub joy: u16,
    pub led: u16,
    pub gimbal: u16,
}

impl Default for CommandCounters {
    fn default() -> Self {
        Self {
            joy: 0,
            led: 0,
            gimbal: 0,
        }
    }
}

/// Message splitter for converting commands to CAN frames
pub struct MessageSplitter;

impl MessageSplitter {
    /// Split a command into 8-byte CAN frames
    pub fn split_command(command: &[u8]) -> Vec<Vec<u8>> {
        let mut can_command_list = Vec::new();
        let chunks = (command.len() + CAN_MAX_DATA_LEN - 1) / CAN_MAX_DATA_LEN;
        
        for i in 0..chunks {
            let start = i * CAN_MAX_DATA_LEN;
            let end = std::cmp::min(start + CAN_MAX_DATA_LEN, command.len());
            can_command_list.push(command[start..end].to_vec());
        }
        
        can_command_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_splitter_exact_size() {
        let command = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let result = MessageSplitter::split_command(&command);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], command);
    }

    #[test]
    fn test_message_splitter_multiple_frames() {
        let command = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let result = MessageSplitter::split_command(&command);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(result[1], vec![9, 10, 11, 12]);
    }

    #[test]
    fn test_message_splitter_uneven_split() {
        let command = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let result = MessageSplitter::split_command(&command);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(result[1], vec![9]);
    }

    #[test]
    fn test_command_counters_default() {
        let counters = CommandCounters::default();
        assert_eq!(counters.joy, 0);
        assert_eq!(counters.led, 0);
        assert_eq!(counters.gimbal, 0);
    }
}
