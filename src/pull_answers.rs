use crate::request_from_pc;
use crate::pull_resource_bookings;
use sqlx::QueryBuilder;

struct AnswerRow {
    id: i64,
    question: String,
    answer: String,
    event_resource_request_id: i64,
}

async fn fetch_answers(event_resource_request_id: i64) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    loop {
        match request_from_pc::get_pc_data(
            &format!("/calendar/v2/event_resource_requests/{}/answers", event_resource_request_id),
            "",
            &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
            &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
        ).await {
            Ok((items, _)) => return Ok(items),
            Err(_e) => {
                //This is almost always an API rate limit issue. Wait 20 seconds and try again.
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            }
        }
    }
}

pub async fn pull_answers(pool: sqlx::PgPool, event_rr_rows: Vec<pull_resource_bookings::EventRrRow>) -> Result<(), Box<dyn std::error::Error>> {
    let total = event_rr_rows.len();

    if total == 0 {
        return Ok(());
    }

    let chunk_size = (total + 19) / 20;
    let chunks: Vec<Vec<pull_resource_bookings::EventRrRow>> = event_rr_rows
        .chunks(chunk_size)
        .map(|c| c.to_vec())
        .collect();

    let mut handles = Vec::new();

    for chunk in chunks {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            let mut all_answer_rows: Vec<AnswerRow> = Vec::new();
            for rr in &chunk {
                let answers = fetch_answers(rr.id).await.unwrap_or_default();
                let mut answer_rows = parse_answer_rows(answers);
                all_answer_rows.append(&mut answer_rows);
            }
            if !all_answer_rows.is_empty() {
                let mut qb = QueryBuilder::new("INSERT INTO answers (id, question, answer, event_resource_request_id) ");
                qb.push_values(all_answer_rows.iter(), |mut b, row| {
                    b.push_bind(row.id)
                        .push_bind(&row.question)
                        .push_bind(&row.answer)
                        .push_bind(row.event_resource_request_id);
                });
                qb.push(" ON CONFLICT (id) DO UPDATE SET question = EXCLUDED.question, answer = EXCLUDED.answer, event_resource_request_id = EXCLUDED.event_resource_request_id");

                if let Err(e) = qb.build().execute(&pool).await {
                    eprintln!("Error inserting answers: {}", e);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

fn parse_answer_rows(items: Vec<serde_json::Value>) -> Vec<AnswerRow> {

    if items.len() == 0 {
        return Vec::new();
    }

    let mut answer_rows = Vec::new();
    for item in items {
        let answer_row = AnswerRow {
            id: item["id"].as_str().unwrap_or_default().parse().unwrap_or_default(),
            question: item["attributes"]["question"]["question"].as_str().unwrap_or_default().to_string(),
            answer: item["attributes"]["answer"].as_str().unwrap_or_default().to_string(),
            event_resource_request_id: item["relationships"]["event_resource_request"]["data"]["id"]
                .as_str()
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        };
        answer_rows.push(answer_row);
    }
    answer_rows
}
