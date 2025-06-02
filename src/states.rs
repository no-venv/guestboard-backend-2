use crate::Mutex;
use crate::Arc;
use crate::database;
pub struct MainState {
    pub db: Arc<Mutex<database::Database>>,
}
pub struct ApiKeyState {
    pub api_key: String,
}
