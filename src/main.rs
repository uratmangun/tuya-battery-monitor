use std::env;
use std::time::Duration;
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
use notify_rust::Notification;
use std::thread;

// Import the system_info module that contains getSystemInfo implementation
mod system_info;
use system_info::SystemInfo;

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

// Function to show notifications
fn show_notification(title: &str, message: &str, urgency: &str) {
    let urgency_level = match urgency {
        "critical" => notify_rust::Urgency::Critical,
        _ => notify_rust::Urgency::Normal,
    };

    if let Err(e) = Notification::new()
        .summary(title)
        .body(message)
        .urgency(urgency_level)
        .show() {
        println!("Failed to show notification: {}", e);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get initial system information
    let sys_info = SystemInfo::get();
    println!("Raw system info: {:?}", sys_info);

    if let Some(sys_info) = sys_info {
        println!("System Information:");
        println!("CPU Temperature: {}°C", 
            sys_info.cpu_temp.map_or("N/A".to_string(), |t| format!("{:.1}", t)));
        println!("Battery: {}% ({})", 
            sys_info.battery_percentage.map_or("N/A".to_string(), |b| format!("{:.1}", b)),
            sys_info.battery_status.unwrap_or("Unknown".to_string()));
    }

    // Start the monitoring loop
    loop {
         println!("checking battery");
        
        if let Some(sys_info) = SystemInfo::get() {
            if let Some(battery) = sys_info.battery_percentage {
                println!("Current battery level: {}%", battery);
                if battery < 20.0 {
                    control_switch(true).await?;
                    show_notification("Tuya charging", "charge", "normal");
                } else if battery > 79.0 {
                    control_switch(false).await?;
                    show_notification("Tuya discharging", "discharging", "normal");
                }
            }
        }

        // Sleep for 5 minutes
        thread::sleep(Duration::from_secs(300));
    }
} 