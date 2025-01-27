use crate::node::{create_node, NodeRepr};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn root(x: NodeRepr) -> NodeRepr {
    create_node(
        "root",
        json!({"channel": 0.0}).as_object().unwrap().clone(),
        vec![x],
    )
}

pub fn sin(x: NodeRepr) -> NodeRepr {
    create_node("sin", Default::default(), vec![x])
}

pub fn mul2(x: NodeRepr, y: NodeRepr) -> NodeRepr {
    create_node("mul", Default::default(), vec![x, y])
}

pub fn add2(x: NodeRepr, y: NodeRepr) -> NodeRepr {
    create_node("add", Default::default(), vec![x, y])
}

pub fn phasor(rate: NodeRepr) -> NodeRepr {
    create_node("phasor", Default::default(), vec![rate])
}

#[derive(Serialize, Deserialize)]
pub struct ConstNodeProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub value: f64,
}

pub fn constant(props: &ConstNodeProps) -> NodeRepr {
    create_node(
        "const",
        serde_json::to_value(&props)
            .unwrap()
            .as_object()
            .unwrap()
            .clone(),
        vec![],
    )
}

#[macro_export]
macro_rules! constant {
    // Match the macro pattern with a key-value pair in the first argument
    ({$($key:ident: $value:expr),*}) => {
        {
            // Create the props struct with the provided key-value pairs
            let props = ConstNodeProps { $($key: $value),* };

            // Call the constant function with the constructed props
            constant(&props)
        }
    };
}

pub use crate::constant;

pub fn cv(x: f64) -> NodeRepr {
    constant!({key: None, value: x})
}

pub fn le(x: NodeRepr, y: NodeRepr) -> NodeRepr {
    create_node("le", Default::default(), vec![x, y])
}

pub fn train(x: NodeRepr) -> NodeRepr {
    le(phasor(x), cv(0.5))
}

#[derive(Serialize, Deserialize)]
pub struct SampleNodeProps {
    pub key: Option<String>,
    pub path: String,
}

pub fn sample(props: &SampleNodeProps, gate: NodeRepr) -> NodeRepr {
    create_node(
        "sample",
        serde_json::to_value(&props)
            .unwrap()
            .as_object()
            .unwrap()
            .clone(),
        vec![gate],
    )
}

#[macro_export]
macro_rules! sample {
    // Match the macro pattern with a key-value pair in the first argument
    ({$($key:ident: $value:expr),*}, $gate:expr) => {
        {
            // Create the props struct with the provided key-value pairs
            let props = SampleNodeProps { $($key: $value),* };

            // Call the constant function with the constructed props
            sample(&props, $gate)
        }
    };
}

pub use crate::sample;
