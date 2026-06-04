use facilities_data_pull_rust::db_pool;
use facilities_data_pull_rust::request_from_pc::get_pc_data;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use chrono::{NaiveDate, NaiveTime};
use sqlx::{PgPool, QueryBuilder};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool = db_pool::create_pool().await.unwrap();
    let app: Router = Router::new()
        .route("/schedule", get(get_schedule))
            .with_state(pool.clone())
        .route("/events", get(get_events))
            .with_state(pool.clone())
        .route("/setup", get(get_setup));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_schedule(State(pool): State<PgPool>, Query(params): Query<ScheduleParams>) -> String {
    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new("SELECT * FROM facilities_schedule WHERE 1=1");
    
    if let Some(gte) = params.date_gte {
        qb.push(" AND date >= ").push_bind(gte);
    }
    if let Some(lte) = params.date_lte {
        qb.push(" AND date <= ").push_bind(lte);
    }

    if let Some(event_instance_id) = params.event_instance_id {
        qb.push(" AND event_instance_id = ").push_bind(event_instance_id);
    }

    qb.push(" ORDER BY date, start_time");

    let rows: Vec<ScheduleRow> = qb.build_query_as().fetch_all(&pool).await.unwrap();
    let res = serde_json::to_string(&rows).unwrap();
    println!("Schedule data: {}", res);
    res
}
async fn get_events(State(pool): State<PgPool>, Query(params): Query<EventParams>) -> String {
    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new("SELECT * FROM event_schedule WHERE 1=1");

    if let Some(event_instance_id) = params.event_instance_id {
        qb.push(" AND event_instance_id = ").push_bind(event_instance_id);
    }
    if let Some(gte) = params.date_gte {
        qb.push(" AND date >= ").push_bind(gte);
    }
    if let Some(lte) = params.date_lte {
        qb.push(" AND date <= ").push_bind(lte);
    }

    qb.push(" ORDER BY date, start_time");

    let rows: Vec<EventRow> = qb.build_query_as().fetch_all(&pool).await.unwrap();
    let res = serde_json::to_string(&rows).unwrap();
    println!("Event data: {}", res);
    res
}

async fn get_setup(Query(params): Query<SetupParams>) -> Result<String, (StatusCode, String)> {
    let id = params.room_setup_id.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "room_setup_id is required".to_string())
    })?;

    let (items, _) = get_pc_data(
        &format!("/calendar/v2/room_setups/{}", id),
        "",
        &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
        &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let setup_info = SetupInfo {
        id,
        name: items[0]["attributes"]["name"].as_str().unwrap_or_default().to_string(),
        url: items[0]["attributes"]["diagram_url"].as_str().unwrap_or_default().to_string(),
    };

    let setup_info = serde_json::to_string(&setup_info).unwrap();

    Ok(setup_info)
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ScheduleRow {
    id: i32,
    date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    resource_name: String,
    event_name: String,
    owner_id: Option<i32>,
    notes: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    question: Option<String>,
    answer: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct EventRow {
    event_instance_id: i32,
    event_name: String,
    date: NaiveDate,
    start_time: NaiveTime,
}

#[derive(serde::Deserialize)]
struct ScheduleParams {
    date_gte: Option<NaiveDate>,
    date_lte: Option<NaiveDate>,
    event_instance_id: Option<i32>,
}
#[derive(serde::Deserialize)]
struct EventParams {
    event_instance_id: Option<i32>,
    date_gte: Option<NaiveDate>,
    date_lte: Option<NaiveDate>,
}
#[derive(serde::Deserialize)]
struct SetupParams {
    room_setup_id: Option<i32>,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct SetupInfo {
    id: i32,
    name: String,
    url: String,
}