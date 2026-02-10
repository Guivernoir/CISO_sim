# CISO Judgment Simulator

> **Tagline:** Every decision is a liability.

A narrative simulation of how security decisions compound into legal outcomes. This isn't a game about stopping hackers—it's a game about surviving discovery.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/security-hardened-green.svg)](#security-architecture)

## What Is This?

You're the new CISO of a mid-sized tech company. The previous security leader "left to pursue other opportunities" after a breach. The board wants "fresh leadership with new ideas."

**The twist:** This game doesn't punish you immediately. It audits you later.

- Make compromises to ship features? The audit trail remembers.
- Bury incidents to avoid panic? Discovery finds them.
- Cut corners on compliance? Regulators have long memories.

Every decision you make is recorded. Every shortcut documented. Every compromise tracked. And in the end, **you face the consequences**.

## Features

### 🎭 Narrative-Driven Gameplay
- 10+ turns of escalating pressure and consequences
- Decisions across strategic direction, incident response, compliance, and politics
- Multiple endings based on your integrity score and risk management

### 📊 Realistic Security Scenarios
- Vendor breach inheritance
- Third-party data sharing dilemmas
- Insider threat incidents
- Ransomware response
- SOC 2 compliance theater
- AI/ML privacy violations
- M&A security debt
- Federal investigations

### 🎯 Meaningful Consequences
- **Immediate impacts**: Budget, political capital, team morale
- **Delayed consequences**: Risks compound, incidents trigger, auditors discover
- **Narrative integrity tracking**: Every lie, every buried incident, every shortcut tracked
- **Four possible endings**: Golden CISO, Lawsuit Survivor, Post-Breach Cleanup, Criminal Investigation

### 🔐 Security-First Architecture
- Written in **Hardened Rust** following zero-trust principles
- Encrypted save files using AES-256-GCM with Argon2 key derivation
- Opaque error handling (never exposes system internals)
- Memory-safe with automatic zeroization of sensitive data
- No unsafe code, no panics, no data leaks

### 🎮 Flexible Decision System
- **TOML-based decisions**: Edit scenarios without recompiling
- **Graceful fallback**: Works with or without external decision files
- **Modding support**: Create custom decision packs
- **Hot-swappable content**: Update scenarios on the fly

## Screenshots

```
╔═══════════════════════════════════════════════════════╗
║                                                       ║
║           CISO JUDGMENT SIMULATOR v1.0                ║
║           A Security Failure RPG                      ║
║                                                       ║
║   Tagline: Every decision is a liability.             ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝

Turn 1 - Q1 - Phase: Inheritance Disaster

CURRENT STATUS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CISO: Alex Chen | Company: TechFlow Solutions
ARR: $45.0M | Board Confidence: 75% | Integrity: 100%
Risk Total: 35 | Budget Available: $1.70M
```

## Installation

### Prerequisites

- **Rust 1.75 or later** ([Install Rust](https://rustup.rs/))
- 50MB free disk space
- Terminal with UTF-8 support

### Quick Start

```bash
# Clone the repository
git clone https://github.com/Guivernoir/CISO-sim.git
cd ciso-simulator

# Build release version
cargo build --release

# Run the game
./target/release/ciso_simulator
```

### With Decision Files (Optional)

```bash
# Create decision data directory
mkdir -p data/decisions

# Copy decision files (if you have custom scenarios)
cp decisions/*.toml data/decisions/

# Run with custom scenarios
./target/release/ciso_simulator
```

## How to Play

### Game Flow

1. **Each Turn Represents ~1 Month** of your tenure as CISO
2. **Review Status**: ARR, budget, risk levels, team morale
3. **Face a Decision**: Strategic choice with multiple options
4. **See Immediate Impact**: Budget changes, political capital shifts
5. **Discover Delayed Consequences**: Risks materialize, incidents trigger
6. **Survive Discovery**: Auditors, regulators, and lawyers examine your choices

### Decision Categories

- **Strategic Direction**: Set security program priorities
- **Incident Response**: Handle active breaches and threats
- **Budget Allocation**: Fight for resources or make compromises
- **Compliance Approach**: Theater vs. substance
- **Team Management**: Retention, hiring, capacity planning
- **Vendor Selection**: Political vs. technical decisions
- **Risk Acceptance**: What risks are you willing to take?
- **Political Navigation**: Board pressure and C-suite dynamics

### Key Metrics

#### Business Metrics
- **ARR (Annual Recurring Revenue)**: Company growth/decline
- **Board Confidence**: Do they trust your judgment?
- **Velocity**: How fast is the team shipping?
- **Churn**: Customer retention impact

#### Security Metrics
- **Risk Vectors**: Data Exposure, Access Control, Detection, Vendor Risk, Insider Threat
- **Total Exposure**: Combined risk across all vectors
- **Mitigation Coverage**: How well risks are managed
- **Time to Critical**: How many turns until disaster?

#### Political Metrics
- **Political Capital**: Currency for pushing back on bad ideas
- **Team Morale**: Will your team follow you into fire?
- **Industry Standing**: Can you get another job after this?
- **Vendor Relationships**: Can you call in favors?

#### Narrative Integrity
- **Integrity Score**: 0-100, tracks honesty and transparency
- **Buried Incidents**: Hidden problems that discovery will find
- **Delayed Escalations**: Incidents you slow-walked
- **Audit Trail Quality**: Clean, Flagged, or Toxic

### Winning vs. Losing

There are four possible endings:

1. **Golden CISO** (Top 5%): You did everything right, maintained integrity, managed risks
2. **Lawsuit Survivor** (Middle 70%): You made it out alive with your career intact
3. **Post-Breach Cleanup** (Bottom 25%): Resume update time
4. **Criminal Investigation** (Bottom 1%): Lawyer up

**The game tracks your Narrative Integrity score.** This is your "multiplier" in lawsuits and regulatory fines:
- **Score > 85**: Good faith, 1.0x penalty
- **Score 50-85**: Negligence, 1.8x penalty  
- **Score < 50**: Bad faith, 3.2x penalty
- **Score < 30 + Multiple Buried Incidents**: Criminal exposure

## Game Mechanics

### Risk Accumulation

Risks don't fail fast—they **accrete silently**:

```rust
Turn 1: Ship insecure API (+35 Data Exposure risk)
Turn 2: Accept bad vendor TOS (+40 Data Exposure risk)
Turn 3: → Risk materializes: Data breach
Turn 4: → Regulators investigate
Turn 5: → Discovery examines your decision trail
```

### Cascading Failures

Risks are interconnected:
- **Poor Access Control** amplifies **Data Exposure**
- **Weak Detection** increases all risks by 50%
- **Vendor Risk** cascades to **Supply Chain Risk**

### Narrative Consistency

The game tracks whether your actions match your words:

- Tell board "security is a priority" then cut SOC budget? **Inconsistency penalty**
- Claim "we have strong controls" then fail audit? **Integrity penalty**
- Public statement "no evidence of misuse" when you know otherwise? **Toxic audit trail**

### Political Capital

You start with limited political capital. Spend it to:
- Push back on bad ideas
- Delay dangerous features
- Demand budget increases
- Challenge vendor selections

**But political capital is finite.** Spend it wisely, or you'll become "Dr. No" who gets ignored.

## Technical Architecture

### Core Design Principles

This project follows **"Hardened Rust"** security standards:

#### 1. Memory Safety
```rust
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SessionToken {
    pub data: [u8; 32],  // Automatically zeroized on drop
}
```

#### 2. Encryption at Rest
```rust
// AES-256-GCM for save file encryption
// Argon2id for key derivation
pub struct GamePersistence {
    encryption_key: [u8; 32],  // Derived from user password
}
```

#### 3. Opaque Error Handling
```rust
pub enum GameError {
    StateCorruption,  // Never exposes parsing details
    SystemFailure,    // Never exposes file paths
    InvalidAction,    // Never exposes internal state
}
```

#### 4. Immutable Infrastructure
- Game state is append-only event log
- Decisions cannot be undone (audit trail preservation)
- All state transitions recorded

### Dependencies

```toml
[dependencies]
# Async runtime - industry standard
tokio = { version = "1.35", features = ["full"] }

# Serialization - battle-tested
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Cryptography - Mozilla-audited
ring = "0.17"

# Memory hygiene
zeroize = { version = "1.7", features = ["derive"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Terminal UI
crossterm = "0.27"
colored = "2.1"
toml = "0.9.11"
textwrap = "0.16.2"
console = "0.16.2"
argon2 = "0.5.3"
rand = "0.8.5"
ratatui = "0.30.0"
```

## Building from Source

### Development Build

```bash
cargo build
cargo run
```

### Release Build (Optimized)

```bash
# Maximum optimization
cargo build --release

# Run optimized binary
./target/release/ciso_simulator
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_risk_accumulation
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy -- -D warnings

# Security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Decision File Format

Decisions can be defined in TOML files for easy modification:

```toml
[[decision]]
turn = 1
title = "The Inheritance"
context = """Your predecessor left you a mess..."""
is_board_pressure = false
decision_category = "StrategicDirection"

[[decision.choice]]
id = "ship_it"
label = "Ship it - Fix it in production"
description = "Let the feature ship..."

[decision.choice.impact_preview]
estimated_arr_change = 2.5
budget_cost = 0.05
timeline_weeks = 2
political_note = "Builds goodwill with engineering"

[decision.choice.impact.risk_delta.changes.DataExposure]
level_delta = 35.0
description = "Unauthenticated bulk data access"

[decision.choice.impact.business_delta]
arr_change = 2.5
velocity_change = 5.0
churn_change = 0.0
confidence_change = 10.0

[decision.choice.impact]
audit_trail = "Flagged"
budget_impact = -0.05
```

## Configuration

### Save File Location

Save files are stored as encrypted `.enc` files:

```
./ciso_save.enc  # Default save location
```

### Decision Data Location

The game looks for decision files in:

1. `./data/decisions/` (relative to working directory)
2. `<executable_dir>/data/decisions/` (relative to binary)

If no TOML files are found, the game uses hardcoded decisions (fully functional).

## Security Architecture

### Threat Model

This game simulates security decisions, so it's built with real security:

#### Encryption
- **Save files**: AES-256-GCM with authenticated encryption
- **Key derivation**: Argon2id with 150,000 iterations
- **Nonce handling**: Counter-based with randomized initialization

#### Memory Safety
- **No unsafe code** in decision-critical paths
- **Zeroization**: Sensitive data cleared on drop
- **No Clone semantics**: Data ownership prevents side-channel attacks

#### Error Handling
- **Opaque errors**: Never expose file paths or internal state
- **No panics**: All errors handled gracefully
- **Graceful degradation**: Missing files don't crash the game

#### Input Validation
- **TOML parsing**: Validated schema with safe defaults
- **Enum conversion**: Invalid values use safe defaults
- **File I/O**: All errors mapped to opaque types

### Privacy

- **No telemetry**: Game doesn't phone home
- **No analytics**: Your decisions stay on your machine
- **Local saves**: All data encrypted locally
- **No cloud sync**: You control your data

## Contributing

Contributions welcome! This project follows security-first development:

### Code Standards

1. **No unsafe code** without security review
2. **All errors must be opaque** (use GameError types)
3. **Memory must be zeroized** for sensitive data
4. **No panics** in production code paths
5. **Document security assumptions** in comments

### Contribution Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow Rust style guide (`cargo fmt`)
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Run security audit (`cargo audit`)
7. Submit a pull request

### Adding New Decisions

To add new scenarios:

1. Create a TOML file in `data/decisions/turn_XX.toml`
2. Test the decision loads correctly
3. Submit PR with decision file and rationale

## License

This project is licensed under the MIT License.

## Acknowledgments

- **Inspiration**: Every CISO who's had to choose between "right" and "shippable"
- **Security principles**: NIST Cybersecurity Framework, OWASP, CIS Controls
- **Narrative design**: Based on real incident response and compliance scenarios
- **Rust community**: For creating a language that makes security the default

## Disclaimer

This is a simulation. Any resemblance to real companies, incidents, or CISOs is purely coincidental. The scenarios are dramatized for gameplay but based on realistic security challenges.

**This game is not**:
- Legal advice
- Security consulting
- A training certification
- A substitute for actual security expertise

**This game is**:
- A narrative exploration of security decision-making
- An illustration of how choices compound over time
- A reminder that "move fast and break things" has consequences
- Entertainment with educational value

## FAQ

### Is this game realistic?

**Yes and no.** The scenarios are based on real security challenges, but compressed for gameplay. Real CISO decisions take months to play out; here they take minutes. The core principle is accurate: **security decisions accrete into legal outcomes**.

### Can I really go to prison in this game?

**In the game, yes.** If your narrative integrity drops below 30 and you've buried multiple incidents, you get the "Criminal Investigation" ending. In reality, criminal charges for CISOs are rare but not unheard of (see: Uber CISO case).

### Why Rust?

Security simulation deserves secure implementation. Rust prevents entire classes of vulnerabilities (buffer overflows, use-after-free, data races) at compile time. For a game about security consequences, it would be hypocritical to use memory-unsafe languages.

### Can I mod the game?

**Yes!** Create TOML decision files and drop them in `data/decisions/`. The game will load them automatically. See [INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md) for the schema.

## Support

- **Issues**: [GitHub Issues](https://github.com/Guivernoir/CISO-sim/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Guivernoir/CISO-sim/discussions)
- **Email**: strukturaenterprise@gmail.com

**Built with Rust 🦀 | Narrative-Driven 📖**

*"Risk doesn't fail fast—it accretes silently."*