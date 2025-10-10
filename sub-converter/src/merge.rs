use crate::ir::{InputMeta, Node, Subscription};

pub fn merge_with_meta(groups: Vec<(Vec<Node>, Option<String>)>) -> Subscription {
    let mut nodes = Vec::new();
    let mut meta = Vec::new();
    for (group_nodes, tag) in groups.into_iter() {
        let start = nodes.len();
        nodes.extend(group_nodes);
        let end = nodes.len();
        meta.push(InputMeta {
            tag,
            range_start: start,
            range_end: end,
        });
    }
    Subscription { nodes, meta }
}
