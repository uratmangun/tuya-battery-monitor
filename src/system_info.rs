use serde::Serialize;
use sysinfo::{System, SystemExt, ComponentExt};
use std::sync::Once;

static INIT: Once = Once::new();
static mut SYSTEM: Option<System> = None;

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub cpu_temp: Option<f64>,
    pub battery_percentage: Option<f64>,
    pub battery_status: Option<String>,
}

impl SystemInfo {
    fn get_system() -> &'static mut System {
        unsafe {
            INIT.call_once(|| {
                SYSTEM = Some(System::new_all());
            });
            SYSTEM.as_mut().unwrap()
        }
    }

    pub fn get() -> Option<Self> {
        let sys = Self::get_system();
        // Force a complete refresh of all system information
        sys.refresh_all();
        sys.refresh_components();
        sys.refresh_components_list();

        // Calculate CPU temperature using the improved method
        let cpu_temp = Some(if sys.components().is_empty() {
            // Try reading from /sys/class/thermal/thermal_zone0/temp directly
            // This ensures we get fresh data every time
            std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
                .map(|s| s.trim().parse::<f64>().map(|t| t / 1000.0).unwrap_or(0.0))
                .unwrap_or(0.0)
        } else {
            sys.components()
                .iter()
                .map(|comp| comp.temperature() as f64)
                .sum::<f64>() / sys.components().len() as f64
        });

        // Get battery information directly from system files
        // This ensures we get fresh data every time
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

    // Add a method to validate the readings
    pub fn validate(&self) -> bool {
        match (self.battery_percentage, self.battery_status.as_deref()) {
            (Some(percentage), Some(status)) => {
                // Check if percentage is within valid range
                if !(0.0..=100.0).contains(&percentage) {
                    return false;
                }
                // Check if status is a valid value
                matches!(status, "Charging" | "Discharging" | "Full" | "Not charging")
            }
            _ => false
        }
    }
} 