# HearthGlow

![HearthGlow Logo](frontend/dist/logo.png)

HearthGlow is a small, local-first app for tracking family star charts, wins, and household goals. This repo contains a Rust backend (axum + SeaORM + SQLite) and a tiny TypeScript-powered static frontend served by the server.

Theme: nostalgic 80's pixel-game — the site and README adopt a retro arcade/hearth vibe.

## Quick start

1. Build and run the server:

```bash
cargo run --release
```

2. Open http://localhost:8080 in a browser. If you want the logo to show up in the site and README, place the `logo.png` file at `frontend/dist/logo.png` (the repository may already include it via assets).

## Code overview

```
Configuration -> Used to bootstrap the application (see `configuration.toml`).
Types -> API request/response types used by the server (see `src/server/types.rs`).
CLI -> Command Line Interface for local control and maintenance.
App -> The primary server logic lives under `src/` and serves the static frontend as well as the HTTP API.
```

## Architecture

```
--------
| Frontend (browser)
--------
     |
     v
--------       -----------
| Axum | ---> | SQLite via SeaORM |
--------       -----------
     |
     v
   Admin / CLI tools
```

If you'd like I can add a small build step to produce optimized frontend assets or a short script to copy the provided `logo.png` into `frontend/dist` automatically.
