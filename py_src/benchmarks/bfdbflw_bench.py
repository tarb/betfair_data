from typing import Sequence
import tarfile
import bz2
import logging
from betfair_data import bflw

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

paths = [ 
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar"
]


def run_with_py_loading():
    def load_tar(file_paths: Sequence[str]):
        for file_path in file_paths:
            with tarfile.TarFile(file_path) as archive:
                for file in archive:
                    f: bz2.BZ2File = bz2.open(archive.extractfile(file))
                    yield bflw.File(file.name, f.read())
        return None

    market_count = 0
    update_count = 0

    for file in load_tar(paths):
        market_count += 1
        for market_books in file:
            for market_book in market_books:
                update_count += 1

        print(f"Market {market_count} Update {update_count} File:{file.file_name}", end='\r')
    print(f"Market {market_count} Update {update_count}")


def run_with_rust_loading():
    market_count = 0
    update_count = 0
    
    for file in bflw.Files(paths, streaming_unique_id=1000):
        market_count += 1

        for market_books in file:
            for market_book in market_books:
                update_count += 1
                print(f"{market_book.market_id} {file.stream_unique_id}")

        print(f"Market {market_count} Update {update_count} File:{file.file_name}", end='\r')
    print(f"Market {market_count} Update {update_count}")


# run_with_py_loading()
run_with_rust_loading()