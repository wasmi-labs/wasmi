window.BENCHMARK_DATA = {
  "lastUpdate": 1690294772544,
  "repoUrl": "https://github.com/paritytech/wasmi",
  "entries": {
    "Wasmi criterion wasmtime": [
      {
        "commit": {
          "author": {
            "name": "paritytech",
            "username": "paritytech"
          },
          "committer": {
            "name": "paritytech",
            "username": "paritytech"
          },
          "id": "b0498bdd1bfeddaf983119c0a7ad8779425f1190",
          "message": "[Do not merge] Publishing benchmarks for graphs",
          "timestamp": "2023-07-25T14:14:26Z",
          "url": "https://github.com/paritytech/wasmi/pull/740/commits/b0498bdd1bfeddaf983119c0a7ad8779425f1190"
        },
        "date": 1690294772525,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5778641,
            "range": "± 17523",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100901516,
            "range": "± 242046",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 152805,
            "range": "± 1905",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 219511,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 315419,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55951,
            "range": "± 1603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 811468,
            "range": "± 2335",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 918257,
            "range": "± 1491",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 918240,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1297772,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1193938,
            "range": "± 2560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1791703,
            "range": "± 2399",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 659115,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1000138,
            "range": "± 1756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 942105,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1801197,
            "range": "± 5152",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1027691,
            "range": "± 2376",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1213639,
            "range": "± 2509",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1647253,
            "range": "± 20427",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3552146,
            "range": "± 5275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1662273,
            "range": "± 3928",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1679627,
            "range": "± 1213",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 866751,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 669848,
            "range": "± 693",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 186821,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 244645,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 18699,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39505,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7534896,
            "range": "± 10234",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1878106,
            "range": "± 1200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3231672,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1283838,
            "range": "± 2517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2418116,
            "range": "± 2119",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2620737,
            "range": "± 6733",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5221896,
            "range": "± 6742",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}