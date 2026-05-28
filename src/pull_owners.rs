use crate::request_from_pc;
use sqlx::QueryBuilder;
use serde_json::Value;

struct OwnerRow {
        id: i64,
        first_name: String,
        last_name: String,
        email: String,
}

async fn fetch_owners() -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {

    let (items, _) = request_from_pc::get_pc_data(
        "/groups/v2/groups/828975/people",
        "",
        &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
        &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
    ).await?;
    
    Ok(items)
}

pub async fn pull_owners(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let items = fetch_owners().await?;

    let mut owner_rows: Vec<OwnerRow> = Vec::new();

    for item in items.iter() {
        match item["type"].as_str().unwrap_or_default() {
            "Person" => owner_rows.push(parse_owner_row(item)),
            _ => (),
        }
    }

    for chunk in owner_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO owners (id, first_name, last_name, email) "
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.first_name)
                .push_bind(&row.last_name)
                .push_bind(&row.email);
        });
        qb.push(
            " ON CONFLICT ON CONSTRAINT owners_pkey DO UPDATE SET first_name = EXCLUDED.first_name, last_name = EXCLUDED.last_name, email = EXCLUDED.email",
        );
        qb.build().execute(&pool).await?;
    }
    
    println!("Owners pulled and stored successfully");

    Ok(())
}

fn parse_owner_row(item: &Value) -> OwnerRow {
    OwnerRow {
        id: item["id"].as_str().unwrap_or_default().parse::<i64>().unwrap_or_default(),
        first_name: item["attributes"]["first_name"].as_str().unwrap_or_default().to_string(),
        last_name: item["attributes"]["last_name"].as_str().unwrap_or_default().to_string(),
        email: item["attributes"]["email_addresses"][0]["address"].as_str().unwrap_or_default().to_string(),
    }
}