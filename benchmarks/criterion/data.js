window.BENCHMARK_DATA = {
  "lastUpdate": 1690294772332,
  "repoUrl": "https://github.com/paritytech/wasmi",
  "entries": {
    "Wasmi criterion benchmark": [
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
        "date": 1690294772313,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3671693,
            "range": "± 13788",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 55461968,
            "range": "± 328509",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 92122,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 128064,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 185107,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55410,
            "range": "± 751",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 319010,
            "range": "± 1437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 420338,
            "range": "± 1763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 458589,
            "range": "± 632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 621240,
            "range": "± 765",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1359609,
            "range": "± 9053",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 731168,
            "range": "± 1098",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1161188,
            "range": "± 5198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1269289,
            "range": "± 11979",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1254190,
            "range": "± 26557",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1552307,
            "range": "± 21083",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1559486,
            "range": "± 10024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1614945,
            "range": "± 16067",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1791746,
            "range": "± 9289",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2590511,
            "range": "± 14468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 741960,
            "range": "± 1946",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 660704,
            "range": "± 753",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 515855,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318778,
            "range": "± 952",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 103514,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 140080,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10218,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37061,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4205992,
            "range": "± 7559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 971520,
            "range": "± 1220",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395403,
            "range": "± 3034",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 710204,
            "range": "± 2066",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1131213,
            "range": "± 1359",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1152216,
            "range": "± 1969",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2303725,
            "range": "± 5191",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}