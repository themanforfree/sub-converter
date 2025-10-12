use serde::{Deserialize, Serialize};

use super::Node;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Subscription {
    pub nodes: Vec<Node>,
}

impl From<Vec<Node>> for Subscription {
    fn from(nodes: Vec<Node>) -> Self {
        Subscription { nodes }
    }
}

impl From<crate::formats::ClashConfig> for Subscription {
    fn from(cfg: crate::formats::ClashConfig) -> Self {
        let nodes = cfg.proxies.into_iter().map(Into::into).collect();
        Subscription { nodes }
    }
}

impl From<crate::formats::SingBoxConfig> for Subscription {
    fn from(cfg: crate::formats::SingBoxConfig) -> Self {
        let nodes = cfg.outbounds.into_iter().map(Into::into).collect();
        Subscription { nodes }
    }
}
