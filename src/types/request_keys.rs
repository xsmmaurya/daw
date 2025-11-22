// /Users/xsm/Documents/workspace/xtras/daw/src/types/request_keys.rs
/// Store current user id in `HttpRequest` extensions
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct CurrentUserId(pub Uuid);
