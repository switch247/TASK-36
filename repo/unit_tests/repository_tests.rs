#[cfg(test)]
mod tests {
    use app_models::repository::Repositories;
    use sqlx::mysql::MySqlPoolOptions;

    #[rocket::async_test]
    async fn repositories_wraps_cloneable_pool() {
        let pool = MySqlPoolOptions::new()
            .connect_lazy("mysql://user:pass@localhost:3306/eagle_exam")
            .expect("lazy pool");

        let repos = Repositories::new(pool.clone());
        let cloned = repos.clone();

        assert_eq!(
            repos.pool.connect_options().get_host(),
            cloned.pool.connect_options().get_host()
        );
        assert_eq!(
            repos.pool.connect_options().get_port(),
            cloned.pool.connect_options().get_port()
        );
        assert_eq!(
            repos.pool.connect_options().get_username(),
            cloned.pool.connect_options().get_username()
        );
    }
}
