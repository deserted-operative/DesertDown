use super::{NodeId, NodeType};

#[derive(Debug, Clone, Copy)]
/// A Job specifies a block that has been extracted by the block parser, and itself needs to be parsed - either by the block or inline parser.
/// Jobs are queued up as blocks are closed by the block parser
pub struct Job {
    // the id of the root node of the block which needs parsing as a part of this job
    pub block: NodeId,

    // store the type of the block, to avoid having to reference into the nodes array again to get the type of the block
    pub node_type: NodeType,
}
