# Tuya Battery Monitor

A Rust application that monitors your system's battery level and automatically controls a smart plug through Home Assistant to manage charging. When the battery level drops below 20%, it turns on the smart plug to start charging, and when it reaches 79%, it turns off the plug to prevent overcharging.

This project uses [Bardi Smart Power Strip](https://bardi.co.id/product/extension-power-strip/) as the smart plug, but you can use any smart plug that's compatible with Home Assistant and tuya.

## Features

- Real-time battery level monitoring
- Automatic smart plug control based on battery levels
- Desktop notifications for battery status
- Systemd service integration for automatic startup
- CPU temperature monitoring
- Built with pure Rust for optimal performance

## Prerequisites

- Rust toolchain (rustc, cargo)
- Linux system with systemd
- Docker and Docker Compose (for Home Assistant)
- Home Assistant setup with a smart plug

## Setting up Home Assistant

1. Create a `docker-compose.yml` file:
```yaml
version: '3'
services:
  homeassistant:
    container_name: homeassistant
    image: "ghcr.io/home-assistant/home-assistant:stable"
    volumes:
      - ./config:/config
      - /etc/localtime:/etc/localtime:ro
    restart: unless-stopped
    privileged: true
    network_mode: host
```

2. Create the configuration directory:
```bash
mkdir config
```

3. Start Home Assistant:
```bash
docker-compose up -d
```

4. Access Home Assistant web interface at `http://localhost:8123`

5. Set up your smart plug following Home Assistant's documentation

6. Generate a Long-Lived Access Token:
   - Go to your profile in Home Assistant
   - Scroll to the bottom
   - Under "Long-Lived Access Tokens" click "Create Token"
   - Save this token for use in the `.env` file

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd tuya
```

2. Create a `.env` file in the project root with your Home Assistant credentials:
```env
HOMEASSISTANT_URL=your_homeassistant_url
HOMEASSISTANT_TOKEN=your_long_lived_access_token
```

## Building

Build the release version:
```bash
cargo build --release
```

This will create an executable at `target/release/tuya`. Copy it to your desired location:
```bash
cp target/release/tuya tuya-monitor
```

## Running

### Manual Run

You can run the application directly:
```bash
./tuya-monitor
```

### Running as a System Service

#### Setting up the Systemd Service

1. Create a `tuya-monitor.service` file with the following content:
```ini
[Unit]
Description=Tuya Battery Monitor Service
After=network.target

[Service]
Type=simple
User=your_username
AmbientCapabilities=CAP_SYS_RAWIO
WorkingDirectory=/path/to/tuya/directory
ExecStart=/path/to/tuya/directory/tuya-monitor
Restart=always
RestartSec=10
Environment=DISPLAY=:0
EnvironmentFile=/path/to/tuya/directory/.env

[Install]
WantedBy=multi-user.target
```

2. Copy the service file to systemd directory:
```bash
sudo cp tuya-monitor.service /etc/systemd/system/
```

3. Reload systemd daemon and enable the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable tuya-monitor
sudo systemctl start tuya-monitor
```

4. Check service status:
```bash
systemctl status tuya-monitor
```

#### Service Management

- Start the service:
```bash
sudo systemctl start tuya-monitor
```

- Stop the service:
```bash
sudo systemctl stop tuya-monitor
```

- Restart the service:
```bash
sudo systemctl restart tuya-monitor
```

- View logs:
```bash
journalctl -u tuya-monitor -f
```

## Configuration

The application monitors battery levels with the following thresholds:
- Below 20%: Turns ON the smart plug to start charging
- Above 79%: Turns OFF the smart plug to prevent overcharging
- Checks every 5 minutes

To modify these thresholds, edit the values in `src/main.rs`.

## System Information CLI Tool

The project includes a CLI tool for checking system information directly:

### Building the CLI Tool

```bash
cd sysinfo-cli
cargo build --release
```

### Usage

The tool provides three main commands:

1. Show all system information:
```bash
./target/release/sysinfo
# or
./target/release/sysinfo all
```

2. Show only CPU temperature:
```bash
./target/release/sysinfo temp
```

3. Show only battery information:
```bash
./target/release/sysinfo battery
```

### Example Output

```
System Information:
  CPU Temperature: 51.9°C
  Battery Level: 72%
  Battery Status: Charging
```

## Troubleshooting

1. If the service fails to start, check the logs:
```bash
journalctl -u tuya-monitor -f
```

2. Verify environment variables:
```bash
systemctl show tuya-monitor
```

3. Check file permissions:
```bash
ls -l tuya-monitor
```

4. If CPU temperature shows as N/A, ensure your user has the right permissions:
```bash
sudo usermod -aG sys your_username
```

## License

MIT License. See [LICENSE](LICENSE) file for details.
