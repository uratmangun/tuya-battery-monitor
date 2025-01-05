use clap::{Parser, Subcommand};
use colored::*;
use sysinfo::{System, SystemExt, ComponentExt};
use std::fs;

#[derive(Parser)]
#[command(name = "sysinfo")]
#[command(about = "A CLI tool to display system information", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show all system information
    All,
    /// Show only temperature
    Temp,
    /// Show only battery information
    Battery,
}

fn get_cpu_temp() -> f64 {
    let mut sys = System::new_all();
    sys.refresh_all();

    if sys.components().is_empty() {
        // Try reading from /sys/class/thermal/thermal_zone0/temp
        std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .map(|s| s.trim().parse::<f64>().map(|t| t / 1000.0).unwrap_or(0.0))
            .unwrap_or(0.0)
    } else {
        sys.components()
            .iter()
            .map(|comp| comp.temperature())
            .sum::<f32>() as f64 / sys.components().len() as f64
    }
}

fn get_battery_info() -> (f64, String) {
    let percentage = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);

    let status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    (percentage, status)
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::All) {
        Commands::All => {
            let temp = get_cpu_temp();
            let (battery_percentage, battery_status) = get_battery_info();
            
            println!("{}:", "System Information".green().bold());
            println!("  {}: {:.1}°C", "CPU Temperature".cyan(), temp);
            println!("  {}: {:.0}%", "Battery Level".cyan(), battery_percentage);
            println!("  {}: {}", "Battery Status".cyan(), battery_status);
        }
        Commands::Temp => {
            let temp = get_cpu_temp();
            println!("{:.1}°C", temp);
        }
        Commands::Battery => {
            let (percentage, status) = get_battery_info();
            println!("{}% ({})", percentage as i32, status);
        }
    }
}
