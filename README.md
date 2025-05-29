# Tuya Battery Monitor

A go application that monitors an Android device's battery level (via ADB) and automatically controls a smart plug through Home Assistant to manage charging. When the battery level drops below a configured threshold (e.g., 20%), it turns on the smart plug to start charging, and when it reaches another threshold (e.g., 80%), it turns off the plug to prevent overcharging.

This project uses a [Bardi Smart Power Strip](https://bardi.co.id/product/extension-power-strip/) as the smart plug, but you can use any smart plug that's compatible with Home Assistant and Tuya.

## Features

- Real-time battery level monitoring via ADB (Android Debug Bridge).
- Automatic smart plug control based on configurable battery levels.
- Manages charging via a systemd service.
- Built with Go.

## Prerequisites

- **Go**: To build the application (version 1.18 or newer recommended).
- **ADB (Android Debug Bridge)**: Installed on the host machine.
- **Android Device**: With Developer Options and Wireless Debugging (or USB Debugging) enabled.
- **Home Assistant**: Setup and running, with your smart plug integrated.

## Setting up Home Assistant

If you haven't already, you can set up Home Assistant using Docker:

1.  Create a `docker-compose.yml` file (e.g., in a directory like `~/homeassistant-docker`):
    ```yaml
    version: '3'
    services:
      homeassistant:
        container_name: homeassistant
        image: "ghcr.io/home-assistant/home-assistant:stable"
        volumes:
          - ./config:/config # Persists Home Assistant configuration
          - /etc/localtime:/etc/localtime:ro
        restart: unless-stopped
        privileged: true # Often needed for Z-Wave/Zigbee sticks, etc.
        network_mode: host # Simplifies network access to localhost
    ```

2.  Create the configuration directory and navigate into it:
    ```bash
    mkdir -p ~/homeassistant-docker/config
    cd ~/homeassistant-docker
    ```

3.  Start Home Assistant:
    ```bash
    docker-compose up -d
    ```

4.  Access Home Assistant web interface at `http://localhost:8123` and complete the initial setup. Make sure to integrate your smart plug.

## Environment Configuration

Create a `.env` file in the root of the `tuya-monitor` project directory with the following content:

```env
HOMEASSISTANT_URL=http://localhost:8123/api
HOMEASSISTANT_TOKEN=YOUR_LONG_LIVED_ACCESS_TOKEN
```

-   **`HOMEASSISTANT_URL`**: The URL to your Home Assistant API. If running `tuya-monitor` with `--network host` and Home Assistant is also on the host (or in a container with `--network host`), `http://localhost:8123/api` should work.
-   **`HOMEASSISTANT_TOKEN`**: A Long-Lived Access Token from Home Assistant.
    1.  In Home Assistant, click on your user profile (bottom left).
    2.  Scroll down to "Long-Lived Access Tokens."
    3.  Click "CREATE TOKEN," give it a name (e.g., `tuya-monitor`), and copy the generated token.


## Building and Running as a Systemd Service

This application is designed to run as a systemd service on Linux.

### 1. Prerequisites for Go Build
- **Go**: Ensure Go (version 1.18 or newer recommended) is installed.
- **Git**: To clone the repository if you haven't already.

### 2. ADB Setup (Wireless Debugging Recommended)

It's recommended to use Wireless Debugging for easier integration.

**On your Android Device:**

1.  Enable **Developer options**.
2.  Enable **Wireless debugging**.
3.  Note the **IP address and port** for connecting (e.g., `192.168.1.100:41235`). Some newer Android versions might also require a pairing step using a different port.

**On your Host Machine (where the service will run):**

1.  **Pair (if needed for your Android version):**
    ```bash
    adb pair <device_ip>:<pairing_port>
    ```
    Enter the pairing code shown on your device.

2.  **Connect to your Android device via Wi-Fi:**
    ```bash
    adb connect <device_ip>:<connection_port>
    ```
    Example: `adb connect 192.168.1.100:41235`

3.  **Verify the connection:**
    ```bash
    adb devices
    ```
    You should see your device listed (e.g., `192.168.1.100:41235 device`). Ensure it says "device" and not "unauthorized" or "offline".

### 3. Build the Go Application

1.  Navigate to the `tuya-monitor` project directory (e.g., `/home/<your path>/tuya`).
2.  Initialize Go modules (if you haven't already):
    ```bash
    go mod init tuya-monitor # Or your preferred module name
    go mod tidy
    ```
3.  Build the executable:
    ```bash
    go build -o tuya-monitor-go main.go
    ```
    This creates an executable file named `tuya-monitor-go` in the project directory.

### 4. Setup the Systemd Service

1.  **Create/Verify the Service File**:
    Ensure you have a `tuya-monitor.service` file in your project directory (e.g., `/home/<your path>/tuya/tuya-monitor.service`) with the correct paths and user. It should look similar to this:

    ```ini
    [Unit]
    Description=Tuya Battery Monitor Service (Go Version)
    After=network.target

    [Service]
    Type=simple
    User=<your linux username> # Replace with your actual username if different
    # AmbientCapabilities=CAP_SYS_RAWIO # May not be needed if ADB is setup for user
    WorkingDirectory=/home/<your path>/tuya # Replace with your actual project path
    ExecStart=/home/<your path>/tuya/tuya-monitor-go # Replace with your actual project path
    Restart=always
    RestartSec=10
    EnvironmentFile=/home/<your path>/tuya/.env # Replace with your actual project path

    [Install]
    WantedBy=multi-user.target
    ```
    **Important:**
    - Update `User`, `WorkingDirectory`, `ExecStart`, and `EnvironmentFile` to match your actual setup.
    - The `AmbientCapabilities=CAP_SYS_RAWIO` line might not be strictly necessary if your user has permissions to run `adb` without sudo and interact with USB devices, or if ADB is running as a system service accessible by the user. Test without it first if unsure.

2.  **Copy the service file to the systemd directory:**
    ```bash
    sudo cp /home/<your path>/tuya/tuya-monitor.service /etc/systemd/system/tuya-monitor.service
    ```
    (Adjust the source path if your service file is named differently or located elsewhere).

3.  **Reload systemd, enable, and start the service:**
    ```bash
    sudo systemctl daemon-reload
    sudo systemctl enable tuya-monitor.service
    sudo systemctl start tuya-monitor.service
    ```

### 5. Checking Logs

To view the logs of the running service:

```bash
sudo systemctl status tuya-monitor.service
journalctl -u tuya-monitor.service -f
```

### 6. Service Management

-   **Start the service:**
    ```bash
    sudo systemctl start tuya-monitor.service
    ```
-   **Stop the service:**
    ```bash
    sudo systemctl stop tuya-monitor.service
    ```
-   **Restart the service:**
    ```bash
    sudo systemctl restart tuya-monitor.service
    ```
-   **Disable auto-start:**
    ```bash
    sudo systemctl disable tuya-monitor.service
    ```

## Troubleshooting (Systemd/Go)

-   **Service Fails to Start**:
    -   Check `sudo systemctl status tuya-monitor.service` and `journalctl -u tuya-monitor.service` for error messages.
    -   Verify all paths in the `.service` file are correct.
    -   Ensure the `tuya-monitor-go` executable has execute permissions (`chmod +x tuya-monitor-go`).
    -   Confirm the `.env` file exists at the specified `EnvironmentFile` path and is readable by the service user.
    -   Test running the `tuya-monitor-go` executable directly from the `WorkingDirectory` as the specified `User` to see if it runs manually.
-   **ADB Issues**:
    -   Ensure `adb devices` shows your device as `device` when run by the user the service runs as (or system-wide if ADB server is system-wide).
    -   The service might not have the same environment (e.g., `PATH` to `adb`) as your interactive shell. You might need to specify the full path to `adb` in the Go code or ensure `adb` is in a standard system path.
-   **Cannot Connect to Home Assistant**:
    -   Verify `HOMEASSISTANT_URL` in `.env` is correct.
    -   Ensure Home Assistant is running and accessible from the machine running the service.
    -   Double-check your `HOMEASSISTANT_TOKEN` in `.env`.
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
