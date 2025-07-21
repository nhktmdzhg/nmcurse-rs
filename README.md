# nmcurse-rs

A fast and secure terminal-based WiFi network manager written in Rust, forked from a C++ project with enhanced performance, safety, and additional features.

![image](screenshot.png)

## Features

- 🚀 **Fast and lightweight** - Optimized Rust implementation
- 🔒 **Secure password handling** - Uses zeroize for memory-safe password storage
- 🖥️ **TUI interface** - Clean ncurses-based terminal interface
- 📶 **Signal strength indicators** - Color-coded signal strength display
- ⚡ **Real-time scanning** - Live network discovery with loading animations
- 🔑 **Password management** - Connect, disconnect, and forget saved networks
- 🎯 **NetworkManager integration** - Seamless integration with system networking

## Dependencies

- **ncurses** - Terminal UI library
- **NetworkManager** - System network management (nmcli)
- **Rust** - Programming language and toolchain

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/nhktmdzhg/nmcurse-rs.git
cd nmcurse-rs

# Build with optimizations
cargo build --release
```

### Optimized Build

For maximum performance:

```bash
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release
```

## Usage

```bash
# Run the application
./target/release/nmcurse

# Or run directly with cargo
cargo run --release
```

### Controls

- **↑/↓** - Navigate networks
- **Enter** - Connect to selected network
- **r** - Rescan for networks
- **d** - Disconnect from current network
- **f** - Forget saved password
- **q/Esc** - Quit application

## System Requirements

- Linux with NetworkManager
- ncurses development libraries
- Rust 1.70+ (Rust 2021 edition)

### Installing Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install libncurses5-dev libncursesw5-dev network-manager
```

**Fedora/RHEL:**
```bash
sudo dnf install ncurses-devel NetworkManager
```

**Arch Linux:**
```bash
sudo pacman -S ncurses networkmanager
```

## Performance Optimizations

The release build includes several optimizations:

- **Size optimization** (`opt-level = "z"`)
- **Link-time optimization** (LTO)
- **Symbol stripping** for smaller binary size
- **Single codegen unit** for better optimization
- **Panic abort** for smaller runtime

## Security Features

- **Memory-safe password handling** with automatic clearing
- **Secure string operations** using the zeroize crate
- **Input validation** and bounds checking
- **Error handling** for robust operation

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original C++ implementation inspiration
- ncurses library for terminal UI capabilities
- NetworkManager for network management functionality

## AUR
```
$AUR_HELPER -S nmcurse-rs-git
```