# Pytron 📦 ✨

**Python Deployment Made Ridiculously Simple**

Pytron is a lightning-fast CLI tool that turns Python scripts into portable, deployable packages that run anywhere - with zero configuration. Stop wrestling with environment setups and dependency hell. Just zip, ship, and run.

## ✨ Why Pytron?

- **Ship Complete Environments**: Package your Python script and ALL its dependencies in a single zip file
- **Run Anywhere**: Execute the same code on any system without installation headaches
- **Blazing Fast**: Built in Rust and powered by `uv` for lightning-quick dependency resolution
- **Zero Config**: No complex setup - just works out of the box
- **Cross-Platform**: Windows, macOS, Linux - we've got you covered

## 🚀 Quick Start

### Package your Python project:
```bash
pytron zip -o myapp.zip
```

### Run it anywhere:
```bash
pytron run myapp.zip main.py
```

That's it! No virtual environments, no pip installs, no "works on my machine" syndrome.

## 🔧 Installation

```bash
cargo install pytron
```

## 🧰 Key Features

### 💼 Single-File Distribution
Package your entire Python application - code, dependencies, data files - into a single, portable zip file.

### 🔒 Environment Isolation
Every execution uses its own isolated Python environment, free from system conflicts.

### ⚡ Optimized Performance
Powered by Rust and `uv` - the fastest Python dependency manager available.

### 🔄 Consistent Execution
What works in development works in production - guaranteed.

### 🌐 Cross-Platform Support
Built to handle Windows, macOS, and Linux idiosyncrasies automatically.

## 📋 Core Commands

### `pytron zip` - Package Your Project
```bash
pytron zip [directory] -o [output.zip] [--ignore-patterns]
```

### `pytron run` - Execute Python Code
```bash
pytron run [UV_ARGS] [ZIPFILE/SCRIPT] [SCRIPT_ARGS]
```

## 💡 Perfect For

- **DevOps Automation**: Distribute operations scripts across systems
- **Data Processing**: Ship data pipelines that run consistently
- **CLI Applications**: Turn Python scripts into easy-to-distribute tools
- **Microservices**: Deploy lightweight Python services with minimal overhead

## 🛠️ Advanced Usage

See the [pytron-example](./pytron-example/) directory for detailed examples of:
- Argument handling
- Dependency management
- Running scripts from archives
- Passing arguments to both `uv` and your Python script

## 📚 Documentation

For full command documentation and advanced usage, run:
```bash
pytron --help
```

## 🔗 Contributing

We welcome contributions! Check out the [CLAUDE.md](./CLAUDE.md) file for development guidelines.

## 📄 License

[Apache License 2.0](LICENSE)