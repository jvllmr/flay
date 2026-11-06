window.BENCHMARK_DATA = {
  "lastUpdate": 1793994592600,
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
      }
    ]
  }
}