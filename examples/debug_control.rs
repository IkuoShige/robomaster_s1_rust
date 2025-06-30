/// Basic RoboMaster control example with debug output
/// This example demonstrates how to use the high-level RoboMaster API with detailed logging

use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{sleep, Duration};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RoboMaster Basic Control Example (Debug Version)");
    println!("=================================================");

    // Initialize RoboMaster controller
    println!("Initializing RoboMaster on can0...");
    let mut robot = RoboMaster::new("can0").await?;
    println!("Connected to RoboMaster on can0");

    // Initialize the robot
    println!("Sending initialization commands...");
    robot.initialize().await?;
    println!("RoboMaster initialized successfully");
    
    // Test simple movement with detailed logging
    println!("\nTesting basic forward movement...");
    
    // Move forward
    println!("Sending forward movement command (vx=0.3)...");
    let forward_cmd = MovementCommand::new().forward(0.3);
    let params = forward_cmd.into_params();
    println!("Movement parameters: vx={:.2}, vy={:.2}, vz={:.2}", params.vx, params.vy, params.vz);
    
    robot.move_robot(params).await?;
    println!("Forward command sent, waiting 2 seconds...");
    sleep(Duration::from_millis(2000)).await;
    
    // Stop with explicit command
    println!("Sending stop command...");
    robot.stop().await?;
    println!("Stop command sent, waiting 1 second...");
    sleep(Duration::from_millis(1000)).await;
    
    // Test LED to verify communication
    println!("\nTesting LED control to verify communication...");
    println!("Setting LED to red...");
    robot.control_led(LedCommand::red().color()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    println!("Setting LED to green...");
    robot.control_led(LedCommand::green().color()).await?;
    sleep(Duration::from_millis(1000)).await;
    
    println!("Turning off LED...");
    robot.control_led(LedCommand::off().color()).await?;
    sleep(Duration::from_millis(500)).await;

    // Test backward movement
    println!("\nTesting backward movement...");
    println!("Sending backward movement command (vx=-0.3)...");
    let backward_cmd = MovementCommand::new().forward(-0.3);
    let params = backward_cmd.into_params();
    println!("Movement parameters: vx={:.2}, vy={:.2}, vz={:.2}", params.vx, params.vy, params.vz);
    
    robot.move_robot(params).await?;
    println!("Backward command sent, waiting 2 seconds...");
    sleep(Duration::from_millis(2000)).await;
    
    // Final stop
    println!("Sending final stop command...");
    robot.stop().await?;
    
    // Shutdown
    println!("\nShutting down...");
    robot.shutdown().await?;
    println!("Example completed successfully!");
    
    Ok(())
}
