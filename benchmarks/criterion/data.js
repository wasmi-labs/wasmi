window.BENCHMARK_DATA = {
  "lastUpdate": 1697459401541,
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
        "date": 1690295226748,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3798532,
            "range": "± 40198",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 55962726,
            "range": "± 357651",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 91417,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 128335,
            "range": "± 1116",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 185855,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 50746,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 317302,
            "range": "± 1005",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 419799,
            "range": "± 1359",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 469046,
            "range": "± 4928",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620567,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1421852,
            "range": "± 23052",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 729103,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1253268,
            "range": "± 3533",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1310970,
            "range": "± 8141",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1398292,
            "range": "± 9097",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1574602,
            "range": "± 5861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1619405,
            "range": "± 8285",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1647186,
            "range": "± 13370",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1940622,
            "range": "± 15542",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2574275,
            "range": "± 13002",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 740835,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 661228,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 517722,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319329,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 104650,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 140822,
            "range": "± 3450",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10255,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37028,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4196982,
            "range": "± 5699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 971871,
            "range": "± 1072",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1393145,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 710943,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1129674,
            "range": "± 839",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1150556,
            "range": "± 1538",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2300525,
            "range": "± 5936",
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
        "date": 1690295738777,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3673530,
            "range": "± 17849",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 55467060,
            "range": "± 321170",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 91762,
            "range": "± 934",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 128106,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 185970,
            "range": "± 440",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 51459,
            "range": "± 1643",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 318832,
            "range": "± 1051",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 419422,
            "range": "± 1182",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 487456,
            "range": "± 1142",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620933,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1490186,
            "range": "± 22721",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 730824,
            "range": "± 821",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1175859,
            "range": "± 14978",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1325212,
            "range": "± 14502",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1251053,
            "range": "± 36968",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1614584,
            "range": "± 6202",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1530408,
            "range": "± 19932",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1739417,
            "range": "± 14968",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1800613,
            "range": "± 10042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2569099,
            "range": "± 18768",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 741108,
            "range": "± 1387",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 661890,
            "range": "± 1489",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 511748,
            "range": "± 628",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318696,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 103510,
            "range": "± 185",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 139992,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10307,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36307,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4388628,
            "range": "± 7478",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 972325,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1396595,
            "range": "± 3549",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 712497,
            "range": "± 1825",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1130741,
            "range": "± 1564",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1150324,
            "range": "± 2116",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2300748,
            "range": "± 10256",
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
        "date": 1690799417144,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3699005,
            "range": "± 15941",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 56089812,
            "range": "± 1257547",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 91135,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 128592,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 186348,
            "range": "± 1925",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55672,
            "range": "± 2134",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 326347,
            "range": "± 1398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 440668,
            "range": "± 3545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 466066,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620537,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1394977,
            "range": "± 14576",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 737907,
            "range": "± 1520",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1128467,
            "range": "± 33581",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1256079,
            "range": "± 31059",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1224459,
            "range": "± 22111",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1633394,
            "range": "± 56903",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1517596,
            "range": "± 36245",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1598187,
            "range": "± 25613",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1803820,
            "range": "± 29729",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2512238,
            "range": "± 46328",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 716636,
            "range": "± 2092",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 659706,
            "range": "± 1582",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 513851,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319694,
            "range": "± 636",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 102515,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 137666,
            "range": "± 1166",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10029,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36454,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4293217,
            "range": "± 7858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 973440,
            "range": "± 1364",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1399764,
            "range": "± 2515",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 718622,
            "range": "± 5050",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133833,
            "range": "± 2081",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1160317,
            "range": "± 5024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2298942,
            "range": "± 3568",
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
        "date": 1690805858898,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3779455,
            "range": "± 24399",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 55985787,
            "range": "± 40566",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 92343,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 130000,
            "range": "± 538",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 190697,
            "range": "± 2664",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53934,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 319330,
            "range": "± 1966",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 424988,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 454956,
            "range": "± 846",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 621257,
            "range": "± 848",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1355114,
            "range": "± 15197",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 744331,
            "range": "± 2936",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1113623,
            "range": "± 33698",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1244067,
            "range": "± 14485",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1231754,
            "range": "± 19041",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1552017,
            "range": "± 40025",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1523554,
            "range": "± 32486",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1621225,
            "range": "± 27926",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1756628,
            "range": "± 39749",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2591285,
            "range": "± 50872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 721723,
            "range": "± 23072",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658921,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 511613,
            "range": "± 1099",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319152,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 102543,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 136916,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10033,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36976,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4228321,
            "range": "± 10528",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 972526,
            "range": "± 1287",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1396100,
            "range": "± 3468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 709436,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1228023,
            "range": "± 114830",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1158114,
            "range": "± 57763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2306688,
            "range": "± 8277",
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
        "date": 1693428929198,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3747828,
            "range": "± 27214",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 55748870,
            "range": "± 380449",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 91061,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 128323,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 184651,
            "range": "± 554",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 50825,
            "range": "± 1477",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 318442,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 418382,
            "range": "± 1085",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 456035,
            "range": "± 5426",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 591346,
            "range": "± 3840",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1339877,
            "range": "± 18347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 738718,
            "range": "± 5485",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1156400,
            "range": "± 5495",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1256237,
            "range": "± 5667",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1240051,
            "range": "± 5155",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1530039,
            "range": "± 6925",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1554122,
            "range": "± 36001",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1603349,
            "range": "± 7767",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1776572,
            "range": "± 17104",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2567732,
            "range": "± 10719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 740339,
            "range": "± 2872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 661860,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 523879,
            "range": "± 1293",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318664,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 103583,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 137800,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10126,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36976,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4239158,
            "range": "± 8158",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 974307,
            "range": "± 3148",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1402282,
            "range": "± 7543",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 719061,
            "range": "± 15038",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133857,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1149355,
            "range": "± 169070",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2303862,
            "range": "± 4128",
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
        "date": 1694436468893,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3786663,
            "range": "± 15034",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 56777463,
            "range": "± 106035",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 93149,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 132094,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 187259,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52030,
            "range": "± 1377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316373,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 420877,
            "range": "± 5350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455007,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620222,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1353072,
            "range": "± 18776",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 744328,
            "range": "± 3199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1125170,
            "range": "± 38217",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1260734,
            "range": "± 35128",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1244950,
            "range": "± 11687",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1580551,
            "range": "± 51653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1517277,
            "range": "± 35619",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1600859,
            "range": "± 22493",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1773391,
            "range": "± 32688",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2666935,
            "range": "± 27357",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 716892,
            "range": "± 28565",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658189,
            "range": "± 450",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 516481,
            "range": "± 551",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318192,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 102980,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 137303,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10057,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36343,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4233775,
            "range": "± 5700",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 972483,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1394836,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 711311,
            "range": "± 2261",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133152,
            "range": "± 1742",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1157554,
            "range": "± 4168",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2304837,
            "range": "± 6011",
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
        "date": 1694511200002,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3819806,
            "range": "± 11181",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 57294169,
            "range": "± 93768",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 93927,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 133146,
            "range": "± 602",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 190372,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53233,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316175,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421601,
            "range": "± 751",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455460,
            "range": "± 1128",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 621068,
            "range": "± 1309",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1349100,
            "range": "± 1719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 750676,
            "range": "± 5050",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1118015,
            "range": "± 26875",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1249136,
            "range": "± 16511",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1232454,
            "range": "± 15164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1698732,
            "range": "± 28256",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1528537,
            "range": "± 28180",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1603435,
            "range": "± 24259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1768670,
            "range": "± 26872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2601325,
            "range": "± 54643",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 715048,
            "range": "± 1367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658169,
            "range": "± 566",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 520781,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318313,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 102803,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 137315,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10136,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36944,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4230469,
            "range": "± 4180",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 1043208,
            "range": "± 691",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395735,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 710548,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133692,
            "range": "± 164507",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1157600,
            "range": "± 134421",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2296148,
            "range": "± 7131",
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
        "date": 1694523656793,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 3842806,
            "range": "± 35435",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 57348040,
            "range": "± 157269",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 93707,
            "range": "± 332",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 132315,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 188898,
            "range": "± 1071",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54486,
            "range": "± 1025",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316460,
            "range": "± 635",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 422253,
            "range": "± 2307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455672,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 621188,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1339738,
            "range": "± 8927",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 742178,
            "range": "± 4066",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1158836,
            "range": "± 37993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1258961,
            "range": "± 22610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1236437,
            "range": "± 32858",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1551878,
            "range": "± 39539",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1591749,
            "range": "± 32709",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1623486,
            "range": "± 31604",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1788466,
            "range": "± 25265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2638270,
            "range": "± 26269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 714781,
            "range": "± 1634",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658687,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 514162,
            "range": "± 896",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 320265,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 101859,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 136609,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 10086,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36961,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4233457,
            "range": "± 3661",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 972971,
            "range": "± 1556",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395858,
            "range": "± 1612",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 711388,
            "range": "± 1451",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133787,
            "range": "± 1991",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1158480,
            "range": "± 2034",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2300565,
            "range": "± 2345",
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
        "date": 1695290570832,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4355925,
            "range": "± 23926",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64995204,
            "range": "± 84950",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103703,
            "range": "± 172",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 146576,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209998,
            "range": "± 867",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56698,
            "range": "± 1792",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 319529,
            "range": "± 2190",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 420446,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 480160,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 575116,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1378579,
            "range": "± 14130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 727949,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1206852,
            "range": "± 8950",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1374408,
            "range": "± 8042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1367847,
            "range": "± 4455",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1583808,
            "range": "± 21243",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1547018,
            "range": "± 14874",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1630619,
            "range": "± 12575",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1792103,
            "range": "± 24116",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2491589,
            "range": "± 12506",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 728502,
            "range": "± 852",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 681399,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 496701,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 318334,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 93605,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129622,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8714,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36077,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3969450,
            "range": "± 3579",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 860121,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1392963,
            "range": "± 3126",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 666943,
            "range": "± 1807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134005,
            "range": "± 1214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1155027,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2349677,
            "range": "± 3001",
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
        "date": 1695721913766,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4782496,
            "range": "± 44759",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 70973140,
            "range": "± 591109",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 111737,
            "range": "± 1184",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 158392,
            "range": "± 2636",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 226446,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 63965,
            "range": "± 2286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 337608,
            "range": "± 2701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 448286,
            "range": "± 2298",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 482276,
            "range": "± 4416",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 612064,
            "range": "± 5454",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1466247,
            "range": "± 17506",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 787213,
            "range": "± 8659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1238763,
            "range": "± 17555",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1336817,
            "range": "± 11502",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1301229,
            "range": "± 16592",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1689428,
            "range": "± 44599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1657200,
            "range": "± 8761",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1720635,
            "range": "± 56550",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1815869,
            "range": "± 11471",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2733763,
            "range": "± 51905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 786768,
            "range": "± 9475",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 724041,
            "range": "± 4895",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 522545,
            "range": "± 4351",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 339162,
            "range": "± 3316",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 100642,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 138224,
            "range": "± 831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 9342,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 38551,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4165077,
            "range": "± 43025",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 897483,
            "range": "± 5120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1417045,
            "range": "± 11774",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 666052,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1137676,
            "range": "± 98846",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1155271,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2359320,
            "range": "± 2901",
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
        "date": 1697457946309,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4332985,
            "range": "± 10015",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 65315696,
            "range": "± 107525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103783,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147014,
            "range": "± 638",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 210910,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56010,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 315823,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 419742,
            "range": "± 864",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455845,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 574745,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1347748,
            "range": "± 18270",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 730939,
            "range": "± 796",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1230886,
            "range": "± 6148",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1259963,
            "range": "± 5967",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1275739,
            "range": "± 10639",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1616550,
            "range": "± 8922",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1553500,
            "range": "± 15701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1612903,
            "range": "± 37997",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1802928,
            "range": "± 10446",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2596268,
            "range": "± 6352",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 738678,
            "range": "± 43593",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 679781,
            "range": "± 2300",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 491236,
            "range": "± 1721",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 330825,
            "range": "± 1117",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94577,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130424,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8957,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36311,
            "range": "± 2058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4072862,
            "range": "± 7149",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 872135,
            "range": "± 2486",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1399652,
            "range": "± 881",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 663099,
            "range": "± 1883",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134084,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1153060,
            "range": "± 1164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2353835,
            "range": "± 2237",
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
        "date": 1697459401518,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4175680,
            "range": "± 13069",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64132196,
            "range": "± 170240",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103079,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147625,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209880,
            "range": "± 1367",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53986,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 315591,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 422289,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455131,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 574374,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1375902,
            "range": "± 6087",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 729415,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1160764,
            "range": "± 4985",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1251274,
            "range": "± 7337",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1242968,
            "range": "± 12784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1732125,
            "range": "± 9880",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1591312,
            "range": "± 29277",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1666428,
            "range": "± 13861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1874625,
            "range": "± 10204",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2646799,
            "range": "± 13595",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 738323,
            "range": "± 35689",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 679435,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 480616,
            "range": "± 825",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 330260,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94750,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130848,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8840,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36304,
            "range": "± 1275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4043600,
            "range": "± 2470",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 864443,
            "range": "± 862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1398949,
            "range": "± 855",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 661402,
            "range": "± 637",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133291,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1152857,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2348688,
            "range": "± 3847",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}