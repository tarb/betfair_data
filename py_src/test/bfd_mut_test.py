import betfair_data as bfd
import logging

logging.basicConfig(level=logging.WARN, format='%(levelname)s %(name)s %(message)s')


paths = [
    "data/2021_10_OctRacingAUPro.tar",
    "data/2021_11_NovRacingAUPro.tar",
    "data/2021_12_DecRacingAUPro.tar",
]

mut = bfd.TarBz2(paths, cumulative_runner_tv=True).iter(mutable=True)
imm = bfd.TarBz2(paths, cumulative_runner_tv=True).iter(mutable=False)

market_count = 0
update_count = 0

for (mut_file, imm_file) in zip(mut, imm):
    market_count += 1

    for (m, i) in zip(mut_file, imm_file):
        assert m.market_id == i.market_id, f"<Market> selection_id {m.market_id} != {i.market_id}"



        for (mr, ir) in zip(m.runners, i.runners):
            assert mr.selection_id == ir.selection_id, f"<Runner> selection_id {mr.selection_id} != {ir.selection_id}"




        update_count += 1
        print(f"Market {market_count} Update {update_count}", end='\r')

