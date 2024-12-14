use serde::{Deserialize, Serialize};
use crate::priority::Priority;

#[derive(Clone, Deserialize, Serialize)]
pub struct Task {
    pub description: String,
    pub priority: Priority,
}
