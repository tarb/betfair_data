import tarfile
import bz2
from typing import Sequence
import betfair_data
import logging
from betfair_data import bflw

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

paths = [ 
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar"
]

def run_with_py_loading():
    market_count = 0
    update_count = 0

    def load_tar(file_paths: Sequence[str]):
        for file_path in file_paths:
            with tarfile.TarFile(file_path) as archive:
                for file in archive:
                    f: bz2.BZ2File = bz2.open(archive.extractfile(file))
                    name = file.name
                    bytes = f.read()

                    yield (name, bytes)
        return None

    for name, bytes in load_tar(paths):
        file = bflw.BflwIter(name, bytes, cumulative_runner_tv=True)

        market_count += 1
        for market_books in file:
            for market_book in market_books:
                update_count += 1

        print(f"Market {market_count} Update {update_count}", end='\r')
    print(f"Market {market_count} Update {update_count}")



def run_with_rust_loading():
    market_count = 0
    update_count = 0
    
    for file in betfair_data.TarBz2(paths, cumulative_runner_tv=True).bflw():
        market_count += 1

        for market_books in file:
            for market_book in market_books:
                update_count += 1

        print(f"Market {market_count} Update {update_count}", end='\r')
    print(f"Market {market_count} Update {update_count}")


run_with_rust_loading()