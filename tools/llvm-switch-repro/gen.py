#!/usr/bin/env python3
"""Generate reproducers for the LLVM switch-to-lookup-table pathology that forces
Wasmi to set `-C llvm-args=--switch-to-lookup=true`.

Three shapes (`--shape`):

  handler  A single function: a `match`/`switch` over a dense `#[repr(u16)]`
           enum (discriminants 0..N) returning a distinct function pointer per
           case. This is the textbook lookup-table candidate. NOTE: on
           rustc 1.96 / LLVM 22.1.2 the *default* pipeline already turns this
           lone switch into a lookup table at every N, so in isolation it does
           NOT reproduce the bug -- it only shows what `-switch-to-lookup=false`
           breaks. Kept for contrast.

  interp   The FAITHFUL shape. Mirrors Wasmi's tail-call dispatch: N handler
           functions, each `#[inline(never)]`, each ending by calling an
           `#[inline(always)]` dispatch fn that contains the N-arm match and
           tail-calling its result. The dispatch match is therefore inlined into
           all N handlers (~N*N switch arms total). This is what actually trips
           LLVM's switch-to-lookup heuristic by default once N is large.

  table    The candidate FIX. Same as `interp`, but dispatch indexes a `static`
           array of function pointers instead of `match`ing -- there is no switch
           for LLVM to (mis)lower, so it stays a single load regardless of the
           flag.

Each handler has a distinct body so identical-code-folding cannot merge them
(which would collapse the dispatch to a constant and hide the issue).

Usage:
    python3 gen.py --lang rust --shape interp --count 1100 --out repro.rs
    python3 gen.py --lang c    --shape interp --count 1100 --out repro.c
"""

import argparse
import sys


# --------------------------------------------------------------------------- C
def c_handler(n):
    hs = "\n".join(f"static int h{i}(void) {{ return {i}; }}" for i in range(n))
    cs = "\n".join(f"        case {i}: return h{i};" for i in range(n))
    return f"""#include <stdint.h>
typedef int (*handler)(void);

{hs}

handler op_code_to_handler(uint16_t code) {{
    switch (code) {{
{cs}
        default: __builtin_unreachable();
    }}
}}
"""


def c_interp(n):
    fwd = "\n".join(f"static unsigned long h{i}(struct St*);" for i in range(n))
    cs = "\n".join(f"        case {i}: return h{i};" for i in range(n))
    defs = "\n".join(
        f"static __attribute__((noinline)) unsigned long h{i}(struct St*s)"
        f"{{ s->acc += {i}; handler nh = dispatch(s->code); return nh(s); }}"
        for i in range(n))
    return f"""#include <stdint.h>
struct St {{ unsigned long acc; uint16_t code; }};
typedef unsigned long (*handler)(struct St*);

{fwd}

/* inlined into every handler below -> N copies of an N-arm switch */
static inline __attribute__((always_inline)) handler dispatch(uint16_t code) {{
    switch (code) {{
{cs}
        default: __builtin_unreachable();
    }}
}}

{defs}

unsigned long run(struct St*s) {{ handler nh = dispatch(s->code); return nh(s); }}
"""


def c_table(n):
    fwd = "\n".join(f"static unsigned long h{i}(struct St*);" for i in range(n))
    tbl = ", ".join(f"h{i}" for i in range(n))
    defs = "\n".join(
        f"static __attribute__((noinline)) unsigned long h{i}(struct St*s)"
        f"{{ s->acc += {i}; handler nh = dispatch(s->code); return nh(s); }}"
        for i in range(n))
    return f"""#include <stdint.h>
struct St {{ unsigned long acc; uint16_t code; }};
typedef unsigned long (*handler)(struct St*);

{fwd}

static const handler TABLE[{n}] = {{ {tbl} }};

/* no switch: a plain indexed load, inlined everywhere */
static inline __attribute__((always_inline)) handler dispatch(uint16_t code) {{
    return TABLE[code];
}}

{defs}

unsigned long run(struct St*s) {{ handler nh = dispatch(s->code); return nh(s); }}
"""


# ------------------------------------------------------------------------ Rust
def rs_prelude(n):
    variants = ", ".join(f"V{i}" for i in range(n))
    return f"""#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum OpCode {{ {variants} }}   // dense, contiguous 0..{n}
"""


def rs_handler(n):
    hs = "\n".join(f"fn h{i}() -> u32 {{ {i} }}" for i in range(n))
    arms = "\n".join(f"        OpCode::V{i} => h{i}," for i in range(n))
    return rs_prelude(n) + f"""pub type Handler = fn() -> u32;
{hs}
#[inline(never)]
pub fn op_code_to_handler(code: OpCode) -> Handler {{
    match code {{
{arms}
    }}
}}
"""


def _rs_handlers(n):
    return "\n".join(
        f"#[inline(never)] fn h{i}(s:&mut St)->u64 "
        f"{{ s.acc=s.acc.wrapping_add({i}); let c=unsafe{{core::mem::transmute::<u16,OpCode>(s.code)}}; dispatch(c)(s) }}"
        for i in range(n))


def rs_interp(n):
    arms = "\n".join(f"        OpCode::V{i} => h{i}," for i in range(n))
    return rs_prelude(n) + f"""pub struct St {{ pub acc: u64, pub code: u16 }}
pub type Handler = fn(&mut St) -> u64;

// #[inline(always)] -> this N-arm match is inlined into every handler below.
#[inline(always)]
fn dispatch(code: OpCode) -> Handler {{
    match code {{
{arms}
    }}
}}

{_rs_handlers(n)}

pub fn run(s:&mut St)->u64 {{ let c=unsafe{{core::mem::transmute::<u16,OpCode>(s.code)}}; dispatch(c)(s) }}
"""


def rs_table(n):
    tbl = ", ".join(f"h{i} as Handler" for i in range(n))
    return rs_prelude(n) + f"""pub struct St {{ pub acc: u64, pub code: u16 }}
pub type Handler = fn(&mut St) -> u64;

static TABLE: [Handler; {n}] = [{tbl}];

// no match: a plain indexed load (unchecked, code is always a valid OpCode).
#[inline(always)]
fn dispatch(code: OpCode) -> Handler {{ unsafe {{ *TABLE.get_unchecked(code as usize) }} }}

{_rs_handlers(n)}

pub fn run(s:&mut St)->u64 {{ let c=unsafe{{core::mem::transmute::<u16,OpCode>(s.code)}}; dispatch(c)(s) }}
"""


GEN = {
    ("c", "handler"): c_handler, ("c", "interp"): c_interp, ("c", "table"): c_table,
    ("rust", "handler"): rs_handler, ("rust", "interp"): rs_interp, ("rust", "table"): rs_table,
}


def main():
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--lang", required=True, choices=["c", "rust"])
    p.add_argument("--shape", default="interp", choices=["handler", "interp", "table"])
    p.add_argument("--count", required=True, type=int, help="number of cases (N)")
    p.add_argument("--out", default=None)
    a = p.parse_args()
    if a.count < 1:
        p.error("--count must be >= 1")
    src = GEN[(a.lang, a.shape)](a.count)
    if a.out is None:
        sys.stdout.write(src)
    else:
        open(a.out, "w").write(src)


if __name__ == "__main__":
    main()
