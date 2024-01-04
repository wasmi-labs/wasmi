window.BENCHMARK_DATA = {
  "lastUpdate": 1704365369774,
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
      },
      {
        "commit": {
          "author": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "committer": {
            "email": "robin.freyler@gmail.com",
            "name": "Robin Freyler",
            "username": "Robbepop"
          },
          "distinct": true,
          "id": "606d1cfcd5c80a3ea58611e4e84e33e8a6e19684",
          "message": "remove result_mut impl since it is no longer used",
          "timestamp": "2023-11-24T22:21:42+01:00",
          "tree_id": "40443203fe0f1cfac5cb2d167bded6094f0527c9",
          "url": "https://github.com/paritytech/wasmi/commit/606d1cfcd5c80a3ea58611e4e84e33e8a6e19684"
        },
        "date": 1700861383648,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6580102,
            "range": "± 6474",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115405661,
            "range": "± 158624",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 176514,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 255618,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 363585,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58034,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1095391,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1146587,
            "range": "± 847",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1229851,
            "range": "± 735",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 17498259,
            "range": "± 8960",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1146558,
            "range": "± 2091",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1708440,
            "range": "± 1159",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 799176,
            "range": "± 19052",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 972068,
            "range": "± 1873",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1038275,
            "range": "± 2058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1874865,
            "range": "± 6658",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1103052,
            "range": "± 4533",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1234182,
            "range": "± 1412",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1628311,
            "range": "± 3060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 3098777,
            "range": "± 8288",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 2190257,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 2422143,
            "range": "± 5390",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 969233,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 798991,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 201811,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 290533,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20888,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 39347,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 34209599,
            "range": "± 8650",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 8492476,
            "range": "± 6333",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 2172643,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 3826899,
            "range": "± 3769",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 1472755,
            "range": "± 6929",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 3280088,
            "range": "± 2323",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 3289768,
            "range": "± 3239",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 7356223,
            "range": "± 20307",
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
          "id": "ad02b0930ecbe12562e197bca1ba11a97de319ae",
          "message": "Clean up benchmarks a bit (#801)\n\n* add new translation benchmark tests\r\n\r\n* clean up benchmarks",
          "timestamp": "2023-11-24T22:40:49+01:00",
          "tree_id": "4aaa0dc8552254d13ce258fb314711643f3530ea",
          "url": "https://github.com/paritytech/wasmi/commit/ad02b0930ecbe12562e197bca1ba11a97de319ae"
        },
        "date": 1700862455073,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6603283,
            "range": "± 32636",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 115713639,
            "range": "± 357951",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 4845381,
            "range": "± 36464",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 1830322,
            "range": "± 13809",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 177121,
            "range": "± 1435",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 256132,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 363042,
            "range": "± 1920",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56715,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1131190,
            "range": "± 3088",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1141929,
            "range": "± 1538",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1239670,
            "range": "± 1774",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 16154996,
            "range": "± 77203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1231699,
            "range": "± 1527",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1719457,
            "range": "± 3110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1784206,
            "range": "± 11199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 2393331,
            "range": "± 9503",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 978428,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 798026,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 200512,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 287016,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20919,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 40948,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 34384189,
            "range": "± 32539",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 8657279,
            "range": "± 14390",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 2209611,
            "range": "± 2275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 3840818,
            "range": "± 10672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1452006,
            "range": "± 2724",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 3275393,
            "range": "± 2163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 3330807,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 7365684,
            "range": "± 7170",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 808978,
            "range": "± 4996",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1967423,
            "range": "± 3260",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1165173,
            "range": "± 2356",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3379422,
            "range": "± 6906",
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
          "id": "933f179cea16bf2e1dd64b1c1ae7f000dd9eb7a7",
          "message": "Clean up `relink_result` impl (#802)\n\ncleanup relink_result impl",
          "timestamp": "2023-11-24T23:01:16+01:00",
          "tree_id": "0966c8b9e2ce8d185cdfe492b7856abea0a22f5f",
          "url": "https://github.com/paritytech/wasmi/commit/933f179cea16bf2e1dd64b1c1ae7f000dd9eb7a7"
        },
        "date": 1700863757650,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 6526574,
            "range": "± 11286",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 114808766,
            "range": "± 140049",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 4790440,
            "range": "± 6880",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 1812746,
            "range": "± 1782",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 174189,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 251999,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 357538,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58210,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 1099483,
            "range": "± 3877",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 1137968,
            "range": "± 937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1235428,
            "range": "± 3005",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 17550068,
            "range": "± 25632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1143649,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1708503,
            "range": "± 2169",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 2191286,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 2390855,
            "range": "± 5087",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 966731,
            "range": "± 1276",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 801195,
            "range": "± 2200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 200396,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 286844,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 20910,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 39420,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 34203876,
            "range": "± 12624",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 8587808,
            "range": "± 4427",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 2173503,
            "range": "± 1467",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 3831285,
            "range": "± 5740",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1447389,
            "range": "± 1587",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 3277607,
            "range": "± 2760",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 3290398,
            "range": "± 3532",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 7350054,
            "range": "± 6924",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 871407,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1919734,
            "range": "± 10416",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1229876,
            "range": "± 918",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3278064,
            "range": "± 12180",
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
          "id": "13dd96169fd9a8d5e5b19e2016ecdf9f0d0386ae",
          "message": "Use register-machine `wasmi` in benchmarks (#804)\n\nuse register-machine wasmi in benchmarks",
          "timestamp": "2023-11-24T23:18:37+01:00",
          "tree_id": "9d5f325262aa373b33dc890fd586260d46394636",
          "url": "https://github.com/paritytech/wasmi/commit/13dd96169fd9a8d5e5b19e2016ecdf9f0d0386ae"
        },
        "date": 1700864759474,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 8629410,
            "range": "± 8670",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 153905854,
            "range": "± 263192",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 6405301,
            "range": "± 16475",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 2454422,
            "range": "± 13243",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 226437,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 328203,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 473139,
            "range": "± 1771",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56946,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 366376,
            "range": "± 810",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 624034,
            "range": "± 1760",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 993795,
            "range": "± 2836",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7501264,
            "range": "± 1172",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1277725,
            "range": "± 1683",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 885477,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1512182,
            "range": "± 1236",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 747437,
            "range": "± 397",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1253546,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 298458,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 338922,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 361174,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 33200,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 39524,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12432636,
            "range": "± 14826",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12648213,
            "range": "± 63423",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3548123,
            "range": "± 3188",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1432526,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2138894,
            "range": "± 17496",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1209474,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1239341,
            "range": "± 1567",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3477347,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 858137,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1932226,
            "range": "± 2618",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1351518,
            "range": "± 1206",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3728296,
            "range": "± 5350",
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
          "id": "834c3fbe313d41e8ddf25f5f2e25b07500605568",
          "message": "Remove superseeded conditional branch instructions (#805)\n\n* add TryFrom<BranchOffset> for BranchOffset16\r\n\r\n* move From impl\r\n\r\n* remove superseeded branch+cmp instructions\r\n\r\n* fix intra doc links\r\n\r\n* refactor implementation of cmp+br instructions\r\n\r\n* apply rustfmt\r\n\r\n* reduce column noise",
          "timestamp": "2023-11-25T11:06:28+01:00",
          "tree_id": "e5547a54cf8c230c29c170b98213f0412694410e",
          "url": "https://github.com/paritytech/wasmi/commit/834c3fbe313d41e8ddf25f5f2e25b07500605568"
        },
        "date": 1700907219072,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 8603188,
            "range": "± 25539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 153592240,
            "range": "± 660164",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 6412659,
            "range": "± 23803",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 2416523,
            "range": "± 16035",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 226688,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 326723,
            "range": "± 1868",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 472080,
            "range": "± 1319",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54178,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 382216,
            "range": "± 1371",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 636029,
            "range": "± 1910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1048558,
            "range": "± 2753",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7626210,
            "range": "± 8437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1305353,
            "range": "± 2285",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 951929,
            "range": "± 4058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1602389,
            "range": "± 5502",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 743775,
            "range": "± 1606",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1306448,
            "range": "± 3973",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 307382,
            "range": "± 585",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 352844,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 372381,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34737,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 41077,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12571886,
            "range": "± 14816",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13056085,
            "range": "± 29544",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3640890,
            "range": "± 14089",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1443353,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2218027,
            "range": "± 4925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1227290,
            "range": "± 3257",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1229319,
            "range": "± 1189",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3559041,
            "range": "± 6231",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 850261,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1961459,
            "range": "± 4631",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1296699,
            "range": "± 4233",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3760432,
            "range": "± 12119",
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
          "id": "80e1d212a26da8950f11fa6c0812bcc70661c3ee",
          "message": "Implement minor branch optimization in executor (#806)\n\n* minor optimization in executor\r\n\r\n* apply rustfmt\r\n\r\n* move utility methods into submodule",
          "timestamp": "2023-11-25T12:03:50+01:00",
          "tree_id": "7a6df8c42c4615c7e4fb08660770082e19ec7906",
          "url": "https://github.com/paritytech/wasmi/commit/80e1d212a26da8950f11fa6c0812bcc70661c3ee"
        },
        "date": 1700910684908,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 8701069,
            "range": "± 32052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 154398666,
            "range": "± 574517",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 6558934,
            "range": "± 34951",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 2422402,
            "range": "± 8768",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 227696,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 327611,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 472334,
            "range": "± 1712",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56138,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 385950,
            "range": "± 590",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 639217,
            "range": "± 978",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1040024,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7627026,
            "range": "± 8019",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1303293,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 949172,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1602995,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 744274,
            "range": "± 803",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1321885,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 317919,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 356193,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 377229,
            "range": "± 1247",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34973,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 39849,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12567821,
            "range": "± 11647",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13088570,
            "range": "± 23860",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3620852,
            "range": "± 3058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1443494,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2220978,
            "range": "± 3404",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1469524,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1372303,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3560223,
            "range": "± 2098",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 858683,
            "range": "± 2294",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1984322,
            "range": "± 2119",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1302589,
            "range": "± 3677",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3749744,
            "range": "± 2921",
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
          "id": "30ab9885e3510d0fd9062c49b018d8d7b817ed47",
          "message": "Add fuel metering translation to benchmarks (#813)\n\nadd fuel metering translation to benchmarks",
          "timestamp": "2023-11-28T21:53:08+01:00",
          "tree_id": "9e0aaa82314b3bcb7f7ec5273cef2bf74a1f786b",
          "url": "https://github.com/paritytech/wasmi/commit/30ab9885e3510d0fd9062c49b018d8d7b817ed47"
        },
        "date": 1701205265938,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8576205,
            "range": "± 16718",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 8596029,
            "range": "± 9361",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 152652780,
            "range": "± 176699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 152826004,
            "range": "± 244923",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6372443,
            "range": "± 10744",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6387077,
            "range": "± 6418",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2411441,
            "range": "± 5533",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2429054,
            "range": "± 3545",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 223992,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 226137,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 324589,
            "range": "± 856",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 325918,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 468134,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 470261,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54918,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 389091,
            "range": "± 2681",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 636606,
            "range": "± 1394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1033576,
            "range": "± 1397",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7532828,
            "range": "± 8468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1262060,
            "range": "± 1213",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 937869,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1602012,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748788,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1338678,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 296598,
            "range": "± 286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 352453,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 371043,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36024,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 39268,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12634987,
            "range": "± 16216",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12738442,
            "range": "± 9232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3621998,
            "range": "± 1518",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1303545,
            "range": "± 1240",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2197264,
            "range": "± 1845",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1226457,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1228000,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3618766,
            "range": "± 10116",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 843594,
            "range": "± 1404",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2058731,
            "range": "± 12030",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1294576,
            "range": "± 3342",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3873117,
            "range": "± 4768",
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
          "id": "52f180c240fc2f4640ab99b54e3798767b797d80",
          "message": "Implement fuel metering for the register-machine `wasmi` engine backend (#810)\n\n* refactor FuelCosts\r\n\r\n- Remove fields and fields accesses by methods.\r\n- Add new methods that make more sense for the new register-machine wasmi engine backend.\r\n- Adjusted both stack-machine and register-machine to the new model.\r\n- Reinvent register-machine translation API for fuel metering.\r\n\r\n* refactor and cleanup of new fuel metering API\r\n\r\n* fix intra doc link\r\n\r\n* move fuel methods around\r\n\r\n* change docs a bit\r\n\r\n* improve docs for field\r\n\r\n* fix formatting in docs\r\n\r\n* use InstrEncoder::append_instr in more places where it makes sense\r\n\r\n* add fuel metering to call, table, memory and global instructions\r\n\r\n* add fuel metering for select instructions\r\n\r\n* add fuel metering for load and store instructions\r\n\r\n* add fuel metering for binary instructions\r\n\r\n* add fuel metering for unary and conversion instructions\r\n\r\n* add fuel metering for return instructions\r\n\r\n* add fuel metering for local.set\r\n\r\nlocal.set also depends on fuel metering for copy instructions which is not yet implemented.\r\n\r\n* add fuel metering to copy instructions\r\n\r\nAdditionally refactored encode_copies to reuse encode_copy for single value copies.\r\nAlso refactor implementation of encode_copy a bit.\r\n\r\n* fuel metering impl cleanups\r\n\r\n- Rename consume_fuel to fuel_instr.\r\n- Use fuel_instr helper method where possible.\r\n\r\n* account total registers used per function for fuel metering\r\n\r\n* add comment for loop consume fuel instructions\r\n\r\n* add FuelInfo type to clean up the fuel metering implementation",
          "timestamp": "2023-11-28T22:20:22+01:00",
          "tree_id": "3f010c247881e247097c9583e8fbd92f3716c699",
          "url": "https://github.com/paritytech/wasmi/commit/52f180c240fc2f4640ab99b54e3798767b797d80"
        },
        "date": 1701206884867,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8822927,
            "range": "± 31419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9229360,
            "range": "± 11878",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 158609266,
            "range": "± 318545",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 166977500,
            "range": "± 687798",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6596567,
            "range": "± 15129",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6917612,
            "range": "± 17716",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2522957,
            "range": "± 10826",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2679487,
            "range": "± 7350",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 235397,
            "range": "± 879",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 246203,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 336191,
            "range": "± 1621",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 353278,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 488904,
            "range": "± 1731",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 520922,
            "range": "± 1959",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54913,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 382316,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 644865,
            "range": "± 2122",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1042061,
            "range": "± 882",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7521579,
            "range": "± 12733",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1343491,
            "range": "± 3668",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 942844,
            "range": "± 1794",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1601318,
            "range": "± 1030",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750020,
            "range": "± 1851",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1363798,
            "range": "± 4314",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 298373,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 364989,
            "range": "± 4063",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 383881,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34745,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 38849,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12668227,
            "range": "± 9963",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13597333,
            "range": "± 6452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3827688,
            "range": "± 4127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1269082,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2285882,
            "range": "± 6341",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1253499,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1237438,
            "range": "± 679",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3594562,
            "range": "± 3591",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 916887,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1960171,
            "range": "± 4264",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1457971,
            "range": "± 7769",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3878525,
            "range": "± 4161",
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
          "id": "11f51bed853e239c1d239f52dba44b47f5732904",
          "message": "Implement resumable calls via register-machine `wasmi` engine backend (#814)\n\n* implement resumable calling via register-machine engine\r\n\r\nThis is not tested properly and may not work, yet.\r\n\r\n* try to fix fuzzing CI\r\n\r\n* fix bug for older rust toolchain (bench CI)\r\n\r\n* make special smoldot tail resume tests pass under regmach\r\n\r\n* apply rustfmt\r\n\r\n* remove outdated comment\r\n\r\nTails calls work when calling host functions in the new register-machine engine.\r\n\r\n* add TestData to resumable calls tests\r\n\r\nAlso use Linker::func_wrap.\r\nNew TestData is unused, yet.\r\n\r\n* test resumable calls for both engine backends",
          "timestamp": "2023-11-30T12:28:57+01:00",
          "tree_id": "86ce8e9fb0dc5773f594e2facc3118906f6f1b51",
          "url": "https://github.com/paritytech/wasmi/commit/11f51bed853e239c1d239f52dba44b47f5732904"
        },
        "date": 1701344220969,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8671350,
            "range": "± 19262",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9077388,
            "range": "± 12485",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 156974588,
            "range": "± 406533",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 164124123,
            "range": "± 527893",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6514889,
            "range": "± 21038",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6851074,
            "range": "± 15714",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2469942,
            "range": "± 5230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2625856,
            "range": "± 10798",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 227024,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 238791,
            "range": "± 6764",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 326620,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 344544,
            "range": "± 2567",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 477332,
            "range": "± 2557",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 507268,
            "range": "± 1212",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54840,
            "range": "± 821",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 394537,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 634951,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1024436,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7594292,
            "range": "± 26576",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1334943,
            "range": "± 893",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 942131,
            "range": "± 1378",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1596827,
            "range": "± 644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 747412,
            "range": "± 817",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1424751,
            "range": "± 1844",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 316560,
            "range": "± 461",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 370204,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 397651,
            "range": "± 11774",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35979,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 38250,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12904835,
            "range": "± 8714",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13543567,
            "range": "± 9462",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4085135,
            "range": "± 15263",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1179279,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2321343,
            "range": "± 15876",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1201474,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1215181,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3513597,
            "range": "± 3794",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 891343,
            "range": "± 1496",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2134992,
            "range": "± 8866",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1360488,
            "range": "± 1692",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4048976,
            "range": "± 4664",
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
          "id": "e2323b2d38f7d83bc182250695cae192cbe6dd01",
          "message": "Remove the stack-machine `wasmi` engine backend (#818)\n\n* remove EngineBackend and conditionals of users\r\n\r\nAlso remove tests testing the stack-machine wasmi engine backend which is about to be removed since those tests can no longer be run.\r\n\r\n* remove code directly related to the stack-machine backend\r\n\r\nNo post-removal clean ups performed in this commit.\r\n\r\n* fix doclinks and minor renamings\r\n\r\n* make caller_results non optional\r\n\r\nThe None variant was only needed for the removed stack-machine engine backend.",
          "timestamp": "2023-12-01T09:22:00+01:00",
          "tree_id": "312a9201791565e618352c0d52bc848be3ae6e1d",
          "url": "https://github.com/paritytech/wasmi/commit/e2323b2d38f7d83bc182250695cae192cbe6dd01"
        },
        "date": 1701419374135,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8676131,
            "range": "± 22325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9124131,
            "range": "± 35415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 157095588,
            "range": "± 225091",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 164782620,
            "range": "± 490225",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6556664,
            "range": "± 24387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6891080,
            "range": "± 20118",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2512688,
            "range": "± 8478",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2663089,
            "range": "± 9662",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 229691,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 240475,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 331801,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 347285,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 489491,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 524607,
            "range": "± 1949",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54724,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 392533,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 641499,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1027770,
            "range": "± 9110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7483633,
            "range": "± 4838",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1312149,
            "range": "± 1014",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 977031,
            "range": "± 1119",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1623070,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 749916,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1288257,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 325389,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 346365,
            "range": "± 4044",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 366456,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34156,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66853,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11536687,
            "range": "± 5534",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12572323,
            "range": "± 14894",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3680475,
            "range": "± 5414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1173430,
            "range": "± 3244",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2157415,
            "range": "± 4950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1227527,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1217415,
            "range": "± 1498",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3401879,
            "range": "± 10620",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 819122,
            "range": "± 915",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1859871,
            "range": "± 3710",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1274756,
            "range": "± 2382",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3889465,
            "range": "± 7779",
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
          "id": "3ac1af977a2b726be0067939231e91df1dd4b86c",
          "message": "Post stack-machine `wasmi` engine backend removal cleanup (#820)\n\n* remove empty trap.rs file\r\n\r\n* refactor resumable call stack usage\r\n\r\nThis refactoring was needed when there were still 2 different wasmi backend engines. Now the situation is simpler again so we can reverse the original refactoring.\r\n\r\n* rename method\r\n\r\n* rename a bunch of Engine and EngineInner methods\r\n\r\n* perform more renamings of engine internals\r\n\r\n* rename function param\r\n\r\n* reorder fields in TaggedTrap::Host variant\r\n\r\n* remove unused TranslationErrorInner variant\r\n\r\n* move bytecode submodule up into engine\r\n\r\n* rename Instruction2 import to just Instruction\r\n\r\n* use UntypedValue wasmi re-export\r\n\r\n* move code_map.rs up into engine module\r\n\r\n* move trap.rs up into engine module\r\n\r\n* move EngineInner executor impls into executor submodule\r\n\r\n* fix intra doc links\r\n\r\n* move trap.rs into executor submodule\r\n\r\n* move stack submodule into executor submodule\r\n\r\n* move executor up into the engine module\r\n\r\n* re-export FuncTranslatorAllocations without alias\r\n\r\n* move non-translation tests into engine/tests submodule\r\n\r\n* move translation tests into translator submodule\r\n\r\n* move translator submodule up into engine module\r\n\r\n* remove EngineInner forwarding methods\r\n\r\n* rename FuncBuilder to ValidatingFuncTranslator",
          "timestamp": "2023-12-02T13:07:06+01:00",
          "tree_id": "57706dd363c1120483f20e1f8616de92573afaec",
          "url": "https://github.com/paritytech/wasmi/commit/3ac1af977a2b726be0067939231e91df1dd4b86c"
        },
        "date": 1701519237589,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8743516,
            "range": "± 14594",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9196887,
            "range": "± 11280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 158208350,
            "range": "± 201165",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 166045886,
            "range": "± 191403",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6540946,
            "range": "± 10255",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6846516,
            "range": "± 11833",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2500495,
            "range": "± 7521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2660782,
            "range": "± 6904",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 228575,
            "range": "± 1164",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 240747,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 330858,
            "range": "± 1120",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 346392,
            "range": "± 1591",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 477674,
            "range": "± 709",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 508682,
            "range": "± 1314",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55317,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 447704,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 644958,
            "range": "± 574",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1008196,
            "range": "± 1209",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7499727,
            "range": "± 12108",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1308928,
            "range": "± 2296",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 945551,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1590498,
            "range": "± 1865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746236,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1323227,
            "range": "± 2882",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 324650,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 356541,
            "range": "± 1274",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 380496,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35649,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66727,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11499662,
            "range": "± 9279",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12993138,
            "range": "± 9742",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3691456,
            "range": "± 3775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1169365,
            "range": "± 717",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2251821,
            "range": "± 6295",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1226940,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1217740,
            "range": "± 1490",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3390114,
            "range": "± 2493",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 820756,
            "range": "± 1307",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1996623,
            "range": "± 3879",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1262658,
            "range": "± 2303",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3979667,
            "range": "± 12033",
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
          "id": "0f4eea96be63061ca1471587a25516f48af16414",
          "message": "Some more minor (forgotten) cleanups (#821)\n\n* rename compiled_funcs_2 field\r\n\r\n* remove unnecessary get_compiled_func_2 method",
          "timestamp": "2023-12-02T13:34:43+01:00",
          "tree_id": "0f7a3d76fa3684543800c561f7259821b66a578b",
          "url": "https://github.com/paritytech/wasmi/commit/0f4eea96be63061ca1471587a25516f48af16414"
        },
        "date": 1701520928428,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8625128,
            "range": "± 20627",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9027139,
            "range": "± 22031",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 155275564,
            "range": "± 209097",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 162795821,
            "range": "± 371560",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6454635,
            "range": "± 15323",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6756856,
            "range": "± 10223",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2466076,
            "range": "± 5645",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2624608,
            "range": "± 7185",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 225490,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 237368,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 326713,
            "range": "± 2229",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 342779,
            "range": "± 897",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 474141,
            "range": "± 1891",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 504943,
            "range": "± 1624",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52916,
            "range": "± 1248",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 446920,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 648028,
            "range": "± 460",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1014745,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7499983,
            "range": "± 2914",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1377963,
            "range": "± 1805",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 945713,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1590558,
            "range": "± 647",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746178,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1323507,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 323108,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 345104,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 369675,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 33747,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66691,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11487635,
            "range": "± 5201",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12985421,
            "range": "± 8904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3649447,
            "range": "± 3563",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1169654,
            "range": "± 758",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2185464,
            "range": "± 6347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1226933,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1216480,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3390631,
            "range": "± 3419",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 930768,
            "range": "± 3493",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2026722,
            "range": "± 4701",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1269194,
            "range": "± 17229",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4027343,
            "range": "± 8762",
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
          "id": "6abc9510fa7fd3ec0cb3e624d8f5a1dc4c650c10",
          "message": "cleanup parameters in translate (#822)\n\n* cleanup parameters in translate\r\n\r\n* apply rustfmt",
          "timestamp": "2023-12-02T17:57:16+01:00",
          "tree_id": "3dd467e81065b5b8daaada362e9194393faef390",
          "url": "https://github.com/paritytech/wasmi/commit/6abc9510fa7fd3ec0cb3e624d8f5a1dc4c650c10"
        },
        "date": 1701536687054,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8633873,
            "range": "± 15744",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9061358,
            "range": "± 21667",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 155105655,
            "range": "± 369677",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 161899448,
            "range": "± 354479",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6532586,
            "range": "± 34168",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6819751,
            "range": "± 20719",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2464720,
            "range": "± 5156",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2613324,
            "range": "± 11670",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 226864,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 237530,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 327582,
            "range": "± 1252",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 342127,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 475425,
            "range": "± 4141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 502788,
            "range": "± 11072",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56556,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 447591,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 646807,
            "range": "± 9332",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1006143,
            "range": "± 431",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7502046,
            "range": "± 3863",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1297548,
            "range": "± 1063",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 945570,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1591129,
            "range": "± 567",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746166,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1339442,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 322977,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 346203,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 370337,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 33961,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66628,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11477564,
            "range": "± 10486",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12986002,
            "range": "± 33369",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3739358,
            "range": "± 3638",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1170263,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2188784,
            "range": "± 2779",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1227328,
            "range": "± 858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1217678,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3394565,
            "range": "± 1558",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 820278,
            "range": "± 2365",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1960232,
            "range": "± 8759",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1224469,
            "range": "± 2226",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4028389,
            "range": "± 9179",
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
          "id": "b02e23d478f79158c07c6b068bf91f388185e0a9",
          "message": "Fix bug in new register-machine executor (#824)\n\nfix bug in executor",
          "timestamp": "2023-12-02T21:56:18+01:00",
          "tree_id": "6cc1dfe1befca2d9865aeee3c0c04fb90b46727c",
          "url": "https://github.com/paritytech/wasmi/commit/b02e23d478f79158c07c6b068bf91f388185e0a9"
        },
        "date": 1701551026694,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8710569,
            "range": "± 48013",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9075650,
            "range": "± 19712",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 157597772,
            "range": "± 750850",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 163164485,
            "range": "± 393604",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6567033,
            "range": "± 18565",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6790765,
            "range": "± 10489",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2464489,
            "range": "± 11416",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2620936,
            "range": "± 11248",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 228169,
            "range": "± 1280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 237671,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 328961,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 343813,
            "range": "± 1271",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 481605,
            "range": "± 1370",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 506475,
            "range": "± 1596",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53778,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 449662,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 661553,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1005806,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7489516,
            "range": "± 9867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1425833,
            "range": "± 5062",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 937692,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1622343,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750166,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1320089,
            "range": "± 4135",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 322111,
            "range": "± 1672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 348497,
            "range": "± 968",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 373777,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35218,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 69430,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11557054,
            "range": "± 12481",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13146555,
            "range": "± 15670",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3616113,
            "range": "± 4374",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1175699,
            "range": "± 2389",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2215832,
            "range": "± 4983",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1226867,
            "range": "± 3504",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1216680,
            "range": "± 1723",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3403208,
            "range": "± 3887",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 807244,
            "range": "± 2002",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1975659,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1192971,
            "range": "± 2450",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3852273,
            "range": "± 5462",
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
          "id": "ed8ce84baca52913b295deef5617fc813fd0940a",
          "message": "Refactor bytecode const utilities (#826)\n\n* refactor AnyConst{16,32}, Const16<T> and Const32<T> APIs\r\n\r\nThey now offer all their API via From and TryFrom impls if possible.\r\n\r\n* apply rustfmt\r\n\r\n* generalize Const16::is_zero method\r\n\r\n* fix SAFETY comment",
          "timestamp": "2023-12-03T13:25:04+01:00",
          "tree_id": "7006cc205d8c03c717f433391386e07915153d18",
          "url": "https://github.com/paritytech/wasmi/commit/ed8ce84baca52913b295deef5617fc813fd0940a"
        },
        "date": 1701606662278,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8556388,
            "range": "± 19978",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 8953547,
            "range": "± 34429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 154388120,
            "range": "± 381576",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 161624943,
            "range": "± 566852",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6432375,
            "range": "± 29221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6751094,
            "range": "± 59237",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2411874,
            "range": "± 5574",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2570423,
            "range": "± 9670",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 224593,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 235968,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 325214,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 339099,
            "range": "± 1319",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 469119,
            "range": "± 1434",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 497636,
            "range": "± 2120",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54090,
            "range": "± 1146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 449701,
            "range": "± 438",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 643480,
            "range": "± 710",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1007901,
            "range": "± 1826",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7484276,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1328792,
            "range": "± 1833",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 939076,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1622703,
            "range": "± 897",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 749753,
            "range": "± 1021",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1321578,
            "range": "± 4155",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 320799,
            "range": "± 1034",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 347825,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 372722,
            "range": "± 1132",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34215,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 67044,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11558116,
            "range": "± 9519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12930823,
            "range": "± 14167",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3633848,
            "range": "± 6382",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1173749,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2211180,
            "range": "± 2736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1224515,
            "range": "± 1507",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1215713,
            "range": "± 1271",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3399701,
            "range": "± 4508",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 815615,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1973463,
            "range": "± 3813",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1220603,
            "range": "± 1737",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3848846,
            "range": "± 5926",
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
          "id": "42371ac0b5235ca02ebda7836c6781a989e2c858",
          "message": "Add `divrem` benchmark test (#827)\n\n* fix naming in fure benchmark test\r\n\r\n* add benchmark test for divrem",
          "timestamp": "2023-12-03T13:57:16+01:00",
          "tree_id": "1c82c377181d8b369dcf98b253bd8aa65e4c54fb",
          "url": "https://github.com/paritytech/wasmi/commit/42371ac0b5235ca02ebda7836c6781a989e2c858"
        },
        "date": 1701608694193,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8398947,
            "range": "± 15234",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 8811219,
            "range": "± 19142",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 151842938,
            "range": "± 486692",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 159247288,
            "range": "± 482626",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6302971,
            "range": "± 11935",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6613000,
            "range": "± 15581",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2394161,
            "range": "± 11636",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2552044,
            "range": "± 11624",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 219460,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 231991,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 316770,
            "range": "± 2044",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 334281,
            "range": "± 514",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 460904,
            "range": "± 2262",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 493237,
            "range": "± 2741",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58474,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 390988,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 629699,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1031547,
            "range": "± 1653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7474671,
            "range": "± 5075",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1324189,
            "range": "± 2163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 942011,
            "range": "± 436",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1672209,
            "range": "± 84881",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 766328,
            "range": "± 2680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1318258,
            "range": "± 2154",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 328106,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 356394,
            "range": "± 4523",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 372787,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35058,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66041,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11476968,
            "range": "± 9848",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6792524,
            "range": "± 11518",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13306782,
            "range": "± 28097",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3699970,
            "range": "± 8402",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1187402,
            "range": "± 1571",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2227622,
            "range": "± 6833",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1240744,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1228899,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3394921,
            "range": "± 1778",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 806020,
            "range": "± 2430",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2040715,
            "range": "± 4304",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1229176,
            "range": "± 1837",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4000022,
            "range": "± 2134",
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
          "id": "63b1a63169c44b287eeb2ef4ef2fda1ac9cee3b4",
          "message": "Optimize divrem with non-zero immediate `rhs` values (#825)\n\n* optimize divrem with non-zero immediate rhs values\r\n\r\n* rename DivRemImm -> DivRemExt\r\n\r\n* add docs to DivRemExt trait\r\n\r\n* use macro to generate divrem instr constructors\r\n\r\n* refactor AnyConst{16,32}, Const16<T> and Const32<T> APIs\r\n\r\nThey now offer all their API via From and TryFrom impls if possible.\r\n\r\n* generalize Const16::is_zero method\r\n\r\n* fix SAFETY comment\r\n\r\n* fix naming in fure benchmark test\r\n\r\n* add benchmark test for divrem",
          "timestamp": "2023-12-03T14:28:54+01:00",
          "tree_id": "35592de261aa93522f3f8f91edee0ec2198e5ceb",
          "url": "https://github.com/paritytech/wasmi/commit/63b1a63169c44b287eeb2ef4ef2fda1ac9cee3b4"
        },
        "date": 1701610593949,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8660109,
            "range": "± 23496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 9104143,
            "range": "± 31457",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 155140415,
            "range": "± 400812",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 161895650,
            "range": "± 215102",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6458644,
            "range": "± 16830",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6765870,
            "range": "± 23259",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2628132,
            "range": "± 13126",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2634411,
            "range": "± 21189",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 223896,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 236738,
            "range": "± 1344",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 323360,
            "range": "± 873",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 338377,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 469352,
            "range": "± 1362",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 503315,
            "range": "± 1894",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54881,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 381364,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 640588,
            "range": "± 859",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1074854,
            "range": "± 2277",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7491960,
            "range": "± 5336",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1300716,
            "range": "± 15755",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 955455,
            "range": "± 1666",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1541684,
            "range": "± 6552",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750532,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1277974,
            "range": "± 1125",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 315112,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 359285,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 391761,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35860,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64703,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11606412,
            "range": "± 23260",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6967230,
            "range": "± 10486",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13048519,
            "range": "± 12225",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3704184,
            "range": "± 5295",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1269556,
            "range": "± 1528",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2267028,
            "range": "± 3032",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1231557,
            "range": "± 1766",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1316309,
            "range": "± 1659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3622447,
            "range": "± 7940",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 861408,
            "range": "± 3247",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2066121,
            "range": "± 3350",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1323261,
            "range": "± 4114",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3859566,
            "range": "± 10160",
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
          "id": "5d4ea7b24648def2ef5b7be188f428a1546b45a2",
          "message": "Introduce `WasmTranslator` trait (#828)\n\n* create new WasmTranslator trait\r\n\r\n* make ValidatingFuncTranslator::current_pos private\r\n\r\n* impl WasmTranslator for ValidatingFuncTranslator\r\n\r\n* rename binding",
          "timestamp": "2023-12-03T15:54:00+01:00",
          "tree_id": "7334f84d6947ff6ec3802cd399f5bfc8e5ad976f",
          "url": "https://github.com/paritytech/wasmi/commit/5d4ea7b24648def2ef5b7be188f428a1546b45a2"
        },
        "date": 1701615636253,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 8565262,
            "range": "± 17105",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 8993257,
            "range": "± 16270",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 152717052,
            "range": "± 285131",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 160537534,
            "range": "± 381579",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 6381779,
            "range": "± 35416",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 6705574,
            "range": "± 17761",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 2438214,
            "range": "± 7215",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 2586697,
            "range": "± 5225",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 227375,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 237892,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 327271,
            "range": "± 1578",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 343731,
            "range": "± 804",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 470890,
            "range": "± 1326",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 503435,
            "range": "± 1079",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56281,
            "range": "± 891",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 378819,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 639885,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1046179,
            "range": "± 2538",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7512514,
            "range": "± 4543",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1272646,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961942,
            "range": "± 588",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1537983,
            "range": "± 1216",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750823,
            "range": "± 2828",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1246580,
            "range": "± 3007",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 315526,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 356471,
            "range": "± 589",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 388106,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35187,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64367,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11422453,
            "range": "± 9489",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6971110,
            "range": "± 10286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13106857,
            "range": "± 32559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3539308,
            "range": "± 4649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1263204,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2258886,
            "range": "± 4741",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1251198,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1318891,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3610693,
            "range": "± 2888",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 866644,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1936912,
            "range": "± 3701",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1225383,
            "range": "± 1181",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3880731,
            "range": "± 9853",
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
          "id": "1ce23700e1cdf46d8c770bb7e46955b277311847",
          "message": "Implement `Module::new_unchecked` (#829)\n\n* use derive(Default) for ReusableAllocations\r\n\r\n* use Self::Allocations\r\n\r\n* make many FuncTranslator methods private\r\n\r\nNone of them needed to be public. This was a simple oversight.\r\n\r\n* impl WasmTranslator for FuncTranslator\r\n\r\n* rename parameter\r\n\r\n* remove unnecessary map_err(Into::into) in macro\r\n\r\n* implement Module::new_unchecked\r\n\r\n* apply rustfmt\r\n\r\n* add translation benchmarks for Module::new_unchecked",
          "timestamp": "2023-12-03T18:54:55+01:00",
          "tree_id": "351aecad1f0e3199a9241b5e91f821c28b186d36",
          "url": "https://github.com/paritytech/wasmi/commit/1ce23700e1cdf46d8c770bb7e46955b277311847"
        },
        "date": 1701626603050,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8988350,
            "range": "± 52714",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9300700,
            "range": "± 28970",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6921155,
            "range": "± 13074",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7247659,
            "range": "± 15422",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 160053661,
            "range": "± 746383",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 166965722,
            "range": "± 487799",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 123830464,
            "range": "± 275779",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 128746323,
            "range": "± 642768",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6667257,
            "range": "± 26001",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6907070,
            "range": "± 21083",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5191982,
            "range": "± 12464",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5396254,
            "range": "± 26511",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2588700,
            "range": "± 19220",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2706695,
            "range": "± 10754",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1993343,
            "range": "± 4907",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2099829,
            "range": "± 9103",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 238537,
            "range": "± 1130",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 247301,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 187904,
            "range": "± 478",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 197502,
            "range": "± 1158",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 343823,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 357370,
            "range": "± 1111",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 268408,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 279361,
            "range": "± 1153",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 495854,
            "range": "± 1834",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 524081,
            "range": "± 1863",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 392705,
            "range": "± 3539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 413226,
            "range": "± 2063",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56271,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 376196,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 652466,
            "range": "± 1668",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1084116,
            "range": "± 4429",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7478768,
            "range": "± 9912",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1240816,
            "range": "± 1632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 952308,
            "range": "± 1374",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1545918,
            "range": "± 1519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748311,
            "range": "± 1265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1302795,
            "range": "± 1959",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 318886,
            "range": "± 957",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 353834,
            "range": "± 611",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 389502,
            "range": "± 875",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34948,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63711,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11625704,
            "range": "± 8059",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7001951,
            "range": "± 10896",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13397525,
            "range": "± 19994",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3723199,
            "range": "± 13867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1551512,
            "range": "± 4624",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2251982,
            "range": "± 7895",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1231501,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1317305,
            "range": "± 1230",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3765557,
            "range": "± 152531",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 774839,
            "range": "± 1013",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1947413,
            "range": "± 7035",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1246131,
            "range": "± 2423",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3958835,
            "range": "± 6526",
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
          "id": "ef174dc82fd2b2cbdd2d3a3273db8476d4f72471",
          "message": "Test the entire Wasm spec test suite with fuel metering enabled (#830)\n\ntest the entire Wasm spec test suite with fuel metering",
          "timestamp": "2023-12-03T22:52:12+01:00",
          "tree_id": "d8bd93ba0bf75fd49c1d857c5605efe7e6fc03ad",
          "url": "https://github.com/paritytech/wasmi/commit/ef174dc82fd2b2cbdd2d3a3273db8476d4f72471"
        },
        "date": 1701640826358,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9037429,
            "range": "± 17269",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9460576,
            "range": "± 94289",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6744822,
            "range": "± 41014",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7054392,
            "range": "± 25092",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 161785906,
            "range": "± 586783",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 167544840,
            "range": "± 411375",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 119843805,
            "range": "± 302659",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 125347470,
            "range": "± 383842",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6674853,
            "range": "± 22052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6967984,
            "range": "± 31460",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5047799,
            "range": "± 9297",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5266626,
            "range": "± 15931",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2607379,
            "range": "± 7117",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2716248,
            "range": "± 8388",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1896052,
            "range": "± 22453",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2012848,
            "range": "± 4544",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 243201,
            "range": "± 753",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 251804,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 181399,
            "range": "± 1021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 190046,
            "range": "± 1131",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 353329,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 365893,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 257830,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 269011,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 506237,
            "range": "± 2194",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 531333,
            "range": "± 998",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 377653,
            "range": "± 1713",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 399891,
            "range": "± 2609",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54383,
            "range": "± 1522",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 376861,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 671993,
            "range": "± 2053",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1093022,
            "range": "± 15470",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481420,
            "range": "± 8265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1242901,
            "range": "± 1492",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 955452,
            "range": "± 2136",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1545300,
            "range": "± 2295",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 747940,
            "range": "± 2228",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1305846,
            "range": "± 3173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 318281,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 362609,
            "range": "± 1584",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 399193,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35230,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63811,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11603037,
            "range": "± 14869",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6995013,
            "range": "± 11083",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13450727,
            "range": "± 22129",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3740798,
            "range": "± 8635",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1419549,
            "range": "± 2920",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2327976,
            "range": "± 6575",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1230809,
            "range": "± 2382",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1316482,
            "range": "± 3674",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3764511,
            "range": "± 151142",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 771213,
            "range": "± 1415",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1943707,
            "range": "± 4863",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1209722,
            "range": "± 3464",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3942262,
            "range": "± 4789",
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
          "id": "050ba4fc758f08bd1a7a6dd05fb48fcab9730f1c",
          "message": "Fix translation bug with `reinterpret` instructions with preserved register inputs (#831)\n\n* improve panic message (forgot formatting)\r\n\r\n* add offending test cases\r\n\r\n* fix bug in translator for reinterpret instructions\r\n\r\n* apply rustfmt\r\n\r\n* apply clippy suggestions\r\n\r\n* add comment to push_storage",
          "timestamp": "2023-12-04T13:08:00+01:00",
          "tree_id": "208de54e1561570a3f09c709e5accd14c5ca3602",
          "url": "https://github.com/paritytech/wasmi/commit/050ba4fc758f08bd1a7a6dd05fb48fcab9730f1c"
        },
        "date": 1701692191988,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8827062,
            "range": "± 17884",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9240946,
            "range": "± 27875",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6714281,
            "range": "± 34785",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7114872,
            "range": "± 14668",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 157816049,
            "range": "± 427005",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 164480554,
            "range": "± 527167",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 119686375,
            "range": "± 219861",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 126439377,
            "range": "± 198357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6564607,
            "range": "± 14221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6848177,
            "range": "± 12215",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5041775,
            "range": "± 12782",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5343791,
            "range": "± 19273",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2505337,
            "range": "± 6426",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2655410,
            "range": "± 8147",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1927016,
            "range": "± 9059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2049387,
            "range": "± 5172",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 234830,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 244750,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 181809,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 191131,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 340020,
            "range": "± 666",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 353812,
            "range": "± 2104",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 258747,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 274002,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 487611,
            "range": "± 2460",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 514958,
            "range": "± 1096",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 380438,
            "range": "± 1669",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 404178,
            "range": "± 1652",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56992,
            "range": "± 1308",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 378263,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 627622,
            "range": "± 2057",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1037487,
            "range": "± 643",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7528967,
            "range": "± 10942",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1277455,
            "range": "± 3950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 960508,
            "range": "± 1205",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1538379,
            "range": "± 2047",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750573,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1263307,
            "range": "± 1846",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 316416,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 366047,
            "range": "± 640",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 398843,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35847,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63905,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11424038,
            "range": "± 17174",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6958640,
            "range": "± 7962",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 14137582,
            "range": "± 36336",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3540887,
            "range": "± 6406",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1260253,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2311062,
            "range": "± 8490",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1250445,
            "range": "± 1539",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1317588,
            "range": "± 652",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3614787,
            "range": "± 7631",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 781430,
            "range": "± 1401",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1945376,
            "range": "± 3483",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1190009,
            "range": "± 1741",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4085171,
            "range": "± 7528",
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
          "id": "41ca0619784b87fc4d3cb010029edd1acc7551c0",
          "message": "Add execution fuzzing (#832)\n\n* add execution fuzzing\r\n\r\n* refactor and cleanup\r\n\r\n* remove Invocation type and store only funcs\r\n\r\n* add execution fuzzing to CI\r\n\r\n* add doc comment\r\n\r\n* re-add StoreLimits to avoid running out of memory during fuzzing\r\n\r\n* add newline\r\n\r\n* improve panic message\r\n\r\n* cleanup and fix fuzzing CI\r\n\r\n* add corpus to CI fuzz cache\r\n\r\n* try to fix CI fuzzing\r\n\r\n* replace actions-rs with dtolnay/rust-toolchain\r\n\r\n* fix fuzz translation CI\r\n\r\n* try to update dependencies to fix proc-macro2\r\n\r\n* Update rust.yml\r\n\r\n* unlock installment of cargo fuzz to fix bug\r\n\r\n* remove debug CI jobs\r\n\r\n* try to fix execution fuzzing CI with previous learnings\r\n\r\n* make cargo-fuzz install locked again\r\n\r\n* lock the correct cargo fuzz installation ...\r\n\r\n* lock cargo-fuzz install on CI again\r\n\r\nThis temporary unlocking fixed the bug.\r\n\r\n* unlock cargo-fuzz installment generally\r\n\r\n* use proper local paths in CI caching\r\n\r\n* fix CI caching",
          "timestamp": "2023-12-04T16:05:30+01:00",
          "tree_id": "60ce14486e7bd8009a1aa52679bfcc1117b7c7e8",
          "url": "https://github.com/paritytech/wasmi/commit/41ca0619784b87fc4d3cb010029edd1acc7551c0"
        },
        "date": 1701702853174,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8797321,
            "range": "± 15060",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9164200,
            "range": "± 49557",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6694984,
            "range": "± 11308",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7090366,
            "range": "± 28483",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 156483067,
            "range": "± 409702",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 163606814,
            "range": "± 388501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 119192780,
            "range": "± 364260",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 125487443,
            "range": "± 252524",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6567524,
            "range": "± 31471",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6853337,
            "range": "± 44221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5035869,
            "range": "± 14756",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5309024,
            "range": "± 24371",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2503687,
            "range": "± 10686",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2645688,
            "range": "± 7540",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1915149,
            "range": "± 7013",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2042450,
            "range": "± 8096",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 232321,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 241052,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 180756,
            "range": "± 948",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 189944,
            "range": "± 1235",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 336238,
            "range": "± 1680",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 348506,
            "range": "± 714",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 257859,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 271483,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 485178,
            "range": "± 1127",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 531117,
            "range": "± 1637",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 378411,
            "range": "± 1581",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 406617,
            "range": "± 2109",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55786,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 382618,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 625906,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1002363,
            "range": "± 1788",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7519990,
            "range": "± 15212",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1292818,
            "range": "± 1341",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961480,
            "range": "± 1201",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1537362,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750358,
            "range": "± 1901",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1277349,
            "range": "± 4910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 315447,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 346866,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 379798,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34941,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 65909,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11434501,
            "range": "± 20620",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6954330,
            "range": "± 12300",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 14180893,
            "range": "± 9926",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3631305,
            "range": "± 8035",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1261788,
            "range": "± 1093",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2192675,
            "range": "± 5009",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1252060,
            "range": "± 2482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1320924,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3617707,
            "range": "± 3703",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 782767,
            "range": "± 2796",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1996857,
            "range": "± 5182",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1238401,
            "range": "± 2532",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4119409,
            "range": "± 18496",
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
          "id": "4a8725e500f00052e19743f1e08389285e67164f",
          "message": "Add differential fuzzing (#833)\n\n* apply rustfmt to fuzz code\r\n\r\n* add differential fuzzing target\r\n\r\n* add differential fuzzing job to CI\r\n\r\n* rename CI job step",
          "timestamp": "2023-12-04T19:03:55+01:00",
          "tree_id": "8e1e83c4a7a6f2b9f665b3917747d5f5e4c402a6",
          "url": "https://github.com/paritytech/wasmi/commit/4a8725e500f00052e19743f1e08389285e67164f"
        },
        "date": 1701713552118,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9022724,
            "range": "± 10753",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9461779,
            "range": "± 16756",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6669127,
            "range": "± 17540",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 6990102,
            "range": "± 16603",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 157720211,
            "range": "± 266895",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 164852973,
            "range": "± 565610",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 117263327,
            "range": "± 315663",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 123105868,
            "range": "± 251831",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6566921,
            "range": "± 10135",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6873668,
            "range": "± 20220",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 4963001,
            "range": "± 3896",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5224494,
            "range": "± 18922",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2550439,
            "range": "± 3069",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2692151,
            "range": "± 22264",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1887559,
            "range": "± 2445",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2024719,
            "range": "± 6103",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 236004,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 246552,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 179328,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 188609,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 344195,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 355483,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 256546,
            "range": "± 522",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 268069,
            "range": "± 441",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 491577,
            "range": "± 855",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 518000,
            "range": "± 979",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 374314,
            "range": "± 1124",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 397121,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54508,
            "range": "± 1204",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 379775,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 684546,
            "range": "± 1641",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1094121,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7507542,
            "range": "± 2904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1321654,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 962027,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1535826,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750678,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1326209,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 315468,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 349091,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 379599,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35540,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66803,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11405133,
            "range": "± 24269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6956892,
            "range": "± 17329",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12900680,
            "range": "± 33119",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3877930,
            "range": "± 7387",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1260540,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2199253,
            "range": "± 13444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1250321,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1319451,
            "range": "± 1114",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3610132,
            "range": "± 3940",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 805450,
            "range": "± 875",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1843189,
            "range": "± 1545",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1227346,
            "range": "± 2773",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4046167,
            "range": "± 3087",
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
          "id": "fdd9364302ce63d80f8ff27375264ad72a98381c",
          "message": "Fix `local.set` preservation bug (#834)\n\n* add testcase\r\n\r\n* fix bug by avoiding no-op local.set translation\r\n\r\n* apply rustfmt\r\n\r\n* apply rustfmt (2)",
          "timestamp": "2023-12-05T12:10:44+01:00",
          "tree_id": "84cb56941e6a4221e09bcd39578a73e8ef829d85",
          "url": "https://github.com/paritytech/wasmi/commit/fdd9364302ce63d80f8ff27375264ad72a98381c"
        },
        "date": 1701775241019,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8894238,
            "range": "± 32456",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9246192,
            "range": "± 21509",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6739850,
            "range": "± 74640",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7094086,
            "range": "± 22440",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 159817820,
            "range": "± 499732",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 165802008,
            "range": "± 489369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 119839405,
            "range": "± 313921",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 126473504,
            "range": "± 290261",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6590771,
            "range": "± 33136",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6901867,
            "range": "± 27169",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5042882,
            "range": "± 13242",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5310190,
            "range": "± 33703",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2536292,
            "range": "± 18093",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2654392,
            "range": "± 11338",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1932459,
            "range": "± 10287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2055601,
            "range": "± 7214",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 237230,
            "range": "± 698",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 249349,
            "range": "± 7656",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 183238,
            "range": "± 806",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 192733,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 345471,
            "range": "± 964",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 357299,
            "range": "± 2413",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 261636,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 273694,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 490050,
            "range": "± 1956",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 521704,
            "range": "± 3357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 377973,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 405898,
            "range": "± 2349",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55968,
            "range": "± 1737",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 376998,
            "range": "± 2519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 627051,
            "range": "± 1572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1028958,
            "range": "± 2638",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7589719,
            "range": "± 11068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1293323,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 956995,
            "range": "± 2373",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1536832,
            "range": "± 3403",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 743845,
            "range": "± 872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1277082,
            "range": "± 3617",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 312513,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 348282,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 380016,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35085,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64349,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11470884,
            "range": "± 14544",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6945728,
            "range": "± 15205",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12937348,
            "range": "± 26908",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3564240,
            "range": "± 8588",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1252052,
            "range": "± 2250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2192866,
            "range": "± 4161",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1224220,
            "range": "± 1631",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1312719,
            "range": "± 2082",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3638370,
            "range": "± 59794",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 797470,
            "range": "± 2231",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1929682,
            "range": "± 8275",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1216232,
            "range": "± 2689",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4051380,
            "range": "± 6221",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "85877331+sergejparity@users.noreply.github.com",
            "name": "Sergejs Kostjucenko",
            "username": "sergejparity"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "83082b3e04903497aad70ecdda99ab127f3ea5ac",
          "message": "Test criterion reports (#836)\n\n* increase translate measurement time\r\n\r\n* increase translate measurement time\r\n\r\n* use mean if slope is null",
          "timestamp": "2023-12-06T16:19:30+01:00",
          "tree_id": "087a6c0bc4cf81a538fb0c5872c86765d4f210d9",
          "url": "https://github.com/paritytech/wasmi/commit/83082b3e04903497aad70ecdda99ab127f3ea5ac"
        },
        "date": 1701875989539,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9663525,
            "range": "± 13997",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9872216,
            "range": "± 20923",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 7573091,
            "range": "± 14375",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7867847,
            "range": "± 16327",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 173984295,
            "range": "± 390355",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 180094315,
            "range": "± 370392",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 136647022,
            "range": "± 180802",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 142860644,
            "range": "± 209405",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 7219599,
            "range": "± 32141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 7473683,
            "range": "± 29355",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5673833,
            "range": "± 8956",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5927653,
            "range": "± 9573",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2810201,
            "range": "± 15641",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2907149,
            "range": "± 5075",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 2178744,
            "range": "± 6539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2321608,
            "range": "± 6429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 254437,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 264471,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 209619,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 217283,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 370414,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 382206,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 301581,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 311672,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 526749,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 550730,
            "range": "± 2303",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 430202,
            "range": "± 842",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 450489,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56798,
            "range": "± 1699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 375756,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 629728,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1024973,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7472251,
            "range": "± 1413",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1346289,
            "range": "± 1241",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1137772,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1533771,
            "range": "± 1227",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748085,
            "range": "± 1025",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1325041,
            "range": "± 9907",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 316142,
            "range": "± 3580",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 349683,
            "range": "± 1088",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 385952,
            "range": "± 4813",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34841,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63727,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11614983,
            "range": "± 32368",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6986189,
            "range": "± 35705",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12921862,
            "range": "± 61125",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3948557,
            "range": "± 13182",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1274245,
            "range": "± 2678",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2244248,
            "range": "± 32228",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1407056,
            "range": "± 186519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1317611,
            "range": "± 952",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3583137,
            "range": "± 4848",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 780865,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1851651,
            "range": "± 6695",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1289205,
            "range": "± 3284",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3914803,
            "range": "± 6616",
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
          "id": "336bb11940677c5a7fd3e5968c06b73b217d4c0b",
          "message": "Fix a bunch of register-machine `wasmi` translation bugs (#838)\n\n* add failing translation tests\r\n\r\n* add debug_assert\r\n\r\n* remove NonZeroUsize runtime check\r\n\r\n* improve docs\r\n\r\n* fix naming of preservation registers and space\r\n\r\nPreviously we called this the \"storage\" space but now we renamed all occurrences to \"preservation\". This should improve code readability.\r\n\r\n* more renamings from storage to preserve\r\n\r\n* make some methods private if possible\r\n\r\n* cleanup push_local method\r\n\r\n* add debug_assert to bump_preserved\r\n\r\n* rename storage -> preserve\r\n\r\n* improve lifetime tracking of preserved registers\r\n\r\nThe new systems starts preserved register amounts at 2 instead of 1. This prevents removal of the slot when popping it from the preservation stack. In order to properly recycle preservation registers again, we now check for all previously removed preservation registers if they are truly removed (amount = 1) and remove them before allocating a new preservation register.\r\n\r\n* add debug_assert to push_preserved\r\n\r\n* add dev docs for new semantics\r\n\r\n* add else provider regression test and fix bug\r\n\r\n* apply rustfmt\r\n\r\n* remove unneeded validation checks\r\n\r\n* add dev comment\r\n\r\n* add missing call to dec_register_usage\r\n\r\n* fix intra doc link\r\n\r\n* apply rustfmt\r\n\r\n* add another if test with missing else\r\n\r\n* simplify fuzz_6.wat test case\r\n\r\n* finalize testcases 5 and 6\r\n\r\n* fix test cases 5 and 6",
          "timestamp": "2023-12-08T11:34:07+01:00",
          "tree_id": "3db817ad649d2cc82224428e51549708b38c5812",
          "url": "https://github.com/paritytech/wasmi/commit/336bb11940677c5a7fd3e5968c06b73b217d4c0b"
        },
        "date": 1702031666434,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8702552,
            "range": "± 19563",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9070012,
            "range": "± 18033",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6833543,
            "range": "± 8510",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7170733,
            "range": "± 21873",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 154992040,
            "range": "± 886314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 161185610,
            "range": "± 567838",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 120337774,
            "range": "± 230709",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 126189338,
            "range": "± 178647",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6421915,
            "range": "± 25247",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6732475,
            "range": "± 23591",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5101307,
            "range": "± 45783",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5349999,
            "range": "± 21123",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2456532,
            "range": "± 4852",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2594706,
            "range": "± 16124",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1954238,
            "range": "± 9462",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2070309,
            "range": "± 5007",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 228930,
            "range": "± 657",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 240103,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 184796,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 191384,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 331519,
            "range": "± 854",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 347649,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 263233,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 274105,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 476098,
            "range": "± 2134",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 504555,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 384812,
            "range": "± 1446",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 405630,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56214,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 375017,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 629789,
            "range": "± 1800",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1014982,
            "range": "± 1187",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7470434,
            "range": "± 3896",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1345063,
            "range": "± 952",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1137598,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1532717,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746778,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1284226,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 314966,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 349332,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 382367,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35220,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63589,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11595533,
            "range": "± 19425",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6968111,
            "range": "± 10195",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12904199,
            "range": "± 9801",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3579256,
            "range": "± 3862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1275504,
            "range": "± 935",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2236214,
            "range": "± 6521",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1403673,
            "range": "± 186773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1316149,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3582018,
            "range": "± 4700",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 784701,
            "range": "± 723",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1856084,
            "range": "± 3461",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1223110,
            "range": "± 3152",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3908184,
            "range": "± 3862",
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
          "id": "bed9ae4ce3ca7e4b3c88883049e047a5244b8e73",
          "message": "Add `Module::validate` API from Wasmtime (#840)\n\n* remove unneeded wasmparser::validate checks in some test cases\r\n\r\n* no longer panic in parser when encountering component model definitions\r\n\r\nInstead return a proper wasmi error indicating usage of unsupported Wasm features.\r\n\r\n* add Module::validate API\r\n\r\n* remove unnecessary temporary buffer\r\n\r\nWe do not need this buffer until we actually plan to perform validation in parllel.",
          "timestamp": "2023-12-08T15:24:29+01:00",
          "tree_id": "ce11dcfee8819167c7d29d536c8c218c58c823dd",
          "url": "https://github.com/paritytech/wasmi/commit/bed9ae4ce3ca7e4b3c88883049e047a5244b8e73"
        },
        "date": 1702045482192,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9004874,
            "range": "± 27512",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9353109,
            "range": "± 86602",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6803329,
            "range": "± 9333",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7217329,
            "range": "± 21343",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 159804315,
            "range": "± 442251",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 166100969,
            "range": "± 514875",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 120064731,
            "range": "± 223809",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 126790422,
            "range": "± 403314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6652648,
            "range": "± 23323",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6925008,
            "range": "± 28436",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5073945,
            "range": "± 14455",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5312636,
            "range": "± 22977",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2518325,
            "range": "± 9429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2648644,
            "range": "± 9922",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1895043,
            "range": "± 5630",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2017376,
            "range": "± 10107",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 236192,
            "range": "± 1027",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 245950,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 180187,
            "range": "± 1317",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 190056,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 343196,
            "range": "± 1521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 357278,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 257076,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 271995,
            "range": "± 1543",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 492426,
            "range": "± 2054",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 520437,
            "range": "± 1389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 373634,
            "range": "± 1486",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 401523,
            "range": "± 1226",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54661,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 374157,
            "range": "± 734",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 624940,
            "range": "± 1641",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1046691,
            "range": "± 1714",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7513437,
            "range": "± 11525",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1302862,
            "range": "± 1070",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 1140568,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1537290,
            "range": "± 4973",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 751751,
            "range": "± 3986",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1264753,
            "range": "± 1383",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 299035,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 354495,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 382064,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35183,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64552,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11410826,
            "range": "± 10764",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7080698,
            "range": "± 12880",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12652428,
            "range": "± 10717",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3766547,
            "range": "± 4018",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1259404,
            "range": "± 4549",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2198834,
            "range": "± 6866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1416386,
            "range": "± 176446",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1321562,
            "range": "± 2960",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3657748,
            "range": "± 34990",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 786135,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1948626,
            "range": "± 10334",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1210243,
            "range": "± 1659",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3755662,
            "range": "± 5369",
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
          "id": "eac84fefbb265c980331a6b6ce1f1c2a33845516",
          "message": "Fix CI `rustfmt` job (#841)\n\n* fix CI rustfmt job\r\n\r\n* apply nightly rustfmt\r\n\r\n* re-apply nightly rustftm",
          "timestamp": "2023-12-08T15:52:50+01:00",
          "tree_id": "c7558f69791c5be7bf36baf85e2ba77c840132bd",
          "url": "https://github.com/paritytech/wasmi/commit/eac84fefbb265c980331a6b6ce1f1c2a33845516"
        },
        "date": 1702047186919,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9144622,
            "range": "± 42362",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9531003,
            "range": "± 41547",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 7591825,
            "range": "± 19388",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7353098,
            "range": "± 13862",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 166017931,
            "range": "± 553652",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 172565123,
            "range": "± 208277",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 125449499,
            "range": "± 282227",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 131817454,
            "range": "± 471767",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6860085,
            "range": "± 85052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 7136669,
            "range": "± 19081",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5300797,
            "range": "± 22468",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5563228,
            "range": "± 30386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2631919,
            "range": "± 9423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2758204,
            "range": "± 4560",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 2018227,
            "range": "± 15060",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2143609,
            "range": "± 15147",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 244927,
            "range": "± 1281",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 255079,
            "range": "± 1948",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 190253,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 199637,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 355754,
            "range": "± 2074",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 367121,
            "range": "± 1570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 269485,
            "range": "± 926",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 284863,
            "range": "± 1386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 510174,
            "range": "± 2169",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 539511,
            "range": "± 10596",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 397648,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 420171,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53025,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 376502,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 643909,
            "range": "± 18960",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1037497,
            "range": "± 1184",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7521707,
            "range": "± 9925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1326007,
            "range": "± 3747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 962987,
            "range": "± 1352",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1540738,
            "range": "± 3115",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750126,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1261216,
            "range": "± 1928",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 308300,
            "range": "± 978",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 346826,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 381314,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34485,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 69271,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11430322,
            "range": "± 23850",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6956944,
            "range": "± 41400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13062105,
            "range": "± 17111",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3688636,
            "range": "± 5418",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1261979,
            "range": "± 1843",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2195271,
            "range": "± 3807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1251451,
            "range": "± 2527",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1318106,
            "range": "± 3225",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3611337,
            "range": "± 3562",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 765475,
            "range": "± 1553",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1933369,
            "range": "± 8758",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1234322,
            "range": "± 1134",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3950036,
            "range": "± 4592",
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
          "id": "dc5076f6f8e2793aeeee6da3404a732b6c74384f",
          "message": "Make CI jobs use `dtolnay/rust-toolchain` instead of `actions-rs` (#842)\n\n* fix miri CI job\r\n\r\n* fix clippy CI job\r\n\r\n* try to fix test coverage CI job\r\n\r\n* fix rustdoc CI job\r\n\r\n* fix test CI job\r\n\r\n* fix build CI job\r\n\r\n* fix audit CI job\r\n\r\n* fix udeps CI job\r\n\r\n* fix build CI job (2)",
          "timestamp": "2023-12-08T16:43:32+01:00",
          "tree_id": "6c497f482aed91d3a53ac97b00102d6004f62a07",
          "url": "https://github.com/paritytech/wasmi/commit/dc5076f6f8e2793aeeee6da3404a732b6c74384f"
        },
        "date": 1702050229900,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9131726,
            "range": "± 18831",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9394664,
            "range": "± 50652",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 7020130,
            "range": "± 12126",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7386391,
            "range": "± 155982",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 164325922,
            "range": "± 209597",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 171076471,
            "range": "± 515923",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 125713541,
            "range": "± 289074",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 132036040,
            "range": "± 276052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6870773,
            "range": "± 21084",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 7081784,
            "range": "± 30927",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5309423,
            "range": "± 11882",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5541712,
            "range": "± 11911",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2619930,
            "range": "± 9226",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2750029,
            "range": "± 13427",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 2009649,
            "range": "± 3202",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2144709,
            "range": "± 17753",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 243244,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 255046,
            "range": "± 1276",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 191548,
            "range": "± 534",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 199590,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 351150,
            "range": "± 1558",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 364496,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 272277,
            "range": "± 1399",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 283846,
            "range": "± 1481",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 504207,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 533843,
            "range": "± 4717",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 396667,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 422446,
            "range": "± 3476",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54112,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 380365,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 642196,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1063250,
            "range": "± 3332",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7512728,
            "range": "± 10482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1391086,
            "range": "± 2171",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 962431,
            "range": "± 1370",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1539628,
            "range": "± 2736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750545,
            "range": "± 2117",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1284367,
            "range": "± 2946",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 307817,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 359759,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 391118,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35450,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63716,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11411218,
            "range": "± 6065",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6960544,
            "range": "± 8428",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13236499,
            "range": "± 34252",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3596910,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1260493,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2276558,
            "range": "± 6313",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1251381,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1320572,
            "range": "± 1599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3617576,
            "range": "± 4493",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 763686,
            "range": "± 1008",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1920369,
            "range": "± 5805",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1215153,
            "range": "± 2559",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3889362,
            "range": "± 8302",
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
          "id": "42b1eb799e583f45836e69df09a79d005184183c",
          "message": "Only store `local.get` on the provider stack out-of-place when necessary (#843)\n\n* only use LocalRefs when necessary\r\n\r\nStoring local.get providers on LocalRefs is expensive. We do this to prevent certain attack vectors. However, for most common and practical Wasm inputs we might not even need to do this. This commit implements a naive safety guard.\r\n\r\n* fix bug",
          "timestamp": "2023-12-08T17:53:58+01:00",
          "tree_id": "4128211426e9c8934a188000b2d9670cbc8a21d4",
          "url": "https://github.com/paritytech/wasmi/commit/42b1eb799e583f45836e69df09a79d005184183c"
        },
        "date": 1702054455873,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 9100482,
            "range": "± 9649",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9445435,
            "range": "± 15426",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 7034821,
            "range": "± 9810",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7380418,
            "range": "± 10265",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 165168984,
            "range": "± 402329",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 182704607,
            "range": "± 608203",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 125176540,
            "range": "± 274570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 132382782,
            "range": "± 245060",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6760488,
            "range": "± 8314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 7064713,
            "range": "± 14491",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 5289629,
            "range": "± 9510",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5533216,
            "range": "± 7576",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2622774,
            "range": "± 5784",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2754695,
            "range": "± 8751",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 2010068,
            "range": "± 7311",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 2139611,
            "range": "± 8221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 240921,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 252618,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 190441,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 199296,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 350503,
            "range": "± 1079",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 365772,
            "range": "± 1100",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 271693,
            "range": "± 489",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 284866,
            "range": "± 2099",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 504571,
            "range": "± 3364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 535238,
            "range": "± 2829",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 398456,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 422104,
            "range": "± 1195",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56713,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 380525,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 638048,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1051566,
            "range": "± 1548",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7515440,
            "range": "± 8113",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1386667,
            "range": "± 1849",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961308,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1537261,
            "range": "± 1451",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750347,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1273569,
            "range": "± 2099",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 309846,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 357257,
            "range": "± 1242",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 388678,
            "range": "± 1347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35698,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63724,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11445039,
            "range": "± 338489",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6980489,
            "range": "± 69316",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13046485,
            "range": "± 41587",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3599834,
            "range": "± 12237",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1261191,
            "range": "± 1523",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2282489,
            "range": "± 3481",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1250274,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1317452,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3607458,
            "range": "± 2079",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 764423,
            "range": "± 934",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1911664,
            "range": "± 6886",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1221662,
            "range": "± 2261",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3882919,
            "range": "± 5921",
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
          "id": "e5bad7a3f8b2013b062d7614f2ed9daa8bddea7d",
          "message": "Adjust benchmarks to make lazy-comp benchmark CI work (#846)\n\nadjust benchmarks to make lazy-comp PR succeed",
          "timestamp": "2023-12-16T12:06:27+01:00",
          "tree_id": "bbf08db21dd656fd43b4c2b69b6696bf4a0fe66f",
          "url": "https://github.com/paritytech/wasmi/commit/e5bad7a3f8b2013b062d7614f2ed9daa8bddea7d"
        },
        "date": 1702724802018,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 8775116,
            "range": "± 14363",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 9153345,
            "range": "± 19454",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 6656660,
            "range": "± 10486",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 7049811,
            "range": "± 18138",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 156846188,
            "range": "± 379604",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 163803196,
            "range": "± 121823",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 118250483,
            "range": "± 481298",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 124850165,
            "range": "± 135031",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 6536709,
            "range": "± 36525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 6795817,
            "range": "± 27301",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 4969547,
            "range": "± 6620",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 5239173,
            "range": "± 14079",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 2463066,
            "range": "± 5927",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 2593526,
            "range": "± 6292",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1835741,
            "range": "± 4659",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1975221,
            "range": "± 4294",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 230515,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 240694,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 177748,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 188346,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 334745,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 349531,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 252095,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 266309,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 478843,
            "range": "± 1030",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 506360,
            "range": "± 1019",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 367342,
            "range": "± 1613",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 393521,
            "range": "± 810",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55750,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 375113,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 642024,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1031284,
            "range": "± 2615",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7591811,
            "range": "± 3687",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1309045,
            "range": "± 1356",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 964987,
            "range": "± 802",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1534313,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 744704,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1255068,
            "range": "± 1019",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 311128,
            "range": "± 1027",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 343203,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 376661,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 33378,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66548,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11490392,
            "range": "± 12103",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6943730,
            "range": "± 33984",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12765130,
            "range": "± 31423",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3560328,
            "range": "± 3028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1248726,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2175314,
            "range": "± 4980",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1402719,
            "range": "± 189541",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1314383,
            "range": "± 3038",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3598124,
            "range": "± 14375",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 840074,
            "range": "± 2926",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2131999,
            "range": "± 26442",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1317681,
            "range": "± 7296",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4019407,
            "range": "± 6978",
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
          "id": "ac00319f69bb3d61a22d83b11d4f70102550a3f0",
          "message": "apply `wasm-opt -Oz` to benchmark Wasm inputs (#847)\n\napply wasm-opt -Oz to benchmark wasm inputs\r\n\r\nwasm-opt version 116 was used",
          "timestamp": "2023-12-16T21:03:35+01:00",
          "tree_id": "4c98aaebb5647161eb7a9ebbb9ace7d0b592afc8",
          "url": "https://github.com/paritytech/wasmi/commit/ac00319f69bb3d61a22d83b11d4f70102550a3f0"
        },
        "date": 1702757030588,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8552179,
            "range": "± 15223",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 8952032,
            "range": "± 12814",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 8539335,
            "range": "± 10833",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6605806,
            "range": "± 20476",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 6891380,
            "range": "± 8965",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 6609986,
            "range": "± 15559",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 153571728,
            "range": "± 308625",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 160596445,
            "range": "± 228059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 153309372,
            "range": "± 191261",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 116247912,
            "range": "± 198714",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 122869550,
            "range": "± 133496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 116841899,
            "range": "± 207034",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6333959,
            "range": "± 16675",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6651474,
            "range": "± 14676",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 6392340,
            "range": "± 14446",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4866065,
            "range": "± 8201",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5120365,
            "range": "± 4231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 4891835,
            "range": "± 52358",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2417415,
            "range": "± 20033",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2572500,
            "range": "± 15423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 2398499,
            "range": "± 5382",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1802226,
            "range": "± 9514",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1933922,
            "range": "± 3862",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 1807158,
            "range": "± 4150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 224100,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 236368,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 223884,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 172732,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 182695,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 172888,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 324771,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 341391,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 325050,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 245821,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 260774,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 245983,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 467230,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 494810,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 465616,
            "range": "± 1077",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 355787,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 382624,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 361300,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56407,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 378024,
            "range": "± 538",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 673094,
            "range": "± 4185",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1096860,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7474482,
            "range": "± 2621",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1328933,
            "range": "± 3959",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 951641,
            "range": "± 1137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1533106,
            "range": "± 1103",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746866,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1240074,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 313880,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 344273,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 377649,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 33516,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63226,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11798938,
            "range": "± 4276",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6971166,
            "range": "± 3544",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12683615,
            "range": "± 12611",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3871607,
            "range": "± 2862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1279063,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2167539,
            "range": "± 5760",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1247889,
            "range": "± 20288",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1321319,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 4045091,
            "range": "± 9565",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 792982,
            "range": "± 695",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2101607,
            "range": "± 4587",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1275359,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4090086,
            "range": "± 5858",
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
          "id": "1b9aae2f4ee5462fc5ef7cc5487808ef60319c96",
          "message": "Implement lazy Wasm to `wasmi` bytecode translation (#844)\n\n* add CompilationMode to Config\r\n\r\n* rename builder::ModuleImports -> ModuleImportsBuilder\r\n\r\n* return reference to GlobalType\r\n\r\n* split ModuleBuilder into its header\r\n\r\n* refactor Wasm module parsing\r\n\r\nThis commit removes all lifetime annotations from parsing related types. This is going to be important since we require the new ModuleHeader type to be stored in the Engine for all lazily compiled Wasm functions for translation purposes.\r\n\r\n* apply rustfmt\r\n\r\n* remove debug printlns\r\n\r\n* fix intra doc link\r\n\r\n* re-export CompilationMode from crate root\r\n\r\n* apply rustfmt\r\n\r\n* silence warning\r\n\r\n* rename FunctionTranslator -> FuncTranslationDriver\r\n\r\n* refactor ArenaIndex impl for CompiledFunc\r\n\r\n* add CompiledFunc -> FuncIdx mapping for ModuleHeader\r\n\r\n* apply rustftm\r\n\r\n* use ModuleHeader info in relink_result\r\n\r\nThis fixes a problem in relink_result that CompiledFunc info is oftentimes results.len() is not available at the time is it required due to uninitialized compiled function entities. Using ModuleHeader instead fixes this issue which should improve codegen in these situations and make codegen non-order dependent.\r\n\r\n* add FuncType::len_results\r\n\r\nRequired in last commit. (oups)\r\n\r\n* use new as uniform translation driver constructor\r\n\r\n* add setup method to the WasmTranslator trait\r\n\r\n* add LazyFuncTranslator type\r\n\r\n* extend Engine[Inner] docs\r\n\r\n* remove len_results field from CompiledFuncEntity\r\n\r\n* add InternalFuncEntity to CodeMap\r\n\r\n- This divides CompiledFuncEntity for eager translation and UncompiledFuncEntity for lazy translation.\r\n- This commit does not yet dispatch on UncompiledFuncEntity during execution of call instructions.\r\n- Furthermore this commit does not yet use the new LazyFuncTranslator to actually translate Wasm functions lazily.\r\n\r\n* make as_compiled method test-only\r\n\r\n* make use of InternalFuncEntity::uninit\r\n\r\n* re-export LazyFuncTranslator from engine module\r\n\r\n* refactor and use new func translators\r\n\r\n* apply clippy suggestions\r\n\r\n* allow dead code temporarily\r\n\r\n* fix intra doc link\r\n\r\n* add lazy translation benchmark tests\r\n\r\n* prevent heap allocations for small Wasm funcs\r\n\r\nWasm function bodies up to 22 bytes will now be stored inline instead of allocated on the heap which should decrease burden on the memory allocator for many small Wasm functions in lazy compilation mode.\r\n\r\n* move block_type into engine submodule\r\n\r\nAlso flatten module/compile submodule.\r\n\r\n* no longer use FunctionBody in translate method\r\n\r\nThis is so that we can later use translate from within the Engine when lazily compiling functions since they do not have a FunctionBody field but just raw bytes and module header information. Fortunately it is possible to restore the FunctionBody from this information.\r\n\r\n* rename Error::Store to Error::Fuel\r\n\r\n* remove unneeded UnsupportedFeatures error\r\n\r\nComponent model validation is already performed by the wasmparser crate.\r\n\r\n* replace ModuleError by Error\r\n\r\nFlatten sub-errors of ModuleError as new variants into Error.\r\nThis allows us to move translation driver routines into engine.\r\n\r\n* make Error type pointer sized\r\n\r\n* fix no_std build\r\n\r\n* refactor TranslationError\r\n\r\n- All functions that returned TranslationError now return wasmi::Error instead.\r\n- Removed TranslationErrorInner and moved variants to outer TranslationError type.\r\n- Moved TranslationError::Validate kind to Error as Error::Wasm.\r\n\r\n* move translation driver into engine submodule\r\n\r\n* improve docs\r\n\r\n* rename translate -> translate_wasm_func\r\n\r\n* improve docs of translate_wasm_func\r\n\r\n* rename FuncTranslator::res field to module\r\n\r\n* use Error instead of Trap in Func::call et.al. and host functions\r\n\r\nThis is a major refactoring that will significantly affect wasmi users unfortunately.\r\nHowever, there is no better alternative to having a unified Error type when introducing lazy Wasm function compilation during execution.\r\nThis requires execution to handle Error which could be a TranslationError due to problems during lazy translation.\r\nThis means, Func::call et.al. also need to return Error instead of Trap.\r\nIf we want to allow host function calls to call Wasm functions and propagate their result we therefore also need to return the Error type from host functions instead of just a Trap.\r\nThis commit handles all of these cases.\r\nFor ease of use we introduced Error::as_trap convenience method.\r\nThe great thing about this is that a unified Error type is closer to how Wasmtime API looks and feels. So we kinda improved our Wasmtime mirror with this commit.\r\n\r\n* refactor func initialization asserts\r\n\r\n* implement lazy compilation during execution\r\n\r\nThis commit requires a follow-up to return wasmi::Error instead of TrapCode from wasmi instruction executor functions so that call instructions can properly forward translation errors.\r\nFurthermore CodeMap::get (where lazy translation happens) is currently not perfectly implemented and might dead lock in malicious usage scenarios. I already know how to fix this in another later commit.\r\n\r\n* rename CodeMap::init_func_v2 -> init_func\r\n\r\n* remove some TODOs\r\n\r\n* return Error from wasmi instruction executors\r\n\r\nThis allows us to properly handle failed lazy translations in call instruction executions.\r\n\r\n* remove usage of Trap\r\n\r\nNow wasmi::Error takes over responsibilities of Trap.\r\nThis make it possible to remove an unnecessary Box indirection.\r\n\r\n* improve CodeMap::get method internals\r\n\r\nThis makes fast path faster and fixes some problems with unfair write access.\r\n\r\n* fix internal doc links\r\n\r\n* fix no_std build\r\n\r\n* rename EngineInner::init_func_v2 -> init_func\r\n\r\n* limit ReusableAllocationStack height to just 1\r\n\r\n* experiment: comment out most translation benchmarks\r\n\r\nCurrently Wasm benchmark CI runs out of memory for spidermonkey lazy unchecked translation. We want to see if there are memory dependencies between the different translation benchmark runs.\r\n\r\n* Revert \"experiment: comment out most translation benchmarks\"\r\n\r\nThis reverts commit 1dd9a1e9c2bb5d076656e54af459c9851308275d.\r\n\r\n* add forgotten buffer.drain call\r\n\r\n* remove commented out code\r\n\r\n* apply wasm-opt -Oz to spidermonkey.wasm (version 116)\r\n\r\n* improve byte slicing\r\n\r\n* use Self::MAX_INLINE_SIZE constant\r\n\r\n* use Self::MAX_INLINE_SIZE in more places\r\n\r\n* use Self::MAX_INLINE_SIZE in more places (2)\r\n\r\n* increase MAX_INLINE_SIZE in SmallByteSlice to 30\r\n\r\n* avoid unnecessary Engine clone\r\n\r\n* remove unnecessary slicing\r\n\r\n* apply clippy suggestions\r\n\r\n* refactor translation benchmark test runner\r\n\r\n* remove direct use of ModuleHeader::engine field\r\n\r\n* fix memory leak due to cyclic Arc usage\r\n\r\nThe cycle existed because Engine held ModuleHeader which itself held Engine.\r\nThe cycle was broken by introducing EngineWeak and make ModuleHeader hold EngineWeak instead of Engine which is just a fancy wrapper around a Weak pointer to an Engine. Therefore Engine access via ModuleHeader now may fail if the Engine does no longer exist. However, due to the fact that ModuleHeader is only accessed via its Engine, this should technically never occure.\r\n\r\n* apply rustfmt\r\n\r\n* make Engine::downgrade method crate private",
          "timestamp": "2023-12-16T21:37:58+01:00",
          "tree_id": "56ec2d2f814c4675b0078ccf41a8c080aa2e0021",
          "url": "https://github.com/paritytech/wasmi/commit/1b9aae2f4ee5462fc5ef7cc5487808ef60319c96"
        },
        "date": 1702759094265,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8502338,
            "range": "± 18479",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 8926769,
            "range": "± 10025",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 8502121,
            "range": "± 11495",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6511922,
            "range": "± 16164",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 6870439,
            "range": "± 16418",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 6509151,
            "range": "± 15489",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 130037744,
            "range": "± 449322",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 140206657,
            "range": "± 229002",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 129884635,
            "range": "± 427840",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 99872873,
            "range": "± 162896",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 109467258,
            "range": "± 330206",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 100072585,
            "range": "± 76061",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6001882,
            "range": "± 13308",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6461970,
            "range": "± 18230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 6011462,
            "range": "± 18813",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4670412,
            "range": "± 13673",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5072816,
            "range": "± 12697",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 4653089,
            "range": "± 9008",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2357971,
            "range": "± 7192",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2545600,
            "range": "± 15173",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 2354655,
            "range": "± 7280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1760885,
            "range": "± 5637",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1945360,
            "range": "± 9969",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 1760668,
            "range": "± 7167",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 224578,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 235470,
            "range": "± 744",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 222774,
            "range": "± 390",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 174009,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 183155,
            "range": "± 1278",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 172996,
            "range": "± 461",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 323051,
            "range": "± 487",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 340205,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 322695,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 245702,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 259807,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 246447,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 464395,
            "range": "± 890",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 495060,
            "range": "± 1601",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 463751,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 355659,
            "range": "± 1173",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 384591,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 355773,
            "range": "± 1848",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53895,
            "range": "± 969",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 378685,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 678263,
            "range": "± 1245",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1128932,
            "range": "± 1490",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7474016,
            "range": "± 3512",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1390075,
            "range": "± 6013",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 951340,
            "range": "± 4425",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1532706,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 746763,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1276578,
            "range": "± 1469",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 318998,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 350471,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 385721,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 34326,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66169,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11617214,
            "range": "± 83904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6975249,
            "range": "± 3853",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12986841,
            "range": "± 13411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3878382,
            "range": "± 5304",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1279057,
            "range": "± 1693",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2218877,
            "range": "± 5940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1248302,
            "range": "± 20012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1319978,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3581281,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 751317,
            "range": "± 1656",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2110914,
            "range": "± 3979",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1251978,
            "range": "± 2259",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4018159,
            "range": "± 7452",
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
          "id": "a3f44daa4ac24a43460930646a10e9cb1582831b",
          "message": "Bump actions/upload-artifact from 3 to 4 (#845)\n\nBumps [actions/upload-artifact](https://github.com/actions/upload-artifact) from 3 to 4.\r\n- [Release notes](https://github.com/actions/upload-artifact/releases)\r\n- [Commits](https://github.com/actions/upload-artifact/compare/v3...v4)\r\n\r\n---\r\nupdated-dependencies:\r\n- dependency-name: actions/upload-artifact\r\n  dependency-type: direct:production\r\n  update-type: version-update:semver-major\r\n...\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2023-12-16T21:40:39+01:00",
          "tree_id": "14c3d687113c722dc5b61885445ef96f12078743",
          "url": "https://github.com/paritytech/wasmi/commit/a3f44daa4ac24a43460930646a10e9cb1582831b"
        },
        "date": 1702759685987,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8656786,
            "range": "± 15439",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9133130,
            "range": "± 27762",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 4022804,
            "range": "± 7596",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6607904,
            "range": "± 116109",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 7076245,
            "range": "± 13741",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 457906,
            "range": "± 3458",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 132506268,
            "range": "± 266442",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 143937869,
            "range": "± 178606",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 56301766,
            "range": "± 119938",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 102667038,
            "range": "± 196119",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 112745551,
            "range": "± 235506",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3704382,
            "range": "± 14948",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6103738,
            "range": "± 9820",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6618644,
            "range": "± 10486",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 2582692,
            "range": "± 6644",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4762158,
            "range": "± 4364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5252425,
            "range": "± 73515",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 242459,
            "range": "± 5616",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2384846,
            "range": "± 10902",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2615714,
            "range": "± 7281",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 967415,
            "range": "± 5389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1822062,
            "range": "± 7212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 2033745,
            "range": "± 9097",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 45247,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 225444,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 240811,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 107438,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 175039,
            "range": "± 528",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 186682,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 24474,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 325504,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 345269,
            "range": "± 1537",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 153273,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 249055,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 265890,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 28511,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 469555,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 504430,
            "range": "± 947",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 212637,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 364913,
            "range": "± 1342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 396414,
            "range": "± 3449",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 32147,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55149,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 383382,
            "range": "± 996",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 664416,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1054363,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481496,
            "range": "± 7320",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1283382,
            "range": "± 1510",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961328,
            "range": "± 1295",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1621555,
            "range": "± 1440",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750229,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1366136,
            "range": "± 1543",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 333910,
            "range": "± 1555",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 368142,
            "range": "± 433",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 399883,
            "range": "± 536",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36211,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64595,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12169699,
            "range": "± 5544",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7212768,
            "range": "± 6284",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13133046,
            "range": "± 5420",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4050316,
            "range": "± 6458",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1264137,
            "range": "± 1054",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2330148,
            "range": "± 6731",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1314264,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1343361,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3533903,
            "range": "± 2343",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 870842,
            "range": "± 1410",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1966665,
            "range": "± 2858",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1254786,
            "range": "± 1951",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3762516,
            "range": "± 7757",
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
          "id": "20bfc9300211949231ed39f442d5c09631e50a48",
          "message": "Fix `Module::validate` method signature (#848)\n\nfix Module::validate method signature",
          "timestamp": "2023-12-17T11:17:55+01:00",
          "tree_id": "6d727ddcf0090a9afcecbb073491dc99afb76ef3",
          "url": "https://github.com/paritytech/wasmi/commit/20bfc9300211949231ed39f442d5c09631e50a48"
        },
        "date": 1702808840914,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8621645,
            "range": "± 18132",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9107910,
            "range": "± 19042",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 4001214,
            "range": "± 12387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6714817,
            "range": "± 35866",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 7153651,
            "range": "± 18736",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 463430,
            "range": "± 3373",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 133488200,
            "range": "± 416197",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 143883060,
            "range": "± 289843",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 55917765,
            "range": "± 103037",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 102529986,
            "range": "± 304213",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 112806057,
            "range": "± 206835",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3839372,
            "range": "± 54903",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6163610,
            "range": "± 15139",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6668486,
            "range": "± 12335",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 2572926,
            "range": "± 12147",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4799753,
            "range": "± 18198",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5280154,
            "range": "± 22928",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 242220,
            "range": "± 3079",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2388963,
            "range": "± 8242",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2606135,
            "range": "± 16663",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 965819,
            "range": "± 4464",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1830617,
            "range": "± 4872",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 2027892,
            "range": "± 6958",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 45134,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 225341,
            "range": "± 1027",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 239899,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 107070,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 174484,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 186407,
            "range": "± 684",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 24556,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 325505,
            "range": "± 642",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 343784,
            "range": "± 543",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 152465,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 247683,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 267733,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 28548,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 468438,
            "range": "± 979",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 503578,
            "range": "± 1021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 211498,
            "range": "± 1306",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 362668,
            "range": "± 826",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 397876,
            "range": "± 1713",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 31705,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57896,
            "range": "± 1571",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 383709,
            "range": "± 1127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 664812,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1054924,
            "range": "± 741",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7497025,
            "range": "± 3888",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1282799,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961874,
            "range": "± 1260",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1622374,
            "range": "± 1124",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 751172,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1368800,
            "range": "± 1687",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 333347,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 370570,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 396452,
            "range": "± 723",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36729,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64866,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12180438,
            "range": "± 15251",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7213681,
            "range": "± 6543",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13289080,
            "range": "± 68866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3783093,
            "range": "± 2868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1265019,
            "range": "± 1063",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2328165,
            "range": "± 6711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1315218,
            "range": "± 948",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1346923,
            "range": "± 1279",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3536144,
            "range": "± 3675",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 871052,
            "range": "± 1228",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1945603,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1179906,
            "range": "± 1381",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3777633,
            "range": "± 11426",
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
          "id": "89b413c9dd5ad9e18a711d817ec1040096c9bccd",
          "message": "Update Wasmi CLI arguments and default config (#849)\n\n* Wasmi CLI: enabled Wasm tail-calls and extended-const\r\n\r\nThose Wasm proposals are not stabilized, yet but we can assume them to be stabilized very soon so enabling them by default is probably a good idea.\r\n\r\n* Wasmi CLI: add --lazy to enable lazy compilation",
          "timestamp": "2023-12-17T14:26:20+01:00",
          "tree_id": "d065f6e1a9e882b105cbd5f4f80032d8914294f4",
          "url": "https://github.com/paritytech/wasmi/commit/89b413c9dd5ad9e18a711d817ec1040096c9bccd"
        },
        "date": 1702820008348,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8846551,
            "range": "± 16081",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9216732,
            "range": "± 16870",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 4032112,
            "range": "± 9657",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6673333,
            "range": "± 16096",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 7116483,
            "range": "± 17641",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 463362,
            "range": "± 2668",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 133145575,
            "range": "± 117430",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 144919049,
            "range": "± 464119",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 56497116,
            "range": "± 69573",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 102850950,
            "range": "± 405389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 113181241,
            "range": "± 199132",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3992572,
            "range": "± 27973",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6218471,
            "range": "± 17745",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6763883,
            "range": "± 26841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 2590849,
            "range": "± 11791",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4842757,
            "range": "± 28751",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5327449,
            "range": "± 17844",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 240703,
            "range": "± 4525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2388985,
            "range": "± 10230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2617199,
            "range": "± 8699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 967130,
            "range": "± 2336",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1820237,
            "range": "± 16446",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 2035070,
            "range": "± 6874",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 45256,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 226564,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 239425,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 107110,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 175048,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 187155,
            "range": "± 1108",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 24553,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 325576,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 345522,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 153278,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 248329,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 266067,
            "range": "± 927",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 28614,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 470365,
            "range": "± 1088",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 505444,
            "range": "± 1535",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 212952,
            "range": "± 785",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 362626,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 394011,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 31906,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54908,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 383250,
            "range": "± 438",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 665150,
            "range": "± 1421",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1070566,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7488523,
            "range": "± 9308",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1282974,
            "range": "± 4117",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961373,
            "range": "± 3109",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1621034,
            "range": "± 1178",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 751066,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1335062,
            "range": "± 2122",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 332594,
            "range": "± 948",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 375005,
            "range": "± 1198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 401821,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 37292,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64844,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12182877,
            "range": "± 5904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7220800,
            "range": "± 19485",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13153463,
            "range": "± 17098",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3782790,
            "range": "± 5026",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1265388,
            "range": "± 1381",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2384978,
            "range": "± 4092",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1316747,
            "range": "± 1435",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1344478,
            "range": "± 2307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3542373,
            "range": "± 4983",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 984551,
            "range": "± 1013",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2106506,
            "range": "± 3231",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1228540,
            "range": "± 3087",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3792731,
            "range": "± 44475",
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
          "id": "f495637497fac49c83c18e9a97a2b048f6b3b9d0",
          "message": "Prepare for `v0.32.0-beta.0` release (#850)\n\n* write changelog for v0.32.0-beta.0 release\r\n\r\n* bump crate versions\r\n\r\n* fix invalid markdown links in changelog\r\n\r\n* move item in changelog from changed to dev.note\r\n\r\n* improve consistency of changelog\r\n\r\n* improve changelog writing\r\n\r\n* describe lazy compilation feature",
          "timestamp": "2023-12-17T14:57:38+01:00",
          "tree_id": "59d6fe661bcd6aa9425bfed893dd0e0f3658667c",
          "url": "https://github.com/paritytech/wasmi/commit/f495637497fac49c83c18e9a97a2b048f6b3b9d0"
        },
        "date": 1702821892910,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8816658,
            "range": "± 31640",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9262164,
            "range": "± 16711",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 4023072,
            "range": "± 6843",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6712580,
            "range": "± 8712",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 7180895,
            "range": "± 9159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 468620,
            "range": "± 2815",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 133473322,
            "range": "± 516194",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 144633758,
            "range": "± 475358",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 55894096,
            "range": "± 80814",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 103033968,
            "range": "± 191935",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 113166291,
            "range": "± 148904",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 4074324,
            "range": "± 22305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6237775,
            "range": "± 20253",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6765484,
            "range": "± 29057",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 2568872,
            "range": "± 6342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4835685,
            "range": "± 10709",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5338850,
            "range": "± 16812",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 248967,
            "range": "± 6907",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2411975,
            "range": "± 23187",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2614884,
            "range": "± 17508",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 953203,
            "range": "± 1587",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1828541,
            "range": "± 4243",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 2031565,
            "range": "± 5645",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 45505,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 227959,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242033,
            "range": "± 516",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 107520,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 175306,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 188132,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 24630,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 329383,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 347641,
            "range": "± 1676",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 152474,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 249792,
            "range": "± 1914",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 267546,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 28594,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 473331,
            "range": "± 2051",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 505974,
            "range": "± 2305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 211417,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 364358,
            "range": "± 1908",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 397618,
            "range": "± 1893",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 32021,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54821,
            "range": "± 1897",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 387872,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 674037,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1058740,
            "range": "± 1411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7491443,
            "range": "± 5784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1305126,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 961453,
            "range": "± 1256",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1621199,
            "range": "± 2042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750764,
            "range": "± 1640",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1334709,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 337114,
            "range": "± 633",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 367125,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 395187,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36473,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64933,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12191297,
            "range": "± 4380",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7237288,
            "range": "± 4940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13151888,
            "range": "± 31814",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3791861,
            "range": "± 11328",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1263777,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2343641,
            "range": "± 4440",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1316327,
            "range": "± 692",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1345878,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3538763,
            "range": "± 2102",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 870124,
            "range": "± 1161",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2007449,
            "range": "± 8708",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1175806,
            "range": "± 8455",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3812472,
            "range": "± 12800",
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
          "id": "32272351a949adde3a0f50f409179770dbbb0882",
          "message": "Improve `Error` API (#851)\n\n* remove Error::{kind_mut, into_kind} methods\r\n\r\n* add Error::downcast[{ref,mut}] methods\r\n\r\nThey replace the Error::{as_host[_mut], into_host} methods.",
          "timestamp": "2023-12-18T10:02:21+01:00",
          "tree_id": "66ddaee4b769a14c20bca2ef467717b7a832eb0b",
          "url": "https://github.com/paritytech/wasmi/commit/32272351a949adde3a0f50f409179770dbbb0882"
        },
        "date": 1702890576080,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8693420,
            "range": "± 10614",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9140988,
            "range": "± 15172",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 3985001,
            "range": "± 6926",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6661018,
            "range": "± 34962",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 7063174,
            "range": "± 16699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 463923,
            "range": "± 1800",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 132918225,
            "range": "± 268771",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 143564023,
            "range": "± 188469",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 55190157,
            "range": "± 134787",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 101841222,
            "range": "± 250613",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 112378088,
            "range": "± 236337",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3729412,
            "range": "± 57144",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6143005,
            "range": "± 10280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6659452,
            "range": "± 24201",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 2540049,
            "range": "± 3866",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4768323,
            "range": "± 8105",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 5247953,
            "range": "± 13358",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 244228,
            "range": "± 2466",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2404076,
            "range": "± 6763",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2625568,
            "range": "± 8580",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 950569,
            "range": "± 2658",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1823212,
            "range": "± 4353",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 2042941,
            "range": "± 15160",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 45397,
            "range": "± 131",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 226270,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 240186,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 107226,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 174853,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 187026,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 24605,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 326505,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 347332,
            "range": "± 1543",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 151828,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 248121,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 266454,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 28210,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 472028,
            "range": "± 1521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 506008,
            "range": "± 1185",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 210456,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 361751,
            "range": "± 1615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 395515,
            "range": "± 1533",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 31752,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57789,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 385340,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 677001,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1087293,
            "range": "± 1968",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7479203,
            "range": "± 9190",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1287094,
            "range": "± 3810",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 965571,
            "range": "± 6614",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1621623,
            "range": "± 1180",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 751001,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1394396,
            "range": "± 4453",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 337255,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 387309,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 413763,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 37840,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66925,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12208789,
            "range": "± 8497",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7244632,
            "range": "± 10887",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13495976,
            "range": "± 25763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4065764,
            "range": "± 8141",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1263238,
            "range": "± 1728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2464867,
            "range": "± 7719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1319663,
            "range": "± 4045",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1346511,
            "range": "± 1572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3542597,
            "range": "± 7507",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 872682,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1917149,
            "range": "± 2600",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1178062,
            "range": "± 2434",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 3799158,
            "range": "± 7401",
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
          "id": "99cae60ec64e028ec71b0819e58b9bc2774a10b4",
          "message": "Add lazy validation compilation mode (#854)\n\n* rename CompilationMode::Lazy -> LazyTranslation\r\n\r\n* add CompilationMode::Lazy\r\n\r\nThis new mode allows for partial Wasm module validation and defers both Wasm translation and validation to first use.\r\n\r\n* add support for --compilation-mode in Wasmi CLI\r\n\r\n* adjust benchmarks for new compilation modes\r\n\r\n* fix internal doc link",
          "timestamp": "2023-12-18T11:43:56+01:00",
          "tree_id": "4330894bf51b08d529fb2049697af5633a486324",
          "url": "https://github.com/paritytech/wasmi/commit/99cae60ec64e028ec71b0819e58b9bc2774a10b4"
        },
        "date": 1702896669698,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8692442,
            "range": "± 38979",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9190927,
            "range": "± 21641",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 3997270,
            "range": "± 16306",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 478198,
            "range": "± 1874",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6581068,
            "range": "± 14367",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 132918491,
            "range": "± 467342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 143856340,
            "range": "± 543604",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 55646937,
            "range": "± 94089",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3800148,
            "range": "± 25692",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 101626348,
            "range": "± 384766",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6174118,
            "range": "± 23355",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6650042,
            "range": "± 22408",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2579845,
            "range": "± 10256",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 260284,
            "range": "± 3646",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4763761,
            "range": "± 21820",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2382063,
            "range": "± 14081",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2595518,
            "range": "± 8819",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 953470,
            "range": "± 11805",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46590,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1809532,
            "range": "± 4915",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 228072,
            "range": "± 803",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242128,
            "range": "± 1101",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 107212,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24473,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 174856,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 330470,
            "range": "± 1745",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 350112,
            "range": "± 22354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 151752,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29226,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 253925,
            "range": "± 1361",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 470605,
            "range": "± 1711",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 506029,
            "range": "± 1584",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 211708,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 33351,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 367068,
            "range": "± 2515",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56410,
            "range": "± 1284",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 386114,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 687216,
            "range": "± 1681",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1068957,
            "range": "± 3907",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7517555,
            "range": "± 10719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1324643,
            "range": "± 2128",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 953138,
            "range": "± 1784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1614047,
            "range": "± 19617",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 750077,
            "range": "± 919",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1328772,
            "range": "± 3238",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 338182,
            "range": "± 1265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 362907,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 388409,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35953,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64683,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12255667,
            "range": "± 18305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7245931,
            "range": "± 17023",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13219690,
            "range": "± 11546",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3771860,
            "range": "± 8820",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1303778,
            "range": "± 4080",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2296303,
            "range": "± 12456",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1316240,
            "range": "± 6000",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1355956,
            "range": "± 2295",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3522697,
            "range": "± 9037",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 871088,
            "range": "± 1781",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2029823,
            "range": "± 5016",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1164566,
            "range": "± 5110",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4053497,
            "range": "± 8706",
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
          "id": "e958d6ab686501136c8ba633f25db1249c514354",
          "message": "Add Wasmtime to differential fuzzing backend (#856)\n\n* improve clarification for CompilationMode::Lazy\r\n\r\n* refactor differential fuzzing\r\n\r\n* use Box<str> instead of String\r\n\r\n* rename T to Func\r\n\r\n* improve Debug and Display of F32 and F64 for NaNs\r\n\r\n* refactor differential fuzzing\r\n\r\n* add docs for call method\r\n\r\n* correct comment\r\n\r\n* cleanup code\r\n\r\n* improve message formatting\r\n\r\n* add Wasmtime differential fuzzing backend",
          "timestamp": "2023-12-18T16:07:04+01:00",
          "tree_id": "870fdb850a834c3365cc7c0d57a1b7aa12d89157",
          "url": "https://github.com/paritytech/wasmi/commit/e958d6ab686501136c8ba633f25db1249c514354"
        },
        "date": 1702912527204,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8808425,
            "range": "± 16816",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9323394,
            "range": "± 27667",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4022559,
            "range": "± 10774",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 487335,
            "range": "± 2619",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6728659,
            "range": "± 21213",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 133835092,
            "range": "± 264657",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 144893044,
            "range": "± 270102",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 56597188,
            "range": "± 85046",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3999459,
            "range": "± 16050",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 103375132,
            "range": "± 364082",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6218528,
            "range": "± 19594",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6716924,
            "range": "± 23189",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2600343,
            "range": "± 6364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 252426,
            "range": "± 6434",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4848676,
            "range": "± 10875",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2394505,
            "range": "± 10674",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2612190,
            "range": "± 18262",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 968563,
            "range": "± 2310",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 47476,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1833164,
            "range": "± 4159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 230864,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 243512,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108313,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 25144,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 177397,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 332272,
            "range": "± 2135",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 349782,
            "range": "± 1900",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 152830,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29590,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 252898,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 475906,
            "range": "± 1733",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 507342,
            "range": "± 2102",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 214135,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 33173,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 368726,
            "range": "± 1113",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52514,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 386131,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 688788,
            "range": "± 4243",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1064120,
            "range": "± 631",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7505411,
            "range": "± 4262",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1436392,
            "range": "± 1991",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 955725,
            "range": "± 1772",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1616148,
            "range": "± 3199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748826,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1334808,
            "range": "± 3019",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 337909,
            "range": "± 491",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 361567,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 387831,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36937,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 69925,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12279748,
            "range": "± 17943",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7257559,
            "range": "± 20203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13301079,
            "range": "± 30466",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3760209,
            "range": "± 3577",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1301441,
            "range": "± 1322",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2307333,
            "range": "± 5590",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1314122,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1357741,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3530887,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 876932,
            "range": "± 1842",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1973453,
            "range": "± 9741",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1156358,
            "range": "± 2065",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4024411,
            "range": "± 5356",
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
          "id": "86d097623592dc499852ea4bb01cace0097d70a4",
          "message": "Add `Error::host` constructor (#857)\n\nadd Error::host constructor\r\n\r\nUnfortunately we cannot add From<HostError> for Error since that conflicts with its other From impls which is a slight bummer to user experience.",
          "timestamp": "2023-12-18T16:17:50+01:00",
          "tree_id": "f26c6254738c0f586ff8e871f18f32dceb4ef82a",
          "url": "https://github.com/paritytech/wasmi/commit/86d097623592dc499852ea4bb01cace0097d70a4"
        },
        "date": 1702913155297,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8938550,
            "range": "± 16601",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9522986,
            "range": "± 296375",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4127661,
            "range": "± 8325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 484581,
            "range": "± 1423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 7105085,
            "range": "± 16256",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 134069209,
            "range": "± 253061",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 145353166,
            "range": "± 356141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 57549721,
            "range": "± 140228",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4112518,
            "range": "± 29525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 110917750,
            "range": "± 122509",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6253431,
            "range": "± 24554",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6772494,
            "range": "± 30991",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2634878,
            "range": "± 6503",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 248260,
            "range": "± 2350",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 5098088,
            "range": "± 9258",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2388154,
            "range": "± 10141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2619986,
            "range": "± 24401",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 974043,
            "range": "± 5434",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46721,
            "range": "± 63",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1960215,
            "range": "± 3400",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 231768,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 245512,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 111117,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24906,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 188578,
            "range": "± 1357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 334583,
            "range": "± 1400",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 352193,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 156462,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29828,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 267796,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 477538,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 512426,
            "range": "± 1854",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 215904,
            "range": "± 1681",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32993,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 388837,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54720,
            "range": "± 2146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 400574,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 686722,
            "range": "± 2259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1077866,
            "range": "± 8511",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7500988,
            "range": "± 5401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1329548,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 979942,
            "range": "± 1146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1610409,
            "range": "± 1054",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748477,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1328240,
            "range": "± 2740",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 338490,
            "range": "± 678",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 359962,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 386453,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36935,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64155,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12261564,
            "range": "± 6534",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7231050,
            "range": "± 5028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13119198,
            "range": "± 18137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3721331,
            "range": "± 5014",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1297960,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2302994,
            "range": "± 13472",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1312908,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1357202,
            "range": "± 1420",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3520085,
            "range": "± 3435",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1335589,
            "range": "± 886",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2284394,
            "range": "± 4344",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1843513,
            "range": "± 1430",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4049957,
            "range": "± 9944",
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
          "id": "85b17e9bfaa905f53dcce0a3c5abfeb5479ce59f",
          "message": "Prepare `v0.32.0-beta.1` release (#858)\n\n* update changelog for v0.32.0-beta.1\r\n\r\n* update changelog (again)\r\n\r\n* bump crate versions",
          "timestamp": "2023-12-18T17:24:09+01:00",
          "tree_id": "860dcc52340fd938df7d11f091b5e9b3a621d541",
          "url": "https://github.com/paritytech/wasmi/commit/85b17e9bfaa905f53dcce0a3c5abfeb5479ce59f"
        },
        "date": 1702917140142,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8780475,
            "range": "± 7658",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9231532,
            "range": "± 5368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4043073,
            "range": "± 6894",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 478901,
            "range": "± 1843",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6603948,
            "range": "± 10580",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 132509312,
            "range": "± 603739",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 143241330,
            "range": "± 517795",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 56074277,
            "range": "± 126694",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3735372,
            "range": "± 18316",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 101641872,
            "range": "± 217078",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6119071,
            "range": "± 4158",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6621266,
            "range": "± 8925",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2593840,
            "range": "± 4351",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 249511,
            "range": "± 2766",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4747893,
            "range": "± 4437",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2378558,
            "range": "± 2570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2600381,
            "range": "± 4185",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 973589,
            "range": "± 1183",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46075,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1803012,
            "range": "± 3009",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 228863,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242251,
            "range": "± 317",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108855,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24941,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 175441,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 329604,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 349247,
            "range": "± 1227",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 153100,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29179,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 251324,
            "range": "± 555",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 472142,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 505387,
            "range": "± 944",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 214064,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32435,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 362293,
            "range": "± 1479",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56306,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 386587,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 692448,
            "range": "± 1090",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1096880,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7499694,
            "range": "± 5358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1335126,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 953384,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1610804,
            "range": "± 7822",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748948,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1354906,
            "range": "± 1683",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 340022,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 362599,
            "range": "± 428",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 388595,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 35746,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64048,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 12260622,
            "range": "± 4450",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7233561,
            "range": "± 6711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13110168,
            "range": "± 14476",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 3797835,
            "range": "± 5722",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1298533,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2292793,
            "range": "± 5437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1312423,
            "range": "± 2494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1357660,
            "range": "± 1055",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3519657,
            "range": "± 2454",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 869373,
            "range": "± 1468",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2146854,
            "range": "± 2684",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1176608,
            "range": "± 3082",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4155501,
            "range": "± 9406",
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
          "id": "f96475ef1135d11858e0859cee97ea07f93c6d3d",
          "message": "Relaxed branch+cmp offset encoding (#860)\n\n* add BranchCmpFallback instruction\r\n\r\nTranslation does not yet actually emit the new instruction. Needs to be implemented in another commit.\r\n\r\n* exchange Err with unreachable! in offset init\r\n\r\n* apply rustfmt\r\n\r\n* remove BranchOffset16::new\r\n\r\nSuperseeded by TryFrom impl.\r\n\r\n* refactor BranchOffset::from_src_to_dst\r\n\r\n* add Error section to docs\r\n\r\n* apply rustfmt\r\n\r\n* remove superseeded replacements\r\n\r\nThese are no longer needed since the BranchI32Eqz and BranchI32Nez instruction have been removed some time ago.\r\n\r\n* encode fallback branch+cmp for 32-bit offsets\r\n\r\nThis is encoding only for forward branches.\r\n\r\n* add #Error doc section to init method\r\n\r\n* reduce line noise\r\n\r\n* implement branch+cmp fallback encoding for forward jumps\r\n\r\n* disable std feature of num-traits dependency",
          "timestamp": "2023-12-20T23:23:05+01:00",
          "tree_id": "7e70e2556011fdfe3bfd7d559cd16ce3ad5c2ef2",
          "url": "https://github.com/paritytech/wasmi/commit/f96475ef1135d11858e0859cee97ea07f93c6d3d"
        },
        "date": 1703111397434,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8773782,
            "range": "± 20276",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9206452,
            "range": "± 27207",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 3993612,
            "range": "± 29205",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 487310,
            "range": "± 1395",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6852158,
            "range": "± 29217",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 135430565,
            "range": "± 175102",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 146158965,
            "range": "± 474106",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 55956483,
            "range": "± 84674",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3900483,
            "range": "± 36327",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 106200174,
            "range": "± 244152",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6245944,
            "range": "± 17920",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6730250,
            "range": "± 14570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2559153,
            "range": "± 4793",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 250245,
            "range": "± 1301",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4944931,
            "range": "± 10696",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2442881,
            "range": "± 4139",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2657395,
            "range": "± 4795",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 964010,
            "range": "± 3861",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46138,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1892302,
            "range": "± 4066",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 230810,
            "range": "± 808",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242956,
            "range": "± 629",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 107264,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24319,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 180228,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 332593,
            "range": "± 1451",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 350667,
            "range": "± 2536",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 151398,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 28880,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 256611,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 480848,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 512955,
            "range": "± 1372",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 211143,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 31975,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 374830,
            "range": "± 1451",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53567,
            "range": "± 1445",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 391212,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 727141,
            "range": "± 752",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1201700,
            "range": "± 1198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7783714,
            "range": "± 12962",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1301030,
            "range": "± 1049",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 946512,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1616643,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 783673,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1351411,
            "range": "± 995",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 291673,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 373932,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 391581,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36772,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66374,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11273266,
            "range": "± 9066",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7036168,
            "range": "± 9578",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13352583,
            "range": "± 10694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4729578,
            "range": "± 2439",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1240268,
            "range": "± 2452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2317440,
            "range": "± 8757",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1270177,
            "range": "± 1316",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1323694,
            "range": "± 2058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3522710,
            "range": "± 14625",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 928874,
            "range": "± 2138",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 3055474,
            "range": "± 13837",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1293693,
            "range": "± 1574",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4594613,
            "range": "± 6936",
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
          "id": "51177f798af8c31d782f2a1554b31edaac73448c",
          "message": "Refactor code map synchronization (#862)\n\n* swap args of CodeMap::init_lazy_func\r\n\r\n* refactor CodeMap synchronization\r\n\r\n- This fixes some flaws in how the CodeMap orchestras synchronization for function initialisation, compilation and queries.\r\n- Furthermore this provides full &self API for CodeMap.\r\n\r\n* remove synchronization overhead for init methods\r\n\r\n They take &mut self so we do not need synchronization until we want support for this feature maybe in the future.\r\n\r\n* fix doc links\r\n\r\n* make InternalFuncEntity private\r\n\r\n* use unreachable hint for happy hot path\r\n\r\n* use inline and cold annotations as hints",
          "timestamp": "2023-12-22T16:11:05+01:00",
          "tree_id": "4485dc910f704e2387fa14e50b92947ea21cc97b",
          "url": "https://github.com/paritytech/wasmi/commit/51177f798af8c31d782f2a1554b31edaac73448c"
        },
        "date": 1703258413292,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8779177,
            "range": "± 30180",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9264278,
            "range": "± 33378",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4102348,
            "range": "± 19041",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 487238,
            "range": "± 2505",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6765033,
            "range": "± 15521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 134863392,
            "range": "± 1805451",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 145837027,
            "range": "± 514856",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 57123918,
            "range": "± 142052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4599676,
            "range": "± 48906",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 104510293,
            "range": "± 202212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6337675,
            "range": "± 52722",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6849325,
            "range": "± 64144",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2674769,
            "range": "± 11348",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 255320,
            "range": "± 4825",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4992071,
            "range": "± 29612",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2414860,
            "range": "± 5893",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2627923,
            "range": "± 7794",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 987183,
            "range": "± 2120",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46897,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1855726,
            "range": "± 6633",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 227237,
            "range": "± 2214",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 239408,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108900,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 25043,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 176646,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 328976,
            "range": "± 4515",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 346160,
            "range": "± 1707",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 154258,
            "range": "± 1369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29664,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 250212,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 471910,
            "range": "± 3870",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 504056,
            "range": "± 1703",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 214317,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32874,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 367278,
            "range": "± 1704",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57393,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 398890,
            "range": "± 4385",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 718347,
            "range": "± 1394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1215763,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7728945,
            "range": "± 4223",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1366283,
            "range": "± 1674",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 951261,
            "range": "± 3221",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1622641,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 785611,
            "range": "± 514",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1341378,
            "range": "± 1282",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 307834,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 369225,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 389394,
            "range": "± 589",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 37180,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66517,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11190143,
            "range": "± 24101",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6982937,
            "range": "± 8928",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13175712,
            "range": "± 17449",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4864727,
            "range": "± 19370",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1255027,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2307564,
            "range": "± 6549",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1264731,
            "range": "± 2701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1327784,
            "range": "± 2603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3534128,
            "range": "± 4100",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 845798,
            "range": "± 3835",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2593359,
            "range": "± 2582",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1169283,
            "range": "± 2978",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4344420,
            "range": "± 12380",
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
          "id": "65a9b8ad0e2b5fe8674935af03d9f6a457012e1b",
          "message": "Prepare release of `v0.32.0-beta.2` (#863)\n\n* fix changelog (minor nit)\r\n\r\n* bump crate versions",
          "timestamp": "2023-12-22T17:04:11+01:00",
          "tree_id": "b4cfd43029abe1c0b0d9ce3f1bd73f6fcffc0b44",
          "url": "https://github.com/paritytech/wasmi/commit/65a9b8ad0e2b5fe8674935af03d9f6a457012e1b"
        },
        "date": 1703261585718,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8782787,
            "range": "± 22496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9286527,
            "range": "± 36405",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4059694,
            "range": "± 11909",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 484378,
            "range": "± 1404",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6749716,
            "range": "± 16569",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 134285217,
            "range": "± 398757",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 145411808,
            "range": "± 451732",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 56870891,
            "range": "± 189173",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4279336,
            "range": "± 69691",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 104890391,
            "range": "± 366920",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6248150,
            "range": "± 31701",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6791119,
            "range": "± 42099",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2615862,
            "range": "± 18665",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 255064,
            "range": "± 1865",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4937250,
            "range": "± 32557",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2409731,
            "range": "± 6660",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2631509,
            "range": "± 14121",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 987516,
            "range": "± 4382",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46925,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1866074,
            "range": "± 7734",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 229997,
            "range": "± 2284",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 241152,
            "range": "± 1735",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 110779,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 25162,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 177692,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 325763,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 345716,
            "range": "± 1631",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 153642,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 30040,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 253417,
            "range": "± 769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 473190,
            "range": "± 2820",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 504922,
            "range": "± 1319",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 216626,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32967,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 368883,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54390,
            "range": "± 1400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 389361,
            "range": "± 2493",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 718108,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1209356,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7722086,
            "range": "± 5416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1358800,
            "range": "± 2312",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 951219,
            "range": "± 1335",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1622219,
            "range": "± 1324",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 784973,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1342973,
            "range": "± 3163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 308006,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 369535,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 388272,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 36349,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 66063,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11196260,
            "range": "± 9118",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6974964,
            "range": "± 7494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 13196876,
            "range": "± 14988",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 5053033,
            "range": "± 18599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1352119,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2328100,
            "range": "± 6127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1262375,
            "range": "± 1844",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1325572,
            "range": "± 2879",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3514175,
            "range": "± 2973",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 851088,
            "range": "± 935",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2524275,
            "range": "± 5155",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1151879,
            "range": "± 2873",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4331393,
            "range": "± 3858",
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
          "id": "105583feb3a493370ffeaf23b42a0f2bf869cc03",
          "message": "Add inline annotations to call exec handlers (#866)\n\nadd inline annotations to call exec handlers",
          "timestamp": "2023-12-25T10:08:33+01:00",
          "tree_id": "94513f147ee5ca60538a0f9873829ca4c37f357a",
          "url": "https://github.com/paritytech/wasmi/commit/105583feb3a493370ffeaf23b42a0f2bf869cc03"
        },
        "date": 1703495821422,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8945079,
            "range": "± 93249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9404109,
            "range": "± 108906",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4166639,
            "range": "± 34083",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 506506,
            "range": "± 6349",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 7057284,
            "range": "± 65916",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 137812068,
            "range": "± 912226",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 147609038,
            "range": "± 1668852",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 57616183,
            "range": "± 325904",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 5461820,
            "range": "± 93025",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 106814021,
            "range": "± 723830",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6477723,
            "range": "± 46899",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6994312,
            "range": "± 61633",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2767568,
            "range": "± 11730",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 264775,
            "range": "± 4439",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 5120322,
            "range": "± 39161",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2444622,
            "range": "± 18305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2665766,
            "range": "± 28163",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 982737,
            "range": "± 7376",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 47333,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1917635,
            "range": "± 9428",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 228929,
            "range": "± 1945",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 241566,
            "range": "± 1013",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108848,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24930,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 179320,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 329236,
            "range": "± 2531",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 347875,
            "range": "± 3036",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 152969,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29609,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 253613,
            "range": "± 1744",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 471330,
            "range": "± 5570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 506556,
            "range": "± 4021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 214831,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 33108,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 377178,
            "range": "± 3130",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57125,
            "range": "± 1084",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 386260,
            "range": "± 3394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 674098,
            "range": "± 5579",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1147690,
            "range": "± 8999",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7533742,
            "range": "± 29084",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1475088,
            "range": "± 9235",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 959146,
            "range": "± 6424",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1626057,
            "range": "± 9383",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 753724,
            "range": "± 2047",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1250670,
            "range": "± 4318",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 352499,
            "range": "± 1240",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 330567,
            "range": "± 4042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 355431,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 32173,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63629,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11316635,
            "range": "± 53910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 7023124,
            "range": "± 46129",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12371929,
            "range": "± 91523",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4659059,
            "range": "± 48344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1249927,
            "range": "± 10494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2128455,
            "range": "± 12332",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1397968,
            "range": "± 12698",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1249989,
            "range": "± 10704",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3514789,
            "range": "± 30211",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 946208,
            "range": "± 13370",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2586627,
            "range": "± 25440",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1154312,
            "range": "± 7882",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4500646,
            "range": "± 40197",
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
          "id": "f21eec9f9b988b50ebeb5dee019564defa363bd4",
          "message": "Refactor `ValueStackPtr` API (#867)\n\n* refactor ValueStackPtr API\r\n\r\n* add some inline annotations\r\n\r\n* Revert \"add some inline annotations\"\r\n\r\nThis reverts commit 637afeadfd6faba4e5936bb0f362c31523e8d6b2.",
          "timestamp": "2023-12-25T19:46:43+01:00",
          "tree_id": "71c32ba73954108e1eddfeb8fa66c57155973379",
          "url": "https://github.com/paritytech/wasmi/commit/f21eec9f9b988b50ebeb5dee019564defa363bd4"
        },
        "date": 1703530397356,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8852064,
            "range": "± 36006",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9299339,
            "range": "± 32813",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 3978777,
            "range": "± 16371",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 499110,
            "range": "± 2230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6790820,
            "range": "± 47192",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 134024076,
            "range": "± 505287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 144989339,
            "range": "± 411877",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 55927960,
            "range": "± 123169",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4091485,
            "range": "± 68287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 105122080,
            "range": "± 301297",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6235824,
            "range": "± 33662",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6684508,
            "range": "± 23518",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2578720,
            "range": "± 11454",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 254511,
            "range": "± 3558",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4883782,
            "range": "± 17581",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2417951,
            "range": "± 17392",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2630436,
            "range": "± 15122",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 958622,
            "range": "± 3025",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 47023,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1875430,
            "range": "± 5151",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 229007,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242296,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108137,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24900,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 178341,
            "range": "± 477",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 333732,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 350511,
            "range": "± 950",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 152117,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29737,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 256190,
            "range": "± 707",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 472677,
            "range": "± 1431",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 508284,
            "range": "± 2582",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 212690,
            "range": "± 563",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 33196,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 371819,
            "range": "± 3745",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54603,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 382548,
            "range": "± 922",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 667682,
            "range": "± 1881",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1141226,
            "range": "± 1382",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7501594,
            "range": "± 36014",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1457010,
            "range": "± 1959",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 949778,
            "range": "± 3403",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1605252,
            "range": "± 11717",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 751081,
            "range": "± 1241",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1225214,
            "range": "± 2550",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 302512,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 331814,
            "range": "± 508",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 355278,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 31930,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 63661,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11217624,
            "range": "± 24434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6967304,
            "range": "± 9670",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12223511,
            "range": "± 18720",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4390194,
            "range": "± 17937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1235193,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2087763,
            "range": "± 3385",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1386431,
            "range": "± 2332",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1232517,
            "range": "± 1949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3444182,
            "range": "± 7852",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 868678,
            "range": "± 4327",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2612424,
            "range": "± 17041",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1220348,
            "range": "± 2838",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4476671,
            "range": "± 18648",
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
          "id": "688d9869541eaf70885a024d7d49fa76b8e382a9",
          "message": "Rename `ValueStackPtr` to `FrameRegisters` (#868)\n\n* remove unused NonCopy type\r\n\r\n* rename ValueStackPtr to FrameRegisters\r\n\r\n* refactor FrameRegisters construction\r\n\r\n* remove unnecessary unsafe fn annotation\r\n\r\n* fix internal docs\r\n\r\n* refactor root_stack_ptr impl",
          "timestamp": "2023-12-25T21:32:35+01:00",
          "tree_id": "e3fafd1da2611ea2610de0121b0b26c469d8f52d",
          "url": "https://github.com/paritytech/wasmi/commit/688d9869541eaf70885a024d7d49fa76b8e382a9"
        },
        "date": 1703536877286,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8729227,
            "range": "± 8530",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9153337,
            "range": "± 14911",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 3975099,
            "range": "± 8378",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 481005,
            "range": "± 1941",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6682968,
            "range": "± 16906",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 133402776,
            "range": "± 417827",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 144348642,
            "range": "± 435827",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 56115721,
            "range": "± 72553",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3893138,
            "range": "± 16425",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 104406870,
            "range": "± 421993",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6162421,
            "range": "± 12924",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6641586,
            "range": "± 9890",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2567970,
            "range": "± 7161",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 248256,
            "range": "± 4183",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4838957,
            "range": "± 13069",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2393436,
            "range": "± 18861",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2605136,
            "range": "± 8572",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 1016398,
            "range": "± 2487",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46677,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1861288,
            "range": "± 16824",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 226826,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 241581,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 106926,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24932,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 177855,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 328432,
            "range": "± 942",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 346427,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 151006,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29623,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 252274,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 472101,
            "range": "± 4987",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 509905,
            "range": "± 1961",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 212877,
            "range": "± 591",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32953,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 369632,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56852,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 380021,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 665772,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1101960,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481955,
            "range": "± 6642",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1403534,
            "range": "± 2080",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 947173,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1618040,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 749433,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1222273,
            "range": "± 2016",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 331766,
            "range": "± 995",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 329552,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 353554,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 32242,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 64292,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11171477,
            "range": "± 28700",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6987534,
            "range": "± 28969",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12225878,
            "range": "± 9626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4371254,
            "range": "± 7660",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1234478,
            "range": "± 663",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2083497,
            "range": "± 2323",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1396558,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1234699,
            "range": "± 1188",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3444281,
            "range": "± 6978",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 864698,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2567680,
            "range": "± 2883",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1149449,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4330049,
            "range": "± 6075",
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
          "id": "bd5b61302e2fc5f6c287db9116c62c2c51e12c07",
          "message": "Add `Sync` impl to `ResumableInvocation` (#870)\n\n* add Sync impl to ResumableInvocation\r\n\r\n* apply rustfmt",
          "timestamp": "2023-12-26T10:59:02+01:00",
          "tree_id": "9a707958f764458a450cb42154084d1c7061afc8",
          "url": "https://github.com/paritytech/wasmi/commit/bd5b61302e2fc5f6c287db9116c62c2c51e12c07"
        },
        "date": 1703585224571,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 8779781,
            "range": "± 42076",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9224010,
            "range": "± 36934",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4005818,
            "range": "± 10770",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 506753,
            "range": "± 2769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 6943456,
            "range": "± 18423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 135248898,
            "range": "± 462664",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 145241346,
            "range": "± 788236",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 56837131,
            "range": "± 209868",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4043336,
            "range": "± 26816",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 107489176,
            "range": "± 342057",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6235871,
            "range": "± 9039",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6690531,
            "range": "± 18611",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2620377,
            "range": "± 9172",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 273260,
            "range": "± 6091",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 4965661,
            "range": "± 7571",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2416473,
            "range": "± 12160",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2632901,
            "range": "± 18991",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 984496,
            "range": "± 3719",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 48129,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1927543,
            "range": "± 10627",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 228336,
            "range": "± 1111",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 242237,
            "range": "± 1888",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 107874,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 25947,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 184662,
            "range": "± 918",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 330824,
            "range": "± 3003",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 349552,
            "range": "± 1841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 152601,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 30542,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 262477,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 478058,
            "range": "± 2815",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 509845,
            "range": "± 1695",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 214390,
            "range": "± 423",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 33971,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 389682,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56020,
            "range": "± 1563",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 380715,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 658963,
            "range": "± 1466",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1106766,
            "range": "± 2501",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7486377,
            "range": "± 12150",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1475475,
            "range": "± 4310",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 946800,
            "range": "± 3833",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1617231,
            "range": "± 3129",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 748131,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1347255,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 327561,
            "range": "± 1950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 332581,
            "range": "± 1007",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 357299,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 32057,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 65573,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11175382,
            "range": "± 6432",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6994135,
            "range": "± 14871",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12422322,
            "range": "± 15189",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4331409,
            "range": "± 8068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1233242,
            "range": "± 627",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2096517,
            "range": "± 3060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1394314,
            "range": "± 1559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1234803,
            "range": "± 830",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3452042,
            "range": "± 6103",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 875172,
            "range": "± 2978",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2538701,
            "range": "± 10563",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1176466,
            "range": "± 2584",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4298839,
            "range": "± 13929",
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
          "id": "1e038b6a889aae0ba2e1566862e3f4c77621ea00",
          "message": "New spelling for Wasmi in the codebase (#872)\n\n* new spelling for Wasmi in readme\r\n\r\n* new Wasmi spelling in md files generally\r\n\r\n* new Wasmi spelling in .rs docs and comments",
          "timestamp": "2024-01-04T11:42:45+01:00",
          "tree_id": "1d22681e64cd3eb50c313a326f544c3c0f5d70a5",
          "url": "https://github.com/paritytech/wasmi/commit/1e038b6a889aae0ba2e1566862e3f4c77621ea00"
        },
        "date": 1704365369755,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 9182417,
            "range": "± 9539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 9605361,
            "range": "± 8159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 4004278,
            "range": "± 4966",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 491631,
            "range": "± 2090",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 7041575,
            "range": "± 9331",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 140257990,
            "range": "± 362270",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 151236837,
            "range": "± 129998",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 55503400,
            "range": "± 37442",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3885169,
            "range": "± 4634",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 107280058,
            "range": "± 79699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 6464386,
            "range": "± 4768",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 6975296,
            "range": "± 6511",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 2573356,
            "range": "± 6966",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 258947,
            "range": "± 3768",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 5029746,
            "range": "± 14695",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 2541052,
            "range": "± 5966",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 2763903,
            "range": "± 9351",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 963745,
            "range": "± 1350",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 46896,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1934427,
            "range": "± 3248",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 239737,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 254289,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 108393,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 24801,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 188103,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 347750,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 367611,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 153087,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 29698,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 264419,
            "range": "± 596",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 496978,
            "range": "± 1066",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 534772,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 213282,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 32979,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 386583,
            "range": "± 1802",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56337,
            "range": "± 736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 384142,
            "range": "± 1101",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 668617,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 1127982,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481596,
            "range": "± 4488",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1303483,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 946093,
            "range": "± 2644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1612548,
            "range": "± 11815",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 749385,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1206766,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 322613,
            "range": "± 1753",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 329788,
            "range": "± 436",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 353369,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 31799,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 67110,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 11148504,
            "range": "± 4130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6978048,
            "range": "± 4393",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 12221088,
            "range": "± 12728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 4394900,
            "range": "± 13891",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1235142,
            "range": "± 2858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 2085234,
            "range": "± 2644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1392832,
            "range": "± 722",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1234190,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3438533,
            "range": "± 1536",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 888757,
            "range": "± 1058",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 2577277,
            "range": "± 2483",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1149710,
            "range": "± 3051",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 4538241,
            "range": "± 72132",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}