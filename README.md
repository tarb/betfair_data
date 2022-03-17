# Betfair Data

Betfair Data is a very fast Betfair historical data file parsing library for python. It supports both the official [Betfair's historic data](https://historicdata.betfair.com/#/home) and self recorded stream files. 

The library is written in Rust and uses advanced performance enhancing techniques, like in place json deserialization and decompressing Bz2/Gzip encoded data on worker threads and is ideal for parsing large quantities of historic data that could otherwise take hours or days to parse.

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

for market in bfd.Files(paths).iter():
    market_count += 1

    for market in file:
        update_count += 1

    print(f"Markets {market_count} Updates {update_count}", end='\r')
print(f"Markets {market_count} Updates {update_count}")

```

## Loading Files

You can read in self recorded stream files. Make sure to set cumulative_runner_tv to False for self recorded files to make sure you get the correct runner and market volumes.
```python
import betfair_data
import glob

paths = glob.glob("data/*.gz")
files = betfair_data.Files(paths, cumulative_runner_tv=False)
```
Or you can read official Betfair Tar archives with bz2 encoded market files.

```python
import betfair_data
import glob

paths = glob.glob("data/*.tar")
files = betfair_data.TarBz2(paths, cumulative_runner_tv=True)
```

Or load the file through any other means and pass the bytes and name into the object constructors.

```python
# generator to read in files
def load_files(paths: str):
    for path in glob.glob(paths, recursive=True):
        with open(path, "rb") as file:
            yield (path, file.read())

# iterate over the files and convert into bflw iterator
for name, bs in load_files("markets/*.json"):
    for market_books in bflw.BflwIter(name, bs):
        for market_book in market_books:
            # do stuff
            pass
```

## Object Types

You can use differnt styles of objects, with pros or depending on your needs

Mutable objects, generally the fastest, but can be hard to use. If you find yourself calling market.copy a lot, you may find immutable faster
``` python
# where files is loaded from a TarBz2 or Files source like above
mut_iter = files.mutable()
for market in mut_iter: # different markets per file
    while market.update(): # update the market in place
        pass
```

Immutable objects, slightly slower but can be easier to use. Equilivent of calling market.copy() on every update but faster, as only objects that change make new copies. 
``` python
immut_iter = files.immutable()
for market_iter in immut_iter: # different files
    for market in market_iter: # each update of a market/file
        pass
```

Betfairlightweight compatible version, drop in replacement for bflw objects. 
```python
bflw_iter = files.bflw()
for file in bflw_iter: # different files
    for market_books in file: # different books per update
        for market in market_books: # each update of a market
            pass
```

## Types
IDE's should automatically detect the types and provide checking and auto complete. See the [pyi stub file](betfair_data.pyi) for a comprehensive view of the types and method available.


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

