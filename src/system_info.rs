use serde::Serialize;
use sysinfo::{System, SystemExt, ComponentExt};

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub cpu_temp: Option<f64>,
    pub battery_percentage: Option<f64>,
    pub battery_status: Option<String>,
}

impl SystemInfo {
    pub fn get() -> Option<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Calculate CPU temperature using the improved method
        let cpu_temp = Some(if sys.components().is_empty() {
            // Try reading from /sys/class/thermal/thermal_zone0/temp
            std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
                .map(|s| s.trim().parse::<f64>().map(|t| t / 1000.0).unwrap_or(0.0))
                .unwrap_or(0.0)
        } else {
            sys.components()
                .iter()
                .map(|comp| comp.temperature() as f64)
                .sum::<f64>() / sys.components().len() as f64
        });

        // Get battery information
        let battery_percentage = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .map(|s| s.trim().parse::<f64>().ok())
            .ok()
            .flatten();

        let battery_status = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
            .map(|s| s.trim().to_string())
            .ok();

        Some(SystemInfo {
            cpu_temp,
            battery_percentage,
            battery_status,
        })
    }
} 