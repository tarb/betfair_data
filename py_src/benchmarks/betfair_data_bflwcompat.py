import tarfile
import bz2
import betfair_data
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')

paths = [ 
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar"
]

# def load_tar(file_paths: Sequence[str]):
#     for file_path in file_paths:
#         with tarfile.TarFile(file_path) as archive:
#             for file in archive:
#                 f: bz2.BZ2File = bz2.open(archive.extractfile(file))
#                 name = file.name
#                 bytes = f.read()

#                 yield (name, bytes)
#     return None

market_count = 0
update_count = 0

for file in betfair_data.TarBz2(paths, cumulative_runner_tv=True).bflw():
    market_count += 1

    for market_books in file:
        for market_book in market_books:
            update_count += 1

    print(f"Market {market_count} Update {update_count}", end='\r')
print(f"Market {market_count} Update {update_count}")
