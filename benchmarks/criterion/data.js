window.BENCHMARK_DATA = {
  "lastUpdate": 1693428929216,
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
      }
    ]
  }
}