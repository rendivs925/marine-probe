# 📊 Live Marine Sensor Data Dashboard

A real-time web dashboard that displays live temperature, salinity, and turbidity data from marine sensors. The system architecture uses MQTT for message brokering, Actix Web for WebSocket serving, and Leptos for reactive frontend rendering.

---

## 🧱 Architecture Overview

`[Sensor Simulator] --> MQTT Broker --> Actix Web (MQTT → WS bridge) --> Leptos Frontend (WebSocket)`

- **Sensor Simulator**: Publishes mock sensor data to MQTT.
- **MQTT Broker**: For example, Mosquitto.
- **Backend (Rust)**:
  - Subscribes to MQTT topic (`probe/data`).
  - Forwards data via WebSocket to frontend clients.
- **Frontend (Leptos + TailwindCSS)**:
  - Connects via WebSocket.
  - Reactively displays sensor data in real time.

---

## 🧩 Backend

### Dependencies

```
actix-web
actix-web-actors
rumqttc
tokio
serde
serde_json
anyhow
```

### Key Components

- `ws_handler`: Upgrades HTTP to WebSocket connection.
- `WsSession`: An actor that pushes messages from a `broadcast::Receiver` to each WebSocket client.
- **MQTT Client** subscribes to `probe/data`, receives `SensorData` in JSON, and sends it to all connected WebSocket clients.

### MQTT Payload Format

```json
{
  "timestamp": "2025-06-07T12:00:00Z",
  "temperature": 26.32,
  "salinity": 33.55,
  "turbidity": 10.48
}
```

---

## 🖥️ Frontend (Leptos)

### Dependencies

```
leptos
chrono
serde
serde_json
wasm-bindgen
```

### Highlights

- Uses WebSocket from `web_sys`.
- Parses JSON into `SensorData` struct.
- Formats timestamp with `chrono`.
- UI styled with Tailwind CSS.
- Reactive updates via `signal()` and `Effect::new`.

### Timestamp Formatting

```rust
let formatted_time = DateTime::parse_from_rfc3339(&d.timestamp)
    .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
    .unwrap_or_else(|_| d.timestamp.clone());
```

### Tailwind UI Sample

```html
<div
  class="bg-white rounded-xl shadow-md p-6 flex flex-col gap-4 border border-gray-300"
>
  <div class="flex justify-between">
    <span class="font-semibold text-gray-600">Timestamp:</span>
    <span>2025-06-07 19:02:11</span>
  </div>
</div>
```

---

## 🧪 Sensor Simulator

### Purpose

Simulates a physical marine sensor by publishing MQTT messages every 2 seconds.

### Notes

- Uses `rand::rng()` and `random_range()` (non-deprecated API).
- Uses `chrono::Utc::now().to_rfc3339()` for timestamp generation.

---

## 🧪 Testing Setup

To set up and test the system, follow these steps:

1.  **Run MQTT broker locally** (e.g., Mosquitto):

    Before proceeding, ensure you have Mosquitto installed. If you don't, here's how to install it on common Linux distributions:

    - **Ubuntu/Debian:**

      ```bash
      sudo apt update
      sudo apt install mosquitto mosquitto-clients
      ```

      Then, start the service:

      ```bash
      sudo systemctl start mosquitto.service
      sudo systemctl enable mosquitto.service # (optional) to start on boot
      ```

    - **Arch Linux:**

      ```bash
      sudo pacman -S mosquitto
      ```

      Then, start the service:

      ```bash
      sudo systemctl start mosquitto.service
      sudo systemctl enable mosquitto.service # (optional) to start on boot
      ```

    - **Fedora:**

      ```bash
      sudo dnf install mosquitto
      ```

      Then, start the service:

      ```bash
      sudo systemctl start mosquitto.service
      sudo systemctl enable mosquitto.service # (optional) to start on boot
      ```

    - **Other Distributions:** Please refer to your distribution's package manager documentation for installing Mosquitto. Once installed, the command to start the service is generally `sudo systemctl start mosquitto.service`.

2.  **Start backend WebSocket bridge**:

    ```bash
    cargo run -p backend
    ```

3.  **Start sensor simulator**:

    ```bash
    cargo run -p arduino-sim
    ```

4.  **Start frontend (Leptos)**:

    ```bash
    trunk serve --open
    ```

---

## ✅ Features

- **Real-time updates** via WebSocket
- **Clean, responsive UI** with Tailwind
- **JSON data parsing** with Serde
- **Timestamp formatting** with Chrono
- **Flexible backend** using Actix + MQTT
