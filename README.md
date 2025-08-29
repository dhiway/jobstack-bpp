# bpp-onest-lite

A Beckn Provider Platform (BPP) backend service that communicates with the Beckn network.

## Features

- REST API for search, on_search,select, on_select ..etc
- Integrates with Beckn protocol
- Uses Axum, SQLx, Redis, and async Rust ecosystem

## Getting Started

### Prerequisites

- Rust (1.77+)
- PostgreSQL & Redis
- Cargo

### Build

```sh
cargo build --release
```

### Run

```sh
cargo run -- config/local.yaml
```

Replace `config/local.yaml` with your configuration file as needed.

## Docker

Build and run the container manually:

```sh
docker build -t bpp-onest-lite .
docker run -p 3009:3009 bpp-onest-lite ./bpp-onest-lite config/local.yaml
```

Or use Docker Compose for multi-service setup:

```sh
docker compose build --no-cache
docker compose up -d
```

**Note:**  
The application is started in the container with:

```yaml
command: ["./bpp-onest-lite", "config/local.yaml"]
```

You can update the config path as needed.

## Configuration

Configuration is loaded from the path you provide as the first argument.