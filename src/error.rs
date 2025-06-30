//! Error types and handling for RoboMaster control

use thiserror::Error;

/// Result type alias for RoboMaster operations
pub type Result<T> = std::result::Result<T, RoboMasterError>;

/// Main error type for RoboMaster operations
#[derive(Error, Debug)]
pub enum RoboMasterError {
    /// CAN interface errors
    #[error("CAN interface error: {0}")]
    CanInterface(#[from] CanError),

    /// Protocol-related errors
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    /// Control system errors
    #[error("Control error: {0}")]
    Control(#[from] ControlError),

    /// Joystick input errors
    #[cfg(feature = "cli")]
    #[error("Joystick error: {0}")]
    Joystick(#[from] JoystickError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout errors
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Robot not initialized
    #[error("Robot not initialized - call initialize() first")]
    NotInitialized,

    /// Robot already initialized
    #[error("Robot already initialized")]
    AlreadyInitialized,

    /// Invalid parameter
    #[error("Invalid parameter: {parameter} = {value}")]
    InvalidParameter { parameter: String, value: String },

    /// Generic error with context
    #[error("RoboMaster error: {message}")]
    Generic { message: String },
}

/// CAN interface specific errors
#[derive(Error, Debug)]
pub enum CanError {
    /// Failed to open CAN interface
    #[error("Failed to open CAN interface '{interface}': {source}")]
    OpenFailed {
        interface: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to send CAN message
    #[error("Failed to send CAN message: {0}")]
    SendFailed(std::io::Error),

    /// Failed to receive CAN message
    #[error("Failed to receive CAN message: {0}")]
    ReceiveFailed(std::io::Error),

    /// Invalid CAN message data length
    #[error("Invalid CAN data length: {length} bytes (max: {max_length})")]
    InvalidDataLength { length: usize, max_length: usize },

    /// Failed to create CAN frame
    #[error("Failed to create CAN frame: {0}")]
    FrameCreation(std::io::Error),

    /// Invalid CAN message format
    #[error("Invalid CAN message: {reason}")]
    InvalidMessage { reason: String },

    /// CAN interface not available
    #[error("CAN interface '{interface}' not available")]
    InterfaceNotAvailable { interface: String },
}

/// Protocol parsing and generation errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// CRC checksum mismatch
    #[error("CRC checksum mismatch: expected {expected:04x}, got {actual:04x}")]
    CrcMismatch { expected: u16, actual: u16 },

    /// Invalid command ID
    #[error("Invalid command ID: {command_id}")]
    InvalidCommandId { command_id: u8 },

    /// Message too short
    #[error("Message too short: expected at least {expected} bytes, got {actual}")]
    MessageTooShort { expected: usize, actual: usize },

    /// Message too long
    #[error("Message too long: maximum {max} bytes, got {actual}")]
    MessageTooLong { max: usize, actual: usize },

    /// Invalid message header
    #[error("Invalid message header: {reason}")]
    InvalidHeader { reason: String },

    /// Unsupported command
    #[error("Unsupported command: {command}")]
    UnsupportedCommand { command: String },

    /// Invalid command length
    #[error("Invalid command length for command ID: {command_id}")]
    InvalidCommandLength { command_id: usize },

    /// Command not found
    #[error("Command not found: {command_id}")]
    CommandNotFound { command_id: usize },
}

/// Control system errors
#[derive(Error, Debug)]
pub enum ControlError {
    /// Speed value out of range
    #[error("Speed out of range: {value} (valid range: {min} to {max})")]
    SpeedOutOfRange { value: f32, min: f32, max: f32 },

    /// Invalid LED color value
    #[error("LED color out of range: {component}={value} (valid range: 0-255)")]
    LedColorOutOfRange { component: String, value: i32 },

    /// Robot movement blocked
    #[error("Robot movement blocked: {reason}")]
    MovementBlocked { reason: String },

    /// Sensor data unavailable
    #[error("Sensor data unavailable: {sensor}")]
    SensorUnavailable { sensor: String },

    /// Control loop error
    #[error("Control loop error: {0}")]
    ControlLoop(String),
}

/// Joystick input errors
#[cfg(feature = "cli")]
#[derive(Error, Debug)]
pub enum JoystickError {
    /// Joystick not found
    #[error("Joystick {id} not found")]
    NotFound { id: u32 },

    /// Failed to read joystick input
    #[error("Failed to read joystick input: {0}")]
    ReadFailed(std::io::Error),

    /// Invalid joystick configuration
    #[error("Invalid joystick configuration: {reason}")]
    InvalidConfig { reason: String },

    /// Joystick disconnected
    #[error("Joystick disconnected")]
    Disconnected,
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to load configuration file
    #[error("Failed to load config from '{path}': {source}")]
    LoadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse configuration
    #[error("Failed to parse config: {0}")]
    ParseFailed(#[from] toml::de::Error),

    /// Invalid configuration value
    #[error("Invalid config value: {key} = {value}")]
    InvalidValue { key: String, value: String },

    /// Missing required configuration
    #[error("Missing required config: {key}")]
    MissingRequired { key: String },
}

impl RoboMasterError {
    /// Create a generic error with a message
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::CanInterface(CanError::SendFailed(_))
            | Self::CanInterface(CanError::ReceiveFailed(_))
            | Self::CanInterface(CanError::InvalidMessage { .. })
            | Self::Timeout { .. } => true,
            Self::CanInterface(CanError::OpenFailed { .. })
            | Self::CanInterface(CanError::InvalidDataLength { .. })
            | Self::CanInterface(CanError::FrameCreation(_))
            | Self::CanInterface(CanError::InterfaceNotAvailable { .. }) => false,
            Self::NotInitialized | Self::AlreadyInitialized => false,
            Self::Protocol(_) => false,
            Self::Control(ControlError::SensorUnavailable { .. }) => true,
            Self::Control(_) => false,
            #[cfg(feature = "cli")]
            Self::Joystick(JoystickError::ReadFailed(_)) => true,
            #[cfg(feature = "cli")]
            Self::Joystick(_) => false,
            Self::Config(_) => false,
            Self::Io(_) => true,
            Self::InvalidParameter { .. } => false,
            Self::Generic { .. } => false,
        }
    }

    /// Get error category for logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::CanInterface(_) => "can",
            Self::Protocol(_) => "protocol",
            Self::Control(_) => "control",
            #[cfg(feature = "cli")]
            Self::Joystick(_) => "joystick",
            Self::Config(_) => "config",
            Self::Io(_) => "io",
            Self::Timeout { .. } => "timeout",
            Self::NotInitialized | Self::AlreadyInitialized => "state",
            Self::InvalidParameter { .. } => "parameter",
            Self::Generic { .. } => "generic",
        }
    }
}

// Convenience constructors
impl From<&str> for RoboMasterError {
    fn from(message: &str) -> Self {
        Self::generic(message)
    }
}

impl From<String> for RoboMasterError {
    fn from(message: String) -> Self {
        Self::generic(message)
    }
}
