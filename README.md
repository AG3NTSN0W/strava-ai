# StravAI Oxide

A self-hosted Rust web application that automatically generates AI-powered titles and descriptions for your Strava activities using Ollama (Llama 3.2).

## Features

- **Strava OAuth Integration** — Connect your Strava account and import activities automatically.
- **AI-Generated Summaries** — Uses a local Ollama instance with Llama 3.2 to generate activity titles and descriptions based on activity data (distance, time, elevation, heart rate, etc.).
- **Background Scheduler** — Periodically fetches new activities and generates summaries on a configurable interval (default: 5 hours).
- **Web Dashboard** — View and edit your activities, configure AI prompts, and toggle auto-update per athlete.
- **SQLite Storage** — Lightweight local database for athletes and activities.
- **Docker Ready** — Includes Docker Compose setup with Ollama sidecar and automatic model pulling.

## Tech Stack

- **Rust** (2024 edition) with **Axum** web framework
- **Askama** HTML templates
- **SQLx** with SQLite
- **Moka** in-memory cache for access tokens
- **Ollama** (Llama 3.2) for AI generation
- **Reqwest** for Strava & Ollama API calls

## Prerequisites

- A [Strava API application](https://www.strava.com/settings/api) with `STRAVA_CLIENT_ID` and `STRAVA_CLIENT_SECRET`
- Docker & Docker Compose (for containerized setup), or Rust toolchain for local development
- ~16GB RAM recommended for Ollama with Llama 3.2

## Configuration

Copy the example environment file and fill in your values:

```bash
cp .env.example .env
```

| Variable | Description | Default |
|---|---|---|
| `STRAVA_CLIENT_ID` | Your Strava app client ID | *required* |
| `STRAVA_CLIENT_SECRET` | Your Strava app client secret | *required* |
| `OLLAMA_URL` | Ollama API endpoint | `http://localhost:11434/api/generate` |
| `STRAVA_INTERVAL` | Hours between scheduled activity fetches | `5` |
| `LOGGING` | Log level (`debug`, `info`, `warn`, `error`) | `info` |

## Running

### Docker Compose (recommended)

```bash
docker compose up -d
```

This starts three services:
- **strava-ai** — The application on port `3400`
- **ollama** — Local LLM server on port `11434`
- **pull-ollama-model** — One-shot container that pulls the Llama 3.2 model

### Local Development

```bash
cargo run
```

The server starts on `http://localhost:3400`.

## Usage

1. Open `http://localhost:3400` in your browser.
2. Connect your Strava account via OAuth.
3. Your activities are imported automatically.
4. Enable **auto-update** in settings to have the AI generate titles and descriptions for new activities.
5. Customize the AI prompt per athlete from the settings page.

## Project Structure

```
src/
├── main.rs                  # Entry point, server setup
├── lib.rs                   # AppState, token caching
├── controllers/             # Route handlers & HTML templates
└── libs/
    ├── strava_client.rs     # Strava API client
    ├── ollama_client.rs     # Ollama API client
    ├── scheduler.rs         # Background task scheduler
    ├── models/              # Data models (Activity, Athlete, Ollama)
    └── repository/          # SQLite repositories
templates/                   # Askama HTML templates
assets/                      # CSS, JS, SVG assets
```
