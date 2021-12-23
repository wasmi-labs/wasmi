use super::super::super::{DropKeep, LabelIdx};
use super::ControlFrame;
use parity_wasm::elements::BlockType;
use validation::func::{require_label, top_label, BlockFrame, StackValueType, StartedWith};
use validation::stack::StackWithLimit;
use validation::{util::Locals, Error};

/// Computes how many values should be dropped and kept for the specific branch.
///
/// # Errors
///
/// If underflow of the value stack detected.
///
/// # Note
///
/// This implementation has not yet been ported to support Wasm `multivalue` proposal.
pub fn compute_drop_keep(
    in_stack_polymorphic_state: bool,
    started_with: StartedWith,
    block_type: BlockType,
    actual_value_stack_height: usize,
    start_value_stack_height: usize,
) -> Result<DropKeep, Error> {
    // Find out how many values we need to keep (copy to the new stack location after the drop).
    let keep = match (started_with, block_type) {
        // A loop doesn't take a value upon a branch. It can return value
        // only via reaching it's closing `End` operator.
        (StartedWith::Loop, _) => 0,
        (_, BlockType::Value(_)) => 1,
        (_, BlockType::NoResult) => 0,
    };
    // Find out how many values we need to drop.
    let drop = if in_stack_polymorphic_state {
        // Polymorphic stack is a weird state. Fortunately, it is always about the code that
        // will not be executed, so we don't bother and return 0 here.
        0
    } else {
        if actual_value_stack_height < start_value_stack_height {
            return Err(Error(format!(
                "Stack underflow detected: value stack height ({}) is lower than minimum stack len ({})",
                actual_value_stack_height,
                start_value_stack_height,
            )));
        }
        if (actual_value_stack_height - start_value_stack_height) < keep {
            return Err(Error(format!(
                "Stack underflow detected: asked to keep {:?} values, but there are only {}",
                keep,
                actual_value_stack_height - start_value_stack_height,
            )));
        }
        (actual_value_stack_height - start_value_stack_height) - keep
    };
    Ok(DropKeep::new(drop, keep))
}

/// Compute [`DropKeep`] for the return statement.
///
/// # Errors
///
/// - If the control flow frame stack is empty.
/// - If the value stack is underflown.
///
/// # Note
///
/// This implementation has not yet been ported to support Wasm `multivalue` proposal.
pub fn drop_keep_return(
    locals: &Locals,
    value_stack: &StackWithLimit<StackValueType>,
    frame_stack: &StackWithLimit<BlockFrame>,
) -> Result<DropKeep, Error> {
    if frame_stack.is_empty() {
        return Err(Error(
            "drop_keep_return can't be called with the frame stack empty".into(),
        ));
    }
    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let deepest = frame_stack
        .len()
        .checked_sub(1)
        .expect("frame_stack is not empty") as u32;
    let frame = require_label(deepest, frame_stack).expect("frame_stack is not empty");
    let drop_keep = compute_drop_keep(
        is_stack_polymorphic,
        frame.started_with,
        frame.block_type,
        value_stack.len(),
        frame.value_stack_len,
    )?;
    Ok(DropKeep::new(
        drop_keep.drop(),
        // Drop all local variables and parameters upon exit.
        drop_keep.keep() + locals.count() as usize,
    ))
}

/// Returns the requested target for branch referred by `depth`.
///
/// # Errors
///
/// - If the `depth` is greater than the current height of the control frame stack.
/// - If the value stack underflowed.
pub fn require_target(
    depth: u32,
    value_stack_height: usize,
    frame_stack: &StackWithLimit<BlockFrame>,
    label_stack: &[ControlFrame],
) -> Result<(LabelIdx, DropKeep), Error> {
    let is_stack_polymorphic = top_label(frame_stack).polymorphic_stack;
    let frame = require_label(depth, frame_stack)?;
    // Get the label by the given `depth`.
    let label = label_stack
        .iter()
        .rev()
        .nth(depth as usize)
        .unwrap_or_else(|| {
            panic!(
                "unexectedly failed to get the branch target at depth {}",
                depth
            )
        });
    let drop_keep = compute_drop_keep(
        is_stack_polymorphic,
        frame.started_with,
        frame.block_type,
        value_stack_height,
        frame.value_stack_len,
    )?;
    Ok((label.br_destination(), drop_keep))
}
