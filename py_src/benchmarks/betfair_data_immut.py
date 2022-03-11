import betfair_data
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

paths = [
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar",
]

market_count = 0
update_count = 0

for file in betfair_data.TarBz2(paths).iter(mutable=False):
    market_count += 1

    for market in file:
        update_count += 1

    print(f"Markets {market_count} Updates {update_count}", end='\r')
print(f"Markets {market_count} Updates {update_count}")
