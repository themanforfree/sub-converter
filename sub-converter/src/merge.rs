use crate::ir::{Node, Subscription};

pub fn merge_subscriptions(groups: Vec<Vec<Node>>) -> Subscription {
    let mut nodes = Vec::new();
    for group_nodes in groups.into_iter() {
        nodes.extend(group_nodes);
    }
    Subscription { nodes }
}
