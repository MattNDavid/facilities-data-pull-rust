use crate::request_from_pc;
use crate::pull_answers;
use chrono::{Local, NaiveDate, NaiveTime, Datelike, Duration};
use sqlx::QueryBuilder;


struct RbRow {
    id: i64,
    date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    event_id: i64,
    event_resource_request_id: Option<i64>,
    event_instance_id: Option<i64>,
    resource_id: i64,
}
struct ResourceRow {
    id: i64,
    name: String,
    kind: bool,
}
#[derive(Clone)]
pub struct Event_rrRow {
    pub id: i64,
    notes: String,
    event_id: i64,
    resource_id: i64,
    room_setup_id: Option<i64>,
    quantity: i32,
}
async fn fetch_resource_bookings() -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), Box<dyn std::error::Error>> {
    let now = Local::now();
    let start_date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
    let end_date = start_date + Duration::days(7);
    
    let (items, includes) = request_from_pc::get_pc_data(
        "/calendar/v2/resource_bookings",
        &format!(
            "where[starts_at][gte]={}&where[ends_at][lte]={}&include=event_resource_request,resource",
            start_date,
            end_date
        ),
        &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
        &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
    ).await?;

    Ok((items, includes))
}
pub async fn pull_resource_bookings(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let (rbs, includes) = fetch_resource_bookings().await?;
    
    let mut event_rr_rows: Vec<Event_rrRow> = Vec::new();
    let mut resource_rows: Vec<ResourceRow> = Vec::new();
    let mut rb_rows: Vec<RbRow> = Vec::new();

    for item in rbs.iter().chain(includes.iter()) {
        match item["type"].as_str().unwrap_or_default() {
            "ResourceBooking" => rb_rows.push(parse_rb_row(item).await),
            "EventResourceRequest" => event_rr_rows.push(parse_event_rr_row(item).await),
            "Resource" => resource_rows.push(parse_resource_row(item).await),
            _ => (),
        }
    }

    resource_rows.sort_unstable_by_key(|r| r.id);
    resource_rows.dedup_by_key(|r| r.id);
    event_rr_rows.sort_unstable_by_key(|r| r.id);
    event_rr_rows.dedup_by_key(|r| r.id);

    let event_rr_rows_copy = event_rr_rows.clone(); // clone for use in async block
    let pool_copy = pool.clone();

    let handle = tokio::spawn(async move {
        pull_answers::pull_answers(pool_copy, event_rr_rows_copy).await
            .unwrap_or_else(|e| eprintln!("Error pulling answers: {}", e));
    });

    for chunk in rb_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO resource_bookings (id, date, start_time, end_time, event_id, event_resource_request_id, event_instance_id, resource_id) ",
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.date)
                .push_bind(&row.start_time)
                .push_bind(&row.end_time)
                .push_bind(row.event_id)
                .push_bind(row.event_resource_request_id)
                .push_bind(row.event_instance_id)
                .push_bind(row.resource_id);
        });
        qb.push(" ON CONFLICT (id) DO UPDATE SET date = EXCLUDED.date, start_time = EXCLUDED.start_time, end_time = EXCLUDED.end_time, event_id = EXCLUDED.event_id, event_resource_request_id = EXCLUDED.event_resource_request_id, event_instance_id = EXCLUDED.event_instance_id, resource_id = EXCLUDED.resource_id");
        qb.build().execute(&pool).await?;
    }

    for chunk in event_rr_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO event_resource_requests (id, notes, event_id, resource_id, room_setup_id, quantity) ",
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.notes)
                .push_bind(row.event_id)
                .push_bind(row.resource_id)
                .push_bind(row.room_setup_id)
                .push_bind(row.quantity);
        });
        qb.push(" ON CONFLICT (id) DO UPDATE SET notes = EXCLUDED.notes, event_id = EXCLUDED.event_id, resource_id = EXCLUDED.resource_id, room_setup_id = EXCLUDED.room_setup_id, quantity = EXCLUDED.quantity");
        qb.build().execute(&pool).await?;
    }

    for chunk in resource_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO resources (id, name, room) ",
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.name)
                .push_bind(&row.kind);
        });
        qb.push(" ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, room = EXCLUDED.room");
        qb.build().execute(&pool).await?;
    }

    tokio::join!(handle).0?;
    println!("Resource bookings pulled and stored successfully");
    Ok(())
}

async fn parse_rb_row(item: &serde_json::Value) -> RbRow {
    let id = item["id"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0);
    let starts_at = item["attributes"]["starts_at"].as_str().unwrap_or_default();
    let ends_at = item["attributes"]["ends_at"].as_str().unwrap_or_default();
    let event_id = item["relationships"]["event"]["data"]["id"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0);
    let event_resource_request_id = item["relationships"]["event_resource_request"]["data"]["id"]
        .as_str()
        .map(|s| s.parse::<i64>().unwrap_or(0));
    let event_instance_id = item["relationships"]["event_instance"]["data"]["id"]
        .as_str()
        .map(|s| s.parse::<i64>().unwrap_or(0));
    let resource_id = item["relationships"]["resource"]["data"]["id"]
        .as_str()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0);
    
    let date_str = starts_at.split('T').next().unwrap_or_default();
    let start_time_str = starts_at.split('T').nth(1).unwrap_or_default().trim_end_matches('Z');
    let end_time_str = ends_at.split('T').nth(1).unwrap_or_default().trim_end_matches('Z');
    
    RbRow {
        id,
        date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_default(),
        start_time: NaiveTime::parse_from_str(start_time_str, "%H:%M:%S").unwrap_or_default(),
        end_time: NaiveTime::parse_from_str(end_time_str, "%H:%M:%S").unwrap_or_default(),
        event_id,
        event_resource_request_id,
        event_instance_id,
        resource_id,
    }
}
async fn parse_event_rr_row(item: &serde_json::Value) -> Event_rrRow {
    Event_rrRow {
        id: item["id"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0),
        notes: item["attributes"]["notes"].as_str().unwrap_or_default().to_string(),
        event_id: item["relationships"]["event"]["data"]["id"]
            .as_str()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0),
        resource_id: item["relationships"]["resource"]["data"]["id"]
            .as_str()
            .unwrap_or("0")
            .parse::<i64>()
            .unwrap_or(0),
        room_setup_id: item["relationships"]["room_setup"]["data"]["id"]
            .as_str()
            .map(|s| s.parse::<i64>().unwrap_or(0)),
        quantity: item["attributes"]["quantity"].as_i64().unwrap_or(0) as i32,
    }
}
async fn parse_resource_row(item: &serde_json::Value) -> ResourceRow {
    let kind = match item["attributes"]["kind"].as_str().unwrap_or_default() {
        "Room" => true,
        "Resource" => false,
        _ => false, 
    };
    ResourceRow {
        id: item["id"].as_str().unwrap_or("0").parse::<i64>().unwrap_or(0),
        name: item["attributes"]["name"].as_str().unwrap_or_default().to_string(),
        kind,
    }
}