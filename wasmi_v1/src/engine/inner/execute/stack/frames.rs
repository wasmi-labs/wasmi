use crate::{
    engine::{ExecRegisterSlice, FuncBody},
    func::WasmFuncEntity,
    Instance,
};
use core::ops::Range;
use wasmi_core::TrapCode;

/// The call frame stack.
#[derive(Debug)]
pub struct FrameStack {
    /// The stack of call frames.
    frames: Vec<StackFrame>,
    /// The maximum depth of recursive or nested function calls.
    ///
    /// # Note
    ///
    /// Trying to push more frames onto the [`FrameStack`] results in
    /// a [`TrapCode::StackOverflow`] trap at runtime.
    maximum_recursion_depth: usize,
}

impl FrameStack {
    /// Creates a new [`FrameStack`] with the given maximum recursion depth.
    pub fn new(maximum_recursion_depth: usize) -> Self {
        Self {
            frames: Vec::default(),
            maximum_recursion_depth,
        }
    }
}

/// A reference to a [`StackFrame`] on the [`Stack`].
///
/// [`Stack`]: [`super::Stack`]
#[derive(Debug, Copy, Clone)]
pub struct StackFrameRef(usize);

/// An allocated frame on the [`Stack`].
///
/// [`Stack`]: [`super::Stack`]
#[derive(Debug)]
pub struct StackFrame {
    /// The region in which the [`StackFrame`] lives on the [`Stack`].
    ///
    /// [`Stack`]: [`super::Stack`]
    pub region: FrameRegion,
    /// The results slice of the [`StackFrame`].
    pub results: ExecRegisterSlice,
    /// The instruction of the function.
    pub func_body: FuncBody,
    /// The instance in which the function has been defined.
    ///
    /// # Note
    ///
    /// The instance is used to inspect and manipulate with data that is
    /// non-local to the function such as linear memories, global variables
    /// and tables.
    pub instance: Instance,
    /// The current program counter.
    ///
    /// # Note
    ///
    /// At instruction dispatch the program counter refers to the dispatched
    /// instructions. After instruction execution the program counter will
    /// refer to the next instruction.
    pub pc: usize,
}

/// The region of a [`StackFrame`] within the [`Stack`].
///
/// [`Stack`]: [`super::Stack`]
#[derive(Debug, Copy, Clone)]
pub struct FrameRegion {
    /// The index to the first register on the global [`Stack`].
    ///
    /// [`Stack`]: [`super::Stack`]
    start: usize,
    /// The amount of registers of the [`StackFrame`] belonging to this [`FrameRegion`].
    len: usize,
}

impl FrameRegion {
    /// Creates a new [`FrameRegion`].
    pub fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    /// Returns the start of the [`FrameRegion`].
    pub fn start(self) -> usize {
        self.start
    }

    /// Returns the end of the [`FrameRegion`].
    pub fn end(self) -> usize {
        self.start() + self.len()
    }

    /// Returns the length of the [`FrameRegion`].
    pub fn len(self) -> usize {
        self.len
    }

    /// Returns the index range of the [`FrameRegion`].
    pub fn range(self) -> Range<usize> {
        self.start()..self.end()
    }

    /// Returns `true` if `other` [`FrameRegion`] directly follows `self`.
    pub fn followed_by(&self, other: &Self) -> bool {
        self.end() == other.start()
    }
}

impl FrameStack {
    /// Returns the length of the call frame stack.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the call frame stack is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Clears the call frame stack, removing all frames.
    pub fn clear(&mut self) {
        self.frames.clear()
    }

    /// Pushes a new Wasm [`StackFrame`] onto the call frame stack.
    ///
    /// Returns a [`StackFrameRef`] refering to the pushed [`StackFrame`].
    ///
    /// # Note
    ///
    /// The `results` refer to the result registers of the previous
    /// [`StackFrame`] on the call frame stack which acts as the caller
    /// of the pushed [`StackFrame`].
    pub(super) fn push_frame(
        &mut self,
        region: FrameRegion,
        results: ExecRegisterSlice,
        wasm_func: &WasmFuncEntity,
    ) -> Result<StackFrameRef, TrapCode> {
        let len = self.len();
        if len == self.maximum_recursion_depth {
            return Err(TrapCode::StackOverflow);
        }
        self.frames.push(StackFrame {
            region,
            results,
            func_body: wasm_func.func_body(),
            instance: wasm_func.instance(),
            pc: 0,
        });
        Ok(StackFrameRef(len))
    }

    /// Pops the last [`StackFrame`] from the call frame stack.
    ///
    /// # Panics
    ///
    /// If the [`FrameStack`] is empty.
    pub fn pop_frame(&mut self) -> StackFrame {
        self.frames
            .pop()
            .expect("unexpected missing frame on the call frame stack")
    }

    /// Returns a shared reference to the last [`StackFrame`] on the call frame stack.
    ///
    /// # Panics
    ///
    /// If the [`FrameStack`] is empty.
    pub fn last_frame(&self) -> &StackFrame {
        self.frames
            .last()
            .expect("unexpected missing frame on the call frame stack")
    }

    /// Returns a shared reference to the last [`StackFrame`] on the call frame stack.
    ///
    /// # Panics
    ///
    /// If the [`FrameStack`] is empty.
    pub fn last_frame_mut(&mut self) -> &mut StackFrame {
        self.frames
            .last_mut()
            .expect("unexpected missing frame on the call frame stack")
    }

    /// Returns a [`StackFrameRef`] pointing to the last [`StackFrame`].
    ///
    /// # Panics
    ///
    /// If the [`FrameStack`] is empty.
    pub fn last_frame_ref(&self) -> StackFrameRef {
        debug_assert!(!self.is_empty());
        StackFrameRef(self.len() - 1)
    }

    /// Returns a shared reference to the [`StackFrame`] referenced by `frame_ref`.
    ///
    /// # Panics
    ///
    /// If `frame_ref` refers to an invalid [`StackFrame`].
    pub fn get_frame_mut(&mut self, frame_ref: StackFrameRef) -> &mut StackFrame {
        &mut self.frames[frame_ref.0]
    }
}
