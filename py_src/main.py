import betfair_data
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

paths = [
    "data/2021_10_OctRacingProAu.tar",
    # "data/2021_11_NovRacingAUPro.tar",
    # "data/2021_12_DecRacingAUPro.tar",
]

count = 0

for i, m in enumerate(betfair_data.Sources(paths)):
    count += 1 # the first market object comes with the first deserialization

    while m.update():
        count += 1
    
    print("> Market {} Update {}".format(i, count), end='\r')

