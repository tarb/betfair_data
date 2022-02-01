# Betfair Data

Betfair Data is a fast Betfair historical data file parsing library for python. It currently supports tar archives containing BZ2 compressed NLJSON files.

## Installation

```
pip install betfair_data
```

Note: requires Python >= 3.6.

## Example

```python
import betfair_data

paths = [
    "data/HistoricalDataFile1.tar",
    "data/HistoricalDataFile2.tar",
]

market_count = 0
update_count = 0

for market in betfair_data.TarBz2(paths):
    market_count += 1
    while market.update():
        update_count += 1

    print("Markets {} Updates {}".format(market_count, update_count), end='\r')

```
