use parity_wasm::elements::BlockType;

pub mod stack;

/// Index of default linear memory.
pub const DEFAULT_MEMORY_INDEX: u32 = 0;
/// Index of default table.
pub const DEFAULT_TABLE_INDEX: u32 = 0;
/// Maximum number of entries in value stack.
pub const DEFAULT_VALUE_STACK_LIMIT: usize = 16384;
/// Maximum number of entries in frame stack.
pub const DEFAULT_FRAME_STACK_LIMIT: usize = 1024;

/// Control stack frame.
#[derive(Debug, Clone)]
pub struct BlockFrame {
	/// Frame type.
	pub frame_type: BlockFrameType,
	/// A signature, which is a block signature type indicating the number and types of result values of the region.
	pub block_type: BlockType,
	/// A label for reference to block instruction.
	pub begin_position: usize,
	/// A label for reference from branch instructions.
	pub branch_position: usize,
	/// A label for reference from end instructions.
	pub end_position: usize,
	/// A limit integer value, which is an index into the value stack indicating where to reset it to on a branch to that label.
	pub value_stack_len: usize,
}

/// Type of block frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockFrameType {
	/// Function frame.
	Function,
	/// Usual block frame.
	Block,
	/// Loop frame (branching to the beginning of block).
	Loop,
	/// True-subblock of if expression.
	IfTrue,
	/// False-subblock of if expression.
	IfFalse,
}
