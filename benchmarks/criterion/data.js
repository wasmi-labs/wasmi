window.BENCHMARK_DATA = {
  "lastUpdate": 1703258413130,
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
        "date": 1701206884693,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 5013914,
            "range": "± 11021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5182508,
            "range": "± 24956",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 87792202,
            "range": "± 578077",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90907980,
            "range": "± 283802",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3748250,
            "range": "± 13317",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3880614,
            "range": "± 8228",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1381991,
            "range": "± 3230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1427067,
            "range": "± 2730",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 136061,
            "range": "± 452",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 143734,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 193639,
            "range": "± 520",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 204975,
            "range": "± 1183",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 280553,
            "range": "± 825",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 300239,
            "range": "± 2495",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53554,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350706,
            "range": "± 1325",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 441187,
            "range": "± 2631",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 595214,
            "range": "± 5187",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7482613,
            "range": "± 9382",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1565387,
            "range": "± 19049",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 618005,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1316543,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 647028,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 685737,
            "range": "± 1334",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 242055,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165118,
            "range": "± 355",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 183264,
            "range": "± 3217",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15212,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 37004,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7754222,
            "range": "± 64468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6350189,
            "range": "± 6027",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1458162,
            "range": "± 2305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1359632,
            "range": "± 1655",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1106945,
            "range": "± 11576",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1067125,
            "range": "± 68866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1143062,
            "range": "± 1028",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2948788,
            "range": "± 1640",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1251832,
            "range": "± 7169",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1609756,
            "range": "± 20549",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1672776,
            "range": "± 13888",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2496389,
            "range": "± 14388",
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
        "date": 1701344220776,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 5092202,
            "range": "± 10885",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5249213,
            "range": "± 12997",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 89116526,
            "range": "± 168998",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 92163044,
            "range": "± 277449",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3811119,
            "range": "± 5306",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3923787,
            "range": "± 4642",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1395271,
            "range": "± 2011",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1455559,
            "range": "± 5536",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 138406,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 146479,
            "range": "± 715",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 196273,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 207374,
            "range": "± 725",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 285345,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 305979,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54955,
            "range": "± 1176",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 362085,
            "range": "± 4896",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444871,
            "range": "± 2480",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 602221,
            "range": "± 5590",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7408917,
            "range": "± 14964",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1533321,
            "range": "± 11290",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613915,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1324118,
            "range": "± 770",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 734362,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 684163,
            "range": "± 773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 203020,
            "range": "± 1173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166250,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 185733,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15566,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 36971,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 9982676,
            "range": "± 73735",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6089853,
            "range": "± 2816",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1354336,
            "range": "± 2052",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1356774,
            "range": "± 1629",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1069949,
            "range": "± 2882",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1129236,
            "range": "± 5301",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1131472,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2944826,
            "range": "± 41352",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1284914,
            "range": "± 9483",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1591288,
            "range": "± 6412",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1648839,
            "range": "± 16061",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2431170,
            "range": "± 7210",
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
        "date": 1701419373969,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4955851,
            "range": "± 9268",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5092834,
            "range": "± 3843",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 86631020,
            "range": "± 285824",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 89646473,
            "range": "± 405554",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3701117,
            "range": "± 12061",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3802600,
            "range": "± 14339",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1360423,
            "range": "± 2348",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1401235,
            "range": "± 2267",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134118,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 141306,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191759,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 199913,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 276933,
            "range": "± 1185",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 296096,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54119,
            "range": "± 803",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350419,
            "range": "± 1045",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 441450,
            "range": "± 4035",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591771,
            "range": "± 30282",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6702839,
            "range": "± 35252",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1503140,
            "range": "± 15115",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613513,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1321926,
            "range": "± 879",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 714641,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 675539,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 228211,
            "range": "± 2669",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166611,
            "range": "± 813",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 184856,
            "range": "± 1952",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15469,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44825,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7333148,
            "range": "± 25197",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6036297,
            "range": "± 8835",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1374511,
            "range": "± 5706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1317851,
            "range": "± 5865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1079484,
            "range": "± 5296",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1096608,
            "range": "± 8768",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1076004,
            "range": "± 4420",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2949449,
            "range": "± 41793",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1205703,
            "range": "± 7862",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1637241,
            "range": "± 11166",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1601172,
            "range": "± 14186",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2472205,
            "range": "± 26236",
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
        "date": 1701519237397,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 5019697,
            "range": "± 10387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5168391,
            "range": "± 17887",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 87790232,
            "range": "± 435562",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90253343,
            "range": "± 216126",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3829291,
            "range": "± 12774",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3907071,
            "range": "± 12422",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1363004,
            "range": "± 4839",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1404039,
            "range": "± 10577",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134445,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 141459,
            "range": "± 186",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191154,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 200091,
            "range": "± 1534",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 279032,
            "range": "± 2764",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 295932,
            "range": "± 1336",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53129,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 347155,
            "range": "± 762",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442154,
            "range": "± 1344",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 594264,
            "range": "± 2414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7482851,
            "range": "± 4439",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1507217,
            "range": "± 7044",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 615034,
            "range": "± 16975",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1320491,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 733751,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 675018,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 260732,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167873,
            "range": "± 971",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 187318,
            "range": "± 698",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15690,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46283,
            "range": "± 669",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7808593,
            "range": "± 59303",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 5998221,
            "range": "± 11798",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1475724,
            "range": "± 3430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1284853,
            "range": "± 5388",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1080845,
            "range": "± 3439",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1038180,
            "range": "± 5512",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1087973,
            "range": "± 4674",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2949554,
            "range": "± 3001",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1208262,
            "range": "± 24565",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1602246,
            "range": "± 8305",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1604703,
            "range": "± 16596",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2451711,
            "range": "± 7068",
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
        "date": 1701520928261,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 5043015,
            "range": "± 12638",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5170570,
            "range": "± 18482",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 88469813,
            "range": "± 542429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90675109,
            "range": "± 117850",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3807084,
            "range": "± 19976",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3896175,
            "range": "± 11555",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1373914,
            "range": "± 3566",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1415399,
            "range": "± 2343",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134417,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 141567,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191155,
            "range": "± 656",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 199244,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 279462,
            "range": "± 859",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 298473,
            "range": "± 1152",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55674,
            "range": "± 1510",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 348195,
            "range": "± 2735",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442583,
            "range": "± 2039",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 595820,
            "range": "± 6342",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7485892,
            "range": "± 10703",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1538707,
            "range": "± 17501",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 612017,
            "range": "± 785",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1321975,
            "range": "± 11192",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 734337,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 1032641,
            "range": "± 1315",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 283788,
            "range": "± 1240",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167211,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 186564,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15406,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44664,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7817741,
            "range": "± 57447",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6217001,
            "range": "± 11426",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1502816,
            "range": "± 3701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1281196,
            "range": "± 5486",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1105581,
            "range": "± 20934",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1036443,
            "range": "± 5999",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1086859,
            "range": "± 2972",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2967464,
            "range": "± 8887",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1212214,
            "range": "± 11882",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1633057,
            "range": "± 10231",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1607970,
            "range": "± 10621",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2449421,
            "range": "± 21835",
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
        "date": 1701536686882,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4958070,
            "range": "± 138929",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5087467,
            "range": "± 14962",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 87148753,
            "range": "± 222107",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 89455087,
            "range": "± 204496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3736414,
            "range": "± 9941",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3806961,
            "range": "± 9605",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1358055,
            "range": "± 16234",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1398813,
            "range": "± 7805",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 133341,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 141349,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 190423,
            "range": "± 635",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 200030,
            "range": "± 1150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 278832,
            "range": "± 646",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 295658,
            "range": "± 974",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53493,
            "range": "± 1024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 347336,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442196,
            "range": "± 2603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 588075,
            "range": "± 1757",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7483301,
            "range": "± 5283",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1512503,
            "range": "± 22777",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613230,
            "range": "± 1075",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1321137,
            "range": "± 1429",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 733681,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 686700,
            "range": "± 3267",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 259647,
            "range": "± 2076",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167113,
            "range": "± 1178",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 186213,
            "range": "± 2814",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15431,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45015,
            "range": "± 1297",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7855202,
            "range": "± 35714",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6020758,
            "range": "± 20689",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1495615,
            "range": "± 5319",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1286064,
            "range": "± 7518",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1086376,
            "range": "± 4562",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1035943,
            "range": "± 48236",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1088059,
            "range": "± 3102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2948605,
            "range": "± 4766",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1208901,
            "range": "± 18972",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1613826,
            "range": "± 9857",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1613599,
            "range": "± 26020",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2447982,
            "range": "± 13440",
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
        "date": 1701551026523,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4937280,
            "range": "± 9881",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5083421,
            "range": "± 13948",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 86556261,
            "range": "± 204764",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 89375902,
            "range": "± 335012",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3740284,
            "range": "± 17642",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3831292,
            "range": "± 31023",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1357347,
            "range": "± 7607",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1407558,
            "range": "± 5548",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 133944,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 141388,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191048,
            "range": "± 1145",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 200428,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 278769,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 298030,
            "range": "± 2095",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53043,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 347253,
            "range": "± 2243",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 446742,
            "range": "± 56649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 589695,
            "range": "± 5867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7472935,
            "range": "± 257496",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1458773,
            "range": "± 7822",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 609255,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1320688,
            "range": "± 1452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 733643,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 681035,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 257269,
            "range": "± 1420",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167033,
            "range": "± 1930",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 185836,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15381,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46450,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 9629024,
            "range": "± 50912",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6050067,
            "range": "± 7330",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1688417,
            "range": "± 2404",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1269074,
            "range": "± 5939",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1083773,
            "range": "± 3795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1028055,
            "range": "± 8605",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1091472,
            "range": "± 2373",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2947984,
            "range": "± 4244",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1211557,
            "range": "± 13873",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1628003,
            "range": "± 9545",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1602380,
            "range": "± 21212",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2463844,
            "range": "± 15876",
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
        "date": 1701606662102,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4976070,
            "range": "± 10184",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5116425,
            "range": "± 13319",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 87441398,
            "range": "± 225415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90027388,
            "range": "± 227889",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3724920,
            "range": "± 20091",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3848058,
            "range": "± 22260",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1366663,
            "range": "± 4995",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1409627,
            "range": "± 5040",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 133681,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 142076,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 190946,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 199856,
            "range": "± 792",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 277554,
            "range": "± 1114",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 297230,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53713,
            "range": "± 2200",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350216,
            "range": "± 7517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442375,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 587340,
            "range": "± 2312",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6736528,
            "range": "± 58027",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1501183,
            "range": "± 20603",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613825,
            "range": "± 775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1322255,
            "range": "± 1892",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 709526,
            "range": "± 3627",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 683192,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 249240,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167201,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 185880,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15682,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46095,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7393494,
            "range": "± 24258",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6115676,
            "range": "± 3613",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1515914,
            "range": "± 991",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1312499,
            "range": "± 6489",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1082434,
            "range": "± 4648",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1097433,
            "range": "± 6867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1076305,
            "range": "± 4698",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2951789,
            "range": "± 3678",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1205473,
            "range": "± 11373",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1680016,
            "range": "± 11661",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1607413,
            "range": "± 9883",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2461642,
            "range": "± 4937",
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
        "date": 1701608694026,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4994700,
            "range": "± 9017",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5160780,
            "range": "± 26461",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 88461483,
            "range": "± 279387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90636275,
            "range": "± 214577",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3754545,
            "range": "± 7137",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3862942,
            "range": "± 5384",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1382078,
            "range": "± 1929",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1429468,
            "range": "± 3636",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134832,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 143344,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 192455,
            "range": "± 892",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 202453,
            "range": "± 505",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 279356,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 298254,
            "range": "± 1271",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56577,
            "range": "± 1457",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349478,
            "range": "± 607",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 441053,
            "range": "± 2046",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 589214,
            "range": "± 4558",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7458100,
            "range": "± 53572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1500700,
            "range": "± 15953",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 614352,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1324468,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 735180,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 678347,
            "range": "± 920",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 226208,
            "range": "± 2977",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 169169,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 189103,
            "range": "± 3355",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15596,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45403,
            "range": "± 799",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 8207605,
            "range": "± 103773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6423713,
            "range": "± 8449",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6165283,
            "range": "± 11468",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1505144,
            "range": "± 2506",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1360003,
            "range": "± 1894",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1089733,
            "range": "± 2365",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1129333,
            "range": "± 2546",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1127744,
            "range": "± 1168",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2945099,
            "range": "± 18614",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1211034,
            "range": "± 9865",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1710636,
            "range": "± 14199",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1642641,
            "range": "± 15623",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2690451,
            "range": "± 11795",
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
        "date": 1701610593789,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4987290,
            "range": "± 22728",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5143431,
            "range": "± 13525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 88056913,
            "range": "± 434676",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 90434519,
            "range": "± 257450",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3751570,
            "range": "± 18598",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3842954,
            "range": "± 10517",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1368457,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1414833,
            "range": "± 7922",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134775,
            "range": "± 400",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 143674,
            "range": "± 1022",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191384,
            "range": "± 1129",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 201892,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 279411,
            "range": "± 2210",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 299433,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 51841,
            "range": "± 1504",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350429,
            "range": "± 1531",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 443971,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 589389,
            "range": "± 5993",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6626851,
            "range": "± 5430",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1533176,
            "range": "± 8454",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 617587,
            "range": "± 61601",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1319735,
            "range": "± 2058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 714275,
            "range": "± 3621",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 717110,
            "range": "± 2018",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 238029,
            "range": "± 3951",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 168147,
            "range": "± 606",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 186793,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15386,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45005,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7352303,
            "range": "± 23199",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6227226,
            "range": "± 15653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6103879,
            "range": "± 17771",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1481719,
            "range": "± 1626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1291480,
            "range": "± 8910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1081005,
            "range": "± 2232",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1103152,
            "range": "± 99133",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1081522,
            "range": "± 1610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2948599,
            "range": "± 3309",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1237782,
            "range": "± 12938",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1618335,
            "range": "± 11515",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1625767,
            "range": "± 12471",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2431901,
            "range": "± 10232",
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
        "date": 1701615636076,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/default",
            "value": 4909716,
            "range": "± 5140",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/fuel",
            "value": 5056434,
            "range": "± 4769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/default",
            "value": 86848114,
            "range": "± 433797",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/fuel",
            "value": 89204361,
            "range": "± 281593",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/default",
            "value": 3700937,
            "range": "± 4567",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/fuel",
            "value": 3799721,
            "range": "± 3532",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/default",
            "value": 1367458,
            "range": "± 3269",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/fuel",
            "value": 1412728,
            "range": "± 2123",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/default",
            "value": 134474,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/fuel",
            "value": 142142,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/default",
            "value": 191842,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/fuel",
            "value": 199676,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/default",
            "value": 277687,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/fuel",
            "value": 298016,
            "range": "± 1159",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55587,
            "range": "± 1589",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 347604,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442457,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591339,
            "range": "± 6509",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481238,
            "range": "± 4206",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1491676,
            "range": "± 31061",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613881,
            "range": "± 334",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1321183,
            "range": "± 6889",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 733778,
            "range": "± 849",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 694661,
            "range": "± 570",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 251299,
            "range": "± 2411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166304,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 186280,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15703,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46569,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7839653,
            "range": "± 19434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6265988,
            "range": "± 8173",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6009469,
            "range": "± 7386",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1514875,
            "range": "± 1269",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1276194,
            "range": "± 4211",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1080035,
            "range": "± 2444",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1043692,
            "range": "± 2932",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1094502,
            "range": "± 1462",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2961401,
            "range": "± 55296",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1216692,
            "range": "± 12307",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1603221,
            "range": "± 14242",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1641642,
            "range": "± 11653",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2648618,
            "range": "± 5526",
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
        "date": 1701626602841,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5036835,
            "range": "± 10287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5166917,
            "range": "± 5266",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4099220,
            "range": "± 5042",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4249372,
            "range": "± 9062",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 87970209,
            "range": "± 168689",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 90724203,
            "range": "± 328426",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71691987,
            "range": "± 61671",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74352092,
            "range": "± 114353",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3773755,
            "range": "± 6886",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3861184,
            "range": "± 8112",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3111868,
            "range": "± 7453",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3212368,
            "range": "± 5631",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1380855,
            "range": "± 3438",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1427220,
            "range": "± 2625",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1128595,
            "range": "± 4254",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1170103,
            "range": "± 7648",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 136483,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 144293,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112880,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120490,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 194497,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 205367,
            "range": "± 1143",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 158952,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 168394,
            "range": "± 303",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 281906,
            "range": "± 1941",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 301521,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 233126,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 251276,
            "range": "± 532",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55405,
            "range": "± 2055",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349918,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442346,
            "range": "± 1664",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591246,
            "range": "± 7935",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6671101,
            "range": "± 18328",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1635281,
            "range": "± 9937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 615939,
            "range": "± 264",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317236,
            "range": "± 1597",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 709308,
            "range": "± 4216",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 690129,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 245621,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166273,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 188473,
            "range": "± 6823",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15544,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45602,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7537149,
            "range": "± 17646",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6225118,
            "range": "± 4475",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6107156,
            "range": "± 3602",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1520528,
            "range": "± 1019",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1284238,
            "range": "± 6090",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1096623,
            "range": "± 2785",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1090802,
            "range": "± 3592",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1081268,
            "range": "± 3271",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2948324,
            "range": "± 1046",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1203699,
            "range": "± 14004",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1623464,
            "range": "± 11172",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1637554,
            "range": "± 21566",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2519854,
            "range": "± 7488",
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
        "date": 1701640826191,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5079448,
            "range": "± 12360",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5215079,
            "range": "± 27566",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4119442,
            "range": "± 15930",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4241865,
            "range": "± 13510",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88680882,
            "range": "± 908162",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91637663,
            "range": "± 231546",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71618685,
            "range": "± 117118",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74114262,
            "range": "± 197072",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3787962,
            "range": "± 10231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3882078,
            "range": "± 10059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3112233,
            "range": "± 7064",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3216099,
            "range": "± 15530",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1392559,
            "range": "± 7522",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1436914,
            "range": "± 5045",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1129121,
            "range": "± 11380",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1171763,
            "range": "± 4113",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 139160,
            "range": "± 1459",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 146824,
            "range": "± 1052",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 113608,
            "range": "± 429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120900,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 197149,
            "range": "± 1129",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 208466,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 160490,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 169389,
            "range": "± 1417",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 285258,
            "range": "± 1221",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 304859,
            "range": "± 1341",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 235728,
            "range": "± 1864",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 252460,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57566,
            "range": "± 1285",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 351010,
            "range": "± 1741",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445737,
            "range": "± 3640",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 595054,
            "range": "± 2857",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6609466,
            "range": "± 125929",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1481987,
            "range": "± 8720",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 617301,
            "range": "± 1117",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1326991,
            "range": "± 200411",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 706115,
            "range": "± 25512",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 682284,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 244025,
            "range": "± 1133",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167280,
            "range": "± 951",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190907,
            "range": "± 7764",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15416,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45944,
            "range": "± 1350",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7377376,
            "range": "± 14346",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6227624,
            "range": "± 9578",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6082503,
            "range": "± 9807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1353030,
            "range": "± 1911",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1289597,
            "range": "± 6044",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1128205,
            "range": "± 2415",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1089201,
            "range": "± 5821",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1078652,
            "range": "± 5095",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2961389,
            "range": "± 115203",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1232755,
            "range": "± 10012",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1606344,
            "range": "± 12636",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1624519,
            "range": "± 19397",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2495292,
            "range": "± 7017",
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
        "date": 1701692191823,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5055780,
            "range": "± 19109",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5175985,
            "range": "± 11442",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4128827,
            "range": "± 67403",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4247209,
            "range": "± 23795",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88664171,
            "range": "± 247325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91272094,
            "range": "± 296210",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 72305850,
            "range": "± 214894",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74696889,
            "range": "± 274627",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3796097,
            "range": "± 17467",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3882576,
            "range": "± 18876",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3121345,
            "range": "± 6396",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3202238,
            "range": "± 13117",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1382142,
            "range": "± 6351",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1423192,
            "range": "± 7549",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1123405,
            "range": "± 4615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1156929,
            "range": "± 3487",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 136932,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 145317,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112955,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 121136,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 195802,
            "range": "± 1179",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 205536,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 159690,
            "range": "± 902",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 167345,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 282210,
            "range": "± 3100",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 302801,
            "range": "± 2150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 233747,
            "range": "± 1267",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 251326,
            "range": "± 1530",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52816,
            "range": "± 1839",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350328,
            "range": "± 972",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 443646,
            "range": "± 6965",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 592697,
            "range": "± 2553",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6667035,
            "range": "± 22710",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1480134,
            "range": "± 5424",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616211,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317497,
            "range": "± 1773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 700194,
            "range": "± 3867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 691966,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 242396,
            "range": "± 1186",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166972,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190191,
            "range": "± 3189",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15384,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45867,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7350646,
            "range": "± 21154",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6588392,
            "range": "± 7275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6094363,
            "range": "± 11218",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1388942,
            "range": "± 3632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1298475,
            "range": "± 7440",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1133468,
            "range": "± 6612",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1099791,
            "range": "± 38986",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1079178,
            "range": "± 4905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2946384,
            "range": "± 2986",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1221092,
            "range": "± 16947",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1613429,
            "range": "± 33950",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1645426,
            "range": "± 9749",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2502722,
            "range": "± 10838",
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
        "date": 1701702852989,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5057583,
            "range": "± 13351",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5201086,
            "range": "± 20084",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4106177,
            "range": "± 13900",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4235363,
            "range": "± 12157",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88378091,
            "range": "± 211039",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91110503,
            "range": "± 327776",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71991827,
            "range": "± 509595",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74208736,
            "range": "± 214787",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3792311,
            "range": "± 24369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3881571,
            "range": "± 13579",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3117232,
            "range": "± 3810",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3233076,
            "range": "± 27595",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1375248,
            "range": "± 4770",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1414914,
            "range": "± 2920",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1124200,
            "range": "± 2647",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1161912,
            "range": "± 4764",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 136219,
            "range": "± 635",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 144018,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112449,
            "range": "± 639",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 119752,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 194468,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 204335,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 158692,
            "range": "± 590",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 166459,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 282177,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 301868,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 232881,
            "range": "± 1059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 249569,
            "range": "± 630",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53290,
            "range": "± 901",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350057,
            "range": "± 1130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442633,
            "range": "± 1952",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591574,
            "range": "± 1760",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6624342,
            "range": "± 11790",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1505937,
            "range": "± 18283",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616051,
            "range": "± 835",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1318063,
            "range": "± 1644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 698918,
            "range": "± 1581",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 677584,
            "range": "± 2001",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 246205,
            "range": "± 2472",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167953,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 190208,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15421,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44901,
            "range": "± 545",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 9184443,
            "range": "± 48500",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6223771,
            "range": "± 6374",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6160903,
            "range": "± 8272",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1372867,
            "range": "± 4385",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1292022,
            "range": "± 6256",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1104659,
            "range": "± 7691",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1091489,
            "range": "± 8440",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1082147,
            "range": "± 2018",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2948482,
            "range": "± 39506",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1205886,
            "range": "± 8906",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1612522,
            "range": "± 8503",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1609113,
            "range": "± 15634",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2495991,
            "range": "± 6922",
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
        "date": 1701713551949,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5017799,
            "range": "± 14458",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5147261,
            "range": "± 13637",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4099352,
            "range": "± 10015",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4218727,
            "range": "± 30750",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 86721608,
            "range": "± 298127",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 89278173,
            "range": "± 268657",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 70446412,
            "range": "± 117552",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 72536801,
            "range": "± 536866",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3743628,
            "range": "± 4390",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3837682,
            "range": "± 5512",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3103166,
            "range": "± 8648",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3181648,
            "range": "± 4977",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1376092,
            "range": "± 1687",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1419709,
            "range": "± 12568",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1124599,
            "range": "± 1309",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1159563,
            "range": "± 1161",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 136068,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 143787,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112418,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 119456,
            "range": "± 118",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 194065,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 203547,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 158672,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 167320,
            "range": "± 314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 281103,
            "range": "± 357",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 299571,
            "range": "± 1296",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 232606,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 249912,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 52894,
            "range": "± 1292",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 378932,
            "range": "± 1022",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445380,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 590407,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6708293,
            "range": "± 13706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1500713,
            "range": "± 14500",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 615310,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317375,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 695421,
            "range": "± 22766",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 682287,
            "range": "± 2105",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 243276,
            "range": "± 1899",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166736,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 188815,
            "range": "± 886",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15294,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45433,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7433248,
            "range": "± 68106",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6223011,
            "range": "± 4012",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6472627,
            "range": "± 8737",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1433614,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1293915,
            "range": "± 5745",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1092427,
            "range": "± 1791",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1118845,
            "range": "± 2867",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1078485,
            "range": "± 2749",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2946839,
            "range": "± 1389",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1222746,
            "range": "± 11317",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1593643,
            "range": "± 8504",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1610577,
            "range": "± 13190",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2501386,
            "range": "± 2319",
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
        "date": 1701775240847,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5130321,
            "range": "± 10696",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5270165,
            "range": "± 9061",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4180818,
            "range": "± 12989",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4293120,
            "range": "± 14684",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 90453837,
            "range": "± 337748",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 93427193,
            "range": "± 227212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 73456883,
            "range": "± 259389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 75429783,
            "range": "± 159985",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3826098,
            "range": "± 9704",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3923358,
            "range": "± 6210",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3161567,
            "range": "± 6081",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3243705,
            "range": "± 9780",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1393087,
            "range": "± 1796",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1441961,
            "range": "± 3090",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1139268,
            "range": "± 3285",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1185303,
            "range": "± 3486",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 139248,
            "range": "± 697",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 147165,
            "range": "± 1667",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 114683,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 121463,
            "range": "± 379",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 198277,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 207480,
            "range": "± 408",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 161555,
            "range": "± 710",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 169679,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 285020,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 305096,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 235436,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 253450,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55997,
            "range": "± 927",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349803,
            "range": "± 773",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 443574,
            "range": "± 2554",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 593241,
            "range": "± 7125",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6660499,
            "range": "± 14832",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1492456,
            "range": "± 11367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616065,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1320912,
            "range": "± 12905",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 697085,
            "range": "± 3422",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 678111,
            "range": "± 1699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 243626,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 168069,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 189996,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15450,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45049,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7343363,
            "range": "± 26033",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6228350,
            "range": "± 6183",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6129919,
            "range": "± 10061",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1384256,
            "range": "± 2464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1306863,
            "range": "± 6467",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1098627,
            "range": "± 2694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1089649,
            "range": "± 5005",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1078444,
            "range": "± 4372",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2954246,
            "range": "± 7304",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1200201,
            "range": "± 11657",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1604191,
            "range": "± 18780",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1636323,
            "range": "± 18440",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2432977,
            "range": "± 12918",
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
        "date": 1701875989367,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5061546,
            "range": "± 7842",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5214463,
            "range": "± 11172",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4129093,
            "range": "± 8781",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4247124,
            "range": "± 7915",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88997680,
            "range": "± 266089",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91132358,
            "range": "± 422322",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71909175,
            "range": "± 219647",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74249010,
            "range": "± 293152",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3769177,
            "range": "± 26505",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3888320,
            "range": "± 6050",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3145190,
            "range": "± 5630",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3212667,
            "range": "± 6963",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1388671,
            "range": "± 4239",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1429740,
            "range": "± 4519",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1134250,
            "range": "± 2985",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1175046,
            "range": "± 4769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 137236,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 145751,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 113833,
            "range": "± 434",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120946,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 195587,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 205478,
            "range": "± 665",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 160520,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 170659,
            "range": "± 798",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 283304,
            "range": "± 813",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 302892,
            "range": "± 1231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 234254,
            "range": "± 898",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 253205,
            "range": "± 1193",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57116,
            "range": "± 2177",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349884,
            "range": "± 644",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445924,
            "range": "± 1855",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591556,
            "range": "± 1804",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7469444,
            "range": "± 13367",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1520787,
            "range": "± 2979",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613127,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317107,
            "range": "± 10599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 729281,
            "range": "± 2280",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 685998,
            "range": "± 884",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 239210,
            "range": "± 2165",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167834,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 187224,
            "range": "± 389",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15443,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44630,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7371002,
            "range": "± 27653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6264607,
            "range": "± 6297",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6069836,
            "range": "± 6433",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1420887,
            "range": "± 2303",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1359425,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1078793,
            "range": "± 4265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1088592,
            "range": "± 6377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1136714,
            "range": "± 1030",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2959733,
            "range": "± 3472",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1217322,
            "range": "± 6955",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1621447,
            "range": "± 8901",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1616270,
            "range": "± 13069",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2484056,
            "range": "± 26648",
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
        "date": 1702031666269,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5078160,
            "range": "± 28191",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5221106,
            "range": "± 16352",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4133440,
            "range": "± 17145",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4261422,
            "range": "± 10954",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88960933,
            "range": "± 240205",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91599476,
            "range": "± 270170",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71999205,
            "range": "± 197283",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74238699,
            "range": "± 347755",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3773758,
            "range": "± 13487",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3903146,
            "range": "± 44171",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3133446,
            "range": "± 10077",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3208263,
            "range": "± 11081",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1397170,
            "range": "± 8601",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1442163,
            "range": "± 8570",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1136349,
            "range": "± 4128",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1176822,
            "range": "± 8850",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 138876,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 145710,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 113199,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 121214,
            "range": "± 564",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 196318,
            "range": "± 1035",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 204395,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 159815,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 168292,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 283620,
            "range": "± 1154",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 302251,
            "range": "± 1432",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 235883,
            "range": "± 1720",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 252413,
            "range": "± 1827",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 56403,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350588,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444372,
            "range": "± 2910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 592450,
            "range": "± 3133",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7465913,
            "range": "± 31150",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1550887,
            "range": "± 10965",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613657,
            "range": "± 25031",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1314410,
            "range": "± 36454",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 732019,
            "range": "± 32293",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 682712,
            "range": "± 3275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 235878,
            "range": "± 2760",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167050,
            "range": "± 706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 186657,
            "range": "± 1692",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15472,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45199,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7357825,
            "range": "± 29417",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6259917,
            "range": "± 10335",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6148147,
            "range": "± 16277",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1464462,
            "range": "± 3022",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1360819,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1080305,
            "range": "± 4753",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1089850,
            "range": "± 5940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1135513,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 3024854,
            "range": "± 110709",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1206045,
            "range": "± 12043",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1632741,
            "range": "± 13224",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1638073,
            "range": "± 15827",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2495592,
            "range": "± 39636",
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
        "date": 1702045482023,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5138477,
            "range": "± 17746",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5316689,
            "range": "± 9178",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4095422,
            "range": "± 4784",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4269392,
            "range": "± 11563",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 90022658,
            "range": "± 143515",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 93394350,
            "range": "± 338974",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 72029747,
            "range": "± 300174",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 75026514,
            "range": "± 569368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3814894,
            "range": "± 7585",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3930898,
            "range": "± 31191",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3102392,
            "range": "± 11883",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3212446,
            "range": "± 10231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1392377,
            "range": "± 6323",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1453390,
            "range": "± 11722",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1125114,
            "range": "± 4707",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1171675,
            "range": "± 4572",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 141761,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 149967,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 114099,
            "range": "± 378",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 122256,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 200554,
            "range": "± 863",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 211143,
            "range": "± 424",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 160752,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 171399,
            "range": "± 506",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 287762,
            "range": "± 1412",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 308438,
            "range": "± 758",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 234472,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 253357,
            "range": "± 476",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 58633,
            "range": "± 1726",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350316,
            "range": "± 1565",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444505,
            "range": "± 1340",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 608551,
            "range": "± 2567",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6623474,
            "range": "± 18865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1475813,
            "range": "± 26062",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616566,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317003,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 698694,
            "range": "± 4050",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 680617,
            "range": "± 2015",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 256966,
            "range": "± 2225",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 166941,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 187732,
            "range": "± 766",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15559,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45757,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7357060,
            "range": "± 11962",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6223533,
            "range": "± 8147",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6085432,
            "range": "± 13251",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1356027,
            "range": "± 2523",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1315668,
            "range": "± 5177",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1110748,
            "range": "± 2851",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1093697,
            "range": "± 3595",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1084104,
            "range": "± 226243",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2956423,
            "range": "± 51703",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1222691,
            "range": "± 10111",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1611394,
            "range": "± 7811",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1613352,
            "range": "± 21912",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2536191,
            "range": "± 31481",
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
        "date": 1702047186748,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5206416,
            "range": "± 16831",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5349127,
            "range": "± 13234",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4142821,
            "range": "± 14327",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4297503,
            "range": "± 7844",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 90756406,
            "range": "± 292930",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 93845324,
            "range": "± 188372",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 72280295,
            "range": "± 194716",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 75512433,
            "range": "± 232318",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3952375,
            "range": "± 12267",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 4066484,
            "range": "± 13769",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3190940,
            "range": "± 8258",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3312768,
            "range": "± 9827",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1398855,
            "range": "± 1748",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1462576,
            "range": "± 36431",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1114691,
            "range": "± 3040",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1164473,
            "range": "± 7520",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 140216,
            "range": "± 758",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 149818,
            "range": "± 1427",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 113007,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120892,
            "range": "± 1135",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 198203,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 209046,
            "range": "± 756",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 158638,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 168710,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 289042,
            "range": "± 2069",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 305841,
            "range": "± 1473",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 233876,
            "range": "± 990",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 251297,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54755,
            "range": "± 962",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350811,
            "range": "± 4229",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445439,
            "range": "± 3570",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591570,
            "range": "± 1104",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6634259,
            "range": "± 31513",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1457609,
            "range": "± 7808",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 615870,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1319647,
            "range": "± 232945",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 714041,
            "range": "± 5425",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 682184,
            "range": "± 1503",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 244375,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 170511,
            "range": "± 2393",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 191276,
            "range": "± 3381",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15383,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46994,
            "range": "± 747",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7345160,
            "range": "± 22327",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6226818,
            "range": "± 7458",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6174539,
            "range": "± 10551",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1352641,
            "range": "± 4275",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1296908,
            "range": "± 8023",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1097546,
            "range": "± 4181",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1099117,
            "range": "± 36263",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1082626,
            "range": "± 4809",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2955946,
            "range": "± 27196",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1229083,
            "range": "± 8828",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1594898,
            "range": "± 11673",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1622503,
            "range": "± 13837",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2560817,
            "range": "± 6223",
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
        "date": 1702050229722,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5080799,
            "range": "± 17387",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5278957,
            "range": "± 29645",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4068419,
            "range": "± 11536",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4223656,
            "range": "± 20173",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 89401215,
            "range": "± 241207",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 92264778,
            "range": "± 249036",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 71405063,
            "range": "± 258540",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 74174382,
            "range": "± 252654",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3811818,
            "range": "± 14230",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3910327,
            "range": "± 8730",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3102978,
            "range": "± 14729",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3193231,
            "range": "± 12153",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1394015,
            "range": "± 2484",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1447488,
            "range": "± 6030",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1117608,
            "range": "± 2592",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1165193,
            "range": "± 7141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 138837,
            "range": "± 353",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 147357,
            "range": "± 525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112318,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120153,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 197233,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 207925,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 158500,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 168490,
            "range": "± 392",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 284712,
            "range": "± 905",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 305139,
            "range": "± 1254",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 232895,
            "range": "± 834",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 252601,
            "range": "± 1365",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53616,
            "range": "± 1780",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350717,
            "range": "± 713",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445073,
            "range": "± 2456",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591274,
            "range": "± 3704",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6647750,
            "range": "± 21610",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1469944,
            "range": "± 11343",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616886,
            "range": "± 5450",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317705,
            "range": "± 1692",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 712754,
            "range": "± 4846",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 680988,
            "range": "± 996",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 238584,
            "range": "± 2459",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167076,
            "range": "± 837",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 188031,
            "range": "± 4636",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15387,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 45082,
            "range": "± 387",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7360974,
            "range": "± 22111",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6219513,
            "range": "± 10391",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6110365,
            "range": "± 7414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1357302,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1297355,
            "range": "± 8032",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1085662,
            "range": "± 2241",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1095973,
            "range": "± 3067",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1080771,
            "range": "± 3647",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2947215,
            "range": "± 51168",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1571276,
            "range": "± 17056",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1609396,
            "range": "± 6490",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1726892,
            "range": "± 40067",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2557540,
            "range": "± 6272",
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
        "date": 1702054455702,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5097527,
            "range": "± 12862",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5253088,
            "range": "± 20813",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4085002,
            "range": "± 5655",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4225470,
            "range": "± 3774",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 88864656,
            "range": "± 396301",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 91869674,
            "range": "± 308314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 70388043,
            "range": "± 226705",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 73448234,
            "range": "± 684897",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3800877,
            "range": "± 4691",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3919564,
            "range": "± 11815",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3082544,
            "range": "± 4488",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3209798,
            "range": "± 12565",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1400055,
            "range": "± 2441",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1460187,
            "range": "± 8567",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1120619,
            "range": "± 4009",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1164018,
            "range": "± 4501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 138922,
            "range": "± 583",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 147677,
            "range": "± 750",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 112755,
            "range": "± 1690",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 120660,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 197547,
            "range": "± 3786",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 208297,
            "range": "± 511",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 159615,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 168995,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 287905,
            "range": "± 1225",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 305075,
            "range": "± 608",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 233197,
            "range": "± 579",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 251805,
            "range": "± 1249",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53365,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 352625,
            "range": "± 1893",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442767,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 592466,
            "range": "± 4150",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6636301,
            "range": "± 12583",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1465422,
            "range": "± 11062",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 615771,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1319023,
            "range": "± 33314",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 713734,
            "range": "± 8323",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 699308,
            "range": "± 1807",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 241785,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 167384,
            "range": "± 1095",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 188151,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15579,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44646,
            "range": "± 1865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7351695,
            "range": "± 24409",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6220222,
            "range": "± 10973",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6160769,
            "range": "± 13686",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1388648,
            "range": "± 2309",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1285009,
            "range": "± 8697",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1087122,
            "range": "± 3277",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1093851,
            "range": "± 8124",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1079769,
            "range": "± 2442",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2947775,
            "range": "± 3652",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1222961,
            "range": "± 12030",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1600737,
            "range": "± 13084",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1614081,
            "range": "± 15154",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2559053,
            "range": "± 14109",
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
        "date": 1702724801833,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/default",
            "value": 5030161,
            "range": "± 15013",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/fuel",
            "value": 5160920,
            "range": "± 9792",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/default",
            "value": 4072406,
            "range": "± 20724",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/fuel",
            "value": 4200515,
            "range": "± 8192",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/default",
            "value": 87800257,
            "range": "± 276138",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/fuel",
            "value": 90021242,
            "range": "± 151902",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/default",
            "value": 70846370,
            "range": "± 136395",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/fuel",
            "value": 73071941,
            "range": "± 153587",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/default",
            "value": 3803952,
            "range": "± 19956",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/fuel",
            "value": 3928717,
            "range": "± 29957",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/default",
            "value": 3131626,
            "range": "± 28001",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/fuel",
            "value": 3242350,
            "range": "± 19150",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/default",
            "value": 1358258,
            "range": "± 1545",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/fuel",
            "value": 1404438,
            "range": "± 4731",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/default",
            "value": 1099951,
            "range": "± 29086",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/fuel",
            "value": 1134551,
            "range": "± 18209",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/default",
            "value": 134653,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/fuel",
            "value": 142552,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/default",
            "value": 111176,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/fuel",
            "value": 118927,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/default",
            "value": 192120,
            "range": "± 1598",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/fuel",
            "value": 200382,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/default",
            "value": 156832,
            "range": "± 620",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/fuel",
            "value": 165487,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/default",
            "value": 277044,
            "range": "± 935",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/fuel",
            "value": 296617,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/default",
            "value": 229061,
            "range": "± 664",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/fuel",
            "value": 246791,
            "range": "± 406",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 57578,
            "range": "± 1639",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350452,
            "range": "± 1158",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442343,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 592248,
            "range": "± 2998",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7488260,
            "range": "± 8637",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1497097,
            "range": "± 21794",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 612649,
            "range": "± 2250",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1316880,
            "range": "± 122719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 728872,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 681991,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 241005,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165923,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 184292,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15735,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 47312,
            "range": "± 1265",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7398614,
            "range": "± 40928",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6263850,
            "range": "± 4451",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6134674,
            "range": "± 17110",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1358344,
            "range": "± 3168",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1360029,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1081079,
            "range": "± 5302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1082317,
            "range": "± 3890",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1136363,
            "range": "± 971",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2951255,
            "range": "± 50187",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1223493,
            "range": "± 9321",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1596163,
            "range": "± 17314",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1613788,
            "range": "± 13983",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2467009,
            "range": "± 23251",
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
        "date": 1702757030409,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5010257,
            "range": "± 17899",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5165413,
            "range": "± 10878",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 5014523,
            "range": "± 15317",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4079247,
            "range": "± 7301",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4232469,
            "range": "± 5201",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 4092254,
            "range": "± 9841",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 87645321,
            "range": "± 302778",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 90239622,
            "range": "± 112452",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 87476609,
            "range": "± 223927",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 71363507,
            "range": "± 154054",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 73859581,
            "range": "± 189571",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 71238874,
            "range": "± 212383",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3747904,
            "range": "± 4315",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3851606,
            "range": "± 29662",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 3741988,
            "range": "± 7925",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3097052,
            "range": "± 11420",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3199254,
            "range": "± 10830",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 3101533,
            "range": "± 4896",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1377638,
            "range": "± 3037",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1425109,
            "range": "± 4715",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 1376690,
            "range": "± 3110",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1117187,
            "range": "± 3412",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1167948,
            "range": "± 3526",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 1117359,
            "range": "± 2765",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135282,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 144048,
            "range": "± 700",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 134860,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 112097,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 119960,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 112135,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 193435,
            "range": "± 407",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 203831,
            "range": "± 767",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 193773,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 157683,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 167604,
            "range": "± 871",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 157737,
            "range": "± 546",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 280570,
            "range": "± 1222",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 301390,
            "range": "± 1080",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 280818,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 231895,
            "range": "± 604",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 252417,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 232026,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54484,
            "range": "± 1341",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349294,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 450281,
            "range": "± 1839",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 601269,
            "range": "± 2497",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7483798,
            "range": "± 6307",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1515108,
            "range": "± 20647",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 617858,
            "range": "± 485",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1318910,
            "range": "± 92025",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 734193,
            "range": "± 886",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 682898,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 261293,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165556,
            "range": "± 614",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 184055,
            "range": "± 838",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15528,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 46286,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7676999,
            "range": "± 47120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6291354,
            "range": "± 4354",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6026725,
            "range": "± 64781",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1866289,
            "range": "± 96640",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1289669,
            "range": "± 7044",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1064488,
            "range": "± 3377",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1043407,
            "range": "± 154272",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1096588,
            "range": "± 60931",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2946099,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1226893,
            "range": "± 10327",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1657734,
            "range": "± 12064",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1624717,
            "range": "± 9232",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2519607,
            "range": "± 8507",
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
        "date": 1702759094074,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5012918,
            "range": "± 14523",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5147347,
            "range": "± 9902",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 5018330,
            "range": "± 12438",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4086549,
            "range": "± 23424",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4195122,
            "range": "± 30480",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 4062555,
            "range": "± 25987",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 76337598,
            "range": "± 183677",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 82445038,
            "range": "± 270979",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 76454623,
            "range": "± 210643",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 62824141,
            "range": "± 83857",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 68735789,
            "range": "± 113975",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 63008734,
            "range": "± 134749",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3618637,
            "range": "± 15063",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3889393,
            "range": "± 12424",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 3623946,
            "range": "± 10228",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3015725,
            "range": "± 7418",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3283174,
            "range": "± 6369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 3009709,
            "range": "± 5218",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1368269,
            "range": "± 3361",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1467602,
            "range": "± 3741",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 1369025,
            "range": "± 2901",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1097407,
            "range": "± 1422",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1202762,
            "range": "± 4711",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 1099364,
            "range": "± 3765",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135162,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143162,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 135152,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111446,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 118875,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 111446,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 193720,
            "range": "± 455",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 204596,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 193814,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 155851,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 166147,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 155806,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 281712,
            "range": "± 1001",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 303145,
            "range": "± 1339",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 281152,
            "range": "± 1386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 230629,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 250175,
            "range": "± 531",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 231412,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55435,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 347464,
            "range": "± 1121",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 442980,
            "range": "± 2253",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 594683,
            "range": "± 1725",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7481064,
            "range": "± 10029",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1583099,
            "range": "± 6254",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613425,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1318913,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 733665,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 676455,
            "range": "± 490",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 262448,
            "range": "± 1048",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 165214,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 183626,
            "range": "± 498",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 15284,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 44835,
            "range": "± 1013",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7770160,
            "range": "± 36990",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6262948,
            "range": "± 4229",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6000276,
            "range": "± 7005",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1358076,
            "range": "± 2268",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1291073,
            "range": "± 7888",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1064611,
            "range": "± 3058",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1041348,
            "range": "± 203450",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1094098,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2947068,
            "range": "± 1646",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1216959,
            "range": "± 9005",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1622369,
            "range": "± 12409",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1614153,
            "range": "± 13250",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2473369,
            "range": "± 3814",
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
        "date": 1702759685810,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5062768,
            "range": "± 25906",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5208829,
            "range": "± 14496",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 2436930,
            "range": "± 4046",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4040877,
            "range": "± 10138",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4195453,
            "range": "± 16758",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 396611,
            "range": "± 2511",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 76580254,
            "range": "± 214828",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 82797873,
            "range": "± 164528",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 32810660,
            "range": "± 125836",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 62001661,
            "range": "± 139326",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 68184770,
            "range": "± 214116",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3033519,
            "range": "± 16266",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3603247,
            "range": "± 8385",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3880235,
            "range": "± 10415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 1556913,
            "range": "± 4493",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 2963294,
            "range": "± 4444",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3239519,
            "range": "± 8726",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 253530,
            "range": "± 2212",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1327503,
            "range": "± 2183",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1430736,
            "range": "± 2093",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 553209,
            "range": "± 1108",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1068406,
            "range": "± 2861",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1168559,
            "range": "± 3767",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 36371,
            "range": "± 167",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135432,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143724,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 66752,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 110811,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 117252,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 18741,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 190912,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 201388,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 93056,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 154900,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 163873,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 21763,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 278398,
            "range": "± 1630",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 298863,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 129894,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 229045,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 247294,
            "range": "± 724",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 24965,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55804,
            "range": "± 788",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350996,
            "range": "± 3160",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 448479,
            "range": "± 2676",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 583222,
            "range": "± 6029",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6534549,
            "range": "± 6233",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1512606,
            "range": "± 6297",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 619104,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1314861,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 687853,
            "range": "± 4204",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 726801,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 263407,
            "range": "± 668",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 182059,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 202647,
            "range": "± 806",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17027,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 51936,
            "range": "± 1131",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7185795,
            "range": "± 8877",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6216939,
            "range": "± 5632",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6554465,
            "range": "± 32426",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1472220,
            "range": "± 2972",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1358881,
            "range": "± 2865",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1175088,
            "range": "± 2439",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1155193,
            "range": "± 2540",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1124665,
            "range": "± 1753",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2943145,
            "range": "± 3294",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1315584,
            "range": "± 10515",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1691572,
            "range": "± 17119",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1648321,
            "range": "± 11314",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2606146,
            "range": "± 38983",
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
        "date": 1702808840733,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5055809,
            "range": "± 19622",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5201888,
            "range": "± 4427",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 2440294,
            "range": "± 5133",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4049826,
            "range": "± 5732",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4190617,
            "range": "± 6014",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 391395,
            "range": "± 3621",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 75618525,
            "range": "± 132009",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 81689883,
            "range": "± 201967",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 32300702,
            "range": "± 73163",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 61943507,
            "range": "± 142137",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 67861628,
            "range": "± 266846",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3048177,
            "range": "± 23238",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3562051,
            "range": "± 8625",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3863622,
            "range": "± 13198",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 1537972,
            "range": "± 3477",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 2968084,
            "range": "± 7429",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3249949,
            "range": "± 3435",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 242946,
            "range": "± 2338",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1322663,
            "range": "± 6986",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1424342,
            "range": "± 4442",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 549814,
            "range": "± 1666",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1067609,
            "range": "± 1520",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1167862,
            "range": "± 2688",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 36795,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 134983,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 142511,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 65923,
            "range": "± 325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 110671,
            "range": "± 204",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 117652,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 18847,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 192237,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 200749,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 92581,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 155291,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 163928,
            "range": "± 529",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 21909,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 276810,
            "range": "± 581",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 296621,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 128575,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 227851,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 245224,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 24834,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 51522,
            "range": "± 945",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350658,
            "range": "± 2848",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 448761,
            "range": "± 3112",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 584782,
            "range": "± 1560",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6536843,
            "range": "± 16132",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1515737,
            "range": "± 12203",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613490,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1315049,
            "range": "± 1302",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 687785,
            "range": "± 2123",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 725198,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 262884,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 181021,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 203065,
            "range": "± 3843",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16981,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 50527,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7183620,
            "range": "± 6020",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6213545,
            "range": "± 2434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6619514,
            "range": "± 6235",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1450695,
            "range": "± 1251",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1358886,
            "range": "± 818",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1176561,
            "range": "± 2021",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1153975,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1123556,
            "range": "± 1722",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2942468,
            "range": "± 2662",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1246909,
            "range": "± 12987",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1658340,
            "range": "± 10906",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1573194,
            "range": "± 19008",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2424070,
            "range": "± 27308",
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
        "date": 1702820008172,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5075662,
            "range": "± 12816",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5221790,
            "range": "± 14989",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 2440441,
            "range": "± 14096",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4058901,
            "range": "± 8020",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4215968,
            "range": "± 18837",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 392817,
            "range": "± 3068",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 76709214,
            "range": "± 237104",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 83023696,
            "range": "± 232602",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 32917409,
            "range": "± 65333",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 62752358,
            "range": "± 121043",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 68552348,
            "range": "± 167686",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3153536,
            "range": "± 41248",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3617955,
            "range": "± 15087",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3905835,
            "range": "± 14415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 1549005,
            "range": "± 4086",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 2980513,
            "range": "± 12881",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3250417,
            "range": "± 8584",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 252747,
            "range": "± 3779",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1323873,
            "range": "± 1233",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1432875,
            "range": "± 3515",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 547936,
            "range": "± 1480",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1071332,
            "range": "± 3464",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1169535,
            "range": "± 6153",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 36033,
            "range": "± 292",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 134296,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 141718,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 65116,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 109581,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 116583,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 18872,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 190818,
            "range": "± 1105",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 201234,
            "range": "± 625",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 91826,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 154814,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 163638,
            "range": "± 351",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 21851,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 278005,
            "range": "± 2501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 297561,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 127984,
            "range": "± 266",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 228252,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 245393,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 24663,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53915,
            "range": "± 1192",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 353786,
            "range": "± 3677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 450852,
            "range": "± 851",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 582816,
            "range": "± 3659",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6533233,
            "range": "± 35559",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1448753,
            "range": "± 13599",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 614081,
            "range": "± 667",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1314817,
            "range": "± 1026",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 690700,
            "range": "± 3082",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 748119,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 264452,
            "range": "± 4862",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 182257,
            "range": "± 3020",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 203088,
            "range": "± 6736",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 17256,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 49891,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7192011,
            "range": "± 5457",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6220002,
            "range": "± 6960",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6520592,
            "range": "± 6856",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1439505,
            "range": "± 4043",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1421233,
            "range": "± 2866",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1194227,
            "range": "± 19153",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1155025,
            "range": "± 2556",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1126135,
            "range": "± 3464",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2945038,
            "range": "± 2101",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1263747,
            "range": "± 8640",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1674465,
            "range": "± 13819",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1592813,
            "range": "± 15943",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2439332,
            "range": "± 22788",
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
        "date": 1702821892733,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5039170,
            "range": "± 12275",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5214395,
            "range": "± 13307",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 2448339,
            "range": "± 5616",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4086833,
            "range": "± 14531",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4237811,
            "range": "± 12307",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 397865,
            "range": "± 2483",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 75781319,
            "range": "± 408121",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 81983088,
            "range": "± 92965",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 33092484,
            "range": "± 41201",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 62934797,
            "range": "± 106207",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 68950383,
            "range": "± 123327",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3189440,
            "range": "± 46909",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3567801,
            "range": "± 8701",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3864382,
            "range": "± 11389",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 1549806,
            "range": "± 3215",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3001985,
            "range": "± 7476",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3276548,
            "range": "± 7315",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 254145,
            "range": "± 3919",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1316050,
            "range": "± 3969",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1417783,
            "range": "± 3706",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 551377,
            "range": "± 2378",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1072191,
            "range": "± 1541",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1175909,
            "range": "± 4088",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 36151,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135473,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143511,
            "range": "± 1156",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 66672,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111065,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 117687,
            "range": "± 466",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 18117,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 191512,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 201164,
            "range": "± 2131",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 92387,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 154671,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 164774,
            "range": "± 438",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 22043,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 276568,
            "range": "± 1175",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 297260,
            "range": "± 1031",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 129068,
            "range": "± 650",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 227972,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 246508,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 24853,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54941,
            "range": "± 730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349223,
            "range": "± 1305",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445223,
            "range": "± 9130",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 573392,
            "range": "± 15530",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7486039,
            "range": "± 48030",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1495764,
            "range": "± 20972",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 616770,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1316853,
            "range": "± 640",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 736487,
            "range": "± 1091",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 698813,
            "range": "± 1410",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 263094,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 176903,
            "range": "± 694",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 197641,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16906,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 51694,
            "range": "± 1120",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 8285748,
            "range": "± 31172",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6274103,
            "range": "± 3876",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6265331,
            "range": "± 9394",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1306130,
            "range": "± 1286",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1360569,
            "range": "± 2878",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1150504,
            "range": "± 2098",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1136038,
            "range": "± 1695",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1142371,
            "range": "± 1661",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2939398,
            "range": "± 6140",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1265524,
            "range": "± 10592",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1626785,
            "range": "± 12154",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1574996,
            "range": "± 17814",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2423477,
            "range": "± 32284",
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
        "date": 1702890575897,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5039010,
            "range": "± 9247",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5188109,
            "range": "± 8376",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 2435068,
            "range": "± 6436",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4086576,
            "range": "± 6828",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/fuel",
            "value": 4231665,
            "range": "± 9162",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/lazy/default",
            "value": 398891,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 74873842,
            "range": "± 260245",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 81210547,
            "range": "± 194159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 32152355,
            "range": "± 113246",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 62170478,
            "range": "± 285314",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/fuel",
            "value": 68379101,
            "range": "± 359206",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/lazy/default",
            "value": 3024504,
            "range": "± 21556",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3561165,
            "range": "± 7285",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3867543,
            "range": "± 18912",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 1544916,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3011073,
            "range": "± 16246",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/fuel",
            "value": 3255750,
            "range": "± 9318",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/lazy/default",
            "value": 248352,
            "range": "± 5046",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1308995,
            "range": "± 3430",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1413305,
            "range": "± 1642",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 549511,
            "range": "± 1718",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1069648,
            "range": "± 2715",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/fuel",
            "value": 1175332,
            "range": "± 2808",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/lazy/default",
            "value": 35449,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135000,
            "range": "± 904",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143977,
            "range": "± 713",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 66154,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 110905,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/fuel",
            "value": 118524,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/lazy/default",
            "value": 18621,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 191494,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 201107,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 91730,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 155142,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/fuel",
            "value": 165917,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/lazy/default",
            "value": 21884,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 275995,
            "range": "± 587",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 296963,
            "range": "± 1141",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 128061,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 228290,
            "range": "± 721",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/fuel",
            "value": 246422,
            "range": "± 1195",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/lazy/default",
            "value": 24369,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53781,
            "range": "± 1322",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350025,
            "range": "± 2030",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444323,
            "range": "± 19055",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 573798,
            "range": "± 2218",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7475246,
            "range": "± 7405",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1460783,
            "range": "± 11217",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 610858,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317551,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 736314,
            "range": "± 722",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 703391,
            "range": "± 940",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 263136,
            "range": "± 1003",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 176651,
            "range": "± 787",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 198403,
            "range": "± 1706",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16517,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 49881,
            "range": "± 1264",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 8250440,
            "range": "± 48102",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6228906,
            "range": "± 8133",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6275431,
            "range": "± 17799",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1333462,
            "range": "± 1366",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1360953,
            "range": "± 2035",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1158840,
            "range": "± 13808",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1136025,
            "range": "± 10989",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1141969,
            "range": "± 2552",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2941534,
            "range": "± 39910",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1277591,
            "range": "± 10135",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1642100,
            "range": "± 18863",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1585315,
            "range": "± 14709",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2423289,
            "range": "± 18644",
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
        "date": 1702896669525,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5162701,
            "range": "± 7767",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5318593,
            "range": "± 8885",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2483149,
            "range": "± 7014",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 428728,
            "range": "± 3419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4171177,
            "range": "± 6706",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 77357972,
            "range": "± 194501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 83261496,
            "range": "± 152699",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 33554957,
            "range": "± 43039",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3832908,
            "range": "± 33775",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 64053129,
            "range": "± 119428",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3688737,
            "range": "± 8318",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 4002607,
            "range": "± 35405",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1582144,
            "range": "± 7369",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 257941,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3086202,
            "range": "± 27039",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1340686,
            "range": "± 3752",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1442740,
            "range": "± 2793",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 557350,
            "range": "± 1829",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 37027,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1081357,
            "range": "± 4856",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135895,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143306,
            "range": "± 780",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 66445,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 19249,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 110926,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 193203,
            "range": "± 203",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 202808,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 92462,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23340,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 154739,
            "range": "± 435",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 281439,
            "range": "± 1594",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 299817,
            "range": "± 1409",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 130140,
            "range": "± 181",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 26230,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 228537,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 55577,
            "range": "± 2362",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349098,
            "range": "± 894",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 444713,
            "range": "± 970",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 591327,
            "range": "± 6988",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7487419,
            "range": "± 2374",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1479410,
            "range": "± 7719",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 618621,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317358,
            "range": "± 956",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 709156,
            "range": "± 1819",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 711992,
            "range": "± 1215",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 238101,
            "range": "± 2137",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 181668,
            "range": "± 5680",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 197889,
            "range": "± 358",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16770,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 50926,
            "range": "± 1020",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7921179,
            "range": "± 41970",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6229732,
            "range": "± 30817",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6257312,
            "range": "± 7937",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1316639,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1357526,
            "range": "± 755",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1161095,
            "range": "± 27881",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1165144,
            "range": "± 986",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1153453,
            "range": "± 3346",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2947600,
            "range": "± 36816",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1275895,
            "range": "± 8149",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1642504,
            "range": "± 9395",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1572953,
            "range": "± 10199",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2408390,
            "range": "± 19953",
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
        "date": 1702912527006,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5208035,
            "range": "± 21041",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5306862,
            "range": "± 10159",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2469875,
            "range": "± 5886",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 426848,
            "range": "± 3054",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4135494,
            "range": "± 13511",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 77200547,
            "range": "± 258044",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 83179943,
            "range": "± 204347",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 33019986,
            "range": "± 92973",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3764770,
            "range": "± 72640",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 63758604,
            "range": "± 459129",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3729556,
            "range": "± 15359",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 4002698,
            "range": "± 31311",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1581800,
            "range": "± 10267",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 260695,
            "range": "± 4781",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3082560,
            "range": "± 20609",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1345421,
            "range": "± 5991",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1449451,
            "range": "± 2385",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 550548,
            "range": "± 1235",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 36992,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1075732,
            "range": "± 7342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 136680,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143145,
            "range": "± 1462",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 65135,
            "range": "± 438",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 18986,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111229,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 194338,
            "range": "± 1021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 204237,
            "range": "± 1924",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 91196,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23127,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 156080,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 279340,
            "range": "± 1165",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 299110,
            "range": "± 1157",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 128240,
            "range": "± 307",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 25050,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 230631,
            "range": "± 699",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53023,
            "range": "± 1131",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 351475,
            "range": "± 2179",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 447568,
            "range": "± 7494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 593127,
            "range": "± 7424",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7495740,
            "range": "± 9910",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1471374,
            "range": "± 5838",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 618495,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317486,
            "range": "± 1698",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 712141,
            "range": "± 26821",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 711299,
            "range": "± 1125",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 240229,
            "range": "± 1623",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 176610,
            "range": "± 1677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 196741,
            "range": "± 3502",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16400,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 51489,
            "range": "± 1234",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7951479,
            "range": "± 98794",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6232128,
            "range": "± 8024",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6308857,
            "range": "± 16714",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1440470,
            "range": "± 3422",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1347273,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1129096,
            "range": "± 4252",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1165991,
            "range": "± 3146",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1154735,
            "range": "± 20601",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2942933,
            "range": "± 289008",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1332985,
            "range": "± 13760",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1732414,
            "range": "± 9730",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1656183,
            "range": "± 10863",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2518142,
            "range": "± 24738",
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
        "date": 1702913155123,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5156395,
            "range": "± 9784",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5312051,
            "range": "± 13354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2433698,
            "range": "± 7075",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 423782,
            "range": "± 2639",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4133489,
            "range": "± 15811",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 77261694,
            "range": "± 164364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 83285725,
            "range": "± 124320",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 32746809,
            "range": "± 101269",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3597039,
            "range": "± 73501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 63509694,
            "range": "± 218853",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3671644,
            "range": "± 22176",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3951474,
            "range": "± 21835",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1549985,
            "range": "± 4399",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 251735,
            "range": "± 1973",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3039575,
            "range": "± 14909",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1348246,
            "range": "± 8215",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1453013,
            "range": "± 1669",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 549902,
            "range": "± 1462",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 37275,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1076811,
            "range": "± 2418",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 137047,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 144806,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 66300,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 19315,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111811,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 194464,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 204479,
            "range": "± 500",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 91755,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23772,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 156804,
            "range": "± 2092",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 280445,
            "range": "± 1195",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 300484,
            "range": "± 671",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 127500,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 25678,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 229556,
            "range": "± 870",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 53797,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 349714,
            "range": "± 977",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 445007,
            "range": "± 3347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 609273,
            "range": "± 19692",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 7491646,
            "range": "± 3355",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1469440,
            "range": "± 5817",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 618560,
            "range": "± 11341",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1317494,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 709405,
            "range": "± 1414",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 718529,
            "range": "± 612",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 243819,
            "range": "± 2676",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 174861,
            "range": "± 1027",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 194205,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16366,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 49367,
            "range": "± 932",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 8090089,
            "range": "± 129438",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6233367,
            "range": "± 2898",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6248384,
            "range": "± 3835",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1454206,
            "range": "± 3285",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1358654,
            "range": "± 1155",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1130897,
            "range": "± 26517",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1164736,
            "range": "± 1260",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1147043,
            "range": "± 3235",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2939856,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1254113,
            "range": "± 16543",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1661583,
            "range": "± 12972",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1590909,
            "range": "± 8837",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2414458,
            "range": "± 13428",
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
        "date": 1702917139936,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5054598,
            "range": "± 21501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5203537,
            "range": "± 15741",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2415074,
            "range": "± 4895",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 422756,
            "range": "± 1678",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4127934,
            "range": "± 14493",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 76033079,
            "range": "± 187059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 82229086,
            "range": "± 246639",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 32407335,
            "range": "± 92054",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3342724,
            "range": "± 22715",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 64214439,
            "range": "± 275399",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3596655,
            "range": "± 20152",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 3891684,
            "range": "± 17249",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1541233,
            "range": "± 9501",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 248049,
            "range": "± 3287",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3021136,
            "range": "± 5950",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1316919,
            "range": "± 3463",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1417078,
            "range": "± 4857",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 538133,
            "range": "± 1913",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 36460,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1080820,
            "range": "± 4695",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135149,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 141830,
            "range": "± 725",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 65119,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 19006,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111446,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 191730,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 201424,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 90963,
            "range": "± 521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23261,
            "range": "± 75",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 156960,
            "range": "± 1168",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 275894,
            "range": "± 1363",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 296061,
            "range": "± 822",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 125509,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 25648,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 230403,
            "range": "± 751",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54904,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350389,
            "range": "± 1190",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 453414,
            "range": "± 9045",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 586813,
            "range": "± 2360",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 6506996,
            "range": "± 27494",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1488392,
            "range": "± 18078",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613592,
            "range": "± 797",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1315421,
            "range": "± 1808",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 697502,
            "range": "± 5347",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 714717,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 263062,
            "range": "± 1262",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 175555,
            "range": "± 391",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 195302,
            "range": "± 850",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16388,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 49379,
            "range": "± 1164",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7190246,
            "range": "± 3786",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6219027,
            "range": "± 3939",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6296061,
            "range": "± 16691",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1375991,
            "range": "± 3532",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1359099,
            "range": "± 887",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1138047,
            "range": "± 1730",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1155290,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1107541,
            "range": "± 4328",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2955316,
            "range": "± 26197",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1262186,
            "range": "± 8685",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1661739,
            "range": "± 21991",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1599724,
            "range": "± 12772",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2474585,
            "range": "± 19537",
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
        "date": 1703111397255,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5191404,
            "range": "± 13722",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5363223,
            "range": "± 16534",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2473269,
            "range": "± 7002",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 430573,
            "range": "± 4791",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4217570,
            "range": "± 12674",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 78255304,
            "range": "± 237092",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 84736714,
            "range": "± 284376",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 33628971,
            "range": "± 294857",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 4188141,
            "range": "± 56687",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 64614542,
            "range": "± 125872",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3808933,
            "range": "± 16590",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 4088513,
            "range": "± 30341",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1580092,
            "range": "± 7456",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 254713,
            "range": "± 3021",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3174133,
            "range": "± 7975",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1340607,
            "range": "± 2940",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1441906,
            "range": "± 6325",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 547348,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 37246,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1081770,
            "range": "± 3663",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 136093,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143985,
            "range": "± 1044",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 66192,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 18999,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111199,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 193970,
            "range": "± 613",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 204447,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 92495,
            "range": "± 354",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23363,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 155985,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 279736,
            "range": "± 1556",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 300173,
            "range": "± 1821",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 128849,
            "range": "± 2148",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 25824,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 229257,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54035,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 350457,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 454327,
            "range": "± 14015",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 580840,
            "range": "± 15921",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5348890,
            "range": "± 8434",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1459312,
            "range": "± 13729",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 620477,
            "range": "± 9452",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1315328,
            "range": "± 1381",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 481797,
            "range": "± 12650",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 710910,
            "range": "± 1353",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 260388,
            "range": "± 947",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 174818,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 191665,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16515,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 50891,
            "range": "± 1496",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7044949,
            "range": "± 11099",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6436509,
            "range": "± 15466",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6234852,
            "range": "± 15653",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1309806,
            "range": "± 3849",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1335725,
            "range": "± 1792",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1123273,
            "range": "± 3541",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 977577,
            "range": "± 1581",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1150740,
            "range": "± 2900",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2952586,
            "range": "± 3799",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1253612,
            "range": "± 18794",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1669678,
            "range": "± 24075",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1587465,
            "range": "± 11106",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2483391,
            "range": "± 9385",
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
        "date": 1703258413113,
        "tool": "cargo",
        "benches": [
          {
            "name": "translate/wasm_kernel/checked/eager/default",
            "value": 5112593,
            "range": "± 21978",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/eager/fuel",
            "value": 5247059,
            "range": "± 22364",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy-translation/default",
            "value": 2470076,
            "range": "± 9556",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/checked/lazy/default",
            "value": 428296,
            "range": "± 2290",
            "unit": "ns/iter"
          },
          {
            "name": "translate/wasm_kernel/unchecked/eager/default",
            "value": 4181762,
            "range": "± 10812",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/default",
            "value": 77219892,
            "range": "± 1054233",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/eager/fuel",
            "value": 83455914,
            "range": "± 285521",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy-translation/default",
            "value": 33254835,
            "range": "± 81415",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/checked/lazy/default",
            "value": 3972964,
            "range": "± 39527",
            "unit": "ns/iter"
          },
          {
            "name": "translate/spidermonkey/unchecked/eager/default",
            "value": 64226346,
            "range": "± 163087",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/default",
            "value": 3762684,
            "range": "± 22035",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/eager/fuel",
            "value": 4039615,
            "range": "± 20059",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy-translation/default",
            "value": 1589381,
            "range": "± 17785",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/checked/lazy/default",
            "value": 256225,
            "range": "± 4525",
            "unit": "ns/iter"
          },
          {
            "name": "translate/pulldown_cmark/unchecked/eager/default",
            "value": 3138935,
            "range": "± 12643",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/default",
            "value": 1331790,
            "range": "± 5029",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/eager/fuel",
            "value": 1429796,
            "range": "± 17990",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy-translation/default",
            "value": 545912,
            "range": "± 2057",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/checked/lazy/default",
            "value": 37309,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "translate/bz2/unchecked/eager/default",
            "value": 1083078,
            "range": "± 11969",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/default",
            "value": 135075,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/eager/fuel",
            "value": 143889,
            "range": "± 688",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy-translation/default",
            "value": 66388,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/checked/lazy/default",
            "value": 19071,
            "range": "± 120",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc20/unchecked/eager/default",
            "value": 111976,
            "range": "± 626",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/default",
            "value": 193759,
            "range": "± 954",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/eager/fuel",
            "value": 202799,
            "range": "± 572",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy-translation/default",
            "value": 93279,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/checked/lazy/default",
            "value": 23574,
            "range": "± 232",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc721/unchecked/eager/default",
            "value": 157753,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/default",
            "value": 280665,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/eager/fuel",
            "value": 300980,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy-translation/default",
            "value": 129824,
            "range": "± 322",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/checked/lazy/default",
            "value": 26047,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "translate/erc1155/unchecked/eager/default",
            "value": 232015,
            "range": "± 1015",
            "unit": "ns/iter"
          },
          {
            "name": "instantiate/wasm_kernel",
            "value": 54984,
            "range": "± 1692",
            "unit": "ns/iter"
          },
          {
            "name": "execute/tiny_keccak",
            "value": 351256,
            "range": "± 726",
            "unit": "ns/iter"
          },
          {
            "name": "execute/rev_complement",
            "value": 457176,
            "range": "± 2536",
            "unit": "ns/iter"
          },
          {
            "name": "execute/regex_redux",
            "value": 587214,
            "range": "± 7677",
            "unit": "ns/iter"
          },
          {
            "name": "execute/count_until",
            "value": 5467426,
            "range": "± 22127",
            "unit": "ns/iter"
          },
          {
            "name": "execute/br_table",
            "value": 1487741,
            "range": "± 19763",
            "unit": "ns/iter"
          },
          {
            "name": "execute/trunc_f2i",
            "value": 613426,
            "range": "± 696",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/bump",
            "value": 1315097,
            "range": "± 1836",
            "unit": "ns/iter"
          },
          {
            "name": "execute/global/get_const",
            "value": 478717,
            "range": "± 970",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/rec",
            "value": 701605,
            "range": "± 815",
            "unit": "ns/iter"
          },
          {
            "name": "execute/factorial/iter",
            "value": 227638,
            "range": "± 5260",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/rec",
            "value": 176563,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_scan",
            "value": 195965,
            "range": "± 841",
            "unit": "ns/iter"
          },
          {
            "name": "execute/recursive_trap",
            "value": 16387,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "execute/call/host/1",
            "value": 51800,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fuse",
            "value": 7447021,
            "range": "± 55574",
            "unit": "ns/iter"
          },
          {
            "name": "execute/divrem",
            "value": 6421030,
            "range": "± 7649",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/rec",
            "value": 6515760,
            "range": "± 74577",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/tail",
            "value": 1303008,
            "range": "± 5034",
            "unit": "ns/iter"
          },
          {
            "name": "execute/fibonacci/iter",
            "value": 1331829,
            "range": "± 2693",
            "unit": "ns/iter"
          },
          {
            "name": "execute/is_even/rec",
            "value": 1150443,
            "range": "± 29552",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/sum_bytes",
            "value": 1107406,
            "range": "± 3070",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/fill_bytes",
            "value": 1106345,
            "range": "± 10422",
            "unit": "ns/iter"
          },
          {
            "name": "execute/memory/vec_add",
            "value": 2956611,
            "range": "± 43065",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/0",
            "value": 1268597,
            "range": "± 12103",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/typed/16",
            "value": 1645105,
            "range": "± 10774",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/0",
            "value": 1571440,
            "range": "± 18359",
            "unit": "ns/iter"
          },
          {
            "name": "overhead/call/untyped/16",
            "value": 2512375,
            "range": "± 8726",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}