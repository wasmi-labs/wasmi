/**
 * \file wasmi/config.h
 *
 * \brief Wasmi-specific extensions to #wasm_config_t
 */

#ifndef WASMI_CONFIG_H
#define WASMI_CONFIG_H

#include <wasm.h>

#ifdef __cplusplus
extern "C" {
#endif

#define WASMI_CONFIG_PROP(ret, name, ty)                                       \
  WASM_API_EXTERN ret wasmi_config_##name##_set(wasm_config_t *, ty);

/**
 * \brief Whether or not fuel is enabled for generated code.
 *
 * When enabled it will enable fuel counting meaning that fuel will be consumed
 * every time a Wasm instruction is executed, and trap when reaching zero.
 *
 * Default value: `false`
 */
WASMI_CONFIG_PROP(void, consume_fuel, bool)

/**
 * \brief Whether or not to ignore Wasm custom sections.
 *
 * When enabled it will ignoe Wasm custom sections when creating a Wasm module.
 *
 * Default value: `false`
 */
WASMI_CONFIG_PROP(void, ignore_custom_sections, bool)

/**
 * \brief Whether or not to Wasm mutable-globals proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_mutable_globals, bool)

/**
 * \brief Whether or not to Wasm multi-value proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_multi_value, bool)

/**
 * \brief Whether or not to Wasm sign-extension proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_sign_extension, bool)

/**
 * \brief Whether or not to Wasm non-trapping-float-to-int-conversions proposal
 * is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_saturating_float_to_int, bool)

/**
 * \brief Whether or not to Wasm bulk-memory-ops proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_bulk_memory, bool)

/**
 * \brief Whether or not to Wasm reference-types proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_reference_types, bool)

/**
 * \brief Whether or not to Wasm tail-call proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_tail_call, bool)

/**
 * \brief Whether or not to Wasm extended-const proposal is enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, wasm_extended_const, bool)

/**
 * \brief Whether or not to floating Wasm point types and operations are
 * enabled.
 *
 * Default value: `true`
 */
WASMI_CONFIG_PROP(void, floats, bool)

/**
 * \brief Different ways Wasmi can compile Wasm bytecode into Wasmi bytecode.
 *
 * The default value is #WASMI_COMPILATION_MODE_EAGER.
 */
enum wasmi_compilation_mode_enum {
  /// Wasmi compiles and validates Wasm bytecode eagerly.
  WASMI_COMPILATION_MODE_EAGER,
  /// Wasmi compiles and validates Wasm bytecode upon first use.
  WASMI_COMPILATION_MODE_LAZY,
  /// Wasmi compiles Wasm bytecode upon first use but validates Wasm bytecode
  /// eagerly.
  WASMI_COMPILATION_MODE_LAZY_TRANSLATION,
};

/**
 * \brief Whether or not to floating Wasm point types and operations are
 * enabled.
 *
 * Default value: #WASMI_COMPILATION_MODE_EAGER
 */
WASMI_CONFIG_PROP(void, compilation_mode, enum wasmi_compilation_mode_enum)

#undef WASMI_CONFIG_PROP

#ifdef __cplusplus
} // extern "C"
#endif

#endif // WASMI_CONFIG_H
