import tarfile
import bz2
from typing import Sequence
import betfair_data as bfd
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

mutable = True
paths = [
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar",
]

def run_with_py_loading():
    market_count = 0
    update_count = 0

    def load_tar(file_paths: Sequence[str]):
        for file_path in file_paths:
            with tarfile.TarFile(file_path) as archive:
                for file in archive:
                    f: bz2.BZ2File = bz2.open(archive.extractfile(file))
                    yield bfd.File(file.name, f.read(), cumulative_runner_tv=True, mutable=mutable)
        return None

    for file in load_tar(paths):
        market_count += 1

        for market in file:
            update_count += 1

        print(f"Market {market_count} Update {update_count} File:{file.file_name}", end='\r')
    print(f"Market {market_count} Update {update_count}")


def run_with_rust_loading():
    market_count = 0
    update_count = 0

    for file in bfd.Files(paths, cumulative_runner_tv=True).iter(mutable=mutable):
        market_count += 1

        for market in file:
            update_count += 1

        print(f"Markets {market_count} Updates {update_count} File:{file.file_name}", end='\r')
    print(f"Markets {market_count} Updates {update_count}")


# run_with_py_loading()
run_with_rust_loading()