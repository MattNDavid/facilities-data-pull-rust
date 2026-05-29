use crate::request_from_pc;
use chrono::{Local, NaiveDate, NaiveTime, Datelike, Duration};
use sqlx::QueryBuilder;

struct EiRow {
    id: i64,
    date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    event_id: i64,
}

struct EventRow {
    id: i64,
    name: String,
    summary: String,
    description: String,
    owner_id: Option<i64>,
}
async fn fetch_event_instances() -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), Box<dyn std::error::Error>> {
    let now = Local::now();
    let today = now.date_naive();
    let start_date = today - Duration::days(today.weekday().num_days_from_sunday() as i64);
    let end_date = start_date + Duration::days(7);


    let (items, includes) = request_from_pc::get_pc_data(
        "/calendar/v2/event_instances",
        &format!("where[starts_at][gte]={}&where[ends_at][lte]={}&include=event,event_times,resource_bookings,tags", start_date, end_date),
        &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
        &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
    ).await?;

    //println!("Fetched {} event instances", items.len());
    //println!("Fetched {} included items", includes.len());
    
    Ok((items, includes))
}

pub async fn pull_event_instances(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let (eis, includes) = fetch_event_instances().await?;

    let mut ei_rows: Vec<EiRow> = Vec::new();
    let mut event_rows: Vec<EventRow> = Vec::new();
    let mut ei_tag_map_rows: Vec<(i64, i64)> = Vec::new();

    for item in eis.iter().chain(includes.iter()) {
        match item["type"].as_str().unwrap_or_default() {
            "EventInstance" => {
                let ei = parse_ei_row(item);
                let tag_rows = parse_ei_tag_map_row(ei.id, item);  // use ei.id first
                ei_rows.push(ei);                                   // then consume
                ei_tag_map_rows.extend(tag_rows);
            },
            "Event" => event_rows.push(parse_event_row(item)),
            _ => (),
        }
    }

    // The same Event can appear in includes once per referencing EventInstance — deduplicate by id
    event_rows.sort_unstable_by_key(|r| r.id);
    event_rows.dedup_by_key(|r| r.id);

    ei_tag_map_rows.sort_unstable();
    ei_tag_map_rows.dedup();

    // Postgres caps bind parameters at 65535; 5 columns * 5000 rows = 25000, safely under the limit
    for chunk in ei_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO event_instances (id, date, start_time, end_time, event_id) ",
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.date)
                .push_bind(&row.start_time)
                .push_bind(&row.end_time)
                .push_bind(row.event_id);
        });
        qb.push(
            " ON CONFLICT (id) DO UPDATE SET \
            date = EXCLUDED.date, start_time = EXCLUDED.start_time, \
            end_time = EXCLUDED.end_time, event_id = EXCLUDED.event_id",
        );
        qb.build().execute(&pool).await?;
    }

    for chunk in event_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO events (id, name, summary, description, owner_id) ",
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.name)
                .push_bind(&row.summary)
                .push_bind(&row.description)
                .push_bind(row.owner_id);
        });
        qb.push(
            " ON CONFLICT (id) DO UPDATE SET \
            name = EXCLUDED.name, summary = EXCLUDED.summary, \
            description = EXCLUDED.description, owner_id = EXCLUDED.owner_id",
        );
        qb.build().execute(&pool).await?;
    }

    for chunk in ei_tag_map_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO event_instance_tag_map (event_instance_id, tag_id) ",
        );
        qb.push_values(chunk, |mut b, (ei_id, tag_id)| {
            b.push_bind(*ei_id)
                .push_bind(*tag_id);
        });
        qb.push(
            " ON CONFLICT ON CONSTRAINT event_instance_tag_map_pkey DO NOTHING",
        );
        qb.build().execute(&pool).await?;
    }
    println!("Event instances pulled and stored successfully");
    Ok(())
}

fn parse_ei_row(item: &serde_json::Value) -> EiRow {
    let id = item["id"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
    let starts_at = item["attributes"]["starts_at"].as_str().unwrap_or_default();
    let ends_at = item["attributes"]["ends_at"].as_str().unwrap_or_default();
    let event_id = item["relationships"]["event"]["data"]["id"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0);

    let date_str = starts_at.split('T').next().unwrap_or_default();
    let start_time_str = starts_at.split('T').nth(1).unwrap_or_default().trim_end_matches('Z');
    let end_time_str = ends_at.split('T').nth(1).unwrap_or_default().trim_end_matches('Z');

    EiRow {
        id,
        date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_default(),
        start_time: NaiveTime::parse_from_str(start_time_str, "%H:%M:%S").unwrap_or_default(),
        end_time: NaiveTime::parse_from_str(end_time_str, "%H:%M:%S").unwrap_or_default(),
        event_id,
    }
}

fn parse_event_row(item: &serde_json::Value) -> EventRow {
    EventRow {
        id: item["id"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0),
        name: item["attributes"]["name"].as_str().unwrap_or_default().to_string(),
        summary: item["attributes"]["summary"].as_str().unwrap_or_default().to_string(),
        description: item["attributes"]["description"].as_str().unwrap_or_default().to_string(),
        owner_id: item["relationships"]["owner"]["data"]["id"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok()),
    }
}

fn parse_ei_tag_map_row(ei_id: i64, item: &serde_json::Value) -> Vec<(i64, i64)> {
    let mut rows = Vec::new();
    if let Some(tags) = item["relationships"]["tags"]["data"].as_array() {
        for tag in tags {
            if let Some(tag_id) = tag["id"].as_str().and_then(|s| s.parse::<i64>().ok()) {
                rows.push((ei_id, tag_id));
            }
        }
    }
    rows
}