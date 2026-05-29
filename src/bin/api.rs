use facilities_data_pull_rust::db_pool;
use axum::{
    extract::State,
    routing::get,
    Router,
};
use chrono::{NaiveDate, NaiveTime};
use sqlx::PgPool;

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

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool = db_pool::create_pool().await.unwrap();
    let app: Router = Router::new()
        .route("/schedule", get(get_schedule))
            .with_state(pool.clone())
        .route("/events", get(get_events))
            .with_state(pool.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_schedule(State(pool): State<PgPool>) -> String {
    let rows: Vec<ScheduleRow> = sqlx::query_as("SELECT * FROM facilities_schedule")
        .fetch_all(&pool)
        .await
        .unwrap();
    let res = serde_json::to_string(&rows).unwrap();
    println!("Schedule data: {}", res);
    res
}
async fn get_events(State(pool): State<PgPool>) -> String {
    let rows: Vec<EventRow> = sqlx::query_as("SELECT * FROM event_schedule")
        .fetch_all(&pool)
        .await
        .unwrap();
    let res = serde_json::to_string(&rows).unwrap();
    println!("Event data: {}", res);
    res
}
