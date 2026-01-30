//! Transport layer for mBot2 communication

use anyhow::{anyhow, Context, Result};
use mbot_core::{MBotSensors, MotorCommand};
use std::time::Duration;
use tracing::{debug, info};

use crate::protocol;

pub enum TransportType {
    #[cfg(feature = "bluetooth")]
    Bluetooth,
    #[cfg(feature = "serial")]
    Serial(String),
    Simulated,
}

pub struct MBotTransport {
    inner: TransportInner,
    // Simulation state
    sim_distance: f32,
    sim_encoder_left: i32,
    sim_encoder_right: i32,
    sim_tick: u64,
}

enum TransportInner {
    #[cfg(feature = "bluetooth")]
    Bluetooth(BluetoothTransport),
    #[cfg(feature = "serial")]
    Serial(SerialTransport),
    Simulated,
}

#[cfg(feature = "bluetooth")]
struct BluetoothTransport {
    // btleplug peripheral would go here
    #[allow(dead_code)]
    connected: bool,
}

#[cfg(feature = "serial")]
struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl MBotTransport {
    pub async fn connect(transport_type: TransportType) -> Result<Self> {
        let inner = match transport_type {
            #[cfg(feature = "bluetooth")]
            TransportType::Bluetooth => {
                let bt = Self::connect_bluetooth().await?;
                TransportInner::Bluetooth(bt)
            }
            #[cfg(feature = "serial")]
            TransportType::Serial(port_name) => {
                let serial = Self::connect_serial(&port_name)?;
                TransportInner::Serial(serial)
            }
            TransportType::Simulated => TransportInner::Simulated,
        };

        Ok(Self {
            inner,
            sim_distance: 100.0,
            sim_encoder_left: 0,
            sim_encoder_right: 0,
            sim_tick: 0,
        })
    }

    #[cfg(feature = "bluetooth")]
    async fn connect_bluetooth() -> Result<BluetoothTransport> {
        use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
        use btleplug::platform::Manager;

        info!("🔍 Scanning for mBot2...");

        let manager = Manager::new()
            .await
            .context("Failed to create Bluetooth manager")?;

        let adapters = manager
            .adapters()
            .await
            .context("Failed to get Bluetooth adapters")?;

        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No Bluetooth adapter found"))?;

        // Start scanning
        adapter
            .start_scan(ScanFilter::default())
            .await
            .context("Failed to start Bluetooth scan")?;

        // Wait for devices
        tokio::time::sleep(Duration::from_secs(5)).await;

        let peripherals = adapter
            .peripherals()
            .await
            .context("Failed to get peripherals")?;

        // Find mBot2 (CyberPi)
        for peripheral in peripherals {
            if let Ok(Some(props)) = peripheral.properties().await {
                let name = props.local_name.unwrap_or_default();
                debug!("Found device: {}", name);

                if name.contains("Makeblock") || name.contains("CyberPi") || name.contains("mBot") {
                    info!("✅ Found mBot2: {}", name);

                    peripheral
                        .connect()
                        .await
                        .context("Failed to connect to mBot2")?;

                    peripheral
                        .discover_services()
                        .await
                        .context("Failed to discover services")?;

                    info!("✅ Connected to mBot2!");

                    return Ok(BluetoothTransport { connected: true });
                }
            }
        }

        Err(anyhow!(
            "mBot2 not found. Make sure it's powered on and Bluetooth is enabled."
        ))
    }

    #[cfg(feature = "serial")]
    fn connect_serial(port_name: &str) -> Result<SerialTransport> {
        info!("📡 Opening serial port: {}", port_name);

        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(100))
            .open()
            .context(format!("Failed to open serial port: {}", port_name))?;

        info!("✅ Serial port opened!");

        Ok(SerialTransport { port })
    }

    pub async fn read_sensors(&mut self) -> Result<MBotSensors> {
        match &mut self.inner {
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_bt) => {
                // TODO: Read from Bluetooth notification characteristic
                // For now, return simulated values
                self.read_simulated()
            }
            #[cfg(feature = "serial")]
            TransportInner::Serial(serial) => {
                // Send read command
                let cmd = protocol::read_ultrasonic_cmd();
                serial.port.write_all(&cmd)?;

                // Read response
                let mut buf = [0u8; 64];
                match serial.port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let distance = protocol::parse_ultrasonic_response(&buf[..n])
                            .unwrap_or(100.0);

                        Ok(MBotSensors {
                            ultrasonic_cm: distance,
                            ..Default::default()
                        })
                    }
                    _ => self.read_simulated(),
                }
            }
            TransportInner::Simulated => self.read_simulated(),
        }
    }

    fn read_simulated(&mut self) -> Result<MBotSensors> {
        self.sim_tick += 1;

        // Simulate varying distance (something approaching and retreating)
        let wave = ((self.sim_tick as f32) * 0.02).sin();
        self.sim_distance = 50.0 + wave * 40.0;

        // Add occasional "close approach"
        if self.sim_tick % 200 > 180 {
            self.sim_distance = 10.0 + (self.sim_tick % 20) as f32;
        }

        // Simulate encoder movement
        self.sim_encoder_left += 5;
        self.sim_encoder_right += 5;

        Ok(MBotSensors {
            timestamp_us: self.sim_tick * 50_000, // 50ms per tick
            ultrasonic_cm: self.sim_distance,
            encoder_left: self.sim_encoder_left,
            encoder_right: self.sim_encoder_right,
            gyro_z: wave * 10.0,
            accel: [wave * 0.5, 0.0, 9.8],
            sound_level: 0.1 + (wave * 0.1).abs(),
            light_level: 0.5,
            quad_rgb: [[200, 200, 200]; 4], // White surface
        })
    }

    pub async fn send_command(&mut self, cmd: &MotorCommand) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "bluetooth")]
            TransportInner::Bluetooth(_bt) => {
                // TODO: Write to Bluetooth characteristic
                debug!(
                    "BT Command: L={} R={} LED={:?}",
                    cmd.left, cmd.right, cmd.led_color
                );
                Ok(())
            }
            #[cfg(feature = "serial")]
            TransportInner::Serial(serial) => {
                // Send motor command
                let motor_cmd = protocol::motor_cmd(cmd.left, cmd.right);
                serial.port.write_all(&motor_cmd)?;

                // Send LED command
                let led_cmd = protocol::led_cmd(cmd.led_color);
                serial.port.write_all(&led_cmd)?;

                // Send pen servo if drawing
                if cmd.pen_angle != 45 {
                    let servo_cmd = protocol::servo_cmd(1, cmd.pen_angle);
                    serial.port.write_all(&servo_cmd)?;
                }

                Ok(())
            }
            TransportInner::Simulated => {
                debug!(
                    "SIM Command: L={} R={} Mode={:?}",
                    cmd.left,
                    cmd.right,
                    if cmd.left < 0 && cmd.right < 0 {
                        "REVERSE"
                    } else if cmd.left > cmd.right {
                        "TURN_RIGHT"
                    } else if cmd.right > cmd.left {
                        "TURN_LEFT"
                    } else {
                        "FORWARD"
                    }
                );
                Ok(())
            }
        }
    }
}
