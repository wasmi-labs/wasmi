window.BENCHMARK_DATA = {
  "lastUpdate": 1700851101814,
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
      },
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
          "id": "0c7411afde32e413f82f09bb6d26e8d395f2c3ac",
          "message": "[Do not merge] Publishing benchmarks for graphs",
          "timestamp": "2023-07-25T14:14:26Z",
          "url": "https://github.com/paritytech/wasmi/pull/740/commits/0c7411afde32e413f82f09bb6d26e8d395f2c3ac"
        },
        "date": 1690295226919,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5761911,
            "range": "± 17182",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100390118,
            "range": "± 724031",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 154018,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 220497,
            "range": "± 1357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 314816,
            "range": "± 900",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54292,
            "range": "± 938",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 813653,
            "range": "± 2142",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 918727,
            "range": "± 1623",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 917513,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1298332,
            "range": "± 1598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1204309,
            "range": "± 1572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1791613,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 661361,
            "range": "± 1755",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1013038,
            "range": "± 3866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 956427,
            "range": "± 3718",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1810330,
            "range": "± 4265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1023998,
            "range": "± 4802",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1204502,
            "range": "± 2593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1632029,
            "range": "± 2881",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3548836,
            "range": "± 47895",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1661084,
            "range": "± 3257",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1682090,
            "range": "± 2726",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 867740,
            "range": "± 1223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 669769,
            "range": "± 565",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 187365,
            "range": "± 1541",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 244499,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19460,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39924,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7527710,
            "range": "± 12984",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1882343,
            "range": "± 4358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3232572,
            "range": "± 6419",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1285176,
            "range": "± 5331",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2424836,
            "range": "± 10170",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2638836,
            "range": "± 10840",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5228125,
            "range": "± 32579",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "6f7904e3b2f96a38f44082f63d446558a8162cef",
          "message": "[Do not merge] Publishing benchmarks for graphs",
          "timestamp": "2023-07-25T14:14:26Z",
          "url": "https://github.com/paritytech/wasmi/pull/740/commits/6f7904e3b2f96a38f44082f63d446558a8162cef"
        },
        "date": 1690295738955,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5802009,
            "range": "± 27825",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100953811,
            "range": "± 280430",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 153250,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 219875,
            "range": "± 1316",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 314777,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56112,
            "range": "± 1106",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 798003,
            "range": "± 1892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 918025,
            "range": "± 828",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 919672,
            "range": "± 994",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1296893,
            "range": "± 1503",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1216012,
            "range": "± 1977",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1791802,
            "range": "± 4868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 661048,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 984347,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 939198,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1799591,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1032400,
            "range": "± 2577",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1232667,
            "range": "± 2097",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1650972,
            "range": "± 2353",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3590377,
            "range": "± 4098",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1664176,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1679981,
            "range": "± 10092",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 864714,
            "range": "± 1463",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 669373,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 187517,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 244911,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 18719,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39717,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7620832,
            "range": "± 5205",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1869312,
            "range": "± 1756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3230467,
            "range": "± 3837",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1282210,
            "range": "± 1374",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2487132,
            "range": "± 5200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3084374,
            "range": "± 3106",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5238793,
            "range": "± 25130",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "41779041+alvicsam@users.noreply.github.com",
            "name": "Alexander Samusev",
            "username": "alvicsam"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "af8c588c9059c5299da812958d7a71dc024d2938",
          "message": "Publishing benchmarks for graphs (#740)\n\n* [Do not merge] Publishing benchmarks for graphs\r\n\r\n* add wasmtime-benchmark-master\r\n\r\n* add dbg ref\r\n\r\n* add collect artifacts\r\n\r\n* switch ci image\r\n\r\n* add publish\r\n\r\n* cp instaed mv\r\n\r\n* add gha\r\n\r\n* fix on\r\n\r\n* add gh-pages\r\n\r\n* disable ref for checkout\r\n\r\n* trim whitespaces\r\n\r\n* add cancel for previous runs\r\n\r\n* add gh token\r\n\r\n* downgrade checkout action\r\n\r\n* add skip-fetch-gh-pages option\r\n\r\n* debug gha\r\n\r\n* add timer for files\r\n\r\n* fix script\r\n\r\n* move script to file\r\n\r\n* rename job\r\n\r\n* restart pipeline\r\n\r\n* add debug messages\r\n\r\n* enable script\r\n\r\n* restart pipeline to add second result to graph\r\n\r\n* remove debug refs\r\n\r\n---------\r\n\r\nCo-authored-by: Robin Freyler <robin.freyler@gmail.com>",
          "timestamp": "2023-07-31T12:23:17+02:00",
          "tree_id": "44ed1b352709151aaf49f40d5f4719aeaeb2c866",
          "url": "https://github.com/paritytech/wasmi/commit/af8c588c9059c5299da812958d7a71dc024d2938"
        },
        "date": 1690799417308,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5684055,
            "range": "± 11046",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 99270220,
            "range": "± 295291",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 151850,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 217450,
            "range": "± 2022",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 310791,
            "range": "± 556",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56793,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 854860,
            "range": "± 3268",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 967556,
            "range": "± 7828",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 972033,
            "range": "± 3120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1365635,
            "range": "± 918",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1097066,
            "range": "± 3380",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1851157,
            "range": "± 1650",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 645597,
            "range": "± 6349",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 944250,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 925758,
            "range": "± 2308",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1749323,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1028064,
            "range": "± 1825",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1188687,
            "range": "± 23883",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1563099,
            "range": "± 1996",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3475704,
            "range": "± 4679",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1666720,
            "range": "± 2162",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1760030,
            "range": "± 3600",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 850278,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 687965,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 190996,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 247197,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19962,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 38602,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7769946,
            "range": "± 4455",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1894652,
            "range": "± 3697",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3308539,
            "range": "± 6400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1241809,
            "range": "± 5179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2651936,
            "range": "± 3204",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2680888,
            "range": "± 6006",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5705647,
            "range": "± 9631",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "983ef37b3e2acf38a898e191c9bbbd2bc2c05da7",
          "message": "Prepare `wasmi` release for version `0.31.0` (#748)\n\n* bump crate versions\r\n\r\n* update wast dependency v0.52.0 -> v0.62.0\r\n\r\n* update criterion from v0.4.0 -> v0.5.0\r\n\r\n* add changelog for v0.31.0 release\r\n\r\n* update changelog\r\n\r\n* update changelog for updated dev. dependencies\r\n\r\n* changed ordering of changelog sections",
          "timestamp": "2023-07-31T14:12:51+02:00",
          "tree_id": "7f10aefbf3d1dfd58d61a7e5d594aba661aefab0",
          "url": "https://github.com/paritytech/wasmi/commit/983ef37b3e2acf38a898e191c9bbbd2bc2c05da7"
        },
        "date": 1690805859063,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5904922,
            "range": "± 66283",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 101248476,
            "range": "± 280233",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 155209,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 221639,
            "range": "± 1286",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 315474,
            "range": "± 2996",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58348,
            "range": "± 1706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 846441,
            "range": "± 2677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 975281,
            "range": "± 9137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 992892,
            "range": "± 5871",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1379810,
            "range": "± 9209",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1118944,
            "range": "± 4880",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1873714,
            "range": "± 15283",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 647140,
            "range": "± 4376",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 930864,
            "range": "± 7240",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1003852,
            "range": "± 22669",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1772071,
            "range": "± 11739",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1060177,
            "range": "± 10137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1223438,
            "range": "± 6497",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1645315,
            "range": "± 10899",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3635065,
            "range": "± 54584",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1756552,
            "range": "± 30431",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1767006,
            "range": "± 13408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 826536,
            "range": "± 1703",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 691897,
            "range": "± 1728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 191606,
            "range": "± 236",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 250350,
            "range": "± 1141",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20008,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 41002,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7660347,
            "range": "± 43091",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1911129,
            "range": "± 3718",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3309069,
            "range": "± 6388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1247440,
            "range": "± 3163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2655873,
            "range": "± 17545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2685057,
            "range": "± 4855",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5735924,
            "range": "± 12623",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49134864+load1n9@users.noreply.github.com",
            "name": "Dean Srebnik",
            "username": "load1n9"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "61f7986c594e6cf6fb8f66d14572d4ab74ffbe3c",
          "message": "typo (#753)\n\nUpdate preview_1.rs",
          "timestamp": "2023-08-30T22:46:24+02:00",
          "tree_id": "fd1c01a871814eb2d52bd9d160e13f478e284430",
          "url": "https://github.com/paritytech/wasmi/commit/61f7986c594e6cf6fb8f66d14572d4ab74ffbe3c"
        },
        "date": 1693428929601,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5734287,
            "range": "± 25265",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100189741,
            "range": "± 407157",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 153867,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 219459,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 312699,
            "range": "± 1210",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54106,
            "range": "± 1349",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 841099,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 966018,
            "range": "± 1756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 968610,
            "range": "± 1072",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1411100,
            "range": "± 4723",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1110278,
            "range": "± 1402",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1862713,
            "range": "± 1061",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 667294,
            "range": "± 25479",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 936763,
            "range": "± 9277",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 916951,
            "range": "± 8012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1875493,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1041620,
            "range": "± 1918",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1191222,
            "range": "± 3017",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1555666,
            "range": "± 3352",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3336132,
            "range": "± 5545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1660574,
            "range": "± 2431",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1761031,
            "range": "± 4232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 864653,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 690401,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 190748,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 245976,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19600,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39500,
            "range": "± 633",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7917672,
            "range": "± 15302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1879067,
            "range": "± 2240",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3291863,
            "range": "± 2603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1248175,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2640498,
            "range": "± 3834",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2680608,
            "range": "± 12790",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5683326,
            "range": "± 93453",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "33dc8721132b5ffd498f906c627497a18c568fed",
          "message": "Add `wasmi` logo (#758)\n\n* add wasmi logo\r\n\r\n* use center alignment that github understands",
          "timestamp": "2023-09-11T14:42:50+02:00",
          "tree_id": "5e3f8ed29d29897cd6c7b46c6fba7900816732af",
          "url": "https://github.com/paritytech/wasmi/commit/33dc8721132b5ffd498f906c627497a18c568fed"
        },
        "date": 1694436469069,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5768868,
            "range": "± 16062",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 101226095,
            "range": "± 196011",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 154226,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 221957,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 315927,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52653,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 841417,
            "range": "± 1430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 969907,
            "range": "± 3868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 969052,
            "range": "± 1543",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1366080,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1171677,
            "range": "± 2416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1852630,
            "range": "± 1747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 671981,
            "range": "± 12516",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 994432,
            "range": "± 18732",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 949439,
            "range": "± 1367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1834158,
            "range": "± 2203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1020147,
            "range": "± 2336",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1185530,
            "range": "± 2130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1605706,
            "range": "± 3097",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3533596,
            "range": "± 13925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1974190,
            "range": "± 11934",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1759386,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 819327,
            "range": "± 1957",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 686861,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 191214,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 246767,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19859,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 38770,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7688584,
            "range": "± 11324",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1900355,
            "range": "± 3663",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3307153,
            "range": "± 1610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1235325,
            "range": "± 3443",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2650322,
            "range": "± 1198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2687074,
            "range": "± 6550",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5759939,
            "range": "± 8324",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yjh465402634@gmail.com",
            "name": "yjh",
            "username": "yjhmelody"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5f4a853ee4eff365ab290d9835d2a16cdb1b8153",
          "message": "chore: fix typos (#761)",
          "timestamp": "2023-09-12T11:28:11+02:00",
          "tree_id": "0190988a55350e2386d9c537a96f630328e82a0a",
          "url": "https://github.com/paritytech/wasmi/commit/5f4a853ee4eff365ab290d9835d2a16cdb1b8153"
        },
        "date": 1694511200190,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5726345,
            "range": "± 18288",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100946154,
            "range": "± 178619",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 154228,
            "range": "± 508",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 220574,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 316097,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52776,
            "range": "± 462",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 847406,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 976294,
            "range": "± 1097",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 994681,
            "range": "± 2572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1367165,
            "range": "± 2717",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1173386,
            "range": "± 2351",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1878965,
            "range": "± 3307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 674092,
            "range": "± 8462",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1039685,
            "range": "± 8694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 988753,
            "range": "± 42673",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1894164,
            "range": "± 12758",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1036587,
            "range": "± 1406",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1199025,
            "range": "± 1057",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1641169,
            "range": "± 6348",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3470853,
            "range": "± 5347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1664208,
            "range": "± 1836",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1759004,
            "range": "± 1096",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 827169,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 686878,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 191516,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 246538,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19520,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39128,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7617756,
            "range": "± 12917",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1896154,
            "range": "± 3892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3310768,
            "range": "± 4960",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1235682,
            "range": "± 1540",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2648193,
            "range": "± 3223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2691618,
            "range": "± 3526",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5694027,
            "range": "± 7312",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "837bfdb7a0fc668b64a00ef56dd09187ef2ce7b8",
          "message": "Bump actions/checkout from 3 to 4 (#755)\n\nBumps [actions/checkout](https://github.com/actions/checkout) from 3 to 4.\r\n- [Release notes](https://github.com/actions/checkout/releases)\r\n- [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)\r\n- [Commits](https://github.com/actions/checkout/compare/v3...3df4ab11eba7bda6032a0b82a6bb43b11571feac)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: actions/checkout\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-major\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2023-09-12T14:54:50+02:00",
          "tree_id": "e24be1c24644d2b7f35f2f3ee07a8b22ae3efbc8",
          "url": "https://github.com/paritytech/wasmi/commit/837bfdb7a0fc668b64a00ef56dd09187ef2ce7b8"
        },
        "date": 1694523657322,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 5741601,
            "range": "± 5510",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 100564848,
            "range": "± 97646",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 155380,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 222031,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 317872,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54367,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 845451,
            "range": "± 1404",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 971990,
            "range": "± 1868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1007896,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1368316,
            "range": "± 2075",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1170415,
            "range": "± 1747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1851128,
            "range": "± 2210",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 658297,
            "range": "± 2233",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 964219,
            "range": "± 26318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 922741,
            "range": "± 817",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1799240,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1034596,
            "range": "± 1079",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1193583,
            "range": "± 1294",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1602933,
            "range": "± 1704",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3466594,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1668634,
            "range": "± 1248",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1759935,
            "range": "± 2435",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 814265,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 686287,
            "range": "± 867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 190184,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 245538,
            "range": "± 1108",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 19494,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39785,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 7578573,
            "range": "± 10459",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1898669,
            "range": "± 2904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3304854,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1230705,
            "range": "± 3380",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2650865,
            "range": "± 2878",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2687846,
            "range": "± 2567",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5697526,
            "range": "± 20897",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6fb940ffffbdcb693390048d14608385cc760a8c",
          "message": "WIP: Register machine `wasmi` execution engine (take 2) (#729)\n\n* remove no longer needed Instruction::ConstRef\r\n\r\n* improve some doc comments\r\n\r\n* rename RegisterSlice to RegisterSpan\r\n\r\n* initial implementation of Wasm call translation\r\n\r\n* refactor ProviderSliceAlloc to RegisterSliceAlloc\r\n\r\n* add proper translation for Wasm calls with more than 3 parameters\r\n\r\n* fix intra doc link\r\n\r\n* add Return2, Return3 and ReturnNez2 instructions\r\n\r\nThese are (probably) more efficient than their ReturnMany and ReturnNezMany respective counterparts because they store the returned registers inline.\r\n\r\n* add translation test for Wasm call translation\r\n\r\nThis also tests the new Return2 and Return3 instructions.\r\n\r\n* fix docs\r\n\r\n* refactor call instructions\r\n\r\nAll call instructions now uniformly require their parameters to be placed in contiguous register spans. This necessitates copy instructions before a call is initiated in some cases. Future plans include to optimise longer sequences of copy instructions but we left that optimisation out for now.\r\n\r\n* refactor return_call wasmi instructions\r\n\r\nThey now have the same form as their nested call counterparts.\r\n\r\n* remove commented out code\r\n\r\n* refactor call_indirect wasmi instructions\r\n\r\n* add InstrEncoder::encode_call_params helper method\r\n\r\n* add Wasm call_indirect translation\r\n\r\nThis is missing translation tests for now.\r\n\r\n* remove WIP todo!()\r\n\r\n* add Wasm call_indirect translation tests\r\n\r\n* make ReturnMany instruction use RegisterSpanIter\r\n\r\n* properly ignore some tests when #[cfg(miri)]\r\n\r\nThis fixes a bug with rust-analyzer not properly identifying some test files due to it enabling miri since some time.\r\n\r\n* make testcase Wasm input more readable\r\n\r\n* refactor ReturnNezMany to use RegisterSpan\r\n\r\nThis also removes the ReturnNezReg2 instruction in favor of the refactored ReturnNezMany.\r\n\r\n* reduce indentation of code a bit\r\n\r\n* add CopySpan instruction and use it when leaving block scopes\r\n\r\n* improve encoding of multiple copy instructions\r\n\r\n* add Wasm return_call[_indirect] translation\r\n\r\nTranslation unit tests are still missing.\r\n\r\n* fix return_call_indirect reachability in translation\r\n\r\n* add translation tests for return_call[_indirect]\r\n\r\n* add Wasm ref.is_null translation\r\n\r\n* remove register_buffer from FuncTranslatorAllocations\r\n\r\n* clear buffer from FuncTranslatorAllocations\r\n\r\n* remove some dead code\r\n\r\n* unsilence some warnings in regmach/mod.rs\r\n\r\n* add BlockType::{params_with, results_with} methods\r\n\r\n* improve Instruction constructors that take Const16<T> inputs\r\n\r\n* add RegisterSpanIter::is_empty method\r\n\r\n* add Wasm br_table translation\r\n\r\n* implement Wasm local.set and local.tee translation\r\n\r\nNo tests provided so far.\r\n\r\n* remove no longer used code\r\n\r\n* implement register defragmentation phase\r\n\r\nThis is required to defragment register space after local.set and local.tee preservations on the emulated value stack.\r\n\r\n* apply clippy suggestion\r\n\r\n* fix bugs in encoding of call instructions\r\n\r\n* fix bug in select instruction encoding\r\n\r\n* add some translation tests for local.set and local.tee\r\n\r\n* fix bug with local.set or local.tee chains\r\n\r\n* fix register defrag offset calculation\r\n\r\n* fix bug that certain copy instructions did not properly defrag\r\n\r\n* fix bug with local.set encoding with preservation\r\n\r\n* add more local.set and local.tee translation tests\r\n\r\n* impl Default for EngineBackend\r\n\r\n* rename FuncHead -> CompiledFuncEntity\r\n\r\n* rename field func_consts -> consts\r\n\r\n* add CompiledFuncEntity::len_cells method\r\n\r\n* improve doc comment\r\n\r\n* refactor regmach CodeMap\r\n\r\n* fix clippy and doc warnings\r\n\r\n* initial implementation of regmach stack\r\n\r\n* fix no_std build\r\n\r\n* apply rustfmt\r\n\r\n* add unsafe annotation to many functions in ValueStackPtr and ValueStack\r\n\r\n* add dev. docs to ValueStack::split\r\n\r\n* apply rustfmt\r\n\r\n* initial implementation of the regmach CallStack\r\n\r\n* add doc comment\r\n\r\n* remove doc comment (not part of PR)\r\n\r\n* use non-relative import\r\n\r\n* initial implementation of the register-machine executor\r\n\r\nAlready implementing some of the many wasmi bytecode executions.\r\n\r\n* implement branch instruction execution\r\n\r\n* add Executor::{get_register[_as], set_register} methods\r\n\r\n* implement copy instruction translation\r\n\r\n* improve TableIdx alignment for bytecode usage\r\n\r\n* refactor Call[Indirect]Params[Imm16] instructions\r\n\r\n* implement execution for internal calls\r\n\r\n* implement imported call execution\r\n\r\nCopying the parameters for host function calls is still missing.\r\n\r\n* add note comment to ValueStack::alloc_call_frame method\r\n\r\n* fix bug: re-instantiate live ValueStackPtr after allocating new call frame\r\n\r\n* add CallOutcome::call constructor\r\n\r\n* unify calling compiled funcs\r\n\r\n* update docs of CallIndirect[0] docs\r\n\r\n* refactor executor call implementation\r\n\r\n* implement indirect call execution\r\n\r\n* prepare executor for tail call implementation\r\n\r\n* implement tail call of internal functions\r\n\r\n* implement tail calling imported functions\r\n\r\n* implement tail calling functions indirectly\r\n\r\n* implement select instruction execution\r\n\r\n* implement ref.func execution\r\n\r\n* implement table.get execution\r\n\r\n* move table execution implementation to submodule\r\n\r\n* move select instruction execution to submodule\r\n\r\n* move call instruction execution to submodule\r\n\r\n* make set_register accept value: Into<UntypedValue>\r\n\r\n* add table.size instruction execution\r\n\r\n* fix bug skipping correct amount of instrs in table.get\r\n\r\n* implement table.set instruction execution\r\n\r\n* implement table.copy instruction execution\r\n\r\n* implement table.init execution\r\n\r\n* implement table.fill instruction execution\r\n\r\n* add EntityGrowError to crate::error\r\n\r\nThis previously was a dependency from crate::{table, memory} into crate::engine which is invalid and should not have happened. Was probably an oversight in the code review.\r\n\r\n* fix lifetime scrwes in impls\r\n\r\n* implement table.grow instruction execution\r\n\r\n* implemented elem.drop instruction execution\r\n\r\n* fix table.grow with delta == 0 return value\r\n\r\n* implement memory.{size, grow} and data.drop instruction execution\r\n\r\n* implement memory.copy instruction execution\r\n\r\n* fix instruction pointer update in memory.copy instruction execution\r\n\r\n* implement memory.fill instruction execution\r\n\r\n* fix docs\r\n\r\n* implement memory.init instruction execution\r\n\r\n* implement global.{get,set} instruction execution\r\n\r\n* implement load instruction execution\r\n\r\n* move fetch_address_offset to parent module\r\n\r\n* implement store instruction execution\r\n\r\n* implement unary instruction execution\r\n\r\n* move execute_unary to parent module\r\n\r\n* implement conversion instruction execution\r\n\r\n* move return instruction implementation to submodule\r\n\r\n* implement comparison instruction execution\r\n\r\n* reorder instruction variants in the executor\r\n\r\n* reorder more instruction variants in the executor\r\n\r\n* implement binary float instruction execution\r\n\r\n* implement integer binary instruction execution\r\n\r\n* implement shift and rotate instruction execution\r\n\r\n* make ValueStack::truncate take generic new_sp\r\n\r\n* fix bug in Executor::ret not dropping call frame values\r\n\r\n* account for executing frame to always be on the stack in Executor::ret\r\n\r\n* move ret into return_ submodule\r\n\r\n* move copy instruction execution to submodule and fix bugs\r\n\r\n* properly use execute_binary_imm16_rev helper method\r\n\r\n* refactor copy_call_params and fix bug with src/dst confusion\r\n\r\n* add missing global.get executor impl\r\n\r\n* fix bug in store_at[_imm16] executor implementation\r\n\r\n* unsilence warnings for the entire executor module\r\n\r\n* unsilence warnings for the stack sub-modules\r\n\r\nAlso remove dead code method.\r\n\r\n* refactor load executor implementation\r\n\r\n* refactor {Wasm,Call}Outcome\r\n\r\nRemoved the Instance field since it can be reconstructed given that the caller is now always guaranteed to be on top of the CallStack while calling a host function.\r\n\r\n* refactor handling of return instructions\r\n\r\n* move CallOutcome into call submodule\r\n\r\n* refactor return implementation\r\n\r\n- unified formerly duplicated code for returning single and multiple values\r\n- now respects that current call frame is always on the stack\r\n- properly implements returning from the root function call back to the host side\r\n- improved performance when popping too many frames to avoid borrow checking issues\r\n\r\n* fix i32 and i64 comparison instruction executions\r\n\r\n* fix dispatch of branch_eqz and branch_nez instructions\r\n\r\n* fix incorrect debug_assert condition\r\n\r\n* properly implement translate_end_unreachable\r\n\r\n* remove unneeded code\r\n\r\n* update the instruction pointer before dispatching a call\r\n\r\n* improve select param decode panic message\r\n\r\n* fix bug with instr_ptr increment of select instructions\r\n\r\n* add forgotten CallStack methods\r\n\r\nThese are required to properly update the instruction pointer before dispatching a call.\r\n\r\n* fix broken assert condition in ValueStack::fill_at\r\n\r\n* fix instr_ptr increment for store instructions\r\n\r\n* fix instr_ptr increment in some table instructions\r\n\r\n* apply rustfmt\r\n\r\n* fix table.init translation and execution for len=0\r\n\r\n* fix table.copy translation & execution for len=0\r\n\r\n* fix table.fill translation & execution for len=0\r\n\r\n* fix memory instr translation & execution with len=0\r\n\r\n* fix bug with executing memory.init with constant params\r\n\r\n* make spec testsuite test regmach engine\r\n\r\nWe ignore all tests that are currently failing and will un-ignore them one-by-one once they are fixed.\r\n\r\n* try: fix bench CI for this PR\r\n\r\nThis is caused by this rustc/LLVM bug: https://github.com/rust-lang/rust/issues/114725\r\n\r\n* fix div/rem translation with lhs=0\r\n\r\nRemoved an overly zealous peephole optimization with x/x -> 1 and x%x -> 0 since with x == 0 the Wasm standard mandates to trap anyways.\r\n\r\n* fix bug in execution of Instruction::Trap\r\n\r\n* fix bug with func local constant ordering\r\n\r\n* fix bug in EngineInner::get_func_const_2\r\n\r\n* fix float_exprs Wasm spec test\r\n\r\nThis was caused by some overzealous peephole optimizations for IEEE floats which could not be applied due to special case rules in the IEEE design, e.g. -0 + 0 -> 0.\r\n\r\n* fix end-of-then reachability in if without else\r\n\r\n* un-ignore \"block\" Wasm spectest\r\n\r\n* un-ignore \"labels\" Wasm spec test\r\n\r\n* un-ignore \"loop\" Wasm spec test\r\n\r\n* add TODO comments to failing Wasm spec tests\r\n\r\n* use try_next_instr_at from try_next_instr\r\n\r\n* fix translation for if(false) without else block\r\n\r\nIn this case the code after the if(false) without else block was mistakenly unreachable.\r\n\r\n* add TODO comments to failing Wasm spec tests\r\n\r\n* un-ignore call_indirect Wasm spec test\r\n\r\n* un-ignore \"binary\" Wasm spec test\r\n\r\n* fix bug in return_call_indirect decode phase\r\n\r\n* fix bug with tail call frame replacement\r\n\r\n* un-ignore tail call Wasm spec tests\r\n\r\n* add TODO comment to all failing Wasm spec tests\r\n\r\nThe TODO comments indicate what make them fail at the moment. With this we can identify which tests may have the same failure origin.\r\n\r\n* fix bug in [return_]call_indirect translation\r\n\r\nThe encoding of the call parameters could override the register that stores the indirect call's table index when the index was stored in the dynamic register space and the parameters had to be copied to form a register span.\r\n\r\n* update the remaining Wasm spec test comments\r\n\r\n* make EngineBackend::RegisterMachine the default\r\n\r\n* make StackMachine default again\r\n\r\n* apply clippy suggestions\r\n\r\n* apply rustfmt\r\n\r\n* fix if control flow translation\r\n\r\n* fix copy instruction encoding ordering\r\n\r\nThis fixes some instances were previous copy instructions overwrite inputs of following ones. We fixed this by ordering the encoded copy instructions after encoding. This may lead to O(n*log(n)) compilation times but this is only tied to Wasm multi-value proposal which is already kinda screwed with respect to linear time compilation.\r\n\r\n* guard against self-overlapping CopySpan (dbg mode)\r\n\r\n* rename overlap -> is_overlapping\r\n\r\n* use 1-indexing for Wasm spec testsuite errors\r\n\r\n* delay updating the cached instance for calls to imported funcs\r\n\r\n* remove unused CompiledFuncEntity::len_instrs field\r\n\r\n* remove unused CompiledFuncEntity::instr_ptr method\r\n\r\n* unsilence warnings in regmach code_map\r\n\r\n* move code_map::regmach into engine::regmach as code_map\r\n\r\nThis change will make it simpler to dissect stack machine engine implementation from register machine engine implementation.\r\n\r\n* move bytecode2 module into regmach as bytecode\r\n\r\nThis will make it simpler to dissect the stack machine engine implementation from the register machine engine implementation.\r\n\r\n* fix compile error due to last commit\r\n\r\n* reduce memory consumption of func translator\r\n\r\nWe achieve this by using an enum to store the func translator allocations of both stack-machine and register-machine since only ever one of them can be active at any time.\r\n\r\n* make stack machine specific translation tests use the correct engine\r\n\r\nPreviously these tests were using the default engine config which might invalidate those tests once the register-machine engine backend becomes the new default.\r\n\r\n* make benchmarks use the register machine backend\r\n\r\nThis should trigger a benchmark CI run on GitHub/GitLab.\r\n\r\n* use absolute import instead of relative\r\n\r\n* move engine::func_builder::regmach into engine::regmach\r\n\r\nThis helps to dissect stack-machine and register-machine engine implementations.\r\n\r\n* refactor ChosenFuncTranslatorAllocations\r\n\r\nNow hides its internal state so that we can remove the underlying FuncTranslatorAllocation types from the API of the engine submodule. Instead only the ChosenFuncTranslatorAllocations are now exported.\r\nAlso this commit removes unnecessary exports from the engine submodule API.\r\n\r\n* remove more unnecessary engine internal exports\r\n\r\n* remove unused code\r\n\r\n* unsilence warnings\r\n\r\n* rename RegisterSliceRef -> ProviderSliceRef\r\n\r\n* remove dead code from register allocator\r\n\r\n* unsilence dead_code warning in regmach::translator::stack submodule\r\n\r\n* remove outdated todo!()\r\n\r\nThis was used to indicate that we wanted to store the definition sites which has long been overhauled and is no longer on the table due to technical complications.\r\n\r\n* convert todo!() into unimplemented!()\r\n\r\n* remove unused imports in regmach test module\r\n\r\n* unsilence unused_imports in regmach test module\r\n\r\n* remove dead code in regmach test module\r\n\r\n* unsilence dead code warnings in regmach test module\r\n\r\n* run Wasm spec testsuite on both wasmi engine backends\r\n\r\n* refactor impl_visit_operator\r\n\r\n* refactor regmach::tests to prepare for separation\r\n\r\n* move regmach translation tests into regmach submodule\r\n\r\n* provide results: RegisterSpan information to host calls\r\n\r\n* fix ValueStack::as_slice taking &mut self instead of &self\r\n\r\n* add ValueStack::as_slice_mut method\r\n\r\n* implement non-root host function calls\r\n\r\n* add safety comments to unsafe blocks\r\n\r\n* implement root host function calls\r\n\r\n* move code sections closer together\r\n\r\n* make it possible to choose the execution engine in wasmi_cli\r\n\r\n* re-enable all benchmarks and use the stack-machine\r\n\r\n* silence incorrect clippy warning\r\n\r\n* only reorder copy instruction if they overwrite each other\r\n\r\n* optimize local.set preservation defragmentation\r\n\r\nWe do this by avoiding or at least limiting the procedure to a conservative subset of all instructions that could have been affected by the register space fragmentation.\r\n\r\n* fix bug in reset of notified_preservation in InstrEncoder\r\n\r\n* improve preservation notification API\r\n\r\n* reorder methods\r\n\r\n* introduce RegisterSpace abstraction\r\n\r\n* fix potential attack vector with local.get preservation\r\n\r\n* make wabt_example test pass again\r\n\r\n* remove unused method\r\n\r\n* add whitespace line\r\n\r\n* fix bug in ProviderStack::push_const_local\r\n\r\n* add dev comment\r\n\r\n* add warning to EngineBackend::RegisterMachine",
          "timestamp": "2023-09-21T11:55:15+02:00",
          "tree_id": "35fdcbd9a29c7d92ab18082240c7b0407846edf0",
          "url": "https://github.com/paritytech/wasmi/commit/6fb940ffffbdcb693390048d14608385cc760a8c"
        },
        "date": 1695290571076,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6544161,
            "range": "± 29842",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 116001220,
            "range": "± 192911",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 178739,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 258004,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 370355,
            "range": "± 1988",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55789,
            "range": "± 1059",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 715115,
            "range": "± 9012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 811886,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 834680,
            "range": "± 1352",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1359132,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1258573,
            "range": "± 3030",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1534548,
            "range": "± 3560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 720213,
            "range": "± 15298",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1049134,
            "range": "± 1904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 982401,
            "range": "± 636",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1667688,
            "range": "± 1098",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1106963,
            "range": "± 3172",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1258438,
            "range": "± 2274",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1715287,
            "range": "± 10561",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3228180,
            "range": "± 3611",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1596268,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1390776,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 679765,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 514771,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 142991,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 198657,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 13789,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39394,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6222645,
            "range": "± 7135",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1583817,
            "range": "± 1257",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2636323,
            "range": "± 1731",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1001848,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2282376,
            "range": "± 2043",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2305998,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 4806714,
            "range": "± 3722",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dc44551ac3ffc5bad352e0c3e9317ec6b2d1a46a",
          "message": "Fix `[return_]call_indirect` miscompilation (#773)\n\nfix [return_]call_indirect translation\r\n\r\nThis bug (issue #768) occurred when the call parameters needed to be copied over to a contiguous span in the dynamic register space and at the same time overwriting the index register.\r\nThe fix is to simply copy the index register to a protected register when detecting this situation.",
          "timestamp": "2023-09-26T11:44:06+02:00",
          "tree_id": "07702b41d98abc198251e2be85c4d0f1040f5abe",
          "url": "https://github.com/paritytech/wasmi/commit/dc44551ac3ffc5bad352e0c3e9317ec6b2d1a46a"
        },
        "date": 1695721913984,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6445661,
            "range": "± 6685",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 113818480,
            "range": "± 320202",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 174426,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 251533,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 359314,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55389,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 657534,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 804563,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 834992,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1300715,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1282696,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1525489,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 741259,
            "range": "± 25132",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1052018,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 980982,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1610755,
            "range": "± 3149",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1127984,
            "range": "± 1551",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1257120,
            "range": "± 752",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1657151,
            "range": "± 14336",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3186433,
            "range": "± 2905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1712502,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1341242,
            "range": "± 1834",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 667441,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 504039,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 145599,
            "range": "± 5593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 192521,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 13209,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 40138,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6261696,
            "range": "± 15057",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1480957,
            "range": "± 4809",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2635938,
            "range": "± 1722",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 991933,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2306759,
            "range": "± 1244",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2250850,
            "range": "± 2237",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 4634307,
            "range": "± 1804",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e1c473110e20a5d053ce049ebca04c1d3ca3b371",
          "message": "Update Wasm proposal support in README (#778)\n\n* replace WASI state with scientist emoji\r\n\r\n* update Wasm proposal support in README\r\n\r\n* add issue links to README\r\n\r\n* apply clippy suggestions (new nightly)\r\n\r\n* apply rustdoc fixes\r\n\r\n* fix trunc_f2i benchmark .wat file\r\n\r\n* update wast in benches\r\n\r\n* downgrade wast dependency to 64.0 again\r\n\r\n* attempt to fix cargo fuzz CI job",
          "timestamp": "2023-10-16T13:58:52+02:00",
          "tree_id": "3c98e785c9cd66e1e58f775b9bd862379239dd17",
          "url": "https://github.com/paritytech/wasmi/commit/e1c473110e20a5d053ce049ebca04c1d3ca3b371"
        },
        "date": 1697457946498,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 7149928,
            "range": "± 31092",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 126109107,
            "range": "± 231994",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 191686,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 277733,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 398291,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55276,
            "range": "± 924",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 610629,
            "range": "± 2861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 756466,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 799710,
            "range": "± 1672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1072205,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1301347,
            "range": "± 2617",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1533019,
            "range": "± 1479",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 737136,
            "range": "± 7119",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1040858,
            "range": "± 2042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 984244,
            "range": "± 858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1666645,
            "range": "± 4563",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1142104,
            "range": "± 3227",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1276718,
            "range": "± 1684",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1715731,
            "range": "± 7563",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3238508,
            "range": "± 6145",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1348391,
            "range": "± 7331",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1221715,
            "range": "± 1054",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 672196,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 473183,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 144235,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190807,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 13892,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 41460,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6083800,
            "range": "± 10431",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1414157,
            "range": "± 2031",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2348389,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 987569,
            "range": "± 1390",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1971571,
            "range": "± 584",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2078609,
            "range": "± 4331",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 4243883,
            "range": "± 3269",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "185f33b631a47a316be0a080b05a35ca688a38ad",
          "message": "Bump actions/checkout from 4.0.0 to 4.1.0 (#770)\n\nBumps [actions/checkout](https://github.com/actions/checkout) from 4.0.0 to 4.1.0.\r\n- [Release notes](https://github.com/actions/checkout/releases)\r\n- [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)\r\n- [Commits](https://github.com/actions/checkout/compare/3df4ab11eba7bda6032a0b82a6bb43b11571feac...8ade135a41bc03ea155e62e844d188df1ea18608)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: actions/checkout\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-minor\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2023-10-16T14:21:31+02:00",
          "tree_id": "0d805187bc6a7c399fdea3d78f865693ac658c5e",
          "url": "https://github.com/paritytech/wasmi/commit/185f33b631a47a316be0a080b05a35ca688a38ad"
        },
        "date": 1697459401791,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6746979,
            "range": "± 56612",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 117377532,
            "range": "± 708240",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 178246,
            "range": "± 1830",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 255754,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 364548,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54747,
            "range": "± 1322",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 641199,
            "range": "± 1318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 763450,
            "range": "± 1617",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 801587,
            "range": "± 4444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1074720,
            "range": "± 4711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1274960,
            "range": "± 5725",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1548040,
            "range": "± 12550",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 762493,
            "range": "± 10943",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1073564,
            "range": "± 2078",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 997255,
            "range": "± 3831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1650518,
            "range": "± 18579",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1181873,
            "range": "± 6305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1352282,
            "range": "± 10666",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1787179,
            "range": "± 9835",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3283717,
            "range": "± 15642",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1345507,
            "range": "± 4684",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1230669,
            "range": "± 7534",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 676565,
            "range": "± 5869",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 474770,
            "range": "± 2469",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 144196,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190748,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 13896,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 40861,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6437786,
            "range": "± 6494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1377409,
            "range": "± 5971",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2353864,
            "range": "± 8763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 990368,
            "range": "± 2525",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1974015,
            "range": "± 2916",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2014945,
            "range": "± 7448",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 4271055,
            "range": "± 12154",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3334ab582eb889b35dbdd263f71365db9a540bad",
          "message": "Bump actions/checkout from 4.1.0 to 4.1.1 (#780)\n\nBumps [actions/checkout](https://github.com/actions/checkout) from 4.1.0 to 4.1.1.\r\n- [Release notes](https://github.com/actions/checkout/releases)\r\n- [Changelog](https://github.com/actions/checkout/blob/main/CHANGELOG.md)\r\n- [Commits](https://github.com/actions/checkout/compare/8ade135a41bc03ea155e62e844d188df1ea18608...b4ffde65f46336ab88eb53be808477a3936bae11)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: actions/checkout\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-patch\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2023-10-18T19:45:03+02:00",
          "tree_id": "6109a82560924d88b0cc3942da036a00a501d8f1",
          "url": "https://github.com/paritytech/wasmi/commit/3334ab582eb889b35dbdd263f71365db9a540bad"
        },
        "date": 1697651606819,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6472900,
            "range": "± 36212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 113687910,
            "range": "± 258913",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 174965,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 252326,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 357532,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55858,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 609748,
            "range": "± 4026",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 761930,
            "range": "± 590",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 798090,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1072474,
            "range": "± 549",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1264257,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1532927,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 733366,
            "range": "± 7261",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1043007,
            "range": "± 2054",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 983599,
            "range": "± 1442",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1635476,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1142741,
            "range": "± 3850",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1261659,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1700618,
            "range": "± 11416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3254158,
            "range": "± 7303",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1344198,
            "range": "± 2402",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1221446,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 671250,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 473142,
            "range": "± 2956",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 143698,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190211,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 13840,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 40691,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6082907,
            "range": "± 4758",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1375556,
            "range": "± 1175",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2347454,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 983281,
            "range": "± 1158",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1970425,
            "range": "± 2270",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2005170,
            "range": "± 1916",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 4241243,
            "range": "± 4639",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65688808fad51385657c3978950817b07aa78534",
          "message": "Fix copy instruction for many cases (#783)\n\n* add RegisterSpan::iter_u16 method\r\n\r\nThis is a more efficient variant of RegisterSpan::iter.\r\n\r\n* use RegisterSpan::iter_u16 were possible\r\n\r\n* minor fix of docs of CopySpan instruction\r\n\r\n* add Instruction::CopySpanRev\r\n\r\nThis commit does not include translation to actually emit CopySpanRev.\r\n\r\n* fix bug in encode_local_set\r\n\r\nReplaced debug_assert with if-conditional and added explanatory comment.\r\n\r\n* disable merging of copies\r\n\r\nWe disable merging of copies since it is a hard problem to dissect and resolve overlapping copy instructions and in some cases it is even impossible. Having merged copies makes this even harder. We need a new way to encode copies to circumvent this scenario in its entirety.\r\n\r\n* assert no overlapping copies after sort\r\n\r\nThe problem with this debug assert is that the sorting does not guarantee that there are no overlapping copies but just makes it less likely in most common cases. We need a new approach to handle copies to properly fix this case.\r\n\r\n* remove unneeded API\r\n\r\n* adjust tests for recent changes (no more copy merge)\r\n\r\n* fix bug\r\n\r\n* remove CopySpanRev\r\n\r\nNo longer needed.\r\n\r\n* fix import warnings\r\n\r\n* apply clippy suggestion\r\n\r\n* fix cargo doc issue",
          "timestamp": "2023-11-09T10:55:24+01:00",
          "tree_id": "09fa4588e9f6a4eebaf1268b7d0d985fb9f12d6a",
          "url": "https://github.com/paritytech/wasmi/commit/65688808fad51385657c3978950817b07aa78534"
        },
        "date": 1699524211394,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 7072061,
            "range": "± 50096",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 120743071,
            "range": "± 295463",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 182788,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 262718,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 375558,
            "range": "± 2045",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56782,
            "range": "± 1603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 679214,
            "range": "± 8683",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 861847,
            "range": "± 2958",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 893667,
            "range": "± 1568",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1294182,
            "range": "± 3891",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1267138,
            "range": "± 7433",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1718475,
            "range": "± 10369",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 758178,
            "range": "± 6535",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1103809,
            "range": "± 5102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1096844,
            "range": "± 7367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1850753,
            "range": "± 5554",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1056658,
            "range": "± 8047",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1263127,
            "range": "± 4926",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1600227,
            "range": "± 4242",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3241457,
            "range": "± 9957",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1682705,
            "range": "± 6298",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1331501,
            "range": "± 6461",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 748660,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 629402,
            "range": "± 14102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 171230,
            "range": "± 2174",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 240190,
            "range": "± 2222",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17756,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 41002,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6985013,
            "range": "± 101437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1692004,
            "range": "± 18408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2825005,
            "range": "± 10164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1146753,
            "range": "± 4601",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2478540,
            "range": "± 5677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2465180,
            "range": "± 14715",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5342141,
            "range": "± 36929",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "0@mcornholio.ru",
            "name": "Yuri Volkov",
            "username": "mutantcornholio"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b7a3ef4742225e02b04f749cdb62eca1ffe87b31",
          "message": "Adding gitspiegel-trigger workflow (#781)\n\n* Adding gitspiegel-trigger workflow\r\n\r\nUsing a workflow to trigger mirroring instead of a webhook allows us to reuse \"Approving workflow runs from public forks\" GitHub feature to somewhat protect us from malicious PRs\r\n\r\n* Update gitspiegel-trigger.yml\r\n\r\n---------\r\n\r\nCo-authored-by: Robin Freyler <robin.freyler@gmail.com>",
          "timestamp": "2023-11-09T11:41:59+01:00",
          "tree_id": "c71a078b777064ed479d3db624a5712b9bd03120",
          "url": "https://github.com/paritytech/wasmi/commit/b7a3ef4742225e02b04f749cdb62eca1ffe87b31"
        },
        "date": 1699526913736,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6540747,
            "range": "± 11135",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115271619,
            "range": "± 212328",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 175993,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 252873,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 361435,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55641,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 671841,
            "range": "± 8350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 859175,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 889151,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1287109,
            "range": "± 3404",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1214135,
            "range": "± 1823",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1716184,
            "range": "± 843",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 794504,
            "range": "± 14983",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1118349,
            "range": "± 3792",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1027672,
            "range": "± 1677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1893162,
            "range": "± 8149",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1103809,
            "range": "± 2094",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1302472,
            "range": "± 21254",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1621927,
            "range": "± 4356",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3289087,
            "range": "± 11410",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1686009,
            "range": "± 4190",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1330294,
            "range": "± 1910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 760312,
            "range": "± 2995",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 607626,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 169854,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 228388,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17175,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 45360,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6939611,
            "range": "± 6236",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1634444,
            "range": "± 28794",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2796791,
            "range": "± 3877",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1135545,
            "range": "± 3344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2458504,
            "range": "± 2679",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2445746,
            "range": "± 2573",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5246307,
            "range": "± 10438",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "194359f69ef2efe3141cb1a25362364e3ca8c86d",
          "message": "Avoid updating `instr_ptr` for tail calls (#785)\n\navoid updating instr_ptr for tail calls\r\n\r\nThis avoids updating the instruction pointer of the current call frame upon a tail call since this call frame is going to be discarded upon a tail call.",
          "timestamp": "2023-11-13T13:44:03+01:00",
          "tree_id": "7f16a7fbdfa6d7a84f9de0a0bbda2780cb6d11ed",
          "url": "https://github.com/paritytech/wasmi/commit/194359f69ef2efe3141cb1a25362364e3ca8c86d"
        },
        "date": 1699879901298,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6543447,
            "range": "± 76855",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 114961238,
            "range": "± 983957",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 173410,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 251094,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 355643,
            "range": "± 633",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57505,
            "range": "± 1747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 666470,
            "range": "± 4249",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 871403,
            "range": "± 1213",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 892175,
            "range": "± 9302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1302088,
            "range": "± 5806",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1199335,
            "range": "± 1589",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1731444,
            "range": "± 3610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 774952,
            "range": "± 2314",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1061038,
            "range": "± 2219",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1043948,
            "range": "± 1169",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1889118,
            "range": "± 4798",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1066209,
            "range": "± 5394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1240647,
            "range": "± 1787",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1651799,
            "range": "± 6986",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3306434,
            "range": "± 13127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1689956,
            "range": "± 10068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1331704,
            "range": "± 2167",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 759098,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 583380,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 172823,
            "range": "± 1183",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 230060,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17199,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39083,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6762172,
            "range": "± 16395",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1698523,
            "range": "± 2851",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2919703,
            "range": "± 3249",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1138923,
            "range": "± 1882",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2461547,
            "range": "± 6646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2448418,
            "range": "± 2432",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5263788,
            "range": "± 7787",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "72be93b3598a995efbd47df715a8910403402586",
          "message": "Implement new copy semantics (#784)\n\n* rename Return[Nez]Many to Return[Nez]Span\r\n\r\n* fix spelling issue\r\n\r\n* adjust Instruction type for new copy encoding\r\n\r\nThis adds some instructions or instruction parameters to account for the new planned copy semantics that replace consecutive copy instructions with a single instruction that handles all the necessary copying between registers.\r\nThis will fix a bug that cannot be fixed with the current copy semantics that involves unavoidable overlapping copy instructions.\r\n\r\nNewly added instruction parameters are:\r\n- Register2\r\n- Register3\r\n- RegisterList\r\n\r\nNewly added instructions are:\r\n- ReturnReg2\r\n- ReturnReg3\r\n- ReturnMany\r\n- ReturnNezReg2\r\n- ReturnNezMany\r\n- Copy2\r\n- CopyMany\r\n\r\nRemoving one instruction parameter:\r\n- CallParams\r\n\r\nAlso the PR associated to this commit will adjust call instruction parameter encodings.\r\n\r\nThis commit does not include execution implementations or execution implementation adjustments of the newly added or changed instructions respectively.\r\n\r\n* implement execution of Instruction::Copy2\r\n\r\n* implement execution of Instruction::CopyMany\r\n\r\n* implement Instruction::ReturnReg{2,3} execution\r\n\r\n* implement Instruction::ReturnNezReg2 execution\r\n\r\n* replace path with use\r\n\r\n* clean up of new copy execution implementation\r\n\r\n* implement Instruction::Return[Nez]Many execution\r\n\r\n* implement new call param copy semantics\r\n\r\nAdjustments for instruction pointer updates is still missing that needs to be altered since amount of parameters is only discovered upon call param copying during execution and no longer before.\r\n\r\n* no longer update instruction pointer for tail calls\r\n\r\nThis is not needed since the caller call frame is discarded during the operation anyways.\r\n\r\n* remove ResolvedCallIndirectParams type\r\n\r\n* improve panic message\r\n\r\n* properly update instruction pointer on non-tail calls\r\n\r\n* apply rustfmt\r\n\r\n* add InstructionPtr::pull and use it where applicable\r\n\r\n* refactor fetch_call_indirect_params\r\n\r\n* refactor CallIndirectParams\r\n\r\nAlso move the Instruction variants to the other instruction parameter variants.\r\n\r\n* add constructors for new Instruction::Register{1,2,3,List}\r\n\r\n* add constructors for new instructions\r\n\r\n* adjust call_[imported,internal] parameter encoding\r\n\r\n* adjust Instruction::[return_]call_indirect encoding\r\n\r\n* adjust Instruction::[return_]call translation and tests\r\n\r\n* implement new encoding for return instructions\r\n\r\n* remove no longer needed Instruction::CallParams\r\n\r\n* adjust br_if encoding when conditionally returning\r\n\r\n* implement new copy semantics for copy instructions\r\n\r\n* remove invalid update to instr_ptr in call execution\r\n\r\n* fix panic message\r\n\r\n* remove InstructionPtr::pull method\r\n\r\nIts use is discouraged since misusing it caused a bug in copying of call parameters.\r\n\r\n* clean up call execution implementation a bit\r\n\r\n* respect overlapping copy spans in execution implementation\r\n\r\n* minor cleanup\r\n\r\n* add copy_span instruction variant for non-overlapping spans\r\n\r\nThe non-overlapping copy_span variants can easily precomputed at compile time and does not require a temporary buffer to avoid invalid copy overwrites.\r\n\r\n* add Instruction::CopyManyNonOverlapping variant\r\n\r\nThis is an optimized version of Instruction::CopyMany that does not require to store its values into a temporary buffer since it assumes that both results and values do not overlap. This assumption is asserted during compilation.\r\n\r\n* rename test\r\n\r\n* improve copy_span overlap detection\r\n\r\nThe new function does a better job at detecting actual copy overlaps. The former function checked if both spans overlapped without respecting the copy operation that only needs to check if the values are overwritten by the results.\r\n\r\n* implement host function parameter passing\r\n\r\nParameter passing for host functions called from root are not yet implemented.\r\n\r\n* add tests for calling host functions from the host side\r\n\r\n* implement host function calling from host side through executor",
          "timestamp": "2023-11-16T20:21:53+01:00",
          "tree_id": "615c956981bd6a3576b0d40328c1da88bcb7a155",
          "url": "https://github.com/paritytech/wasmi/commit/72be93b3598a995efbd47df715a8910403402586"
        },
        "date": 1700162989961,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6639926,
            "range": "± 23618",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115133505,
            "range": "± 89776",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 175603,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 254058,
            "range": "± 3960",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 360725,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57198,
            "range": "± 1232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 681703,
            "range": "± 5606",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 862454,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 896475,
            "range": "± 3852",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1280863,
            "range": "± 3244",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1193701,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1720868,
            "range": "± 4786",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 757721,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1054849,
            "range": "± 1917",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1037562,
            "range": "± 1572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1783383,
            "range": "± 3401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1114052,
            "range": "± 2513",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1284395,
            "range": "± 3733",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1681723,
            "range": "± 6724",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3267977,
            "range": "± 6729",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1684684,
            "range": "± 6647",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1331068,
            "range": "± 1506",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 762324,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 579901,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 170636,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 229084,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17128,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39893,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6674281,
            "range": "± 7414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1634309,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2800292,
            "range": "± 2370",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1135745,
            "range": "± 2966",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2467020,
            "range": "± 5355",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2448239,
            "range": "± 3650",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5258022,
            "range": "± 3953",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7911bd2c728832cdbd136c28f29bfb298b2ef1b8",
          "message": "Refactor `local.set` register preservation (#786)\n\nrefactor local.set register preservation\r\n\r\nThis fixes a bug in preservation of `local.set` for local variables that have been pushed multiple times onto the stack upon preservation and implements recycling of preservation slots to reduce register pressure.",
          "timestamp": "2023-11-19T13:54:48+01:00",
          "tree_id": "4d2b80dccf7cb1e5824b9c378ccf1d24a319f080",
          "url": "https://github.com/paritytech/wasmi/commit/7911bd2c728832cdbd136c28f29bfb298b2ef1b8"
        },
        "date": 1700398889630,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6420094,
            "range": "± 32469",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 113569405,
            "range": "± 97356",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 173135,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 249099,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 354974,
            "range": "± 741",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54808,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 695256,
            "range": "± 3702",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 862293,
            "range": "± 982",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 888098,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1301533,
            "range": "± 1084",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1189187,
            "range": "± 1656",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1702904,
            "range": "± 2525",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 770704,
            "range": "± 9819",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1091065,
            "range": "± 1628",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1057399,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1794701,
            "range": "± 3124",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1075201,
            "range": "± 1437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1248909,
            "range": "± 2258",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1661841,
            "range": "± 2153",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3356043,
            "range": "± 4411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1680625,
            "range": "± 8766",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1342515,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 763646,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 574302,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 169295,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 227363,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17110,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 38249,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6687815,
            "range": "± 8950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1636984,
            "range": "± 1633",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2776896,
            "range": "± 3773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1115242,
            "range": "± 1000",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2395981,
            "range": "± 2680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2450610,
            "range": "± 1644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5268512,
            "range": "± 12628",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d1bc10cfce3f5df91540c0cc6b76d88f09ef8343",
          "message": "bugfix: clear preservations when resetting register alloc (#787)",
          "timestamp": "2023-11-19T14:10:17+01:00",
          "tree_id": "f427db050ca56752cc9b33deabf17bb29ad5a6e3",
          "url": "https://github.com/paritytech/wasmi/commit/d1bc10cfce3f5df91540c0cc6b76d88f09ef8343"
        },
        "date": 1700399743231,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6524370,
            "range": "± 12043",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115516683,
            "range": "± 368786",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 175360,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 251630,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 360541,
            "range": "± 3324",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54875,
            "range": "± 1223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 698892,
            "range": "± 1103",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 865230,
            "range": "± 929",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 899020,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1312451,
            "range": "± 1315",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1238768,
            "range": "± 2064",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1717035,
            "range": "± 1154",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 766955,
            "range": "± 5122",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1119327,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1049704,
            "range": "± 968",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1886410,
            "range": "± 4350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1076957,
            "range": "± 2918",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1273750,
            "range": "± 3008",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1638255,
            "range": "± 26430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3339859,
            "range": "± 9176",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 1695130,
            "range": "± 10232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 1378431,
            "range": "± 1074",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 757684,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 578172,
            "range": "± 2222",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 170697,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 226384,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17185,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39644,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 6786644,
            "range": "± 6369",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1623928,
            "range": "± 2845",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 2778455,
            "range": "± 1280",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1135047,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 2428458,
            "range": "± 7869",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 2375361,
            "range": "± 7926",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 5332414,
            "range": "± 3902",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "eb0f1aa1c24dae37b8c468617a8e391abd885223",
          "message": "Implement cmp+branch instruction fusion (#789)\n\n* add 16-bit BranchOffset16 utility type\r\n\r\n* move new BranchOffset16 to regmach module\r\n\r\n* create new utility BranchBinOpInstr[Imm] types\r\n\r\n* add fused cmp+branch instructions\r\n\r\nTranslation and tests has not yet been implemented in this commit.\r\n\r\n* fixed signedness of some branch_cmp_imm instructions\r\n\r\n* implement fused cmp+branch_nez instruction translation\r\n\r\n* remove invalid debug_assert\r\n\r\n* add minimal test for fused cmp+branch instruction translation\r\n\r\n* change count_until.wat benchmark to allow for fused cmp+branch\r\n\r\n* fix bug in InstrEncoder::encode_branch_nez\r\n\r\n* more fixes for the same bug\r\n\r\n* add another test\r\n\r\n* fix bug with default encoding\r\n\r\n* special fusing cmp+br with cmp={eq,ne} and rhs=0\r\n\r\n* rename internal function\r\n\r\n* make cmp+branch fusion possible for uninit offsets\r\n\r\n* add TODO comment for future\r\n\r\n* do not fuse cmp+branch if cmp stores into a local\r\n\r\n* apply rustfmt\r\n\r\n* no longer optimize local.set when result is a local\r\n\r\nAlso reformat the code using the new more readable let-else syntax.\r\n\r\n* apply manual less verbose formatting\r\n\r\n* separate reg and imm variants in cmp+branch fusion\r\n\r\n* implement branch_eqz cmp+branch fusion\r\n\r\n* add some more cmp+branch fusion translation tests\r\n\r\n* extend new loop_backward test\r\n\r\n* apply rustfmt\r\n\r\n* extend another test case to be more generic\r\n\r\n* extend another test\r\n\r\n* extend block_forward test\r\n\r\n* extend block_forward_no_copy test\r\n\r\n* extend if_forward_multi_value test\r\n\r\n* extend if_forward test",
          "timestamp": "2023-11-20T23:21:17+01:00",
          "tree_id": "668b68aa0d29657cc735319d70a2bd6e8aaa0243",
          "url": "https://github.com/paritytech/wasmi/commit/eb0f1aa1c24dae37b8c468617a8e391abd885223"
        },
        "date": 1700519323761,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6663042,
            "range": "± 10615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 117078712,
            "range": "± 141574",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 177887,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 256532,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 365728,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58422,
            "range": "± 1117",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1108422,
            "range": "± 9645",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1141535,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1230823,
            "range": "± 1594",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1753930,
            "range": "± 1811",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1158027,
            "range": "± 1254",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1729376,
            "range": "± 4196",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 798361,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 970032,
            "range": "± 1516",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1046065,
            "range": "± 1836",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1935745,
            "range": "± 8015",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1131320,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1257993,
            "range": "± 1521",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1648780,
            "range": "± 9680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3404927,
            "range": "± 14979",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 2190906,
            "range": "± 879",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 2383613,
            "range": "± 7593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 974788,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 799183,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 200147,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 286624,
            "range": "± 8784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20848,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39322,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 8771127,
            "range": "± 6866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 2173865,
            "range": "± 9858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3829752,
            "range": "± 11605",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1446599,
            "range": "± 23646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 3277260,
            "range": "± 2851",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3292338,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 7349302,
            "range": "± 8536",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ea3a29ce66ca116489b7bbcec520c5415ace17b2",
          "message": "Allow `local.set` optimization with active preservation (#792)\n\n* allow local.set optimization with active preservation\r\n\r\n* apply rustfmt",
          "timestamp": "2023-11-21T21:50:11+01:00",
          "tree_id": "cd2247bb3352086ef17e87d39f927b85211d9447",
          "url": "https://github.com/paritytech/wasmi/commit/ea3a29ce66ca116489b7bbcec520c5415ace17b2"
        },
        "date": 1700600308792,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6760511,
            "range": "± 12976",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 117601635,
            "range": "± 227636",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 180831,
            "range": "± 345",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 260520,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 370865,
            "range": "± 1224",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55812,
            "range": "± 878",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1118896,
            "range": "± 1818",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1140169,
            "range": "± 1475",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1232352,
            "range": "± 3442",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 1701593,
            "range": "± 5659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1135493,
            "range": "± 1329",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1712115,
            "range": "± 512",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 807387,
            "range": "± 3661",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 974178,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1047578,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1891919,
            "range": "± 6812",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1110947,
            "range": "± 2547",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1253069,
            "range": "± 1518",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1656765,
            "range": "± 1338",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3534254,
            "range": "± 7544",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 2178993,
            "range": "± 4681",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 2442822,
            "range": "± 5274",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 970006,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 802753,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 200009,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 284608,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20858,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39881,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 8605619,
            "range": "± 4482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 2196261,
            "range": "± 2745",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3807721,
            "range": "± 11378",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1445358,
            "range": "± 3950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 3285530,
            "range": "± 5096",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3320212,
            "range": "± 2845",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 7392607,
            "range": "± 3651",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "91a956b581c6ebe9c002f572236c1515ceb7d3eb",
          "message": "Fuse `i32.{and,or, xor}` + [`i32.eqz`] + `br_if` Wasm instructions (#796)\n\n* add i32.branch_{and,or,xor}[_imm] instructions\r\n\r\nThere is no need for i64 counterparts since in Wasm only i32 types are used as conditional \"bool\" types.\r\n\r\n* add i32.branch_{nand, nor, xnor}[_imm] instructions\r\n\r\nWe added these instruction to provide optimizations for encode_eqz.\r\n\r\n* rename new branch instructions\r\n\r\n* add fusion of i32.{and,or,xor} + i32.eqz\r\n\r\n* add forgotten i32.{and,or,xor}+i32.eqz+branch translations\r\n\r\n* add fuse benchmark to showcase perf gains\r\n\r\n* bump count_until limit to make it less noisy\r\n\r\n* fix bug in executor for new fuse instructions\r\n\r\n* add i32.{and,or,xor} + i32.eqz fusion tests\r\n\r\n* add i32.{and,or,xor} + i32.eqz + br_if fuse tests",
          "timestamp": "2023-11-24T17:10:58+01:00",
          "tree_id": "571c75c0a78ff4718186ef0640059a66e303cdfe",
          "url": "https://github.com/paritytech/wasmi/commit/91a956b581c6ebe9c002f572236c1515ceb7d3eb"
        },
        "date": 1700842710351,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6613907,
            "range": "± 9746",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 116849394,
            "range": "± 605391",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 178372,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 259090,
            "range": "± 1882",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 367259,
            "range": "± 3420",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57129,
            "range": "± 373",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1094233,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1139359,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1233542,
            "range": "± 2797",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 17136961,
            "range": "± 34937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1184006,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1713966,
            "range": "± 1390",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 800874,
            "range": "± 4327",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1011948,
            "range": "± 1109",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1064970,
            "range": "± 1170",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1938417,
            "range": "± 1134",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1165854,
            "range": "± 4649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1306827,
            "range": "± 2899",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1693603,
            "range": "± 7464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3423167,
            "range": "± 12213",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 2154938,
            "range": "± 18828",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 2439485,
            "range": "± 1132",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 973522,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 802136,
            "range": "± 532",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 199834,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 286097,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20849,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39742,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 34511604,
            "range": "± 28561",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 8597082,
            "range": "± 4094",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 2193921,
            "range": "± 1205",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3809668,
            "range": "± 5388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1445471,
            "range": "± 1028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 3287408,
            "range": "± 1510",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3347234,
            "range": "± 1811",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 7396385,
            "range": "± 2665",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5bea360983e4c7f382bd77ef243c1902113e2d4c",
          "message": "Add `Instruction::branch_i64_{eqz,nez}` instructions (#797)\n\n* add Instruction::branch_i64_{eqz,nez} instructions\r\n\r\n* add tests\r\n\r\n* apply rustfmt\r\n\r\n* apply clippy suggestions",
          "timestamp": "2023-11-24T19:30:59+01:00",
          "tree_id": "1911de31f70aaf89b4a52c600c78b380c3c33ef8",
          "url": "https://github.com/paritytech/wasmi/commit/5bea360983e4c7f382bd77ef243c1902113e2d4c"
        },
        "date": 1700851101797,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6600422,
            "range": "± 30332",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115684142,
            "range": "± 306812",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 178568,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 258640,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 368849,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58189,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1103064,
            "range": "± 3038",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1144622,
            "range": "± 1967",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1263652,
            "range": "± 5559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 16771923,
            "range": "± 43465",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1235157,
            "range": "± 1390",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1715803,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 845487,
            "range": "± 45598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1087647,
            "range": "± 1578",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1061311,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1933630,
            "range": "± 2610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1176516,
            "range": "± 4645",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1365923,
            "range": "± 3466",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1728645,
            "range": "± 12373",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3498590,
            "range": "± 17171",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 2210634,
            "range": "± 8593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 2435699,
            "range": "± 12456",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 980186,
            "range": "± 4714",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 804226,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 210089,
            "range": "± 10538",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 286670,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20929,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 40578,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 34372861,
            "range": "± 25203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 8722819,
            "range": "± 49311",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 2176081,
            "range": "± 36811",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3838963,
            "range": "± 3465",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1454579,
            "range": "± 3819",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 3299471,
            "range": "± 4781",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3303202,
            "range": "± 4735",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 7341621,
            "range": "± 8821",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}