# 🤖 mBot2 RuVector - AI at the Edge

**Give your mBot2 a nervous system!** This project integrates [RuVector's](https://github.com/ruvnet/ruvector) DAG-based AI with the Makeblock mBot2 robot, enabling:

- 🧠 **Emotional AI** - The robot develops "moods" based on sensor input
- 🎨 **Artistic Drawing** - Attach a pen and watch it create mood-based art
- 🎮 **Interactive Games** - Play tic-tac-toe against your robot
- 📊 **Real-time Dashboard** - Visualize the robot's neural state

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     mBot2 RuVector System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    Bluetooth/Serial    ┌──────────────────┐   │
│  │   mBot2      │ ◄─────────────────────► │  Companion App   │   │
│  │  (CyberPi)   │                         │   (Laptop)       │   │
│  │              │                         │                  │   │
│  │  • Sensors   │                         │  • RuVector AI   │   │
│  │  • Motors    │                         │  • SONA Learning │   │
│  │  • LEDs      │                         │  • Web Dashboard │   │
│  │  • Pen Servo │                         │                  │   │
│  └──────────────┘                         └──────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
cd mbot_ruvector

# Build Rust companion app
cargo build --release
```

### 2. Run in Simulation (No Hardware)

```bash
# Test the AI without mBot2 connected
cargo run --bin mbot-companion -- --simulate
```

### 3. Connect to Real mBot2

```bash
# Via Bluetooth
cargo run --bin mbot-companion -- --bluetooth

# Via USB Serial
cargo run --bin mbot-companion -- --serial /dev/ttyUSB0
```

### 4. Start the Web Dashboard

```bash
cd web
npm install
npm start
# Open http://localhost:3000
```

## Project Structure

```
mbot_ruvector/
├── crates/
│   ├── mbot-core/          # Core AI logic (works on laptop and ESP32)
│   ├── mbot-companion/     # Laptop companion app
│   │   └── src/bin/
│   │       ├── tictactoe.rs  # Tic-tac-toe game
│   │       └── draw.rs       # Emotional art drawing
│   └── mbot-embedded/      # Direct ESP32 deployment (advanced)
├── web/                    # Real-time dashboard
└── docs/                   # Additional documentation
```

## The AI: RuVector Nervous System

The robot uses RuVector's DAG-based "nervous system" with these key concepts:

### Reflex Modes

| Mode | Tension | Behavior |
|------|---------|----------|
| 😌 **Calm** | < 0.20 | Gentle wandering, learning enabled |
| 🔍 **Active** | 0.20 - 0.55 | Curious exploration |
| ⚡ **Spike** | 0.55 - 0.85 | Excited, fast movements |
| 🛡️ **Protect** | > 0.85 | Back away from danger |

### Key Metrics

- **Tension**: Deviation from equilibrium (0-1). High tension = stressed
- **Coherence**: Internal consistency (0-1). High coherence = stable
- **Energy**: Depletes with high tension, recovers when calm
- **Curiosity**: Peaks when something interesting (but not threatening) is detected

## Drawing Mode 🖊️

Attach a servo-controlled pen to the back of your mBot2!

### Hardware Setup

```
      mBot2
    ┌─────────┐
    │         │
    │  [pen]  │  ← Servo on Port 1
    │    │    │     Angle 45° = up
    │    ▼    │     Angle 90° = down
    └─────────┘
```

### Run Drawing Mode

```bash
# Emotional spirograph art
cargo run --bin mbot-draw

# Tic-tac-toe (attach pen first!)
cargo run --bin mbot-tictactoe
```

## Games

### Tic-Tac-Toe

Play against the robot! It draws X's and O's on paper.

```bash
cargo run --bin mbot-tictactoe
```

- You are X, Robot is O
- Robot uses minimax + learned patterns
- Gets smarter over time with SONA learning

## Web Dashboard

Real-time visualization of the robot's neural state.

```bash
cd web && npm start
```

Features:
- Live tension/coherence meters
- Reflex mode indicator
- Sensor readings
- Robot visualization

## Dependencies

- [RuVector](https://github.com/ruvnet/ruvector) - AI nervous system
- [btleplug](https://crates.io/crates/btleplug) - Bluetooth LE
- [serialport](https://crates.io/crates/serialport) - USB Serial

## Hardware

- **Robot**: [Makeblock mBot2](https://www.makeblock.com/pages/mbot2)
- **Controller**: CyberPi (ESP32-based)
- **Sensors**: Ultrasonic, Quad RGB, Gyroscope, Accelerometer
- **Optional**: Servo + pen holder for drawing

## Future Plans

- [ ] Direct ESP32 deployment (no laptop needed)
- [ ] Voice control integration
- [ ] Multi-robot swarms
- [ ] Camera-based object tracking
- [ ] SONA learning persistence

## License

MIT OR Apache-2.0

## Credits

- RuVector by [ruvnet](https://github.com/ruvnet)
- mBot2 by [Makeblock](https://www.makeblock.com)
