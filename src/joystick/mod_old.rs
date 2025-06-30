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

/// Joystic

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

    /// Process joystick input and convert to movement parameters
    pub fn process_input(&mut self, x: f32, y: f32, rotation: f32) -> Result<MovementParams, RoboMasterError> {
        self.last_input = Instant::now();

        // Apply deadzone
        let x = self.apply_deadzone(x);
        let y = self.apply_deadzone(y);
        let rotation = self.apply_deadzone(rotation);

        // Apply speed multiplier
        let movement = MovementParams {
            vx: y * self.max_speed,  // Forward/backward
            vy: x * self.max_speed,  // Strafe left/right
            vz: rotation * self.max_speed,  // Rotation
        };

        Ok(movement)
    }

    /// Check if input has timed out (no recent input)
    pub fn is_input_timeout(&self) -> bool {
        self.last_input.elapsed() > self.timeout
    }

    /// Get safe movement (zeros) when input times out
    pub fn get_safe_movement(&self) -> MovementParams {
        MovementParams {
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
        }
    }

    /// Apply deadzone to input value
    fn apply_deadzone(&self, value: f32) -> f32 {
        if value.abs() < self.deadzone {
            0.0
        } else {
            // Scale the value to maintain smooth transition
            let sign = value.signum();
            let magnitude = (value.abs() - self.deadzone) / (1.0 - self.deadzone);
            sign * magnitude.clamp(0.0, 1.0)
        }
    }

    /// Get current deadzone setting
    pub fn deadzone(&self) -> f32 {
        self.deadzone
    }

    /// Get current max speed setting
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Get input timeout setting
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for JoystickController {
    fn default() -> Self {
        Self::new()
    }
}

/// Joystick input event
#[derive(Debug, Clone, Copy)]
pub struct JoystickEvent {
    /// X-axis input (-1.0 to 1.0)
    pub x: f32,
    /// Y-axis input (-1.0 to 1.0) 
    pub y: f32,
    /// Rotation input (-1.0 to 1.0)
    pub rotation: f32,
    /// Button states (placeholder for future implementation)
    pub buttons: u32,
}

impl Default for JoystickEvent {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            buttons: 0,
        }
    }
}

/// Joystick configuration
#[derive(Debug, Clone)]
pub struct JoystickConfig {
    /// Device path or identifier
    pub device: String,
    /// Axis mappings
    pub axis_mapping: AxisMapping,
    /// Button mappings
    pub button_mapping: ButtonMapping,
    /// Calibration settings
    pub calibration: CalibrationSettings,
}

/// Axis mapping configuration
#[derive(Debug, Clone)]
pub struct AxisMapping {
    /// X-axis (strafe)
    pub x_axis: u8,
    /// Y-axis (forward/backward)
    pub y_axis: u8,
    /// Rotation axis
    pub rotation_axis: u8,
    /// Invert axis flags
    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_rotation: bool,
}

/// Button mapping configuration
#[derive(Debug, Clone)]
pub struct ButtonMapping {
    /// Emergency stop button
    pub emergency_stop: Option<u8>,
    /// LED control button
    pub led_toggle: Option<u8>,
    /// Speed modifier button
    pub speed_boost: Option<u8>,
}

/// Calibration settings
#[derive(Debug, Clone)]
pub struct CalibrationSettings {
    /// Center point offsets
    pub center_x: f32,
    pub center_y: f32,
    pub center_rotation: f32,
    /// Scale factors
    pub scale_x: f32,
    pub scale_y: f32,
    pub scale_rotation: f32,
}

impl Default for JoystickConfig {
    fn default() -> Self {
        Self {
            device: "/dev/input/js0".to_string(),
            axis_mapping: AxisMapping {
                x_axis: 0,
                y_axis: 1,
                rotation_axis: 2,
                invert_x: false,
                invert_y: true,  // Typically Y-axis is inverted
                invert_rotation: false,
            },
            button_mapping: ButtonMapping {
                emergency_stop: Some(0),
                led_toggle: Some(1),
                speed_boost: Some(2),
            },
            calibration: CalibrationSettings {
                center_x: 0.0,
                center_y: 0.0,
                center_rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                scale_rotation: 1.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joystick_controller_creation() {
        let controller = JoystickController::new();
        assert_eq!(controller.deadzone(), 0.1);
        assert_eq!(controller.max_speed(), 1.0);
    }

    #[test]
    fn test_deadzone_application() {
        let controller = JoystickController::new().with_deadzone(0.2);
        
        // Test deadzone filtering
        let result = controller.process_input(0.1, 0.1, 0.1).unwrap();
        assert_eq!(result.vx, 0.0);
        assert_eq!(result.vy, 0.0);
        assert_eq!(result.vz, 0.0);
        
        // Test above deadzone
        let result = controller.process_input(0.5, 0.5, 0.5).unwrap();
        assert!(result.vx > 0.0);
        assert!(result.vy > 0.0);
        assert!(result.vz > 0.0);
    }

    #[test]
    fn test_speed_multiplier() {
        let mut controller = JoystickController::new()
            .with_deadzone(0.0)  // No deadzone for this test
            .with_max_speed(0.5);
        
        let result = controller.process_input(1.0, 1.0, 1.0).unwrap();
        assert_eq!(result.vx, 0.5);
        assert_eq!(result.vy, 0.5);
        assert_eq!(result.vz, 0.5);
    }

    #[test]
    fn test_input_timeout() {
        let mut controller = JoystickController::new()
            .with_timeout(Duration::from_millis(100));
        
        // Fresh input should not timeout
        let _ = controller.process_input(0.5, 0.5, 0.0);
        assert!(!controller.is_input_timeout());
        
        // Wait and check timeout
        std::thread::sleep(Duration::from_millis(150));
        assert!(controller.is_input_timeout());
    }

    #[test]
    fn test_safe_movement() {
        let controller = JoystickController::new();
        let safe = controller.get_safe_movement();
        assert_eq!(safe.vx, 0.0);
        assert_eq!(safe.vy, 0.0);
        assert_eq!(safe.vz, 0.0);
    }

    #[test]
    fn test_joystick_event_default() {
        let event = JoystickEvent::default();
        assert_eq!(event.x, 0.0);
        assert_eq!(event.y, 0.0);
        assert_eq!(event.rotation, 0.0);
        assert_eq!(event.buttons, 0);
    }
}
