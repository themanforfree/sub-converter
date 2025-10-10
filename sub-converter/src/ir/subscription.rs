use serde::{Deserialize, Serialize};

use super::Node;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputMeta {
    pub tag: Option<String>,
    pub range_start: usize,
    pub range_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Subscription {
    pub nodes: Vec<Node>,
    pub meta: Vec<InputMeta>,
}
