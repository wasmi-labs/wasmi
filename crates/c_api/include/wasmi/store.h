/**
 * \file wasmi/store.h
 *
 * \brief Wasmi-specific extensions to #wasm_store_t
 */

#ifndef WASMI_STORE_H
#define WASMI_STORE_H

#include <wasm.h>
#include <wasmi/error.h>

#define own

#ifdef __cplusplus
extern "C" {
#endif

/**
 * \typedef wasmi_store_t
 * \brief Convenience alias for #wasmi_store_t
 *
 * \struct wasmi_store
 * \brief Storage of WebAssembly objects
 *
 * A store is the unit of isolation between WebAssembly instances in an
 * embedding of Wasmi. Values in one #wasmi_store_t cannot flow into
 * another #wasmi_store_t.
 *
 * Objects stored within a #wasmi_store_t are referenced with integer handles.
 * This means that most APIs require that the store be explicitly passed in,
 * which is done via #wasmi_context_t. It is safe to move a #wasmi_store_t
 * to any thread at any time. Though, a store generally cannot be concurrently
 * used.
 */
typedef struct wasmi_store wasmi_store_t;

/**
 * \typedef wasmi_context_t
 * \brief Convenience alias for #wasmi_context
 *
 * \struct wasmi_context
 * \brief An interior pointer into a #wasmi_store_t which is used as
 * "context" for many functions.
 *
 * This context pointer is used throughout Wasmi's API.
 * It can be acquired from #wasmi_store_context or #wasmi_caller_context.
 * The context pointer for a store is the same for the entire lifetime of a
 * store, so it can safely be stored adjacent to a #wasmi_store_t itself.
 *
 * - Usage of a #wasmi_context_t must not outlive the original #wasmi_store_t.
 * - Additionally #wasmi_context_t can only be used in situations where it has
 *   explicitly been granted access to doing so.
 * - Finalizers cannot use #wasmi_context_t because they are not given access to
 * it.
 */
typedef struct wasmi_context wasmi_context_t;

/**
 * \brief Creates a new Wasmi store within the specified engine.
 *
 * \param engine the compilation environment with configuration this store is
 * connected to
 * \param data user-provided data to store, can later be acquired with
 * #wasmi_context_get_data.
 * \param finalizer an optional finalizer for `data`
 *
 * This function creates a fresh store with the provided configuration settings.
 * The returned store must be deleted with #wasmi_store_delete.
 */
WASM_API_EXTERN own wasmi_store_t *
wasmi_store_new(wasm_engine_t *engine, void *data, void (*finalizer)(void *));

/**
 * \brief Deletes a Wasmi store.
 */
WASM_API_EXTERN void wasmi_store_delete(own wasmi_store_t *store);

/**
 * \brief Returns a interior #wasmi_context_t pointer to the Wasmi store.
 */
WASM_API_EXTERN wasmi_context_t *wasmi_store_context(wasmi_store_t *store);

/**
 * \brief Returns the user-specified data associated with the specified store
 */
WASM_API_EXTERN void *wasmi_context_get_data(const wasmi_context_t *context);

/**
 * \brief Overwrites the user-specified data associated with the Wasmi store.
 *
 * Note that this does not execute the original finalizer for the provided data,
 * and the original finalizer will be executed for the provided data when the
 * store is deleted.
 */
WASM_API_EXTERN void wasmi_context_set_data(wasmi_context_t *context,
                                            void *data);

/**
 * \brief Set fuel to this context's store for wasm to consume while executing.
 *
 * For this method to work fuel consumption must be enabled via
 * #wasmi_config_consume_fuel_set. By default a store starts with 0 fuel
 * for wasm to execute with (meaning it will immediately trap).
 * This function must be called for the store to have
 * some fuel to allow WebAssembly to execute.
 *
 * Note that when fuel is entirely consumed it will cause wasm to trap.
 *
 * If fuel is not enabled within this store then an error is returned. If fuel
 * is successfully added then NULL is returned.
 */
WASM_API_EXTERN wasmi_error_t *wasmi_context_set_fuel(wasmi_context_t *store,
                                                      uint64_t fuel);

/**
 * \brief Returns the amount of fuel remaining in this context's store.
 *
 * If fuel consumption is not enabled via #wasmi_config_consume_fuel_set
 * then this function will return an error. Otherwise `NULL` is returned and the
 * fuel parameter is filled in with fuel consumed so far.
 *
 * Also note that fuel, if enabled, must be originally configured via
 * #wasmi_context_set_fuel.
 */
WASM_API_EXTERN wasmi_error_t *
wasmi_context_get_fuel(const wasmi_context_t *context, uint64_t *fuel);

#ifdef __cplusplus
} // extern "C"
#endif

#undef own

#endif // WASMI_STORE_H
