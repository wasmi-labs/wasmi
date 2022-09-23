/// Test profiles collected during the Wasm spec test run.
#[derive(Debug, Default)]
pub struct TestProfile {
    /// The total amount of executed `.wast` directives.
    directives: usize,
    /// The amount of executed [`WasmDirective::Module`].
    module: usize,
    /// The amount of executed [`WasmDirective::QuoteModule`].
    quote_module: usize,
    /// The amount of executed [`WasmDirective::AssertMalformed`].
    assert_malformed: usize,
    /// The amount of executed [`WasmDirective::AssertInvalid`].
    assert_invalid: usize,
    /// The amount of executed [`WasmDirective::Register`].
    register: usize,
    /// The amount of executed [`WasmDirective::Invoke`].
    invoke: usize,
    /// The amount of executed [`WasmDirective::AssertTrap`].
    assert_trap: usize,
    /// The amount of executed [`WasmDirective::AssertReturn`].
    assert_return: usize,
    /// The amount of executed [`WasmDirective::AssertExhaustion`].
    assert_exhaustion: usize,
    /// The amount of executed [`WasmDirective::AssertUnlinkable`].
    assert_unlinkable: usize,
    /// The amount of executed [`WasmDirective::AssertException`].
    assert_exception: usize,
}

impl TestProfile {
    /// Bumps the amount of directives.
    pub fn bump_directives(&mut self) {
        self.directives += 1;
    }

    /// Bumps the amount of [`WasmDirective::Module`] directives.
    pub fn bump_module(&mut self) {
        self.module += 1;
    }

    /// Bumps the amount of [`WasmDirective::QuoteModule`] directives.
    pub fn bump_quote_module(&mut self) {
        self.quote_module += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertMalformed`] directives.
    pub fn bump_assert_malformed(&mut self) {
        self.assert_malformed += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertInvalid`] directives.
    pub fn bump_assert_invalid(&mut self) {
        self.assert_invalid += 1;
    }

    /// Bumps the amount of [`WasmDirective::Register`] directives.
    pub fn bump_register(&mut self) {
        self.register += 1;
    }

    /// Bumps the amount of [`WasmDirective::Invoke`] directives.
    pub fn bump_invoke(&mut self) {
        self.invoke += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertTrap`] directives.
    pub fn bump_assert_trap(&mut self) {
        self.assert_trap += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertReturn`] directives.
    pub fn bump_assert_return(&mut self) {
        self.assert_return += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertExhaustion`] directives.
    pub fn bump_assert_exhaustion(&mut self) {
        self.assert_exhaustion += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertUnlinkable`] directives.
    pub fn bump_assert_unlinkable(&mut self) {
        self.assert_unlinkable += 1;
    }

    /// Bumps the amount of [`WasmDirective::AssertException`] directives.
    pub fn bump_assert_exception(&mut self) {
        self.assert_exception += 1;
    }
}
