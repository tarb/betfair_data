# Betfair Data

Betfair Data is a very fast Betfair historical data file parsing library for python. It currently supports tar archives containing BZ2 compressed NLJSON files (the standard format provided by [Betfair's historic data portal](https://historicdata.betfair.com/#/home)).

The library is written in Rust and uses advanced performance enhancing techniques, like in place json deserialization and decompressing Bz2 encoded data on worker threads and is ideal for parsing large quantities of historic data that could otherwise take hours or days to parse.

## Installation

```
pip install betfair_data
```

Note: requires Python >= 3.6.


## Example

```python
import betfair_data

paths = [
    "data/2021_12_DecRacingAUPro.tar",
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
]

market_count = 0
update_count = 0

for market in betfair_data.TarBz2(paths):
    market_count += 1
    update_count += 1
    
    while market.update():
        update_count += 1

    print(f"Markets {market_count} Updates {update_count}", end='\r')
print(f"Markets {market_count} Updates {update_count}")

```
## Types
IDE's should automatically detect the types and provide checking and auto complete. See the [pyi stub file](betfair_data.pyi) for a comprehensive view of the types and method available.

<br />

## Benchmarks

| Betfair Data (this)  | [Betfairlightweight](https://github.com/liampauling/betfair/) |
| ---------------------|---------------------|
| 3m 37sec             | 1hour 1min 45sec    |
| ~101 markets/sec     | ~6 markets/sec      |
| ~768,000 updates/sec | ~45,500 updates/sec |

Benchmarks were run against 3 months of Australian racing markets comprising roughly 22,000 markets. Benchmarks were run on a M1 Macbook Pro with 32GB ram.

These results should only be used as a rough comparison, different machines, different sports and even different months can effect the performance and overall markets/updates per second.

No disrespect is intended towards betfairlightweight, which remains an amazing library and a top choice for working with the Betfair API. Every effort was made to have its benchmark below run as fast as possible, and any improvements are welcome.

<br>

Betfair_Data benchmark show in the example above.
<details><summary>Betfairlightweight Benchmark</summary>

```python
from typing import Sequence 

import unittest.mock
import tarfile
import bz2
import betfairlightweight

trading = betfairlightweight.APIClient("username", "password", "appkey")
listener = betfairlightweight.StreamListener(
    max_latency=None, lightweight=True, update_clk=False, output_queue=None, cumulative_runner_tv=True, calculate_market_tv=True
)

paths = [ 
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar"
]

def load_tar(file_paths: Sequence[str]):
    for file_path in file_paths:
        with tarfile.TarFile(file_path) as archive:
            for file in archive:
                yield bz2.open(archive.extractfile(file))
    return None

market_count = 0
update_count = 0

for file_obj in load_tar(paths):
    with unittest.mock.patch("builtins.open", lambda f, _: f):  
        stream = trading.streaming.create_historical_generator_stream(
            file_path=file_obj,
            listener=listener,
        )
        gen = stream.get_generator()

        market_count += 1
        for market_books in gen():
            for market_book in market_books:
                update_count += 1

    print(f"Markets {market_count} Updates {update_count}", end='\r')
print(f"Markets {market_count} Updates {update_count}")

```
</details>

<br>
<br>


## Logging

Logging can be enabled and warnings are emitted for IO and JSON errors

```python
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')
```
Example logged errors

```log
WARNING betfair_data source: data/2021_10_OctRacingAUPro.tar file: PRO/2021/Oct/4/30970292/1.188542184.bz2 err: (JSON Parse Error) expected value at line 1480 column 1
WARNING betfair_data source: data/2021_10_OctRacingAUPro.tar file: PRO/2021/Oct/8/30985584/1.188739324.bz2 err: (JSON Parse Error) expected `:` at line 1 column 909
WARNING betfair_data source: data/2021_10_OctRacingAUPro.tar file: PRO/2021/Oct/8/30985584/1.188739325.bz2 err: (JSON Parse Error) expected `:` at line 1 column 904
WARNING betfair_data source: data/2021_10_OctRacingAUPro.tar file: PRO/2021/Oct/15/31001342/1.189124831.bz2 err: (JSON Parse Error) expected value at line 1335 column 1
```

