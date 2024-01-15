use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;

mod file;
mod upload;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Du hast.");

    let app = Router::new()
        .route("/upload", post(upload::upload))
        .layer(DefaultBodyLimit::max(8000000))
        .route("/uploads/:id", get(file::get_file))
        .with_state(Arc::new(InternalAppState::new().await));
    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .expect("Cannot bind to ip/port.");
    axum::serve(listener, app)
        .await
        .expect("Cannot serve webserver.");
}
pub struct InternalAppState {
    pool: DatabaseConnection,
}

pub type AppState = Arc<InternalAppState>;

impl InternalAppState {
    pub async fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("d");

        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(100)
            .min_connections(3)
            .connect_timeout(Duration::from_secs(3))
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(0))
            .max_lifetime(Duration::from_secs(120));

        let pool = Database::connect(opt).await.expect("d");

        Self { pool }
    }

    pub fn get_pool(&self) -> &DatabaseConnection {
        &self.pool
    }
}
