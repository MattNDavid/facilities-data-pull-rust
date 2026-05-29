use facilities_data_pull_rust::{db_pool, pull_event_instances, pull_resource_bookings, pull_owners, pull_tag_groups};

#[tokio::main]
async fn main() {
    let start_time = std::time::Instant::now();
    
    dotenv::dotenv().ok();

    let pool = db_pool::create_pool().await.expect("Failed to create database pool");
    
    let ei_pool = pool.clone();
    let ei_handle = tokio::spawn(async move {
        if let Err(e) = pull_event_instances::pull_event_instances(ei_pool.clone()).await {
            eprintln!("Error pulling event instances: {}", e);
        }
    });

    let rb_pool = pool.clone();
    let rb_handle = tokio::spawn(async move { 
        if let Err(e) = pull_resource_bookings::pull_resource_bookings(rb_pool.clone()).await {
            eprintln!("Error pulling resource bookings: {}", e);
        }
    });

    let owners_pool = pool.clone();
    let owners_handle = tokio::spawn(async move {
        if let Err(e) = pull_owners::pull_owners(owners_pool.clone()).await {
            eprintln!("Error pulling owners: {}", e);
        }
    });

    let tag_groups_pool = pool.clone();
    let tag_groups_handle = tokio::spawn(async move {
        if let Err(e) = pull_tag_groups::pull_tag_groups(tag_groups_pool.clone()).await {
            eprintln!("Error pulling tag groups: {}", e);
        }
    });



    let _ = tokio::join!(ei_handle, rb_handle, owners_handle, tag_groups_handle);

    println!("Time to pull and load all data: {:.2}s", start_time.elapsed().as_secs_f64());
}
