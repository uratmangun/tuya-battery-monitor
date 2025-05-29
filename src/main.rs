use std::env;
use std::time::Duration;
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
use std::thread;

// Import the system_info module that contains getSystemInfo implementation
mod system_info;
use system_info::SystemInfo;
use std::process::Command;
use tokio::time::sleep;


// Function to control the switch (equivalent to turnOffSwitch4)
async fn control_switch(turn_on: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    let homeassistant_url = env::var("HOMEASSISTANT_URL")?;
    let homeassistant_token = env::var("HOMEASSISTANT_TOKEN")?;
    
    // Determine the endpoint based on the action
    let endpoint = if turn_on { "turn_on" } else { "turn_off" };
    let url = format!("{}/services/switch/{}", homeassistant_url, endpoint);

    // Create HTTP client
    let client = Client::new();
    
    // Make the request to Home Assistant API
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", homeassistant_token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "entity_id": "switch.smart_power_strip_socket_4"
        }))
        .send()
        .await?;

    println!("Switch control response: {:?}", response.status());
    Ok(())
}


async fn get_adb_battery_level() -> Result<f32, Box<dyn std::error::Error>> {
    let output = Command::new("adb")
        .args(&["shell", "dumpsys", "battery"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("ADB command error: {}\nEnsure ADB is installed, the device is connected, developer options & USB debugging are enabled, and the PC is authorized.", stderr);
        return Err(format!("adb command failed. Is the device connected and authorized? Details: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with("level:") {
            // Example line: "  level: 85"
            let parts: Vec<&str> = trimmed_line.split(':').collect();
            if parts.len() == 2 {
                if let Ok(level) = parts[1].trim().parse::<f32>() {
                    return Ok(level);
                }
            }
        }
    }
    eprintln!("Failed to parse battery level from ADB output. Full output:\n{}", stdout);
    Err("Could not parse battery level from adb output. Check ADB connection and device state.".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get initial system information
    let sys_info = SystemInfo::get();
    println!("Raw system info: {:?}", sys_info);

    if let Some(sys_info) = sys_info {
        if sys_info.validate() {
            println!("System Information:");
            println!("CPU Temperature: {}°C", 
                sys_info.cpu_temp.map_or("N/A".to_string(), |t| format!("{:.1}", t)));
            println!("Battery: {}% ({})", 
                sys_info.battery_percentage.map_or("N/A".to_string(), |b| format!("{:.1}", b)),
                sys_info.battery_status.unwrap_or("Unknown".to_string()));
        } else {
            println!("Warning: Initial system information validation failed");
        }
    }

    // Start the monitoring loop
    loop {
        println!("checking battery");
        
        if let Some(sys_info) = SystemInfo::get() {
            if !sys_info.validate() {
                println!("Warning: Invalid system information received, skipping this iteration");
                thread::sleep(Duration::from_secs(60)); // Wait for 1 minute before retrying
                continue;
            }

            match get_adb_battery_level().await {
                Ok(battery_level) => {
                    println!("Current ADB battery level: {}%", battery_level);
                    if battery_level < 20.0 {
                        println!("Battery level is below 20% ({}), attempting to turn on charger.", battery_level);
                        if let Err(e) = control_switch(true).await {
                            eprintln!("Failed to turn on switch: {}", e);
                            eprintln!("Switch Error: Failed to turn ON charger: {}", e);
                        } else {
                            println!("Tuya Charging: Battery at {}%, charger ON.", battery_level);
                        }
                    } else if battery_level > 80.0 {
                        println!("Battery level is above 80% ({}), attempting to turn off charger.", battery_level);
                        if let Err(e) = control_switch(false).await {
                            eprintln!("Failed to turn off switch: {}", e);
                            eprintln!("Switch Error: Failed to turn OFF charger: {}", e);
                        } else {
                            println!("Tuya Discharging: Battery at {}%, charger OFF.", battery_level);
                        }
                    } else {
                        println!("Battery level is {}% (between 20% and 80%). No action needed.", battery_level);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get ADB battery level: {}. Check ADB setup and device connection/authorization.", e);
                    eprintln!("ADB Error: Failed to get battery level. Error: {}", e);
                }
            }
        } else {
            println!("Warning: Failed to get system information");
            thread::sleep(Duration::from_secs(60)); // Wait for 1 minute before retrying
        }

        // Sleep for 5 minutes
        sleep(Duration::from_secs(300)).await;
    }
} 