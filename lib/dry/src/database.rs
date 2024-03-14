use errors::TicketsResult;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub async fn connect() -> TicketsResult<Pool<Postgres>> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    Ok(pool)
}
