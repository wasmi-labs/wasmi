/**
 * \file wasmi/engine.h
 *
 * \brief Wasmi-specific extensions to #wasm_engine_t
 */

#ifndef WASMI_ENGINE_H
#define WASMI_ENGINE_H

#include <wasm.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * \brief Create a new reference to the same underlying engine.
 *
 * This function clones the reference-counted pointer to the internal object,
 * and must be freed using #wasm_engine_delete.
 */
WASM_API_EXTERN wasm_engine_t *wasmi_engine_clone(wasm_engine_t *engine);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // WASMI_ENGINE_H
