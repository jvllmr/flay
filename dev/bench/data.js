window.BENCHMARK_DATA = {
  "lastUpdate": 1795620565534,
  "repoUrl": "https://github.com/jvllmr/flay",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "committer": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "id": "23047da2883fdd0a77edb0098c6dceada09e4a91",
          "message": "ci(benchmark): benchmark bundling, publish results to Pages",
          "timestamp": "2026-11-02T05:50:25Z",
          "url": "https://github.com/jvllmr/flay/pull/368/commits/23047da2883fdd0a77edb0098c6dceada09e4a91"
        },
        "date": 1793994505410,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 0.8274223436734505,
            "unit": "iter/sec",
            "range": "stddev: 0.023228945743220198",
            "extra": "mean: 1.2085726324000006 sec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.05890024775293794,
            "unit": "iter/sec",
            "range": "stddev: 0.08623389843803594",
            "extra": "mean: 16.97785727819999 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "committer": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "id": "03b466f375572871c5a37d972278cec818b6ea7c",
          "message": "ci(benchmark): benchmark bundling, publish results to Pages",
          "timestamp": "2026-11-02T05:50:25Z",
          "url": "https://github.com/jvllmr/flay/pull/368/commits/03b466f375572871c5a37d972278cec818b6ea7c"
        },
        "date": 1793994591846,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 1.6069756457598945,
            "unit": "iter/sec",
            "range": "stddev: 0.017496751586691554",
            "extra": "mean: 622.2869665999993 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.09753691809750518,
            "unit": "iter/sec",
            "range": "stddev: 0.156083262551261",
            "extra": "mean: 10.252528165800005 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jan@vllmr.dev",
            "name": "Jan Vollmer",
            "username": "jvllmr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6d1c1a66b72faab7e4956387070baf51c45a0264",
          "message": "ci(benchmark): benchmark bundling, publish results to Pages (#368)\n\nCo-authored-by: Mistral Vibe <vibe@mistral.ai>",
          "timestamp": "2026-11-06T19:53:24Z",
          "tree_id": "ec82c6b4f781ddb8f0a094f35bb44450bc141434",
          "url": "https://github.com/jvllmr/flay/commit/6d1c1a66b72faab7e4956387070baf51c45a0264"
        },
        "date": 1793995002246,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 0.7934637282795272,
            "unit": "iter/sec",
            "range": "stddev: 0.010678623043949701",
            "extra": "mean: 1.2602970549999895 sec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.05527344047302882,
            "unit": "iter/sec",
            "range": "stddev: 0.3215057984620612",
            "extra": "mean: 18.0918718184 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "committer": {
            "name": "jvllmr",
            "username": "jvllmr"
          },
          "id": "37c68a828a3c7f1efd9c9727148a5d05020212f8",
          "message": "perf: cache module traversals, keep file data off the python boundary",
          "timestamp": "2026-11-06T19:53:29Z",
          "url": "https://github.com/jvllmr/flay/pull/373/commits/37c68a828a3c7f1efd9c9727148a5d05020212f8"
        },
        "date": 1794006034846,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.809216821848001,
            "unit": "iter/sec",
            "range": "stddev: 0.0018350016978363444",
            "extra": "mean: 172.1402438000041 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.7721761654805451,
            "unit": "iter/sec",
            "range": "stddev: 0.04340209413172951",
            "extra": "mean: 1.2950412674000034 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jan@vllmr.dev",
            "name": "Jan Vollmer",
            "username": "jvllmr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "aac67205bc9c0d4fa52e2c8b3f9a88723292d28d",
          "message": "perf: cache module traversals, keep file data off the python boundary (#373)\n\nCo-authored-by: Mistral Vibe <vibe@mistral.ai>",
          "timestamp": "2026-11-06T23:28:48Z",
          "tree_id": "1f2f873f5601999c5cea9584cc26c559c54dd74e",
          "url": "https://github.com/jvllmr/flay/commit/aac67205bc9c0d4fa52e2c8b3f9a88723292d28d"
        },
        "date": 1794007809239,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.297707805163391,
            "unit": "iter/sec",
            "range": "stddev: 0.001929189763327271",
            "extra": "mean: 188.7608823999983 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.767768849932967,
            "unit": "iter/sec",
            "range": "stddev: 0.004435580538464765",
            "extra": "mean: 1.3024753479999986 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jan@vllmr.dev",
            "name": "Jan Vollmer",
            "username": "jvllmr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "e5cf5705a62d9af999b6598a96b1088e3dfdbab5",
          "message": "test(integration): cover bundling flay, clonf and pydantic end to end (#374)\n\nCo-authored-by: Mistral Vibe <vibe@mistral.ai>",
          "timestamp": "2026-11-07T00:20:42Z",
          "tree_id": "8df3807cc14052ac5b3fcdfaa965d55d95262fca",
          "url": "https://github.com/jvllmr/flay/commit/e5cf5705a62d9af999b6598a96b1088e3dfdbab5"
        },
        "date": 1794010953754,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.106779595130869,
            "unit": "iter/sec",
            "range": "stddev: 0.0022863301591297775",
            "extra": "mean: 195.81812399999876 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.6707406028016434,
            "unit": "iter/sec",
            "range": "stddev: 0.028696202806974918",
            "extra": "mean: 1.4908893182000014 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jan@vllmr.dev",
            "name": "Jan Vollmer",
            "username": "jvllmr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ace741c202a12cd44b1c583fc988543375396978",
          "message": "feat(treeshake): fold constant if/elif/else chains (#375)\n\nCo-authored-by: Mistral Vibe <vibe@mistral.ai>",
          "timestamp": "2026-11-07T19:24:50Z",
          "tree_id": "43d412881f55467f1a4eba2986ad021bcb06c054",
          "url": "https://github.com/jvllmr/flay/commit/ace741c202a12cd44b1c583fc988543375396978"
        },
        "date": 1794079591557,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 6.609580809994842,
            "unit": "iter/sec",
            "range": "stddev: 0.0028797184920077443",
            "extra": "mean: 151.29552519999834 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.7168672025459208,
            "unit": "iter/sec",
            "range": "stddev: 0.4973020363921582",
            "extra": "mean: 1.3949585033999967 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6edc99fcd90cd2d6367a74e1daf4389963c400f9",
          "message": "chore(deps): update all non-major dependencies (#376)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-09T02:46:18Z",
          "tree_id": "18061e9affb03df77ba3798089e8e4c65a271cce",
          "url": "https://github.com/jvllmr/flay/commit/6edc99fcd90cd2d6367a74e1daf4389963c400f9"
        },
        "date": 1794192500726,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.51963273087976,
            "unit": "iter/sec",
            "range": "stddev: 0.0018822221501131756",
            "extra": "mean: 181.1714744000028 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.644271753374069,
            "unit": "iter/sec",
            "range": "stddev: 0.08773511324208114",
            "extra": "mean: 1.5521400631999966 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c3f688ed1453cd563834499924425f537186311b",
          "message": "chore(deps): update all pre-commit dependencies (#377)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-09T05:31:33Z",
          "tree_id": "e1258aecd8d4199db7f822e7167275324b2027cc",
          "url": "https://github.com/jvllmr/flay/commit/c3f688ed1453cd563834499924425f537186311b"
        },
        "date": 1794202382730,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 6.502797673932668,
            "unit": "iter/sec",
            "range": "stddev: 0.0024198484561155057",
            "extra": "mean: 153.77996519999897 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.865603741973119,
            "unit": "iter/sec",
            "range": "stddev: 0.04500659606145239",
            "extra": "mean: 1.1552630280000045 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "017b0f24dec8acca1ffef9385605bcad8cbb6b48",
          "message": "chore(deps): update all pre-commit dependencies (#378)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-16T03:07:43Z",
          "tree_id": "b1d58d727f32b960e4b64d5b63380f2149e8d1fe",
          "url": "https://github.com/jvllmr/flay/commit/017b0f24dec8acca1ffef9385605bcad8cbb6b48"
        },
        "date": 1794798575372,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.113532405522522,
            "unit": "iter/sec",
            "range": "stddev: 0.001451494841097288",
            "extra": "mean: 195.5595312000014 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.6777312244049128,
            "unit": "iter/sec",
            "range": "stddev: 0.03214098961440808",
            "extra": "mean: 1.4755111819999995 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "779e1a7119c9646a2d7d34839a85e90f5621b6bf",
          "message": "chore(deps): update all rust non-major dependencies to 0.0.22 (#379)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-16T06:07:58Z",
          "tree_id": "433cb845f48977a8a767bc7740ddf315442e6a89",
          "url": "https://github.com/jvllmr/flay/commit/779e1a7119c9646a2d7d34839a85e90f5621b6bf"
        },
        "date": 1794809370610,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 6.786052162281982,
            "unit": "iter/sec",
            "range": "stddev: 0.0013200381953004366",
            "extra": "mean: 147.3610836000006 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.882564892665703,
            "unit": "iter/sec",
            "range": "stddev: 0.08660509031502947",
            "extra": "mean: 1.133061158800001 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c2e01021ebea3aac8bcfa3f018fa3c3b1074a92c",
          "message": "chore(deps): update pre-commit hook astral-sh/uv-pre-commit to v0.13.9 (#380)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T15:50:15Z",
          "tree_id": "7f602b0483bda3318de87bde285ac1d6f60fe144",
          "url": "https://github.com/jvllmr/flay/commit/c2e01021ebea3aac8bcfa3f018fa3c3b1074a92c"
        },
        "date": 1795017429704,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 4.925409407836792,
            "unit": "iter/sec",
            "range": "stddev: 0.0021699434201761115",
            "extra": "mean: 203.0288077999984 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.6758542045731083,
            "unit": "iter/sec",
            "range": "stddev: 0.02696839947407087",
            "extra": "mean: 1.4796090536000037 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7b61aa006687787ea809dd4697c3ee58b80efbb5",
          "message": "chore(deps): lock file maintenance (#386)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-18T22:03:13Z",
          "tree_id": "62c68d691d3288cbc9bdcba21b302854e5cd417c",
          "url": "https://github.com/jvllmr/flay/commit/7b61aa006687787ea809dd4697c3ee58b80efbb5"
        },
        "date": 1795039496343,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.6976940033016525,
            "unit": "iter/sec",
            "range": "stddev: 0.03912315970246511",
            "extra": "mean: 175.50960079999527 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.8131396972742841,
            "unit": "iter/sec",
            "range": "stddev: 0.06958745371354264",
            "extra": "mean: 1.2298009841999942 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8799518ba875df28d56136acc144f553e431b607",
          "message": "chore(deps): update dependency pytest to >=9.2.0,<9.3.0 (#381)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-19T07:10:02Z",
          "tree_id": "9a34cfd850094c932ec2d02540d2e926eb86a0aa",
          "url": "https://github.com/jvllmr/flay/commit/8799518ba875df28d56136acc144f553e431b607"
        },
        "date": 1795072328257,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 7.104788569236457,
            "unit": "iter/sec",
            "range": "stddev: 0.0011404986111648691",
            "extra": "mean: 140.75014199999885 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.8942434615085266,
            "unit": "iter/sec",
            "range": "stddev: 0.02334008946643279",
            "extra": "mean: 1.1182636978000033 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ee940f010d608117a2c03c983742cee6d1decf25",
          "message": "chore(deps): update all pre-commit dependencies (#387)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-23T02:15:22Z",
          "tree_id": "a2721396379f1785e8338f056be9599b54fb02c1",
          "url": "https://github.com/jvllmr/flay/commit/ee940f010d608117a2c03c983742cee6d1decf25"
        },
        "date": 1795400234205,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 5.0957485236525315,
            "unit": "iter/sec",
            "range": "stddev: 0.0017257925481276844",
            "extra": "mean: 196.2420232000028 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.6838182774842559,
            "unit": "iter/sec",
            "range": "stddev: 0.02235584475436981",
            "extra": "mean: 1.4623768228000074 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "be1e60a917159981f8f19a53726643866f6b75f9",
          "message": "chore(deps): update all rust non-major dependencies to 0.0.23 (#388)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-23T07:41:48Z",
          "tree_id": "aff7f90e1421f621394e9f0ce3c8fc3702580b80",
          "url": "https://github.com/jvllmr/flay/commit/be1e60a917159981f8f19a53726643866f6b75f9"
        },
        "date": 1795419987178,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 3.95707549065737,
            "unit": "iter/sec",
            "range": "stddev: 0.0682215108309681",
            "extra": "mean: 252.71188339999924 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.8565696951806723,
            "unit": "iter/sec",
            "range": "stddev: 0.05690778475754927",
            "extra": "mean: 1.1674473258000035 sec\nrounds: 5"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "29139614+renovate[bot]@users.noreply.github.com",
            "name": "renovate[bot]",
            "username": "renovate[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "68ca74bedb89c5379212b50d018dd355139dc771",
          "message": "chore(deps): update mshick/add-pr-comment action to v3 (#385)\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-11-25T16:27:51+01:00",
          "tree_id": "d53a2bd4b01915fa16c5bc3e12b571d068d84836",
          "url": "https://github.com/jvllmr/flay/commit/68ca74bedb89c5379212b50d018dd355139dc771"
        },
        "date": 1795620564758,
        "tool": "pytest",
        "benches": [
          {
            "name": "tests/test_integration.py::test_integration_pre_commit",
            "value": 6.866712260262779,
            "unit": "iter/sec",
            "range": "stddev: 0.007931485057460618",
            "extra": "mean: 145.630100999999 msec\nrounds: 5"
          },
          {
            "name": "tests/test_integration.py::test_integration_fastapi",
            "value": 0.795251926487574,
            "unit": "iter/sec",
            "range": "stddev: 0.17129453982046983",
            "extra": "mean: 1.257463159399998 sec\nrounds: 5"
          }
        ]
      }
    ]
  }
}