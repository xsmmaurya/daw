// /Users/xsm/Documents/workspace/xtras/daw/src/utils/pagination.rs
use actix_web::{HttpRequest, HttpResponseBuilder};

/// Extracts pagination params from headers
pub fn get_pagination_params(req: &HttpRequest) -> (i64, i64, i64) {
    let page = req.headers()
        .get("X-Requested-Page")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1); // ✅ Default: Page 1

    let limit = req.headers()
        .get("X-Requested-Limit")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10); // ✅ Default: Limit 10

    let skip = (page - 1) * limit;
    (page, limit, skip)
}

/// Sets pagination headers in response
pub fn set_pagination_headers(
    response: &mut actix_web::HttpResponseBuilder, 
    total: i64, 
    total_pages: i64, 
    page: i64, 
    limit: i64
) {
    response.insert_header(("X-Total-Count", total.to_string()));
    response.insert_header(("X-Total-Pages", total_pages.to_string()));
    response.insert_header(("X-Current-Page", page.to_string()));
    response.insert_header(("X-Limit", limit.to_string()));
    response.insert_header(("X-Requested-Page", page.to_string()));
    response.insert_header(("X-Requested-Limit", limit.to_string()));
}
