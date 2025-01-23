use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::node::{NodeRepr, ShallowNodeRepr};
use serde::ser::{Serialize, SerializeTuple, Serializer};
// use rpds::RedBlackTreeMap;

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    Create(i32, String),
    AppendChild(i32, i32, u32),
    SetProperty(i32, String, serde_json::Value),
    ActivateRoots(Vec<i32>),
    Commit,
}

impl PartialOrd for Instruction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Instruction {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Instruction::Create(_, _), Instruction::Create(_, _)) => Ordering::Equal,
            (Instruction::Create(_, _), _) => Ordering::Less,
            (_, _) => Ordering::Equal,
        }
    }
}

impl Serialize for Instruction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Create(hash, kind) => {
                let mut tup = serializer.serialize_tuple(3)?;
                let tag: i32 = 0;
                tup.serialize_element(&tag)?;
                tup.serialize_element(hash)?;
                tup.serialize_element(kind)?;
                tup.end()
            }
            Self::AppendChild(parent_hash, child_hash, output_channel) => {
                let mut tup = serializer.serialize_tuple(4)?;
                let tag: i32 = 2;
                tup.serialize_element(&tag)?;
                tup.serialize_element(parent_hash)?;
                tup.serialize_element(child_hash)?;
                tup.serialize_element(output_channel)?;
                tup.end()
            }
            Self::SetProperty(hash, key, value) => {
                let mut tup = serializer.serialize_tuple(4)?;
                let tag: i32 = 3;
                tup.serialize_element(&tag)?;
                tup.serialize_element(hash)?;
                tup.serialize_element(key)?;
                tup.serialize_element(value)?;
                tup.end()
            }
            Self::ActivateRoots(roots) => {
                let mut tup = serializer.serialize_tuple(2)?;
                let tag: i32 = 4;
                tup.serialize_element(&tag)?;
                tup.serialize_element(roots)?;
                tup.end()
            }
            Self::Commit => {
                let mut tup = serializer.serialize_tuple(1)?;
                let tag: i32 = 5;
                tup.serialize_element(&tag)?;
                tup.end()
            }
        }
    }
}

pub fn reconcile(
    node_map: &mut BTreeMap<i32, ShallowNodeRepr>,
    roots: &Vec<NodeRepr>,
) -> Vec<Instruction> {
    let mut visited: HashSet<i32> = HashSet::new();
    let mut queue: VecDeque<&NodeRepr> = VecDeque::from_iter(roots.iter());
    let mut instructions: Vec<Instruction> = Vec::new();

    loop {
        match queue.pop_front() {
            None => break,
            Some(next) => {
                if visited.contains(&next.hash) {
                    continue;
                }

                // Mount
                if !node_map.contains_key(&next.hash) {
                    // Create node
                    instructions.push(Instruction::Create(next.hash, next.kind.clone()));

                    // Append child
                    for child in next.children.iter() {
                        instructions.push(Instruction::AppendChild(
                            next.hash,
                            child.hash,
                            child.output_channel,
                        ));
                    }

                    node_map.insert(next.hash, next.into());
                }

                // Props
                for (name, value) in &next.props {
                    // TODO: Only add the instruction if the prop value != existing prop value
                    instructions.push(Instruction::SetProperty(
                        next.hash,
                        name.clone(),
                        value.clone(),
                    ));
                }

                // Visit children
                for child in next.children.iter() {
                    queue.push_back(child);
                }

                visited.insert(next.hash);
            }
        }
    }

    // Activate roots
    instructions.push(Instruction::ActivateRoots(
        roots.iter().map(|n| n.hash).collect::<Vec<i32>>(),
    ));

    // Commit
    instructions.push(Instruction::Commit);

    // Sort so that creates land before appends, etc
    instructions.sort();
    instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::prelude::*;
    use serde_json::json;
    use serde_json::Value::Null;

    #[test]
    fn it_works() {
        let graph = root(phasor(constant!({key: None, value: 110.0})));
        let mut node_map = BTreeMap::new();
        let roots = vec![graph];
        let instructions = reconcile(&mut node_map, &roots);

        assert_eq!(
            instructions,
            vec![
                Instruction::Create(186452590, "root".to_string()),
                Instruction::Create(-173258319, "phasor".to_string()),
                Instruction::Create(-93964182, "const".to_string()),
                Instruction::AppendChild(186452590, -173258319, 0),
                Instruction::SetProperty(186452590, "channel".to_string(), json!(0.0)),
                Instruction::AppendChild(-173258319, -93964182, 0),
                Instruction::SetProperty(-93964182, "key".to_string(), Null),
                Instruction::SetProperty(-93964182, "value".to_string(), json!(110.0)),
                Instruction::ActivateRoots(vec![186452590]),
                Instruction::Commit,
            ]
        );
    }
}
