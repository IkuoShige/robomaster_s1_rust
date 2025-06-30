/// Sensor monitoring example for RoboMaster
/// This example demonstrates how to read and monitor robot sensor data

use robomaster_rust::RoboMaster;
use tokio::time::{Duration, interval};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("RoboMaster Sensor Monitoring Example");
    println!("====================================");

    // Initialize RoboMaster controller
    let mut robot = RoboMaster::new("can0").await?;
    println!("Connected to RoboMaster on can0");

    // Initialize the robot
    robot.initialize().await?;
    
    let mut monitor_interval = interval(Duration::from_millis(500)); // 2 Hz monitoring
    let mut counter = 0;
    
    println!("Starting sensor monitoring loop...");
    println!("(Press Ctrl+C to exit)");
    println!();
    
    loop {
        monitor_interval.tick().await;
        
        // Receive and process messages
        robot.receive_messages().await?;
        
        // Get current counters (example of internal state monitoring)
        let counters = robot.get_counters();
        
        // Display monitoring information
        counter += 1;
        println!("=== Monitor Update #{} ===", counter);
        println!("Interface: {}", robot.interface_name());
        println!("Command Counters:");
        println!("  Joy: {}", counters.joy);
        println!("  LED: {}", counters.led);
        println!("  Gimbal: {}", counters.gimbal);
        println!();
        
        // Note: Actual sensor data reading would be implemented here
        // For now, we're monitoring the internal command state
        
        // Send a periodic touch command to keep the connection alive
        if counter % 10 == 0 {
            robot.send_touch().await?;
            println!("Sent keep-alive touch command");
        }
        
        // Exit after 60 updates (30 seconds)
        if counter >= 60 {
            break;
        }
    }
    
    // Cleanup
    robot.shutdown().await?;
    println!("Sensor monitoring example completed!");
    
    Ok(())
}
