;; /// @file inc_i32.cpp
;; #include <emscripten.h> // macro EMSCRIPTEN_KEEPALIVE
;; #include <stdint.h>
;; extern "C" {
;; uint32_t EMSCRIPTEN_KEEPALIVE inc_i32(uint32_t param) {
;;      return ++param;
;; }
;; } // extern "C"
(module
 (type $0 (func (param i32) (result i32)))
 (type $1 (func))
 (import "env" "memoryBase" (global $import$0 i32))
 (import "env" "memory" (memory $0 256))
 (import "env" "table" (table 0 anyfunc))
 (import "env" "tableBase" (global $import$3 i32))
 (global $global$0 (mut i32) (i32.const 0))
 (global $global$1 (mut i32) (i32.const 0))
 (export "_inc_i32" (func $0))
 (export "__post_instantiate" (func $2))
 (func $0 (type $0) (param $var$0 i32) (result i32)
  (i32.add
   (get_local $var$0)
   (i32.const 1)
  )
 )
 (func $1 (type $1)
  (nop)
 )
 (func $2 (type $1)
  (block $label$0
   (set_global $global$0
    (get_global $import$0)
   )
   (set_global $global$1
    (i32.add
     (get_global $global$0)
     (i32.const 5242880)
    )
   )
   (call $1)
  )
 )
 ;; custom section "dylink", size 5
)

