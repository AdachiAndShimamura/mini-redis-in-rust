# MiniRedis-Rust

MiniRedis-Rust is a simple Redis server implemented in Rust, inspired by Tokio's mini-redis project. It aims to help learn Rust, asynchronous programming, and building simple distributed systems.

## Features

- **Asynchronous Programming**: Powered by the Tokio framework, MiniRedis-Rust supports asynchronous IO operations, providing high-performance concurrent processing.
- **Simple and Understandable**: The code structure is straightforward, well-commented, suitable for Rust beginners learning asynchronous and network programming.
- **Basic Redis Commands Support**: MiniRedis-Rust implements some basic Redis commands such as SET, GET, DEL, etc.

## Quick Start

1. Clone the repository:

    ```bash
    git clone https://github.com/your-username/mini-redis-rust.git
    cd mini-redis-rust
    ```

2. Build and run:

    ```bash
    cargo build
    cargo run src/bin/server
   cargo run src/bin/client
    ```

3. It listens on `127.0.0.1:6379` by default. Connect using a Redis client.

## Supported Redis Commands

- **SET key value**: Set the value of key.
- **GET key**: Get the value of key.
- **DEL key**: Delete key and its associated value.

## Configuration

MiniRedis-Rust uses default configuration, listening on `127.0.0.1:6379`. You can configure the listening address and port by modifying the `config.toml` file.

