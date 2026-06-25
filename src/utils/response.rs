use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
    pub total: u64,
    pub pages: u64,
    pub page: u64,
    pub page_size: u64,
}

impl Pagination {
    pub fn new(total: u64, page: u64, page_size: u64) -> Self {
        let pages = if page_size > 0 {
            (total + page_size - 1) / page_size
        } else {
            0
        };
        Self {
            total,
            pages,
            page,
            page_size,
        }
    }
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            pagination: None,
        }
    }

    pub fn with_pagination(data: T, pagination: Pagination) -> Self {
        Self {
            data,
            pagination: Some(pagination),
        }
    }
}
