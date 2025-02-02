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
                let existing = node_map.entry(next.hash).or_insert_with(|| {
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

                    next.into()
                });

                // Props
                for (name, value) in &next.props {
                    let do_write = match existing.props.get(name) {
                        Some(v) => v != value,
                        None => true,
                    };

                    if do_write {
                        existing
                            .props
                            .entry(name)
                            .and_modify(|e| *e = value.clone())
                            .or_insert(value.clone());

                        instructions.push(Instruction::SetProperty(
                            next.hash,
                            name.clone(),
                            value.clone(),
                        ));
                    }
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

    #[test]
    fn basic() {
        let mut node_map = BTreeMap::new();
        let graph = vec![root(phasor(constant!({key: None, value: 110.0})))];
        let instructions = reconcile(&mut node_map, &graph);

        insta::assert_json_snapshot!(instructions);
    }

    #[test]
    fn distinguish_by_props() {
        let voice =
            |path| sample!({key: None, path: path}, train(constant!({key: None, value: 1.0})));
        let mut node_map = BTreeMap::new();
        let graph = vec![
            voice(String::from("test.wav")),
            voice(String::from("test2.wav")),
        ];
        let instructions = reconcile(&mut node_map, &graph);

        insta::assert_json_snapshot!(instructions);
    }

    #[test]
    fn structural_equality_with_value_change() {
        let mut node_map = BTreeMap::new();

        let voice = |key, freq| {
            sin(mul2(
                cv(2.0 * std::f64::consts::PI),
                phasor(constant!({key: Some(key), value: freq})),
            ))
        };

        let graph = vec![add2(
            voice(String::from("v1"), 110.0),
            voice(String::from("v2"), 111.0),
        )];

        // For the sake of this test, we don't care about this first pass
        let _ = reconcile(&mut node_map, &graph);

        // Now we build a second graph aiming to see that the instruction
        // set produced contains only a single SetProperty
        let graph2 = vec![add2(
            voice(String::from("v1"), 112.0),
            voice(String::from("v2"), 111.0),
        )];

        let instructions = reconcile(&mut node_map, &graph2);
        insta::assert_json_snapshot!(instructions);
    }
}
