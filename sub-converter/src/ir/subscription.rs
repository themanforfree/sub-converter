use serde::{Deserialize, Serialize};

use super::Node;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Subscription {
    pub nodes: Vec<Node>,
}
