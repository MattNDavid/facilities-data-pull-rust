# facilities-data-pull-rust

A Rust ETL pipeline that fetches facilities-related calendar data from [Planning Center Online (PCO)](https://www.planningcenteronline.com/) and loads it into a PostgreSQL database.

## What it does

Pulls the following data from the PCO Calendar and Groups APIs and upserts it into Postgres:

| Data | PCO Endpoint | Tables |
|---|---|---|
| Event instances & tags | `/calendar/v2/event_instances` | `event_instances`, `events`, `event_instance_tag_map` |
| Resource bookings | `/calendar/v2/resource_bookings` | `resource_bookings`, `event_resource_requests`, `resources` |
| Form answers | `/calendar/v2/event_resource_requests/{id}/answers` | `answers` |
| Owners | `/groups/v2/groups/828975/people` | `owners` |
| Tag groups & tags | `/calendar/v2/tag_groups` | `tag_groups`, `tags`, `tag_groups_tags_map` |

Each run fetches data for the current month plus 7 days. All inserts use `ON CONFLICT` upsert, so re-runs are safe.

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

3. Create the database tables. No migration tooling is bundled — create the tables to match the schema expected by the upsert queries in `src/pull_*.rs`.

## Running

```
cargo run --release
```

The app spawns four concurrent tasks (event instances, resource bookings, owners, tag groups) and prints elapsed time on completion.

## Architecture

```
main.rs
├── pull_event_instances.rs   — events, event instances, tag associations
├── pull_resource_bookings.rs — bookings, resources, ERRs
│   └── pull_answers.rs       — form answers (20 parallel sub-tasks)
├── pull_owners.rs            — people / owners from a PCO group
└── pull_tag_groups.rs        — tag groups and tags

request_from_pc.rs            — paginated PCO HTTP client (basic auth, 100/page)
db_pool.rs                    — sqlx PgPool with max 5 connections
```

Key design decisions:
- **Pagination** is handled automatically (100 items per page, offset tracking).
- **Batching** splits inserts into chunks of 5000 rows to stay within PostgreSQL's bind-parameter limit.
- **Parallelism** for answers: event resource request IDs are split into 20 concurrent tokio tasks.
- **Retry** logic on answer fetches: sleeps 20 seconds and retries once on API failure.

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `sqlx` | PostgreSQL client |
| `reqwest` | HTTP client for PCO API |
| `serde` / `serde_json` | JSON deserialization |
| `chrono` | Date range calculation |
| `dotenv` | `.env` file loading |
