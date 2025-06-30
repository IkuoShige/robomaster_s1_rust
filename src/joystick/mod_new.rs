/// Joystick input handling module
/// This module provides joystick input processing for robot control

use crate::command::MovementParams;
use crate::error::RoboMasterError;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Controller input structure
#[derive(Debug, Clone, Copy, Default)]
pub struct ControllerInput {
    /// Left stick X axis (-1.0 to 1.0)
    pub left_stick_x: f32,
    /// Left stick Y axis (-1.0 to 1.0)
    pub left_stick_y: f32,
    /// Right stick X axis (-1.0 to 1.0)
    pub right_stick_x: f32,
    /// Right stick Y axis (-1.0 to 1.0)
    pub right_stick_y: f32,
    /// Left trigger (0.0 to 1.0)
    pub left_trigger: f32,
    /// Right trigger (0.0 to 1.0)
    pub right_trigger: f32,
    /// Face button states
    pub face_button_north: bool,
    pub face_button_south: bool,
    pub face_button_east: bool,
    pub face_button_west: bool,
    /// Shoulder button states
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    /// D-pad states
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    /// Menu buttons
    pub start_pressed: bool,
    pub select_pressed: bool,
}

/// Joystick manager for handling controller input
pub struct JoystickManager {
    /// Current controller input state
    current_input: Option<ControllerInput>,
    /// Deadzone for analog inputs
    deadzone: f32,
    /// Input timeout
    timeout: Duration,
    /// Last input timestamp
    last_input: Instant,
}

impl JoystickManager {
    /// Create a new joystick manager
    pub async fn new() -> Result<Self, RoboMasterError> {
        Ok(Self {
            current_input: None,
            deadzone: 0.1,
            timeout: Duration::from_millis(100),
            last_input: Instant::now(),
        })
    }

    /// Get current controller input
    pub async fn get_input(&mut self) -> Result<Option<ControllerInput>, RoboMasterError> {
        // For now, return mock input for testing
        // In a real implementation, this would read from a gamepad library
        let now = Instant::now();
        if now.duration_since(self.last_input) > self.timeout {
            // Simulate no controller input
            Ok(None)
        } else {
            // Simulate some basic input
            Ok(Some(ControllerInput::default()))
        }
    }

    /// Set deadzone for analog inputs
    pub fn set_deadzone(&mut self, deadzone: f32) {
        self.deadzone = deadzone.clamp(0.0, 1.0);
    }

    /// Set input timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
}

/// Joystick controller for robot input processing
pub struct JoystickController {
    /// Deadzone for joystick inputs (0.0 to 1.0)
    deadzone: f32,
    /// Maximum speed multiplier
    max_speed: f32,
    /// Last input timestamp
    last_input: Instant,
    /// Input timeout
    timeout: Duration,
}

impl Default for JoystickController {
    fn default() -> Self {
        Self::new()
    }
}

impl JoystickController {
    /// Create a new joystick controller
    pub fn new() -> Self {
        Self {
            deadzone: 0.1,
            max_speed: 1.0,
            last_input: Instant::now(),
            timeout: Duration::from_millis(500),
        }
    }

    /// Set joystick deadzone
    pub fn with_deadzone(mut self, deadzone: f32) -> Self {
        self.deadzone = deadzone.clamp(0.0, 1.0);
        self
    }

    /// Set maximum speed multiplier
    pub fn with_max_speed(mut self, max_speed: f32) -> Self {
        self.max_speed = max_speed.clamp(0.0, 2.0);
        self
    }

    /// Set input timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Process raw joystick input and convert to robot movement
    pub fn process_input(&mut self, x: f32, y: f32, rotation: f32) -> Result<MovementParams, RoboMasterError> {
        self.last_input = Instant::now();

        // Apply deadzone
        let x_filtered = if x.abs() < self.deadzone { 0.0 } else { x };
        let y_filtered = if y.abs() < self.deadzone { 0.0 } else { y };
        let rotation_filtered = if rotation.abs() < self.deadzone { 0.0 } else { rotation };

        // Scale by maximum speed
        let vx = (y_filtered * self.max_speed).clamp(-1.0, 1.0);
        let vy = (x_filtered * self.max_speed).clamp(-1.0, 1.0);
        let vz = (rotation_filtered * self.max_speed).clamp(-1.0, 1.0);

        Ok(MovementParams { vx, vy, vz })
    }

    /// Check if input has timed out
    pub fn has_input_timeout(&self) -> bool {
        self.last_input.elapsed() > self.timeout
    }

    /// Get current deadzone
    pub fn deadzone(&self) -> f32 {
        self.deadzone
    }

    /// Get current max speed
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Get input timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Configuration options for joystick controller
#[derive(Debug, Clone)]
pub struct JoystickConfig {
    /// Invert Y axis
    pub invert_y: bool,
    /// Invert rotation axis  
    pub invert_rotation: bool,
}

impl Default for JoystickConfig {
    fn default() -> Self {
        Self {
            invert_y: false,
            invert_rotation: false,
        }
    }
}

/// Advanced joystick controller with additional features
#[derive(Debug, Clone)]
pub struct AdvancedJoystickController {
    /// Base controller
    base: JoystickController,
    /// Configuration
    config: JoystickConfig,
    /// Calibration data
    calibration: CalibrationData,
}

/// Calibration data for joystick
#[derive(Debug, Clone)]
pub struct CalibrationData {
    /// Center positions
    pub center_y: f32,
    pub center_rotation: f32,
    /// Scale factors
    pub scale_y: f32,
    pub scale_rotation: f32,
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            center_y: 0.0,
            center_rotation: 0.0,
            scale_y: 1.0,
            scale_rotation: 1.0,
        }
    }
}

impl AdvancedJoystickController {
    /// Create a new advanced joystick controller
    pub fn new() -> Self {
        Self {
            base: JoystickController::new(),
            config: JoystickConfig::default(),
            calibration: CalibrationData::default(),
        }
    }

    /// With custom configuration
    pub fn with_config(mut self, config: JoystickConfig) -> Self {
        self.config = config;
        self
    }

    /// With custom calibration
    pub fn with_calibration(mut self, calibration: CalibrationData) -> Self {
        self.calibration = calibration;
        self
    }

    /// Process input with advanced features
    pub fn process_advanced_input(&mut self, input: ControllerInput) -> Result<MovementParams, RoboMasterError> {
        let mut y = input.left_stick_y;
        let mut rotation = input.right_stick_x;

        // Apply calibration
        y = (y - self.calibration.center_y) * self.calibration.scale_y;
        rotation = (rotation - self.calibration.center_rotation) * self.calibration.scale_rotation;

        // Apply configuration
        if self.config.invert_y {
            y = -y;
        }
        if self.config.invert_rotation {
            rotation = -rotation;
        }

        self.base.process_input(input.left_stick_x, y, rotation)
    }
}

impl Default for AdvancedJoystickController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = JoystickController::new();
        assert_eq!(controller.deadzone(), 0.1);
        assert_eq!(controller.max_speed(), 1.0);
    }

    #[test]
    fn test_deadzone_application() {
        let mut controller = JoystickController::new().with_deadzone(0.2);
        
        // Test deadzone filtering
        let result = controller.process_input(0.1, 0.1, 0.1).unwrap();
        assert_eq!(result.vx, 0.0);
        assert_eq!(result.vy, 0.0);
        assert_eq!(result.vz, 0.0);
        
        // Test normal input (outside deadzone)
        let result = controller.process_input(0.5, 0.5, 0.5).unwrap();
        assert_ne!(result.vx, 0.0);
        assert_ne!(result.vy, 0.0);
        assert_ne!(result.vz, 0.0);
    }

    #[test]
    fn test_speed_scaling() {
        let mut controller = JoystickController::new().with_max_speed(0.5);
        
        let result = controller.process_input(1.0, 1.0, 1.0).unwrap();
        assert!(result.vx.abs() <= 0.5);
        assert!(result.vy.abs() <= 0.5);
        assert!(result.vz.abs() <= 0.5);
    }

    #[test]
    fn test_input_clamping() {
        let mut controller = JoystickController::new();
        
        let result = controller.process_input(2.0, -2.0, 1.5).unwrap();
        assert!(result.vx >= -1.0 && result.vx <= 1.0);
        assert!(result.vy >= -1.0 && result.vy <= 1.0);
        assert!(result.vz >= -1.0 && result.vz <= 1.0);
    }

    #[test]
    fn test_controller_input_default() {
        let input = ControllerInput::default();
        assert_eq!(input.left_stick_x, 0.0);
        assert_eq!(input.left_stick_y, 0.0);
        assert!(!input.face_button_north);
        assert!(!input.start_pressed);
    }

    #[test]
    fn test_advanced_controller() {
        let config = JoystickConfig {
            invert_y: true,
            invert_rotation: false,
        };
        
        let mut advanced = AdvancedJoystickController::new().with_config(config);
        
        let input = ControllerInput {
            left_stick_x: 0.5,
            left_stick_y: 0.5,
            right_stick_x: 0.3,
            ..Default::default()
        };
        
        let result = advanced.process_advanced_input(input).unwrap();
        assert_eq!(result.vx, -0.5); // Y is inverted
        assert_eq!(result.vy, 0.5);
        assert_eq!(result.vz, 0.3);
    }
}
