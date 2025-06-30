/// Joystick control example for RoboMaster
/// This example demonstrates basic joystick-like control simulation

use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{Duration, interval};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RoboMaster Simulated Joystick Control Example");
    println!("==============================================");
    println!("This example simulates joystick control for testing purposes.");

    // Initialize RoboMaster controller
    let mut robot = RoboMaster::new("can0").await?;
    println!("Connected to RoboMaster on can0");

    // Initialize the robot
    robot.initialize().await?;
    
    // Set initial LED color (green = ready)
    robot.control_led(LedCommand::green().color()).await?;
    
    let mut update_interval = interval(Duration::from_millis(100)); // 10 Hz control loop
    let mut cycle_count = 0;
    
    println!("Starting simulated control loop...");
    
    // Simulate a sequence of movements
    for i in 0..50 {  // Run for 5 seconds
        update_interval.tick().await;
        
        // Create different movement patterns based on cycle
        let movement_cmd = match i / 10 {
            0 => MovementCommand::new().forward(0.3),      // Forward
            1 => MovementCommand::new().strafe_right(0.3), // Strafe right
            2 => MovementCommand::new().rotate_right(0.5), // Rotate
            3 => MovementCommand::new()                     // Combined movement
                .forward(0.2)
                .strafe_right(0.1)
                .rotate_right(0.2),
            _ => MovementCommand::new(),                    // Stop
        };
        
        robot.move_robot(movement_cmd.into_params()).await?;
        
        // Change LED color based on movement pattern  
        if i % 10 == 0 {
            let led_color = match i / 10 {
                0 => LedCommand::blue().color(),   // Forward = Blue
                1 => LedCommand::red().color(),    // Strafe = Red  
                2 => LedCommand::white().color(),  // Rotate = White
                3 => LedCommand::green().color(),  // Combined = Green
                _ => LedCommand::off().color(),    // Stop = Off
            };
            robot.control_led(led_color).await?;
        }
        
        // Send periodic touch command
        if i % 20 == 0 {
            robot.send_touch().await?;
        }
        
        cycle_count += 1;
    }
    
    // Cleanup
    robot.stop().await?;
    robot.control_led(LedCommand::off().color()).await?;
    robot.shutdown().await?;
    
    println!("Simulated joystick control example completed!");
    println!("Executed {} control cycles", cycle_count);
    
    Ok(())
}
