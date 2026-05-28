use crate::request_from_pc;
use sqlx::QueryBuilder;

struct TagGroupRow {
    id: i64,
    name: String,
}

struct TagRow {
    id: i64,
    name: String,
    color: String,
}

async fn fetch_tag_groups() -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), Box<dyn std::error::Error>> {
    let (items, includes) = request_from_pc::get_pc_data(
        "/calendar/v2/tag_groups",
        "&include=tags",
        &std::env::var("PC_USERNAME").expect("PC_USERNAME must be set"),
        &std::env::var("PC_PASSWORD").expect("PC_PASSWORD must be set"),
    ).await?;

    Ok((items, includes))
}

pub async fn pull_tag_groups(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let (tag_groups, includes) = fetch_tag_groups().await?;

    let mut tag_group_rows: Vec<TagGroupRow> = Vec::new();
    let mut tag_rows: Vec<TagRow> = Vec::new();
    let mut tag_group_tag_map_rows: Vec<(i64, i64)> = Vec::new();


    for item in tag_groups.iter().chain(includes.iter()) {
        match item["type"].as_str().unwrap_or_default() {
            "TagGroup" => {
                let tg = parse_tag_group_row(item);
                let map_rows = parse_tag_group_tags_map_row(tg.id, item);
                tag_group_rows.push(tg);
                tag_group_tag_map_rows.extend(map_rows);
            },
            "Tag" => tag_rows.push(parse_tag_row(item)),
            _ => (),
        }
    }

    tag_rows.sort_unstable_by_key(|r| r.id);
    tag_rows.dedup_by_key(|r| r.id);

    for chunk in tag_group_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO tag_groups (id, name) "
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.name);
        });
        qb.push(
            " ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name",
        );
        qb.build().execute(&pool).await?;
    }

    for chunk in tag_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO tags (id, name, color) "
        );
        qb.push_values(chunk, |mut b, row| {
            b.push_bind(row.id)
                .push_bind(&row.name)
                .push_bind(&row.color);
        });
        qb.push(
            " ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, color = EXCLUDED.color",
        );
        qb.build().execute(&pool).await?;
    }
    
    for chunk in tag_group_tag_map_rows.chunks(5000) {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO tag_groups_tags_map (tag_group_id, tag_id) "
        );
        qb.push_values(chunk, |mut b, (tg_id, tag_id)| {
            b.push_bind(*tg_id)
                .push_bind(*tag_id);
        });
        qb.push(
            " ON CONFLICT (tag_group_id, tag_id) DO NOTHING",
        );
        qb.build().execute(&pool).await?;
    }

    println!("Tag groups and tags pulled and stored successfully");
    
    Ok(())
}

fn parse_tag_group_row(item: &serde_json::Value) -> TagGroupRow {
    TagGroupRow {
        id: item["id"].as_str().unwrap_or_default().parse::<i64>().unwrap_or_default(),
        name: item["attributes"]["name"].as_str().unwrap_or_default().to_string(),
    }
}

fn parse_tag_row(item: &serde_json::Value) -> TagRow {
    TagRow {
        id: item["id"].as_str().unwrap_or_default().parse::<i64>().unwrap_or_default(),
        name: item["attributes"]["name"].as_str().unwrap_or_default().to_string(),
        color: item["attributes"]["color"].as_str().unwrap_or_default().to_string(),
    }
}

fn parse_tag_group_tags_map_row(tg_id: i64, item: &serde_json::Value) -> Vec<(i64, i64)> {
    let mut rows = Vec::new();
    if let Some(tags) = item["relationships"]["tags"]["data"].as_array() {
        for tag in tags {
            if let Some(tag_id) = tag["id"].as_str().and_then(|s| s.parse::<i64>().ok()) {
                rows.push((tg_id, tag_id));
            }
        }
    }
    rows
}