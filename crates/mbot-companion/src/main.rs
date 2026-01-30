//! mBot2 Companion - Runs RuVector AI on laptop, controls mBot2 via Bluetooth/Serial
//!
//! Usage:
//!   mbot-companion --bluetooth           # Connect via Bluetooth
//!   mbot-companion --serial /dev/ttyUSB0 # Connect via USB serial
//!   mbot-companion --simulate            # Run without hardware (testing)

use anyhow::{Context, Result};
use clap::Parser;
use mbot_core::{HomeostasisState, MBotBrain, MBotSensors, MotorCommand, ReflexMode};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn, Level};

mod protocol;
mod transport;

use transport::{MBotTransport, TransportType};

#[derive(Parser, Debug)]
#[command(name = "mbot-companion")]
#[command(about = "RuVector AI companion for mBot2", long_about = None)]
struct Args {
    /// Connect via Bluetooth
    #[arg(long)]
    bluetooth: bool,

    /// Connect via serial port
    #[arg(long)]
    serial: Option<String>,

    /// Simulate without hardware
    #[arg(long)]
    simulate: bool,

    /// Control loop frequency in Hz
    #[arg(long, default_value = "20")]
    freq: u32,

    /// Enable drawing mode (pen attached)
    #[arg(long)]
    draw: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("🤖 mBot2 RuVector Companion starting...");

    // Determine transport type
    let transport_type = if args.simulate {
        info!("📡 Running in SIMULATION mode");
        TransportType::Simulated
    } else if args.bluetooth {
        #[cfg(feature = "bluetooth")]
        {
            info!("📡 Connecting via Bluetooth...");
            TransportType::Bluetooth
        }
        #[cfg(not(feature = "bluetooth"))]
        {
            anyhow::bail!("Bluetooth support not compiled. Rebuild with: cargo build --features bluetooth");
        }
    } else if let Some(port) = &args.serial {
        #[cfg(feature = "serial")]
        {
            info!("📡 Connecting via Serial: {}", port);
            TransportType::Serial(port.clone())
        }
        #[cfg(not(feature = "serial"))]
        {
            let _ = port;
            anyhow::bail!("Serial support not compiled. Rebuild with: cargo build --features serial");
        }
    } else {
        info!("📡 No connection specified, running in SIMULATION mode");
        info!("   Use --bluetooth or --serial <port> for real hardware");
        TransportType::Simulated
    };

    // Create transport
    let transport = MBotTransport::connect(transport_type).await?;
    let transport = Arc::new(Mutex::new(transport));

    // Create brain
    let brain = Arc::new(Mutex::new(MBotBrain::new()));

    // Run main loop
    run_main_loop(transport, brain, args.freq, args.draw).await
}

async fn run_main_loop(
    transport: Arc<Mutex<MBotTransport>>,
    brain: Arc<Mutex<MBotBrain>>,
    freq: u32,
    draw_mode: bool,
) -> Result<()> {
    let tick_duration = Duration::from_secs_f64(1.0 / freq as f64);
    let mut last_tick = Instant::now();
    let mut tick_count = 0u64;

    info!("🧠 Starting AI loop at {}Hz", freq);
    if draw_mode {
        info!("🖊️  Drawing mode ENABLED");
    }

    // Stats tracking
    let mut total_loop_time = Duration::ZERO;
    let mut max_loop_time = Duration::ZERO;

    loop {
        let loop_start = Instant::now();

        // Read sensors
        let sensors = {
            let mut t = transport.lock().await;
            t.read_sensors().await?
        };

        // Process through brain
        let (state, mut cmd) = {
            let mut b = brain.lock().await;
            b.tick(&sensors)
        };

        // Override pen state if not in draw mode
        if !draw_mode {
            cmd.pen_angle = 45; // Keep pen up
        }

        // Send motor commands
        {
            let mut t = transport.lock().await;
            t.send_command(&cmd).await?;
        }

        // Track timing
        let loop_time = loop_start.elapsed();
        total_loop_time += loop_time;
        max_loop_time = max_loop_time.max(loop_time);
        tick_count += 1;

        // Print status periodically
        if tick_count % (freq as u64) == 0 {
            // Every second
            print_status(&sensors, &state, tick_count, total_loop_time, max_loop_time);
        }

        // Maintain loop timing
        let elapsed = last_tick.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await;
        } else if elapsed > tick_duration * 2 {
            warn!("Loop running slow: {:?} > {:?}", elapsed, tick_duration);
        }
        last_tick = Instant::now();
    }
}

fn print_status(
    sensors: &MBotSensors,
    state: &HomeostasisState,
    tick_count: u64,
    total_time: Duration,
    max_time: Duration,
) {
    let avg_time = total_time / tick_count as u32;
    let mode_icon = match state.reflex {
        ReflexMode::Calm => "😌",
        ReflexMode::Active => "🔍",
        ReflexMode::Spike => "⚡",
        ReflexMode::Protect => "🛡️",
    };

    println!(
        "\n╔══════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║  {} {:?}  │  Tension: {:.2}  │  Coherence: {:.2}  │  Energy: {:.2}",
        mode_icon, state.reflex, state.tension, state.coherence, state.energy
    );
    println!(
        "╠══════════════════════════════════════════════════════════════╣"
    );
    println!(
        "║  📏 Distance: {:>6.1} cm  │  🔊 Sound: {:.2}  │  💡 Light: {:.2}",
        sensors.ultrasonic_cm, sensors.sound_level, sensors.light_level
    );
    println!(
        "║  ⚙️  Encoders: L={:>5} R={:>5}  │  🌀 Gyro: {:>6.1}°/s",
        sensors.encoder_left, sensors.encoder_right, sensors.gyro_z
    );
    println!(
        "╠══════════════════════════════════════════════════════════════╣"
    );
    println!(
        "║  ⏱️  Tick: {:>6}  │  Avg: {:>4}µs  │  Max: {:>4}µs",
        tick_count,
        avg_time.as_micros(),
        max_time.as_micros()
    );
    println!(
        "╚══════════════════════════════════════════════════════════════╝"
    );
}
