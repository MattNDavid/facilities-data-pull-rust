# facilities-data-pull-rust

A Rust ETL pipeline that syncs facilities calendar data from [Planning Center Online (PCO)](https://www.planningcenteronline.com/) into PostgreSQL, plus a JSON API and React frontend for viewing schedules.

## What it does

### Sync (`cargo run --bin sync`)

Pulls the following data from the PCO Calendar and Groups APIs and upserts it into Postgres:

| Data | PCO Endpoint | Tables |
|---|---|---|
| Event instances & tags | `/calendar/v2/event_instances` | `event_instances`, `events`, `event_instance_tag_map` |
| Resource bookings | `/calendar/v2/resource_bookings` | `resource_bookings`, `event_resource_requests`, `resources` |
| Form answers | `/calendar/v2/event_resource_requests/{id}/answers` | `answers` |
| Owners | `/groups/v2/groups/828975/people` | `owners` |
| Tag groups & tags | `/calendar/v2/tag_groups` | `tag_groups`, `tags`, `tag_groups_tags_map` |

Each run fetches data for the current day plus 7 days. All inserts use `ON CONFLICT` upsert, so re-runs are safe.

### API server (`cargo run --bin api`)

An Axum HTTP server on `127.0.0.1:3000` with three endpoints:

| Endpoint | Query params | Description |
|---|---|---|
| `GET /schedule` | `date_gte`, `date_lte`, `event_instance_id`, `event_name` | Facilities schedule with owner and setup info |
| `GET /events` | `date_gte`, `date_lte`, `event_instance_id` | Event list |
| `GET /setup` | `room_setup_id` (required) | Room setup name and diagram URL from PCO |

### Frontend

A React + Vite app (in `frontend/`) that displays the event list and room setup diagrams. Route is determined by the `?setup=<id>` query param — with it shows a setup diagram viewer, without it shows the events list.

## Prerequisites

- Rust (2024 edition)
- PostgreSQL database
- Planning Center Online account with a Personal Access Token

## Setup

1. Clone the repo and copy the environment file:

   ```
   cp .env.example .env
   ```

2. Fill in `.env`:

   ```env
   PC_USERNAME=<your PCO app ID>
   PC_PASSWORD=<your PCO secret / personal access token>
   DATABASE_URL=postgresql://<user>:<password>@<host>/<dbname>
   ```

3. Create the database tables to match the schema expected by the upsert queries in the `pull_*.rs` modules. No migration tooling is bundled.

## Running

**Sync data from PCO:**
```
cargo run --release --bin sync
```

**Start the API server:**
```
cargo run --release --bin api
```

**Start the frontend dev server:**
```
cd frontend
npm install
npm run dev
```

## Architecture

```
src/
├── bin/
│   ├── sync.rs               — CLI entry point; runs four concurrent sync tasks
│   └── api.rs                — Axum API server
├── pull_event_instances.rs   — events, event instances, tag associations
├── pull_resource_bookings.rs — bookings, resources, event resource requests
├── pull_answers.rs           — form answers (20 parallel sub-tasks)
├── pull_owners.rs            — people / owners from a PCO group
├── pull_tag_groups.rs        — tag groups and tags
├── request_from_pc.rs        — paginated PCO HTTP client (basic auth, 100/page)
└── db_pool.rs                — sqlx PgPool with max 5 connections

frontend/src/
├── App.jsx                   — route switcher (events list vs. setup viewer)
├── EventsList.jsx            — event schedule view
└── SetupViewer.jsx           — room setup diagram viewer
```

Key design decisions:
- **Pagination** is handled automatically (100 items per page, offset tracking).
- **Batching** splits inserts into chunks of 5000 rows to stay within PostgreSQL's bind-parameter limit.
- **Parallelism** for answers: event resource request IDs are split into 20 concurrent tokio tasks.
- **Retry** PCO API rate limit is 100 requests per 20 seconds. Answer fetches sleep 20 seconds and retry once on API failure.

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `axum` | HTTP API server |
| `sqlx` | PostgreSQL client |
| `reqwest` | HTTP client for PCO API |
| `serde` / `serde_json` | JSON deserialization |
| `chrono` | Date range calculation |
| `dotenv` | `.env` file loading |
