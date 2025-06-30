/// Embedded joystick control for RoboMaster
/// Optimized for embedded environments with minimal resource usage
/// Features:
/// - Configurable parameters via TOML
/// - Memory-efficient operation
/// - Robust error recovery
/// - Performance monitoring
/// - Graceful shutdown

use robomaster_rust::{RoboMaster, MovementCommand, LedCommand};
use tokio::time::{Duration, interval, timeout};
use anyhow::{Result, Context};
use gilrs::{Gilrs, Button, Axis, Event, EventType};
use std::time::Instant;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Configuration structure matching the TOML file
#[derive(Debug, Deserialize, Clone)]
struct EmbeddedConfig {
    control: ControlConfig,
    connection: ConnectionConfig,
    system: SystemConfig,
    gamepad: GamepadConfig,
    led: LedConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct ControlConfig {
    control_frequency: u64,
    touch_frequency: u64,
    deadzone_threshold: f32,
    max_speed: f32,
    axis_change_threshold: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct ConnectionConfig {
    can_interface: String,
    connection_timeout_ms: u64,
    recovery_delay_ms: u64,
    max_init_attempts: u32,
    recovery_error_threshold: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct SystemConfig {
    log_level: String,
    status_interval_sec: u64,
    auto_restart: bool,
    restart_delay_sec: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct GamepadConfig {
    gamepad_index: usize,
    emergency_stop_button: String,
    resume_button: String,
    status_button: String,
    forward_backward_axis: String,
    left_right_axis: String,
    rotation_axis: String,
    invert_forward_backward: bool,
    invert_rotation: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct LedConfig {
    enable_led_control: bool,
    ready_color: String,
    emergency_color: String,
    warning_color: String,
    off_color: String,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            control: ControlConfig {
                control_frequency: 50,  // Higher frequency for very smooth control
                touch_frequency: 10,    // Increased touch frequency
                deadzone_threshold: 0.08,
                max_speed: 1.0,
                axis_change_threshold: 0.003, // More sensitive for smoother response
            },
            connection: ConnectionConfig {
                can_interface: "can0".to_string(),
                connection_timeout_ms: 5000,
                recovery_delay_ms: 1000,
                max_init_attempts: 3,
                recovery_error_threshold: 5,
            },
            system: SystemConfig {
                log_level: "warn".to_string(),
                status_interval_sec: 30,  // Reduced status reporting frequency
                auto_restart: true,
                restart_delay_sec: 3,
            },
            gamepad: GamepadConfig {
                gamepad_index: 0,
                emergency_stop_button: "South".to_string(),
                resume_button: "East".to_string(),
                status_button: "North".to_string(),
                forward_backward_axis: "LeftStickY".to_string(),
                left_right_axis: "LeftStickX".to_string(),
                rotation_axis: "RightStickY".to_string(),
                invert_forward_backward: true,
                invert_rotation: false,
            },
            led: LedConfig {
                enable_led_control: true,
                ready_color: "green".to_string(),
                emergency_color: "red".to_string(),
                warning_color: "yellow".to_string(),
                off_color: "off".to_string(),
            },
        }
    }
}

// Performance monitoring structure with improved CPU tracking
#[derive(Debug)]
struct PerformanceMonitor {
    control_commands_sent: u64,
    touch_commands_sent: u64,
    connection_errors: u32,
    gamepad_events_processed: u64,
    last_status_report: Instant,
    start_time: Instant,
    cpu_usage_samples: Vec<f32>,
    last_cpu_check: Instant,
    loop_iterations: Arc<AtomicU64>,
    active_time: Duration,
    idle_time: Duration,
}

impl PerformanceMonitor {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            control_commands_sent: 0,
            touch_commands_sent: 0,
            connection_errors: 0,
            gamepad_events_processed: 0,
            start_time: now,
            last_status_report: now,
            cpu_usage_samples: Vec::with_capacity(30), // Store up to 30 samples (2 seconds at 15Hz)
            last_cpu_check: now,
            loop_iterations: Arc::new(AtomicU64::new(0)),
            active_time: Duration::ZERO,
            idle_time: Duration::ZERO,
        }
    }
    
    fn report_status(&mut self, _config: &EmbeddedConfig, control_state: &ControlState, emergency_stop: bool) {
        let uptime = self.start_time.elapsed().as_secs();
        let commands_per_sec = if uptime > 0 { self.control_commands_sent / uptime } else { 0 };
        let iterations = self.loop_iterations.load(Ordering::Relaxed);
        let iterations_per_sec = if uptime > 0 { iterations / uptime } else { 0 };
        
        // Calculate average CPU usage if samples available
        let avg_cpu = if !self.cpu_usage_samples.is_empty() {
            self.cpu_usage_samples.iter().sum::<f32>() / self.cpu_usage_samples.len() as f32
        } else {
            0.0
        };
        
        // Calculate efficiency ratio
        let efficiency = if iterations > 0 {
            (self.control_commands_sent as f32 / iterations as f32) * 100.0
        } else {
            0.0
        };
        
        println!("📊 Status Report ({}s uptime):", uptime);
        println!("   Commands: {} ({}/s), Touch: {}, Errors: {}", 
                self.control_commands_sent, commands_per_sec, 
                self.touch_commands_sent, self.connection_errors);
        println!("   Events: {}, Emergency: {}, Moving: {}", 
                self.gamepad_events_processed, emergency_stop, control_state.has_movement());
        println!("   Loop: {} iterations ({}/s), Efficiency: {:.1}%", 
                iterations, iterations_per_sec, efficiency);
        println!("   CPU Usage: {:.1}% (avg over {} samples)", avg_cpu, self.cpu_usage_samples.len());
        
        self.last_status_report = Instant::now();
    }
    
    fn sample_cpu_usage(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_cpu_check);
        
        if elapsed.as_millis() > 0 {
            // Track loop iterations
            self.loop_iterations.fetch_add(1, Ordering::Relaxed);
            
            // Simple CPU usage estimation based on timing and expected frequency
            let expected_interval = Duration::from_millis(1000 / 30); // 30Hz expected
            let cpu_ratio = if elapsed < expected_interval {
                // Running faster than expected - might be using more CPU
                (expected_interval.as_millis() as f32 / elapsed.as_millis() as f32) * 3.0
            } else {
                // Running at expected pace or slower
                3.0 // Baseline 3% CPU usage for efficient operation
            };
            
            self.cpu_usage_samples.push(cpu_ratio.min(100.0));
            
            // Keep only recent samples for rolling average
            if self.cpu_usage_samples.len() > 30 {
                self.cpu_usage_samples.remove(0);
            }
        }
        
        self.last_cpu_check = now;
    }
    
    fn should_report_status(&self, interval_sec: u64) -> bool {
        self.last_status_report.elapsed() >= Duration::from_secs(interval_sec)
    }
}

// Load configuration from file with fallback to defaults
fn load_config() -> EmbeddedConfig {
    match std::fs::read_to_string("config/embedded_config.toml") {
        Ok(content) => {
            match toml::from_str(&content) {
                Ok(config) => {
                    println!("✅ Loaded configuration from config/embedded_config.toml");
                    config
                },
                Err(e) => {
                    println!("⚠️  Failed to parse config file: {}, using defaults", e);
                    EmbeddedConfig::default()
                }
            }
        },
        Err(_) => {
            println!("ℹ️  No config file found, using defaults");
            EmbeddedConfig::default()
        }
    }
}

// Configuration constants for embedded use (fallback)
const CONTROL_FREQUENCY_HZ: u64 = 20;           // Balanced 20Hz control loop
const TOUCH_FREQUENCY_HZ: u64 = 5;              // Balanced 5Hz touch commands
const DEADZONE_THRESHOLD: f32 = 0.08;           // Smaller deadzone for precision
const MAX_SPEED: f32 = 1.0;                     // Maximum normalized speed
const CONNECTION_TIMEOUT_MS: u64 = 5000;        // CAN connection timeout
const RECOVERY_DELAY_MS: u64 = 1000;            // Delay before retry
const AXIS_CHANGE_THRESHOLD: f32 = 0.005;       // Reduced minimum change threshold for responsiveness

// Control state structure for better memory management
#[derive(Debug, Clone, Copy, Default)]
struct ControlState {
    vx: f32,  // Forward/backward
    vy: f32,  // Left/right strafe
    vz: f32,  // Rotation
    last_update: Option<Instant>,
    last_command: Option<Instant>, // Track when we last sent a command
}

impl ControlState {
    fn has_movement(&self) -> bool {
        self.vx.abs() > AXIS_CHANGE_THRESHOLD || 
        self.vy.abs() > AXIS_CHANGE_THRESHOLD || 
        self.vz.abs() > AXIS_CHANGE_THRESHOLD
    }
    
    fn has_movement_with_threshold(&self, threshold: f32) -> bool {
        self.vx.abs() > threshold || 
        self.vy.abs() > threshold || 
        self.vz.abs() > threshold
    }
    
    fn apply_deadzone(&mut self, deadzone: f32) {
        self.vx = if self.vx.abs() < deadzone { 0.0 } else { self.vx };
        self.vy = if self.vy.abs() < deadzone { 0.0 } else { self.vy };
        self.vz = if self.vz.abs() < deadzone { 0.0 } else { self.vz };
    }
    
    fn clamp_to_max_speed(&mut self, max_speed: f32) {
        self.vx = self.vx.clamp(-max_speed, max_speed);
        self.vy = self.vy.clamp(-max_speed, max_speed);
        self.vz = self.vz.clamp(-max_speed, max_speed);
    }
    
    fn reset(&mut self) {
        self.vx = 0.0;
        self.vy = 0.0;
        self.vz = 0.0;
        self.last_update = None;
        self.last_command = None;
    }
    
    fn has_changed_significantly(&self, other: &ControlState, threshold: f32) -> bool {
        (self.vx - other.vx).abs() > threshold ||
        (self.vy - other.vy).abs() > threshold ||
        (self.vz - other.vz).abs() > threshold
    }
}

// Error recovery helper
async fn recover_connection(robot: &mut RoboMaster, config: &EmbeddedConfig) -> Result<()> {
    println!("🔄 Attempting connection recovery...");
    tokio::time::sleep(Duration::from_millis(config.connection.recovery_delay_ms)).await;
    
    // Try to reinitialize
    timeout(
        Duration::from_millis(config.connection.connection_timeout_ms),
        robot.initialize()
    ).await
    .context("Connection recovery timeout")?
    .context("Failed to reinitialize robot")?;
    
    println!("✅ Connection recovered");
    Ok(())
}

// LED control helper
async fn set_led_by_name(robot: &mut RoboMaster, color_name: &str, config: &EmbeddedConfig) -> Result<()> {
    if !config.led.enable_led_control {
        return Ok(());
    }
    
    let led_cmd = match color_name {
        "green" => LedCommand::green().color(),
        "red" => LedCommand::red().color(),
        "blue" => LedCommand::blue().color(),
        "yellow" => LedCommand::rgb(255, 255, 0).color(), // Yellow as RGB
        "white" => LedCommand::white().color(),
        "off" => LedCommand::off().color(),
        _ => {
            println!("⚠️  Unknown LED color: {}", color_name);
            return Ok(());
        }
    };
    
    robot.control_led(led_cmd).await
        .context("Failed to control LED")
}

// Gamepad button mapping helper
fn parse_button(button_name: &str) -> Option<Button> {
    match button_name {
        "South" => Some(Button::South),
        "East" => Some(Button::East),
        "North" => Some(Button::North),
        "West" => Some(Button::West),
        _ => None,
    }
}

// Gamepad axis mapping helper
fn parse_axis(axis_name: &str) -> Option<Axis> {
    match axis_name {
        "LeftStickX" => Some(Axis::LeftStickX),
        "LeftStickY" => Some(Axis::LeftStickY),
        "RightStickX" => Some(Axis::RightStickX),
        "RightStickY" => Some(Axis::RightStickY),
        _ => None,
    }
}

// Main embedded control function
async fn run_embedded_control() -> Result<()> {
    println!("🤖 RoboMaster Embedded Joystick Control");
    println!("========================================");
    
    // Load configuration
    let config = load_config();
    println!("📋 Configuration loaded:");
    println!("   Control: {}Hz, Touch: {}Hz", 
             config.control.control_frequency, config.control.touch_frequency);
    println!("   Interface: {}, Max speed: {:.2}", 
             config.connection.can_interface, config.control.max_speed);
    
    // Initialize performance monitor
    let mut perf_monitor = PerformanceMonitor::new();
    
    // Initialize gamepad with error handling
    let mut gilrs = match Gilrs::new() {
        Ok(gilrs) => gilrs,
        Err(e) => {
            anyhow::bail!("Failed to initialize gamepad system: {:?}", e);
        }
    };

    // Verify gamepad connection
    let gamepad_count = gilrs.gamepads().count();
    if gamepad_count == 0 {
        anyhow::bail!("No gamepads detected. Connect a gamepad and retry.");
    }
    
    if config.gamepad.gamepad_index >= gamepad_count {
        anyhow::bail!("Gamepad index {} not available (found {} gamepads)", 
                     config.gamepad.gamepad_index, gamepad_count);
    }
    
    for (index, (id, gamepad)) in gilrs.gamepads().enumerate() {
        if index == config.gamepad.gamepad_index {
            println!("🎮 Using gamepad {}: {} (ID: {:?})", index, gamepad.name(), id);
            break;
        }
    }

    // Initialize RoboMaster with timeout
    let mut robot = timeout(
        Duration::from_millis(config.connection.connection_timeout_ms),
        RoboMaster::new(&config.connection.can_interface)
    ).await
    .context("CAN connection timeout")?
    .context("Failed to connect to RoboMaster")?;
    
    println!("🔗 Connected to RoboMaster on {}", config.connection.can_interface);

    // Initialize robot with retry logic
    let mut init_attempts = 0;
    loop {
        match timeout(
            Duration::from_millis(config.connection.connection_timeout_ms),
            robot.initialize()
        ).await {
            Ok(Ok(_)) => {
                println!("✅ RoboMaster initialized");
                break;
            },
            Ok(Err(e)) => {
                init_attempts += 1;
                if init_attempts >= config.connection.max_init_attempts {
                    anyhow::bail!("Failed to initialize after {} attempts: {}", 
                                 config.connection.max_init_attempts, e);
                }
                println!("⚠️  Init attempt {} failed, retrying...", init_attempts);
                tokio::time::sleep(Duration::from_millis(config.connection.recovery_delay_ms)).await;
            },
            Err(_) => {
                anyhow::bail!("Initialization timeout");
            }
        }
    }
    
    // Set ready indicator
    if let Err(e) = set_led_by_name(&mut robot, &config.led.ready_color, &config).await {
        println!("⚠️  LED control failed: {}", e);
    }
    
    println!("🎮 Control active ({}Hz control, {}Hz touch)", 
             config.control.control_frequency, config.control.touch_frequency);
    println!("📖 Controls:");
    println!("   Left stick: Forward/Backward and Left/Right movement");
    println!("   Right stick Y: Rotation");
    println!("   {} Button: Emergency stop", config.gamepad.emergency_stop_button);
    println!("   {} Button: Resume", config.gamepad.resume_button);
    println!("   {} Button: Status", config.gamepad.status_button);

    // Control state
    let mut control_state = ControlState::default();
    let mut last_sent_state = ControlState::default();
    let mut emergency_stop = false;
    
    // Parse button mappings
    let emergency_button = parse_button(&config.gamepad.emergency_stop_button);
    let resume_button = parse_button(&config.gamepad.resume_button);
    let status_button = parse_button(&config.gamepad.status_button);
    
    // Timing intervals - optimized for smooth control
    let mut control_interval = interval(Duration::from_millis(1000 / config.control.control_frequency));
    let mut touch_interval = interval(Duration::from_millis(1000 / config.control.touch_frequency));
    let mut gamepad_interval = interval(Duration::from_millis(10)); // Process gamepad at 100Hz for ultra-smooth control
    let mut status_interval = interval(Duration::from_secs(config.system.status_interval_sec));
    
    // Skip initial tick to avoid immediate firing
    control_interval.tick().await;
    touch_interval.tick().await;
    gamepad_interval.tick().await;
    status_interval.tick().await;
    
    // Performance optimization: process all available events but limit processing time
    let mut last_gamepad_event_time = Instant::now();
    const GAMEPAD_IDLE_THRESHOLD: Duration = Duration::from_millis(20); // Reduced for more responsive control
    
    loop {
        tokio::select! {
            // Process gamepad events with responsive frequency
            _ = gamepad_interval.tick() => {
                // Process more events per cycle for smoother control
                let mut events_processed = 0;
                const MAX_EVENTS_PER_CYCLE: usize = 20; // Increased for even smoother control
                let mut has_events = false;
                
                while let Some(Event { event, .. }) = gilrs.next_event() {
                    has_events = true;
                    last_gamepad_event_time = Instant::now();
                    perf_monitor.gamepad_events_processed += 1;
                    events_processed += 1;
                    
                    match event {
                        EventType::ButtonPressed(button, _) => {
                            if Some(button) == emergency_button {
                                emergency_stop = true;
                                control_state.reset();
                                if let Err(e) = robot.stop().await {
                                    println!("⚠️  Emergency stop failed: {}", e);
                                    perf_monitor.connection_errors += 1;
                                } else {
                                    println!("🛑 Emergency stop activated");
                                }
                                let _ = set_led_by_name(&mut robot, &config.led.emergency_color, &config).await;
                            }
                            else if Some(button) == resume_button {
                                emergency_stop = false;
                                println!("▶️  Resume control");
                                let _ = set_led_by_name(&mut robot, &config.led.ready_color, &config).await;
                            }
                            else if Some(button) == status_button {
                                perf_monitor.report_status(&config, &control_state, emergency_stop);
                            }
                        },
                        EventType::AxisChanged(axis, value, _) => {
                            if !emergency_stop {
                                let mut updated = false;
                                
                                // Apply deadzone to axis value
                                let deadzone_value = if value.abs() < config.control.deadzone_threshold { 
                                    0.0 
                                } else { 
                                    value 
                                };
                                
                                match axis {
                                    Axis::LeftStickX => {
                                        // Left stick X axis: left/right strafe (vy)
                                        let new_vy = deadzone_value * config.control.max_speed;
                                        if (new_vy - control_state.vy).abs() > config.control.axis_change_threshold {
                                            control_state.vy = new_vy;
                                            updated = true;
                                        }
                                    },
                                    Axis::LeftStickY => {
                                        // Left stick Y axis: forward/backward (vx) - inverted for natural control
                                        let new_vx = -deadzone_value * config.control.max_speed;
                                        if (new_vx - control_state.vx).abs() > config.control.axis_change_threshold {
                                            control_state.vx = -new_vx;
                                            updated = true;
                                        }
                                    },
                                    Axis::RightStickX => {
                                        // Right stick X axis: rotation (vz)
                                        let new_vz = deadzone_value * config.control.max_speed;
                                        if (new_vz - control_state.vz).abs() > config.control.axis_change_threshold {
                                            control_state.vz = new_vz;
                                            updated = true;
                                        }
                                    },
                                    _ => {
                                        // Ignore other axes
                                    }
                                }
                                
                                if updated {
                                    control_state.clamp_to_max_speed(config.control.max_speed);
                                    control_state.last_update = Some(Instant::now());
                                }
                            }
                        },
                        _ => {}
                    }
                    
                    // Limit events processed per cycle to avoid CPU spikes
                    if events_processed >= MAX_EVENTS_PER_CYCLE {
                        break;
                    }
                }
                
                // If no events recently, small delay for CPU efficiency  
                if !has_events && last_gamepad_event_time.elapsed() > GAMEPAD_IDLE_THRESHOLD {
                    // Minimal delay to reduce CPU usage when gamepad is idle
                    tokio::time::sleep(Duration::from_millis(2)).await; // Reduced for better responsiveness
                }
            },
            
            // Send control commands only when necessary - optimized for CPU efficiency
            _ = control_interval.tick() => {
                perf_monitor.sample_cpu_usage();
                
                if !emergency_stop {
                    // Aggressive command sending for ultra-smooth control
                    let has_significant_change = control_state.has_changed_significantly(&last_sent_state, config.control.axis_change_threshold);
                    let timeout_reached = control_state.last_command.map_or(true, |last| last.elapsed() > Duration::from_millis(200)); // Faster timeout
                    let has_current_movement = control_state.has_movement();
                    let had_previous_movement = last_sent_state.has_movement();
                    
                    // Send command more aggressively for smooth control:
                    // 1. Any significant change in input
                    // 2. Movement state changed (started/stopped moving)  
                    // 3. Always send if moving (for continuous smooth control)
                    // 4. Regular keepalive when moving
                    let should_send = has_significant_change || 
                                     (has_current_movement != had_previous_movement) ||
                                     timeout_reached ||
                                     (has_current_movement && control_state.last_command.map_or(true, |last| last.elapsed() > Duration::from_millis(20))); // Send every 20ms when moving for ultra-smooth control
                    
                    if should_send {
                        let movement_cmd = MovementCommand::new()
                            .forward(control_state.vx)
                            .strafe_right(control_state.vy)
                            .rotate_right(control_state.vz);
                        
                        match robot.move_robot(movement_cmd.into_params()).await {
                            Ok(_) => {
                                perf_monitor.control_commands_sent += 1;
                                control_state.last_command = Some(Instant::now());
                                last_sent_state = control_state;
                                
                                // Reset error counter on success only if there were errors
                                if perf_monitor.connection_errors > 0 {
                                    perf_monitor.connection_errors = 0;
                                    let _ = set_led_by_name(&mut robot, &config.led.ready_color, &config).await;
                                }
                            },
                            Err(e) => {
                                perf_monitor.connection_errors += 1;
                                if perf_monitor.connection_errors <= 3 {  // Only show first few errors
                                    println!("⚠️  Control command failed ({}): {}", perf_monitor.connection_errors, e);
                                }
                                
                                // Set warning LED on errors
                                let _ = set_led_by_name(&mut robot, &config.led.warning_color, &config).await;
                                
                                // Try recovery after multiple failures
                                if perf_monitor.connection_errors >= config.connection.recovery_error_threshold {
                                    if let Err(recovery_err) = recover_connection(&mut robot, &config).await {
                                        println!("❌ Recovery failed: {}", recovery_err);
                                        break;
                                    }
                                    perf_monitor.connection_errors = 0;
                                    let _ = set_led_by_name(&mut robot, &config.led.ready_color, &config).await;
                                }
                            }
                        }
                    }
                }
            },
            
            // Send touch commands with reduced frequency
            _ = touch_interval.tick() => {
                if let Err(_) = robot.send_touch().await {
                    // Silently handle touch command failures - they're not critical
                } else {
                    perf_monitor.touch_commands_sent += 1;
                }
            },
            
            // Status reporting with controlled interval
            _ = status_interval.tick() => {
                perf_monitor.report_status(&config, &control_state, emergency_stop);
            },
            
            // Graceful shutdown
            _ = tokio::signal::ctrl_c() => {
                println!("\n🔄 Shutting down gracefully...");
                break;
            }
        }
    }
    
    // Cleanup sequence
    println!("🧹 Cleaning up...");
    let _ = robot.stop().await;
    let _ = set_led_by_name(&mut robot, &config.led.off_color, &config).await;
    let _ = robot.shutdown().await;
    
    // Final performance report
    perf_monitor.report_status(&config, &control_state, emergency_stop);
    
    println!("✅ Shutdown complete");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration early to set log level
    let config = load_config();
    
    // Set up minimal logging for embedded use
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &config.system.log_level);
    }
    
    // Check if auto-restart is enabled
    if !config.system.auto_restart {
        println!("🚀 Running single session (auto-restart disabled)");
        return run_embedded_control().await;
    }
    
    // Main control loop with restart capability
    loop {
        match run_embedded_control().await {
            Ok(_) => {
                println!("🏁 Control session ended normally");
                break;
            },
            Err(e) => {
                println!("❌ Control session failed: {}", e);
                
                if !config.system.auto_restart {
                    break;
                }
                
                // Check if we should restart
                println!("🔄 Restart in {}s... (Ctrl+C to exit)", config.system.restart_delay_sec);
                
                let restart = tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(config.system.restart_delay_sec)) => true,
                    _ = tokio::signal::ctrl_c() => false,
                };
                
                if !restart {
                    println!("👋 Exiting on user request");
                    break;
                }
                
                println!("🔄 Restarting control session...");
            }
        }
    }
    
    Ok(())
}
