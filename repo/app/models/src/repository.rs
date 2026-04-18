use sqlx::MySqlPool;

#[derive(Clone)]
pub struct Repositories {
    pub pool: MySqlPool,
}

impl Repositories {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
