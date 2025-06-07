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

// getADBBatteryLevel retrieves the battery level from a specific ADB-connected device by model
func getADBBatteryLevel() (float32, error) {
	targetModel := "2201117TY"
	var deviceSerial string

	// 1. List devices and find the target model's serial ID
	cmdDevices := exec.Command("adb", "devices", "-l")
	outputDevices, err := cmdDevices.CombinedOutput()
	if err != nil {
		return 0, fmt.Errorf("adb devices -l command failed: %w. Output: %s", err, string(outputDevices))
	}

	linesDevices := strings.Split(string(outputDevices), "\n")
	foundDevice := false
	for _, line := range linesDevices {
		if strings.Contains(line, "model:"+targetModel) {
			fields := strings.Fields(line)
			if len(fields) > 0 {
				deviceSerial = fields[0]
				foundDevice = true
				log.Printf("Found device %s with serial %s", targetModel, deviceSerial)
				break
			}
		}
	}

	if !foundDevice {
		if strings.Contains(string(outputDevices), "List of devices attached") && len(linesDevices) <= 2 { // Check if output indicates no devices beyond header
			return 0, fmt.Errorf("no ADB devices found. Output: %s", string(outputDevices))
		}
		return 0, fmt.Errorf("device model %s not found. ADB devices output: %s", targetModel, string(outputDevices))
	}

	// 2. Get battery level for the specific device
	cmdBattery := exec.Command("adb", "-s", deviceSerial, "shell", adbBatteryLevelCmd) // adbBatteryLevelCmd is "dumpsys battery"
	outputBattery, err := cmdBattery.CombinedOutput()
	if err != nil {
		return 0, fmt.Errorf("adb -s %s shell %s command failed: %w. Output: %s", deviceSerial, adbBatteryLevelCmd, err, string(outputBattery))
	}

	linesBattery := strings.Split(string(outputBattery), "\n")
	for _, line := range linesBattery {
		trimmedLine := strings.TrimSpace(line)
		if strings.HasPrefix(trimmedLine, "level:") {
			parts := strings.Split(trimmedLine, ":")
			if len(parts) == 2 {
				levelStr := strings.TrimSpace(parts[1])
				level, errConv := strconv.ParseFloat(levelStr, 32)
				if errConv == nil {
					return float32(level), nil
				}
				log.Printf("Failed to parse level string '%s': %v", levelStr, errConv)
			}
		}
	}

	log.Printf("Failed to parse battery level for device %s from ADB output. Full output:\n%s", deviceSerial, string(outputBattery))
	return 0, fmt.Errorf("could not parse battery level for device %s. Check ADB connection and device state", deviceSerial)
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
