use super::virtual_library::{BlockShape, VirtualBlock};
use crate::model::Block;

pub const LIB_NAME: &str = "simulink/Signal Routing";

fn goto_tag_label(block: &Block) -> Option<String> {
    block
        .properties
        .get("GotoTag")
        .map(|s| s.trim().to_string())
        .or_else(|| Some("A".to_string()))
}

pub const BLOCKS: &[VirtualBlock] = &[
    VirtualBlock {
        name: "BusCreator",
        aliases: &[],
        ins: 2,
        outs: 1,
        shape: BlockShape::FilledBlack,
        ..VirtualBlock::DEFAULT
    },
    VirtualBlock {
        name: "BusSelector",
        aliases: &[],
        ins: 1,
        outs: 2,
        shape: BlockShape::FilledBlack,
        ..VirtualBlock::DEFAULT
    },
    VirtualBlock {
        name: "Goto",
        aliases: &[],
        ins: 1,
        outs: 0,
        shape: BlockShape::Goto,
        compute_instance_label: Some(goto_tag_label),
        ..VirtualBlock::DEFAULT
    },
    VirtualBlock {
        name: "From",
        aliases: &[],
        ins: 0,
        outs: 1,
        shape: BlockShape::From,
        compute_instance_label: Some(goto_tag_label),
        ..VirtualBlock::DEFAULT
    },
];

pub fn get_blocks() -> &'static [VirtualBlock] {
    BLOCKS
}

pub fn is_simulink_signal_routing_name(name: &str) -> bool {
    let norm = name.trim().replace('\\', "/").to_ascii_lowercase();
    norm == "simulink/signal routing" || norm.starts_with("simulink/signal routing/")
}
