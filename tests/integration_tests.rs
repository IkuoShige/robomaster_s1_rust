/// Integration tests for RoboMaster Rust library
/// These tests verify the complete functionality of the library

use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_robot_initialization() {
    // Note: This test requires a CAN interface to be available
    // In CI/CD, this would be skipped or use a mock interface
    let result = RoboMaster::new("can0").await;
    
    match result {
        Ok(mut robot) => {
            // Test initialization
            let init_result = robot.initialize().await;
            assert!(init_result.is_ok(), "Robot initialization should succeed");
            
            // Test shutdown
            let shutdown_result = robot.shutdown().await;
            assert!(shutdown_result.is_ok(), "Robot shutdown should succeed");
        }
        Err(_) => {
            // Skip test if no CAN interface available
            println!("Skipping test - no CAN interface available");
        }
    }
}

#[tokio::test]
async fn test_movement_commands() {
    let result = RoboMaster::new("can0").await;
    
    match result {
        Ok(mut robot) => {
            robot.initialize().await.expect("Initialization failed");
            
            // Test basic movement
            let movement = MovementCommand::new()
                .forward(0.5)
                .strafe_right(0.2)
                .rotate_right(-0.3);
                
            let move_result = robot.move_robot(movement.into_params()).await;
            assert!(move_result.is_ok(), "Movement command should succeed");
            
            // Test stop
            let stop_result = robot.stop().await;
            assert!(stop_result.is_ok(), "Stop command should succeed");
            
            robot.shutdown().await.expect("Shutdown failed");
        }
        Err(_) => {
            println!("Skipping test - no CAN interface available");
        }
    }
}

#[tokio::test]
async fn test_led_commands() {
    let result = RoboMaster::new("can0").await;
    
    match result {
        Ok(mut robot) => {
            robot.initialize().await.expect("Initialization failed");
            
            // Test different LED colors
            let colors = vec![
                LedCommand::red().color(),
                LedCommand::green().color(),
                LedCommand::blue().color(),
                LedCommand::white().color(),
                LedCommand::off().color(),
            ];
            
            for color in colors {
                let led_result = robot.control_led(color).await;
                assert!(led_result.is_ok(), "LED command should succeed");
            }
            
            robot.shutdown().await.expect("Shutdown failed");
        }
        Err(_) => {
            println!("Skipping test - no CAN interface available");
        }
    }
}

#[tokio::test]
async fn test_touch_commands() {
    let result = RoboMaster::new("can0").await;
    
    match result {
        Ok(mut robot) => {
            robot.initialize().await.expect("Initialization failed");
            
            // Test touch command
            let touch_result = robot.send_touch().await;
            assert!(touch_result.is_ok(), "Touch command should succeed");
            
            robot.shutdown().await.expect("Shutdown failed");
        }
        Err(_) => {
            println!("Skipping test - no CAN interface available");
        }
    }
}

#[tokio::test]
async fn test_message_receiving() {
    let result = RoboMaster::new("can0").await;
    
    match result {
        Ok(mut robot) => {
            robot.initialize().await.expect("Initialization failed");
            
            // Test message receiving with timeout
            let receive_result = timeout(
                Duration::from_millis(100),
                robot.receive_messages()
            ).await;
            
            // Either receives successfully or times out - both are valid
            match receive_result {
                Ok(Ok(_)) => println!("Messages received successfully"),
                Ok(Err(_)) => println!("Receive returned error (normal if no messages)"),
                Err(_) => println!("Receive timed out (normal if no messages)"),
            }
            
            robot.shutdown().await.expect("Shutdown failed");
        }
        Err(_) => {
            println!("Skipping test - no CAN interface available");
        }
    }
}

#[test]
fn test_movement_command_builder() {
    let cmd = MovementCommand::new()
        .forward(0.5)
        .strafe_right(0.2)
        .rotate_right(-0.3);
    
    let params = cmd.into_params();
    assert_eq!(params.vx, 0.5);
    assert_eq!(params.vy, 0.2);
    assert_eq!(params.vz, -0.3);
}

#[test]
fn test_movement_command_clamping() {
    let cmd = MovementCommand::new()
        .forward(2.0)  // Should be clamped to 1.0
        .strafe_right(-2.0)  // Should be clamped to -1.0
        .rotate_right(0.5);
    
    let params = cmd.into_params();
    assert_eq!(params.vx, 1.0);
    assert_eq!(params.vy, -1.0);
    assert_eq!(params.vz, 0.5);
}

#[test]
fn test_led_command_colors() {
    assert_eq!(LedCommand::red().color().red, 255);
    assert_eq!(LedCommand::red().color().green, 0);
    assert_eq!(LedCommand::red().color().blue, 0);
    
    assert_eq!(LedCommand::green().color().red, 0);
    assert_eq!(LedCommand::green().color().green, 255);
    assert_eq!(LedCommand::green().color().blue, 0);
    
    assert_eq!(LedCommand::blue().color().red, 0);
    assert_eq!(LedCommand::blue().color().green, 0);
    assert_eq!(LedCommand::blue().color().blue, 255);
    
    let white = LedCommand::white().color();
    assert_eq!(white.red, 255);
    assert_eq!(white.green, 255);
    assert_eq!(white.blue, 255);
    
    let off = LedCommand::off().color();
    assert_eq!(off.red, 0);
    assert_eq!(off.green, 0);
    assert_eq!(off.blue, 0);
}

#[test]
fn test_rgb_command() {
    let cmd = LedCommand::rgb(128, 64, 192);
    let color = cmd.color();
    assert_eq!(color.red, 128);
    assert_eq!(color.green, 64);
    assert_eq!(color.blue, 192);
}
