<div align="center">
<hr>
<img src="https://img.shields.io/badge/Made_with_♥️_in-Brazil-ED186A?style=for-the-badge"><br>
<a href="https://discord.gg/q8HzGuHuDY">
  <img src="https://img.shields.io/discord/990369410344701964?color=333&label=Support&logo=discord&style=for-the-badge" alt="Discord">
</a>
<a href="https://github.com/Ecliptia/Aelira/releases">
  <img alt="GitHub Release" src="https://img.shields.io/github/v/release/Ecliptia/Aelira?style=for-the-badge&logo=github&color=333">
</a>
<br>
<a href="https://github.com/Ecliptia/Aelira">
  <img alt="GitHub forks" src="https://img.shields.io/github/forks/Ecliptia/Aelira?style=for-the-badge&logo=github&color=333">
</a>
<a href="https://github.com/Ecliptia/Aelira">
  <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/Ecliptia/Aelira?style=for-the-badge&logo=github&color=333">
</a>
<a href="https://github.com/sponsors/1lucas1apk">
  <img alt="GitHub Sponsors" src="https://img.shields.io/github/sponsors/1lucas1apk?style=for-the-badge&logo=github&color=333">
</a>
<br>
<h3>Aelira — Does the wind ever stop? 🎐</h3>
</div>
<hr>

## Table of Contents

- [The Project](#the-project)
- [Installation](#installation)
- [Configuration](#configuration)
- [How it Works](#how-it-works)
- [License](#license)
- [Support](#support)
- [Special Thanks](#special-thanks)

## The Project

**Aelira** is an exploration of the Rust ecosystem. The name, derived from roots meaning "air" or "wind", reflects the goal of creating something that flows effortlessly through the system.

This project was started as a dedicated path to master **Rust**. It's a journey into memory safety, strict typing, and high-level orchestration. While many environments allow for breaking performance barriers through native bindings, Aelira chooses Rust to handle the complex state management and safety, while leveraging the raw speed of **C** for audio processing.

## Installation

```bash
# Clone the repository
git clone https://github.com/Ecliptia/Aelira.git

# Enter the workshop
cd Aelira

# Build the project
cargo build --release

# Run
./target/release/aelira
```

## Configuration

Aelira uses a simple `config.toml` file.

```toml
[server]
host = "0.0.0.0"
port = 3030
password = "youshallnotpass"

[cluster]
workers = 0 # 0 = Auto-detect
```

## How it Works

- **Orchestration (Rust):** Handles sessions, players, API routes, and logic safety.

## License

This project is licensed under the [Open Software License ("OSL") v. 3.0](LICENSE) – see the [LICENSE](LICENSE) file for details.

## Support

If you have questions or just want to talk about development, join our Discord: [Ecliptia Discord](https://discord.gg/q8HzGuHuDY).

## Special Thanks

Thanks to everyone who supports my work and the Ecliptia organization. Every contributor and sponsor helps keep these projects alive.

Have a great day! :)
