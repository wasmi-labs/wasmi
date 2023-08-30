window.BENCHMARK_DATA = {
  "lastUpdate": 1693428929622,
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
      }
    ]
  }
}