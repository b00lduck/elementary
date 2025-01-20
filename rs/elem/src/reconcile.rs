use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::node::{NodeRepr, ShallowNodeRepr};
use serde_json::json;
// use rpds::RedBlackTreeMap;

pub fn reconcile(
    node_map: &mut BTreeMap<i32, ShallowNodeRepr>,
    roots: &Vec<NodeRepr>,
) -> serde_json::Value {
    let mut visited: HashSet<i32> = HashSet::new();
    let mut queue: VecDeque<&NodeRepr> = VecDeque::new();
    let mut instructions = serde_json::Value::Array(vec![]);

    for root in roots.iter() {
        // TODO: ref?
        queue.push_back(root);
    }

    while !queue.is_empty() {
        let next = queue.pop_front().unwrap();

        if visited.contains(&next.hash) {
            continue;
        }

        // Mount
        node_map.entry(next.hash).or_insert_with(|| {
            // Create node
            instructions
                .as_array_mut()
                .unwrap()
                .push(json!([0, next.hash, next.kind]));

            // Append child
            for child in next.children.iter() {
                instructions.as_array_mut().unwrap().push(json!([
                    2,
                    next.hash,
                    child.hash,
                    child.output_channel
                ]));
            }

            next.into()
        });

        // Props
        for (name, value) in &next.props {
            // TODO: Only add the instruction if the prop value != existing prop value
            instructions
                .as_array_mut()
                .unwrap()
                .push(json!([3, next.hash, name, value]));
        }

        for child in next.children.iter() {
            queue.push_back(child);
        }

        visited.insert(next.hash);
    }

    // Activate roots
    instructions.as_array_mut().unwrap().push(json!([
        4,
        roots.iter().map(|n| n.hash).collect::<Vec<i32>>()
    ]));

    // Commit
    instructions.as_array_mut().unwrap().push(json!([5]));

    // Sort so that creates land before appends, etc
    instructions
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a[0].as_i64().cmp(&b[0].as_i64()));

    instructions
}
