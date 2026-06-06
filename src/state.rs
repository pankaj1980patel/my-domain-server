use mongodb::Database;

/// Shared application state injected into every handler.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub jwt_secret: String,
}
