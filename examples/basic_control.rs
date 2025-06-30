/// Basic RoboMaster control example
/// This example demonstrates how to use the high-level RoboMaster API

use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{sleep, Duration};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RoboMaster Basic Control Example");
    println!("================================");

    // Initialize RoboMaster controller
    let mut robot = RoboMaster::new("can0").await?;
    println!("Connected to RoboMaster on can0");

    // Initialize the robot
    robot.initialize().await?;
    
    // Test LED colors
    println!("Testing LED colors...");
    robot.control_led(LedCommand::red().color()).await?;
    sleep(Duration::from_millis(500)).await;
    
    robot.control_led(LedCommand::green().color()).await?;
    sleep(Duration::from_millis(500)).await;
    
    robot.control_led(LedCommand::blue().color()).await?;
    sleep(Duration::from_millis(500)).await;
    
    robot.control_led(LedCommand::white().color()).await?;
    sleep(Duration::from_millis(500)).await;

    // Test movement commands
    println!("Testing movement...");
    
    // Move forward
    let forward_cmd = MovementCommand::new().forward(0.3);
    robot.move_robot(forward_cmd.into_params()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    // Stop
    robot.stop().await?;
    sleep(Duration::from_millis(500)).await;
    
    // Move backward
    let backward_cmd = MovementCommand::new().forward(-0.3);
    robot.move_robot(backward_cmd.into_params()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    // Stop
    robot.stop().await?;
    sleep(Duration::from_millis(500)).await;
    
    // Strafe right
    let strafe_cmd = MovementCommand::new().strafe_right(0.3);
    robot.move_robot(strafe_cmd.into_params()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    // Stop
    robot.stop().await?;
    sleep(Duration::from_millis(500)).await;
    
    // Rotate
    let rotate_cmd = MovementCommand::new().rotate_right(0.5);
    robot.move_robot(rotate_cmd.into_params()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    // Stop
    robot.stop().await?;
    sleep(Duration::from_millis(500)).await;
    
    // Combined movement
    let combined_cmd = MovementCommand::new()
        .forward(0.2)
        .strafe_right(0.1)
        .rotate_right(0.2);
    robot.move_robot(combined_cmd.into_params()).await?;
    sleep(Duration::from_millis(1500)).await;
    
    // Final stop
    robot.stop().await?;
    
    // Turn off LED
    robot.control_led(LedCommand::off().color()).await?;
    
    // Shutdown
    robot.shutdown().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
