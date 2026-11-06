window.BENCHMARK_DATA = {
  "lastUpdate": 1794007809709,
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
      }
    ]
  }
}