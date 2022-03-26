# Betfair Data

Betfair Data is a very fast, Rust based, Betfair historical data parsing library for python. It supports both the official [Betfair's historic data](https://historicdata.betfair.com/#/home) and self recorded stream files. 

## Installation

```
pip install betfair_data
```

Note: requires Python >= 3.7.

## Example

```python
import betfair_data as bfd

paths = [
    "data/2021_12_DecRacingAUPro.tar",
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
]

market_count = 0
update_count = 0

for file in bfd.Files(paths).iter():
    market_count += 1

    for market in file:
        update_count += 1

    print(f"Markets {market_count} Updates {update_count}", end='\r')
print(f"Markets {market_count} Updates {update_count}")

```

## Loading Files

You can read in files quickly in a worker thread using the provided ```Files``` utility. It supports reading in bz2, gzip or uncompressed (.json) stream files or .tar or .zip archives containing such files. 
```python

paths = [
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.zip",
    "data/self_recorded_market.gz",
    "data/uncompressed_market.json",

]
files = bfd.Files(paths).iter()
```
Or you can use the glob library to find and select all the paths automatically. 

```python
import glob

paths = glob.glob("data/betfair_official/*.tar") + glob.glob("data/self_recorded/*.gz")
files = bfd.Files(paths).iter()
```

You can also load the file through any other means and pass the raw bytes and name into the File object constructor.

```python
# generator to read in files
def load_files(paths: str):
    for path in glob.glob(paths, recursive=True):
        with open(path, "rb") as file:
            yield bfd.File(path, file.read())

for file in load_files("markets/*.json"):
    for market in file:
        pass
```

## Benchmarks

Running over 3 months of Australian racing data on a 2021 M1 Macbook Pro.

| betfair_data (mutable) | betfair_data (immutable/bflw compat) | betfairlightweight lightweight=True | betfairlightweight lightweight=False|
| -----------------------|--------------------------------------|-------------------------------------|-------------------------------------|
| 5m 12sec               | 6m 50sec                             | 1hour 1min 45sec                    | 3hours 46mins 39sec                 |
| ~70 markets/sec        | ~53.5 markets/sec                    | ~6 markets/sec                      | ~1.62 markets/sec                   |
| ~534,200 updates/sec   | ~406,500 updates/sec                 | ~45,500 updates/sec                 | ~12,250 updates/sec                 |


## Types
IDE's should automatically detect the types and provide checking and auto complete. See the [pyi stub file](betfair_data.pyi) for a comprehensive view of the types and method available.


## Betfairlightweight
We also support a format that is a drop in replacement for ```betfairlightweight``` objects. We have rigorously tested it against betfairlightweight to ensure a complete match of its structure, any differences should be submitted as issues with the corresponding differences and the data used to create them.
```py
files = betfair_data.Files(paths).bflw()
```

```py
from betfair_data import bflw

file_bytes = ...
file = bflw.File("file_name", file_bytes)
```

## Logging

Logging can be enabled and warnings are emitted for IO and JSON errors.

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

