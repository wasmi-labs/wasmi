use parity_wasm::elements::BlockType;

pub mod stack;

/// Index of default linear memory.
pub const DEFAULT_MEMORY_INDEX: u32 = 0;
/// Index of default table.
pub const DEFAULT_TABLE_INDEX: u32 = 0;

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
	/// Boolean which signals whether value stack became polymorphic. Value stack starts in non-polymorphic state and
	/// becomes polymorphic only after an instruction that never passes control further is executed,
	/// i.e. `unreachable`, `br` (but not `br_if`!), etc.
	pub polymorphic_stack: bool,
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
