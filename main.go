package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/joho/godotenv"
)

const (
	homeAssistantEntityID = "switch.smart_power_strip_socket_4"
	adbBatteryLevelCmd    = "dumpsys battery"
	loopInterval          = 1 * time.Minute
	lowBatteryThreshold   = 20.0
	highBatteryThreshold  = 80.0
)

// Config holds application configuration
type Config struct {
	HomeAssistantURL   string
	HomeAssistantToken string
}

// loadConfig loads configuration from environment variables
func loadConfig() (*Config, error) {
	err := godotenv.Load() // Load .env file
	if err != nil {
		log.Println("No .env file found, relying on existing environment variables")
	}

	url := os.Getenv("HOMEASSISTANT_URL")
	if url == "" {
		return nil, fmt.Errorf("HOMEASSISTANT_URL not set")
	}

	token := os.Getenv("HOMEASSISTANT_TOKEN")
	if token == "" {
		return nil, fmt.Errorf("HOMEASSISTANT_TOKEN not set")
	}

	return &Config{
		HomeAssistantURL:   url,
		HomeAssistantToken: token,
	}, nil
}

// controlSwitch sends a command to Home Assistant to turn the switch on or off
func controlSwitch(cfg *Config, turnOn bool) error {
	service := "turn_off"
	if turnOn {
		service = "turn_on"
	}

	url := fmt.Sprintf("%s/services/switch/%s", cfg.HomeAssistantURL, service)

	payload := map[string]string{
		"entity_id": homeAssistantEntityID,
	}
	jsonPayload, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("failed to marshal JSON payload: %w", err)
	}

	req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonPayload))
	if err != nil {
		return fmt.Errorf("failed to create HTTP request: %w", err)
	}

	req.Header.Set("Authorization", "Bearer "+cfg.HomeAssistantToken)
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send request to Home Assistant: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("Home Assistant API request failed with status %s", resp.Status)
	}

	log.Printf("Switch control response: %s", resp.Status)
	return nil
}

// getADBBatteryLevel retrieves the battery level from an ADB-connected device
func getADBBatteryLevel() (float32, error) {
	cmd := exec.Command("adb", "shell", adbBatteryLevelCmd)
	output, err := cmd.CombinedOutput() // CombinedOutput includes both stdout and stderr

	if err != nil {
		return 0, fmt.Errorf("adb command failed: %w. Output: %s\nEnsure ADB is installed, the device is connected, developer options & USB debugging are enabled, and the PC is authorized.", err, string(output))
	}

	lines := strings.Split(string(output), "\n")
	for _, line := range lines {
		trimmedLine := strings.TrimSpace(line)
		if strings.HasPrefix(trimmedLine, "level:") {
			parts := strings.Split(trimmedLine, ":")
			if len(parts) == 2 {
				levelStr := strings.TrimSpace(parts[1])
				level, err := strconv.ParseFloat(levelStr, 32)
				if err == nil {
					return float32(level), nil
				}
			}
		}
	}
	log.Printf("Failed to parse battery level from ADB output. Full output:\n%s", string(output))
	return 0, fmt.Errorf("could not parse battery level from adb output. Check ADB connection and device state")
}

func main() {
	cfg, err := loadConfig()
	if err != nil {
		log.Fatalf("Failed to load configuration: %v", err)
	}

	log.Println("Starting Tuya Monitor (Go version)...")

	for {
		log.Println("Checking battery...")

		batteryLevel, err := getADBBatteryLevel()
		if err != nil {
			log.Printf("Failed to get ADB battery level: %v. Check ADB setup and device connection/authorization.", err)
			log.Printf("ADB Error: %v", err) // Simplified error logging
			time.Sleep(loopInterval) // Wait before retrying
			continue
		}

		log.Printf("Current ADB battery level: %.1f%%", batteryLevel)

		if batteryLevel < lowBatteryThreshold {
			log.Printf("Battery level is below %.0f%% (%.1f%%), attempting to turn on charger.", lowBatteryThreshold, batteryLevel)
			if err := controlSwitch(cfg, true); err != nil {
				log.Printf("Failed to turn on switch: %v", err)
				log.Printf("Switch Error: Failed to turn ON charger: %v", err)
			} else {
				log.Printf("Tuya Charging: Battery at %.1f%%, charger ON.", batteryLevel)
			}
		} else if batteryLevel > highBatteryThreshold {
			log.Printf("Battery level is above %.0f%% (%.1f%%), attempting to turn off charger.", highBatteryThreshold, batteryLevel)
			if err := controlSwitch(cfg, false); err != nil {
				log.Printf("Failed to turn off switch: %v", err)
				log.Printf("Switch Error: Failed to turn OFF charger: %v", err)
			} else {
				log.Printf("Tuya Discharging: Battery at %.1f%%, charger OFF.", batteryLevel)
			}
		} else {
			log.Printf("Battery level is %.1f%% (between %.0f%% and %.0f%%). No action needed.", batteryLevel, lowBatteryThreshold, highBatteryThreshold)
		}

		log.Printf("Waiting for %.0f minutes...", loopInterval.Minutes())
		time.Sleep(loopInterval)
	}
}
