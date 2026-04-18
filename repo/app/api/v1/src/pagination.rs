use rocket::form::FromForm;

#[derive(Debug, Clone, FromForm)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub filter: Option<String>,
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        let page = self.page.unwrap_or(1).max(1);
        (page - 1) * self.limit()
    }

    pub fn limit(&self) -> u32 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn sort_order_sql(&self) -> &'static str {
        match self.sort_order.as_deref() {
            Some(v) if v.eq_ignore_ascii_case("asc") => "ASC",
            _ => "DESC",
        }
    }
}
