<p align="center">
    <img src="assets/logo.svg" width="220">
    <br>
    <a href="https://github.com/joaofelipegalvao/todo-cli/releases">
        <img src="https://img.shields.io/github/v/release/joaofelipegalvao/todo-cli?style=flat&labelColor=1C2C2E&color=C96329&logo=GitHub&logoColor=white"></a>
    <a href="https://crates.io/crates/todo-cli">
        <img src="https://img.shields.io/crates/v/todo-cli?style=flat&labelColor=1C2C2E&color=C96329&logo=Rust&logoColor=white"></a>
    <br>
    <a href="https://github.com/joaofelipegalvao/todo-cli/actions/workflows/ci.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/joaofelipegalvao/todo-cli/ci.yml?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=GitHub%20Actions&logoColor=BEC5C9&label=ci"></a>
    <a href="https://github.com/joaofelipegalvao/todo-cli/actions/workflows/release.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/joaofelipegalvao/todo-cli/release.yml?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=GitHub%20Actions&logoColor=BEC5C9&label=release"></a>
    <a href="https://opensource.org/licenses/MIT">
        <img src="https://img.shields.io/badge/License-MIT-BEC5C9?style=flat&labelColor=1C2C2E"></a>
</p>

<h3 align="center">
  A fast, powerful, and colorful task manager for the terminal — built with Rust
</h3>

<p align="center">
  <a href="#-installation">Installation</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="GUIDE.md">Documentation</a> •
  <a href="#-contributing">Contributing</a>
</p>

<p align="center">
  <img src="assets/todo-demo.gif" alt="todo-cli demo">
</p>

## 📦 Installation

### Via Cargo

```bash
cargo install todo-cli
```

### From Source

```bash
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli
cargo install --path .
```

### Pre-built Binaries

Download the latest binary from the [Releases](https://github.com/joaofelipegalvao/todo-cli/releases) page:

| Platform | Binary |
|----------|--------|
| Linux (x86_64) | `todo-linux-amd64` |
| macOS (Apple Silicon) | `todo-darwin-arm64` |
| macOS (Intel) | `todo-darwin-amd64` |

## 🚀 Quick Start

```bash
# Add tasks
todo add "Setup database" --project Backend --priority high --due tomorrow
todo add "Write migrations" --project Backend --depends-on 1
todo add "Weekly review" --due "next monday" --recurrence weekly

# View and filter
todo list
todo list --project Backend --status pending --sort due

# Complete and track
todo done 1
todo stats
```

For the full command reference, see [GUIDE.md](GUIDE.md).

## 🤝 Contributing

Contributions are welcome — bug fixes, new features, documentation improvements, and ideas are all appreciated!

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.

```bash
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli
cargo build
cargo test
```

Found a bug? [Open an issue](https://github.com/joaofelipegalvao/todo-cli/issues/new). Have a question? Start a [discussion](https://github.com/joaofelipegalvao/todo-cli/discussions).

## Contributors

Thanks goes to these wonderful people ✨

<a href="https://github.com/joaofelipegalvao/todo-cli/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=joaofelipegalvao/todo-cli" />
</a>

Made with [contrib.rocks](https://contrib.rocks).

## License

Licensed under the [MIT License](./LICENSE).

Copyright © 2026-present, [João Felipe Galvão](https://github.com/joaofelipegalvao)
