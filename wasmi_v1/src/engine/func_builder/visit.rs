use super::{FuncBuilder, FuncValidator, /* RelativeDepth ,*/ TranslationError};
// use crate::module::{BlockType, FuncIdx, FuncTypeIdx, GlobalIdx, MemoryIdx, TableIdx};
use wasmparser::{BinaryReaderError, VisitOperator};

impl<'alloc, 'parser> FuncBuilder<'alloc, 'parser> {
    /// Translates into `wasmi` bytecode if the current code path is reachable.
    fn validate_then_translate<V, F>(
        &mut self,
        validate: V,
        translator: F,
    ) -> Result<(), TranslationError>
    where
        V: FnOnce(&mut FuncValidator) -> Result<(), BinaryReaderError>,
        F: FnOnce(&mut Self) -> Result<(), TranslationError>,
    {
        validate(&mut self.validator)?;
        translator(self)?;
        Ok(())
    }
}

macro_rules! define_supported_visit_operator {
    ($( fn $visit:ident $(( $($arg:ident: $argty:ty),* ))? => fn $translate:ident)*) => {
        $(
            fn $visit(&mut self, offset: usize $($(,$arg: $argty)*)?) -> Self::Output {
                self.validate_then_translate(
                    |v| v.$visit(offset $($(,$arg)*)?),
                    |this| {
                        this.$translate($($($arg),*)?)
                    },
                )
            }
        )*
    };
}

macro_rules! define_unsupported_visit_operator {
    // The outer layer of repetition represents how all operators are
    // provided to the macro at the same time.
    //
    // The `$op` name is bound to the `Operator` variant name. The
    // payload of the operator is optionally specified (the `$(...)?`
    // clause) since not all instructions have payloads. Within the payload
    // each argument is named and has its type specified.
    //
    // The `$visit` name is bound to the corresponding name in the
    // `VisitOperator` trait that this corresponds to.
    ($( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident)*) => {
        $(
            fn $visit(&mut self, offset: usize $($(,$arg: $argty)*)?) -> Self::Output {
                self.validator.$visit(offset $($(,$arg)*)?).map_err(Into::into)
            }
        )*
    }
}

impl<'alloc, 'parser> VisitOperator<'parser> for FuncBuilder<'alloc, 'parser> {
    type Output = Result<(), TranslationError>;

    for_each_supported_operator!(define_supported_visit_operator);
    for_each_unsupported_operator!(define_unsupported_visit_operator);

    fn visit_br_table(
        &mut self,
        offset: usize,
        table: wasmparser::BrTable<'parser>,
    ) -> Self::Output {
        let table_cloned = table.clone();
        self.validate_then_translate(
            |v| v.visit_br_table(offset, table_cloned),
            |this| {
                this.translate_br_table(table)
            }
        )
    }
}
