window.BENCHMARK_DATA = {
  "lastUpdate": 1701205265796,
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
        "date": 1697651606589,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4235479,
            "range": "± 12699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64501886,
            "range": "± 185016",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 104675,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 148032,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 211803,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55405,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 317613,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421836,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 454989,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620695,
            "range": "± 590",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1435294,
            "range": "± 8298",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 737220,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1263536,
            "range": "± 6796",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1249727,
            "range": "± 13286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1251046,
            "range": "± 16442",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1664009,
            "range": "± 79306",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1580498,
            "range": "± 10137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1678281,
            "range": "± 13545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1930126,
            "range": "± 22911",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2552022,
            "range": "± 12793",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 765993,
            "range": "± 2163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 677981,
            "range": "± 3064",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 598986,
            "range": "± 1471",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 340148,
            "range": "± 1996",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 96482,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 128780,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8878,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36968,
            "range": "± 121",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4084311,
            "range": "± 5034",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 857959,
            "range": "± 1042",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1398603,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 664654,
            "range": "± 1691",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1133710,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1148424,
            "range": "± 2437",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2350257,
            "range": "± 5610",
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
        "date": 1699524211232,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4270551,
            "range": "± 23691",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 65280909,
            "range": "± 104589",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103861,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 148123,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 213398,
            "range": "± 313",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55077,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 327569,
            "range": "± 5578",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 425293,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 457751,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 650737,
            "range": "± 409",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1355276,
            "range": "± 19243",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 732210,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1315658,
            "range": "± 4870",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1430805,
            "range": "± 13834",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1387615,
            "range": "± 4904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1724713,
            "range": "± 5314",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1705611,
            "range": "± 9113",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1752999,
            "range": "± 11861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 2009884,
            "range": "± 14238",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2662134,
            "range": "± 16650",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 739911,
            "range": "± 36407",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 661112,
            "range": "± 1350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 494510,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 328524,
            "range": "± 482",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94514,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129953,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8847,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37091,
            "range": "± 135",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4172542,
            "range": "± 4147",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 855819,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1400581,
            "range": "± 2255",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 661540,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1182249,
            "range": "± 861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1213200,
            "range": "± 11707",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2474279,
            "range": "± 135935",
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
        "date": 1699526913570,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4209087,
            "range": "± 9900",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64737717,
            "range": "± 160913",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 104136,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147474,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 213962,
            "range": "± 793",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53674,
            "range": "± 1002",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 322628,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 428387,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 454898,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 649779,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1345741,
            "range": "± 12649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 732201,
            "range": "± 733",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1162384,
            "range": "± 6622",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1251019,
            "range": "± 6949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1221236,
            "range": "± 9415",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1566195,
            "range": "± 5040",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1548218,
            "range": "± 16600",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1586623,
            "range": "± 12365",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1771926,
            "range": "± 14985",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2539135,
            "range": "± 5756",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 741509,
            "range": "± 16536",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 660293,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 499268,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319174,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 96387,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130927,
            "range": "± 1177",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8925,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36196,
            "range": "± 1005",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3987047,
            "range": "± 10788",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 864834,
            "range": "± 2221",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1399951,
            "range": "± 6028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 660894,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1181686,
            "range": "± 1052",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1211357,
            "range": "± 877",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2341864,
            "range": "± 2277",
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
        "date": 1699879901102,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4242804,
            "range": "± 33504",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 65721936,
            "range": "± 508400",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 105567,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147289,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 210731,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55105,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 322861,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 424521,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 457588,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 650610,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1362049,
            "range": "± 28159",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 731670,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1162941,
            "range": "± 6031",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1265272,
            "range": "± 5865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1219749,
            "range": "± 4372",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1548782,
            "range": "± 2862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1570138,
            "range": "± 12582",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1617685,
            "range": "± 12754",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1790127,
            "range": "± 14725",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2564302,
            "range": "± 5995",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 740896,
            "range": "± 37403",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 660889,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 485556,
            "range": "± 946",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319700,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94487,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129763,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8860,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37142,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3971382,
            "range": "± 3689",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 856321,
            "range": "± 1164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1400676,
            "range": "± 2029",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 663046,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1182247,
            "range": "± 1284",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1213057,
            "range": "± 1533",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2347559,
            "range": "± 4769",
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
        "date": 1700162989779,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4218068,
            "range": "± 27055",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64209442,
            "range": "± 220917",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 104080,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 146444,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209368,
            "range": "± 925",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53878,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316070,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421307,
            "range": "± 930",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455580,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 574595,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1392207,
            "range": "± 9340",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 731395,
            "range": "± 585",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1182868,
            "range": "± 7979",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1275918,
            "range": "± 2669",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1228209,
            "range": "± 8699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1580494,
            "range": "± 47700",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1684133,
            "range": "± 5248",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1772296,
            "range": "± 28706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1831009,
            "range": "± 6401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2586462,
            "range": "± 31811",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 737836,
            "range": "± 2127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 680773,
            "range": "± 1355",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 500473,
            "range": "± 1462",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 320316,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94064,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130193,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8812,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37088,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3968796,
            "range": "± 7250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 874501,
            "range": "± 1686",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1451371,
            "range": "± 3214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 662198,
            "range": "± 6198",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1136517,
            "range": "± 2983",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1156184,
            "range": "± 2679",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2349845,
            "range": "± 145651",
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
        "date": 1700398889471,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4143586,
            "range": "± 31128",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 63817257,
            "range": "± 179443",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103005,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 145869,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209181,
            "range": "± 1633",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58089,
            "range": "± 786",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316058,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 420071,
            "range": "± 987",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 457064,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620357,
            "range": "± 1867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1368398,
            "range": "± 19294",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 738462,
            "range": "± 530",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1180655,
            "range": "± 7419",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1265677,
            "range": "± 7105",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1216802,
            "range": "± 6143",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1563227,
            "range": "± 39193",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1554934,
            "range": "± 10917",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1609533,
            "range": "± 9293",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1780055,
            "range": "± 13102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2532684,
            "range": "± 43663",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 778859,
            "range": "± 55050",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 659015,
            "range": "± 649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 490475,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 319648,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94477,
            "range": "± 146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129312,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8783,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36920,
            "range": "± 1438",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3960374,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 856588,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395051,
            "range": "± 821",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 664971,
            "range": "± 998",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134806,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1148670,
            "range": "± 1350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2409732,
            "range": "± 719",
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
        "date": 1700399743054,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4164130,
            "range": "± 21122",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 63982534,
            "range": "± 105068",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103345,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147473,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 208675,
            "range": "± 2419",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53565,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 319782,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 425864,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 456273,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 620788,
            "range": "± 1629",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1346809,
            "range": "± 5444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 737366,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1178263,
            "range": "± 8938",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1268109,
            "range": "± 8560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1226968,
            "range": "± 8173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1607390,
            "range": "± 60391",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1557037,
            "range": "± 12401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1603653,
            "range": "± 12056",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1770954,
            "range": "± 16383",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2547654,
            "range": "± 49623",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 778898,
            "range": "± 32509",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658618,
            "range": "± 598",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 491266,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 321569,
            "range": "± 933",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94299,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129255,
            "range": "± 1826",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8781,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36139,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3962742,
            "range": "± 4398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 857357,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395393,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 665079,
            "range": "± 2468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134485,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1149270,
            "range": "± 878",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2348269,
            "range": "± 3528",
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
        "date": 1700519323605,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4204875,
            "range": "± 20688",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64632218,
            "range": "± 267350",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 104068,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147427,
            "range": "± 624",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 213055,
            "range": "± 593",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55607,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316605,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421323,
            "range": "± 655",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455490,
            "range": "± 496",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 546117,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1398829,
            "range": "± 6804",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 737047,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1182418,
            "range": "± 6676",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1256536,
            "range": "± 19733",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1465774,
            "range": "± 5281",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1554164,
            "range": "± 9738",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1548992,
            "range": "± 10736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1605540,
            "range": "± 42091",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1780447,
            "range": "± 13615",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2581655,
            "range": "± 2603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 747694,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658643,
            "range": "± 1250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 491447,
            "range": "± 1246",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 320037,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94842,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129705,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8792,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36085,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4041409,
            "range": "± 2107",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 856723,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1395996,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 664477,
            "range": "± 607",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134769,
            "range": "± 1023",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1149342,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2342792,
            "range": "± 1867",
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
        "date": 1700600308637,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4193795,
            "range": "± 10116",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64621755,
            "range": "± 169736",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 104614,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147066,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209164,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55762,
            "range": "± 1266",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 316244,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421968,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 458409,
            "range": "± 1193",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 547396,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1378008,
            "range": "± 7709",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 759750,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1178572,
            "range": "± 8556",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1269993,
            "range": "± 7049",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1219692,
            "range": "± 5247",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1561511,
            "range": "± 53054",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1547204,
            "range": "± 9175",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1618073,
            "range": "± 11599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1785999,
            "range": "± 12804",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2524558,
            "range": "± 37713",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 749398,
            "range": "± 1325",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 658692,
            "range": "± 413",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 488648,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 322184,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94397,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129373,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8806,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37060,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4007095,
            "range": "± 4956",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 857961,
            "range": "± 3396",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1396624,
            "range": "± 2173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 666259,
            "range": "± 1315",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1136023,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1160785,
            "range": "± 1225",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2371127,
            "range": "± 187216",
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
        "date": 1700842710182,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4169885,
            "range": "± 19054",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 63672617,
            "range": "± 258584",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 102774,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 147271,
            "range": "± 585",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209049,
            "range": "± 1459",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52738,
            "range": "± 838",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 317220,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 421367,
            "range": "± 592",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 458951,
            "range": "± 2558",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5457578,
            "range": "± 4500",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1330638,
            "range": "± 12528",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 737575,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1180884,
            "range": "± 3970",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1265884,
            "range": "± 20646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1219898,
            "range": "± 9479",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1885475,
            "range": "± 14301",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1692652,
            "range": "± 10509",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1747330,
            "range": "± 16317",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1786843,
            "range": "± 20264",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2594094,
            "range": "± 8974",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 747400,
            "range": "± 872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 660014,
            "range": "± 1083",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 491023,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 320238,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94535,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129303,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8875,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 36875,
            "range": "± 96",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 10612091,
            "range": "± 9035",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 3967532,
            "range": "± 4111",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 857638,
            "range": "± 1190",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1394274,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 663425,
            "range": "± 1632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1134422,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1151400,
            "range": "± 2656",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2350471,
            "range": "± 8112",
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
        "date": 1700851101635,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4179337,
            "range": "± 17937",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 64134773,
            "range": "± 223540",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103467,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 145690,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209057,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54167,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 324476,
            "range": "± 2035",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 423504,
            "range": "± 484",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 457393,
            "range": "± 818",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5458931,
            "range": "± 10615",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1341753,
            "range": "± 17951",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 731204,
            "range": "± 510",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1162163,
            "range": "± 7493",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1270653,
            "range": "± 19823",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1226342,
            "range": "± 3326",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1613820,
            "range": "± 61102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1548749,
            "range": "± 9043",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1595975,
            "range": "± 15868",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1769290,
            "range": "± 9901",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2548647,
            "range": "± 38774",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 742659,
            "range": "± 36872",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 661213,
            "range": "± 3028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 490760,
            "range": "± 1199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 328300,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94500,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 129857,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8799,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37012,
            "range": "± 1263",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 10601053,
            "range": "± 15553",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4022261,
            "range": "± 3088",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 854901,
            "range": "± 1033",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1399073,
            "range": "± 1776",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 663526,
            "range": "± 5515",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1182015,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1211765,
            "range": "± 1499",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2531158,
            "range": "± 3616",
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
        "date": 1700861383479,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4142970,
            "range": "± 11761",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 63453718,
            "range": "± 56039",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 101512,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 144841,
            "range": "± 262",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 206549,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56546,
            "range": "± 728",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 315482,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 422400,
            "range": "± 442",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455358,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5469547,
            "range": "± 16949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1386235,
            "range": "± 17949",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 730545,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0/typed",
            "value": 1178968,
            "range": "± 11800",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1/typed",
            "value": 1300532,
            "range": "± 4734",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4/typed",
            "value": 1389138,
            "range": "± 3839",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16/typed",
            "value": 1598172,
            "range": "± 6283",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_0",
            "value": 1635398,
            "range": "± 23688",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_1",
            "value": 1705771,
            "range": "± 6513",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_4",
            "value": 1779455,
            "range": "± 17595",
            "unit": "ns/iter"
          },
          {
            "name": "execute/bare_call_16",
            "value": 2574090,
            "range": "± 6027",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_bump",
            "value": 738384,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global_const",
            "value": 679671,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_recursive",
            "value": 483675,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial_iterative",
            "value": 321343,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_ok",
            "value": 94361,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130548,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8813,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "execute/host_calls",
            "value": 37069,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 10602166,
            "range": "± 2744",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_rec",
            "value": 4034844,
            "range": "± 1559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_tail",
            "value": 864134,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci_iter",
            "value": 1394225,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_is_even",
            "value": 661994,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_sum",
            "value": 1136193,
            "range": "± 3676",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_fill",
            "value": 1153328,
            "range": "± 1241",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory_vec_add",
            "value": 2352281,
            "range": "± 7113",
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
        "date": 1700862454907,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4119312,
            "range": "± 10175",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 62864790,
            "range": "± 239875",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 2709472,
            "range": "± 10709",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 976429,
            "range": "± 8520",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 101605,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 144705,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 208411,
            "range": "± 495",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53927,
            "range": "± 672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 315931,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 422202,
            "range": "± 619",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 454645,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5467721,
            "range": "± 4916",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1345383,
            "range": "± 8371",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 730221,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 738221,
            "range": "± 383",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 679381,
            "range": "± 448",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 488544,
            "range": "± 953",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 320646,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 94075,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130027,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8816,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36982,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 10597873,
            "range": "± 7358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 3958920,
            "range": "± 6842",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 863617,
            "range": "± 1695",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1393130,
            "range": "± 1322",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 661268,
            "range": "± 1147",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1133343,
            "range": "± 1059",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1152188,
            "range": "± 641",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2351326,
            "range": "± 4355",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1167487,
            "range": "± 8202",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1567721,
            "range": "± 12647",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1570171,
            "range": "± 11667",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2535219,
            "range": "± 208537",
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
        "date": 1700863757493,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4193274,
            "range": "± 14728",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 63962868,
            "range": "± 111338",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 2774122,
            "range": "± 16154",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 992228,
            "range": "± 1186",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 103670,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 145835,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 209973,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55788,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 315588,
            "range": "± 1588",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 425526,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 455638,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5466137,
            "range": "± 5149",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1443300,
            "range": "± 37417",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 732347,
            "range": "± 4031",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 737733,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 680348,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 487149,
            "range": "± 921",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 320208,
            "range": "± 456",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 94205,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 130268,
            "range": "± 2084",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 8842,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36600,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 10641666,
            "range": "± 36663",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 3957347,
            "range": "± 5811",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 865671,
            "range": "± 16409",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 2007267,
            "range": "± 1784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 663553,
            "range": "± 1271",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1132838,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1153692,
            "range": "± 1672",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2348534,
            "range": "± 3573",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1238740,
            "range": "± 14735",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1618850,
            "range": "± 7604",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1618081,
            "range": "± 8228",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2577490,
            "range": "± 20552",
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
        "date": 1700864759304,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4973358,
            "range": "± 10837",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 86351040,
            "range": "± 112909",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 3765254,
            "range": "± 9938",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 1326291,
            "range": "± 5671",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 134949,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 191169,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 278175,
            "range": "± 1492",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58671,
            "range": "± 1716",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349367,
            "range": "± 1268",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 441983,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 590274,
            "range": "± 11347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7381327,
            "range": "± 20903",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1489347,
            "range": "± 19984",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 599745,
            "range": "± 975",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317299,
            "range": "± 1690",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 730853,
            "range": "± 1372",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 687745,
            "range": "± 1433",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 327310,
            "range": "± 1379",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 169855,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 184890,
            "range": "± 1427",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15814,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36998,
            "range": "± 1725",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7317389,
            "range": "± 11217",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6699847,
            "range": "± 14937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1476382,
            "range": "± 2430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1352687,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1078406,
            "range": "± 11468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1003375,
            "range": "± 12536",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1118735,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2937682,
            "range": "± 1492",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1232865,
            "range": "± 18381",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1618677,
            "range": "± 10294",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1667684,
            "range": "± 11374",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2480428,
            "range": "± 8532",
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
        "date": 1700907218917,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4969347,
            "range": "± 11330",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 86659122,
            "range": "± 162970",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 3744386,
            "range": "± 20693",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 1330203,
            "range": "± 3731",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 135251,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 191854,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 277606,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56335,
            "range": "± 1427",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350404,
            "range": "± 861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445216,
            "range": "± 3179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 596873,
            "range": "± 5567",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7476220,
            "range": "± 5176",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1526642,
            "range": "± 9078",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 620789,
            "range": "± 673",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1319135,
            "range": "± 8942",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 685073,
            "range": "± 21712",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 745209,
            "range": "± 1200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 347203,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165379,
            "range": "± 556",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 185111,
            "range": "± 1700",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15467,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 37088,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 8114201,
            "range": "± 64505",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6659032,
            "range": "± 7139",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1692847,
            "range": "± 852",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1359467,
            "range": "± 639",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1067606,
            "range": "± 2807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1068712,
            "range": "± 9474",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1143651,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2954863,
            "range": "± 1666",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1245051,
            "range": "± 7877",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1663258,
            "range": "± 50565",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1673044,
            "range": "± 35405",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2511581,
            "range": "± 37163",
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
        "date": 1700910684737,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel",
            "value": 4943471,
            "range": "± 17415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey",
            "value": 85970945,
            "range": "± 188305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark",
            "value": 3688105,
            "range": "± 33010",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2",
            "value": 1330213,
            "range": "± 2876",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20",
            "value": 135810,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721",
            "value": 192525,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155",
            "value": 278016,
            "range": "± 503",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55158,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350062,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444438,
            "range": "± 3830",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 595400,
            "range": "± 6063",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7485796,
            "range": "± 2315",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1548528,
            "range": "± 7068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 618654,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1316694,
            "range": "± 1749",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 654427,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 674408,
            "range": "± 1547",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 223662,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165583,
            "range": "± 993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 184280,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15344,
            "range": "± 1107",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36986,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7853197,
            "range": "± 30999",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6255017,
            "range": "± 7847",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1377238,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1359264,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1075808,
            "range": "± 3683",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1073829,
            "range": "± 6272",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1144818,
            "range": "± 1058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2949109,
            "range": "± 4278",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1232619,
            "range": "± 3718",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1600751,
            "range": "± 13199",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1661017,
            "range": "± 11827",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2381626,
            "range": "± 24140",
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
        "date": 1701205265780,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4919086,
            "range": "± 14826",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 4943047,
            "range": "± 18718",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 86640615,
            "range": "± 300947",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 86481643,
            "range": "± 248468",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3709010,
            "range": "± 10455",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3696964,
            "range": "± 8123",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1312443,
            "range": "± 3883",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1316548,
            "range": "± 4873",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 133522,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 133817,
            "range": "± 436",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 189514,
            "range": "± 260",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 190638,
            "range": "± 16296",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 274956,
            "range": "± 805",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 274274,
            "range": "± 729",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 59403,
            "range": "± 1734",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 352555,
            "range": "± 472",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 446185,
            "range": "± 9231",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 593739,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7462179,
            "range": "± 10265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1601444,
            "range": "± 32663",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616167,
            "range": "± 42986",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1320042,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 705390,
            "range": "± 2861",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 681988,
            "range": "± 1023",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 251360,
            "range": "± 1203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166681,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 187972,
            "range": "± 4674",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15403,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36480,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7256390,
            "range": "± 21219",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6144108,
            "range": "± 9000",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1363826,
            "range": "± 3233",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1353904,
            "range": "± 6172",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1087013,
            "range": "± 7941",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1076403,
            "range": "± 6625",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1125696,
            "range": "± 1040",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2939402,
            "range": "± 2880",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1233770,
            "range": "± 12444",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1713338,
            "range": "± 11169",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1667592,
            "range": "± 12568",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2462308,
            "range": "± 14741",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}