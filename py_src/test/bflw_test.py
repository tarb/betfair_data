from glob import glob
from types import NoneType
from typing import List, Sequence 
import tarfile
import bz2
import math
import gzip
import os
import unittest.mock
import betfairlightweight
import betfair_data

# types
from betfair_data import bflw
import betfairlightweight.resources.bettingresources as bflw
import betfairlightweight.resources.streamingresources as bflws

#  paths to test files
# paths = glob("data/*Racing*")
paths = glob("data/stream/*")
# paths = [
#     "data/2021_12_DecRacingAUPro.tar",
#     "data/2021_10_OctRacingAUPro.tar",
#     "data/2021_11_NovRacingAUPro.tar",
# ]

bfd_source = betfair_data.Files(paths, cumulative_runner_tv=False).bflw()

def bflw_source(file_paths: Sequence[str]):
    trading = betfairlightweight.APIClient("username", "password", "appkey")
    listener = betfairlightweight.StreamListener(
        max_latency=None, update_clk=False, output_queue=None, cumulative_runner_tv=False, calculate_market_tv=False
    )

    for file_path in file_paths:
        if os.path.isfile(file_path):
            ext = os.path.splitext(file_path)[1]
            if ext == '.tar':
                with tarfile.TarFile(file_path) as archive:
                    for file in archive:
                        f = bz2.open(archive.extractfile(file))
                        with unittest.mock.patch("builtins.open", lambda f, _: f):  
                            stream = trading.streaming.create_historical_generator_stream(
                                file_path=f,
                                listener=listener,
                            )
                            yield stream.get_generator()
            elif ext == '.gz':
                with gzip.open(file_path, mode='r') as f:   
                    with unittest.mock.patch("builtins.open", lambda f, _: f):  
                        stream = trading.streaming.create_historical_generator_stream(
                            file_path=f,
                            listener=listener,
                        )
                        yield stream.get_generator()
    return None

# run the tests, panics out on fail
def run_test():
    updates = 0
    files = 0
    for (bfd_file, bflw_gen) in zip(bfd_source, bflw_source(paths)): 
        files += 1        
        row = 0

        for (bfd_mbs, bflw_mbs) in zip(bfd_file, bflw_gen()):
            row += 1
            assert len(bflw_mbs) == len(bflw_mbs), f"{bfd_file.file_name}:{row} <File> len market_books {[ m.market_id for m in bfd_mbs]} != {[ m.market_id for m in bflw_mbs]}"
            
            for (bfd_m, bflw_m) in zip(bfd_mbs, bflw_mbs):
                bflw_m: bflw.MarketBook
                updates += 1
                test_MarketBook(bfd_m, bflw_m, bfd_file.file_name, row)
        print(f"Updates Tested {bfd_file.file_name} files:{files} updates:{updates}", end='\r')
    print(f"Updates Tested files:{files} updates:{updates}")

def test_MarketBook(bfd_m: bfd.MarketBook, bflw_m: bflw.MarketBook, file_name: str, row: int):
    assert bfd_m.bet_delay == bflw_m.bet_delay, f"{file_name}:{row} <MarketBook> bet_delay {bfd_m.bet_delay} != {bflw_m.bet_delay}"
    assert bfd_m.bsp_reconciled == bflw_m.bsp_reconciled, f"{file_name}:{row} <MarketBook> bsp_reconciled {bfd_m.bsp_reconciled} != {bflw_m.bsp_reconciled}"
    assert bfd_m.complete == bflw_m.complete, f"{file_name}:{row} <MarketBook> complete {bfd_m.complete} != {bflw_m.complete}"
    assert bfd_m.cross_matching == bflw_m.cross_matching, f"{file_name}:{row} <MarketBook> cross_matching {bfd_m.cross_matching} != {bflw_m.cross_matching}"
    assert bfd_m.inplay == bflw_m.inplay, f"{file_name}:{row} <MarketBook> inplay {bfd_m.inplay} != {bflw_m.inplay}"
    assert bfd_m.is_market_data_delayed == bflw_m.is_market_data_delayed, f"{file_name}:{row} <MarketBook> is_market_data_delayed {bfd_m.is_market_data_delayed} != {bflw_m.is_market_data_delayed}"
    assert bfd_m.last_match_time == bflw_m.last_match_time, f"{file_name}:{row} <MarketBook> last_match_time {bfd_m.last_match_time} != {bflw_m.last_match_time}"
    assert bfd_m.market_id == bflw_m.market_id, f"{file_name}:{row} <MarketBook> market_id {bfd_m.market_id} != {bflw_m.market_id}"
    assert bfd_m.number_of_active_runners == bflw_m.number_of_active_runners, f"{file_name}:{row} <MarketBook> number_of_active_runners {bfd_m.number_of_active_runners} != {bflw_m.number_of_active_runners}"
    assert bfd_m.number_of_runners == bflw_m.number_of_runners, f"{file_name}:{row} <MarketBook> number_of_runners {bfd_m.number_of_runners} != {bflw_m.number_of_runners}"
    assert bfd_m.number_of_winners == bflw_m.number_of_winners, f"{file_name}:{row} <MarketBook> number_of_winners {bfd_m.number_of_winners} != {bflw_m.number_of_winners}"
    assert bfd_m.publish_time == bflw_m.publish_time, f"{file_name}:{row} <MarketBook> publish_time {bfd_m.publish_time} != {bflw_m.publish_time}"
    assert bfd_m.publish_time_epoch == bflw_m.publish_time_epoch, f"{file_name}:{row} <MarketBook> publish_time_epoch {bfd_m.publish_time_epoch} != {bflw_m.publish_time_epoch}"
    assert bfd_m.runners_voidable == bflw_m.runners_voidable, f"{file_name}:{row} <MarketBook> runners_voidable {bfd_m.runners_voidable} != {bflw_m.runners_voidable}"
    assert bfd_m.status == bflw_m.status, f"{file_name}:{row} <MarketBook> status {bfd_m.status} != {bflw_m.status}"
    assert bfd_m.total_available == bflw_m.total_available, f"{file_name}:{row} <MarketBook> total_available {bfd_m.total_available} != {bflw_m.total_available}"
    assert bfd_m.total_matched == bflw_m.total_matched, f"{file_name}:{row} <MarketBook> total_matched {bfd_m.total_matched} != {bflw_m.total_matched}"
    assert bfd_m.version == bflw_m.version, f"{file_name}:{row} <MarketBook> version {bfd_m.version} != {bflw_m.version}"
    test_MarketDefinition(bfd_m.market_definition, bflw_m.market_definition, file_name, row)
    for bfd_rb, bflw_rb in zip(bfd_m.runners, bflw_m.runners):
        test_RunnerBook(bfd_rb, bflw_rb, file_name, row) 

def test_MarketDefinition(bfd_md: bfd.MarketDefinition, bflw_md: bflws.MarketDefinition, file_name: str, row: int):
    assert bfd_md.bet_delay == bflw_md.bet_delay, f"{file_name}:{row} <MarketDefinition> bet_delay {bfd_md.bet_delay} != {bflw_md.bet_delay}"
    assert bfd_md.betting_type == bflw_md.betting_type, f"{file_name}:{row} <MarketDefinition> betting_type {bfd_md.betting_type} != {bflw_md.betting_type}"
    assert bfd_md.bsp_market == bflw_md.bsp_market, f"{file_name}:{row} <MarketDefinition> bsp_market {bfd_md.bsp_market} != {bflw_md.bsp_market}"
    assert bfd_md.bsp_reconciled == bflw_md.bsp_reconciled, f"{file_name}:{row} <MarketDefinition> bsp_reconciled {bfd_md.bsp_reconciled} != {bflw_md.bsp_reconciled}"
    assert bfd_md.complete == bflw_md.complete, f"{file_name}:{row} <MarketDefinition> complete {bfd_md.complete} != {bflw_md.complete}"
    assert bfd_md.country_code == bflw_md.country_code, f"{file_name}:{row} <MarketDefinition> country_code {bfd_md.country_code} != {bflw_md.country_code}"
    assert bfd_md.cross_matching == bflw_md.cross_matching, f"{file_name}:{row} <MarketDefinition> cross_matching {bfd_md.cross_matching} != {bflw_md.cross_matching}"
    assert bfd_md.discount_allowed == bflw_md.discount_allowed, f"{file_name}:{row} <MarketDefinition> discount_allowed {bfd_md.discount_allowed} != {bflw_md.discount_allowed}"
    assert bfd_md.event_id == bflw_md.event_id, f"{file_name}:{row} <MarketDefinition> event_id {bfd_md.event_id} != {bflw_md.event_id}"
    assert bfd_md.event_type_id == bflw_md.event_type_id, f"{file_name}:{row} <MarketDefinition> event_type_id {bfd_md.event_type_id} != {bflw_md.event_type_id}"
    assert bfd_md.in_play == bflw_md.in_play, f"{file_name}:{row} <MarketDefinition> in_play {bfd_md.in_play} != {bflw_md.in_play}"
    assert bfd_md.market_base_rate == bflw_md.market_base_rate, f"{file_name}:{row} <MarketDefinition> market_base_rate {bfd_md.market_base_rate} != {bflw_md.market_base_rate}"
    assert bfd_md.market_time == bflw_md.market_time, f"{file_name}:{row} <MarketDefinition> market_time {bfd_md.market_time} != {bflw_md.market_time}"
    assert bfd_md.market_type == bflw_md.market_type, f"{file_name}:{row} <MarketDefinition> market_type {bfd_md.market_type} != {bflw_md.market_type}"
    assert bfd_md.number_of_active_runners == bflw_md.number_of_active_runners, f"{file_name}:{row} <MarketDefinition> number_of_active_runners {bfd_md.number_of_active_runners} != {bflw_md.number_of_active_runners}"
    assert bfd_md.number_of_winners == bflw_md.number_of_winners, f"{file_name}:{row} <MarketDefinition> number_of_winners {bfd_md.number_of_winners} != {bflw_md.number_of_winners}"
    assert bfd_md.open_date == bflw_md.open_date, f"{file_name}:{row} <MarketDefinition> open_date {bfd_md.open_date} != {bflw_md.open_date}"
    assert bfd_md.persistence_enabled == bflw_md.persistence_enabled, f"{file_name}:{row} <MarketDefinition> persistence_enabled {bfd_md.persistence_enabled} != {bflw_md.persistence_enabled}"
    assert bfd_md.regulators == bflw_md.regulators, f"{file_name}:{row} <MarketDefinition> regulators {bfd_md.regulators} != {bflw_md.regulators}"
    assert bfd_md.runners_voidable == bflw_md.runners_voidable, f"{file_name}:{row} <MarketDefinition> runners_voidable {bfd_md.runners_voidable} != {bflw_md.runners_voidable}"
    assert bfd_md.settled_time == bflw_md.settled_time, f"{file_name}:{row} <MarketDefinition> settled_time {bfd_md.settled_time} != {bflw_md.settled_time}"
    assert bfd_md.status == bflw_md.status, f"{file_name}:{row} <MarketDefinition> status {bfd_md.status} != {bflw_md.status}"
    assert bfd_md.suspend_time == bflw_md.suspend_time, f"{file_name}:{row} <MarketDefinition> suspend_time {bfd_md.suspend_time} != {bflw_md.suspend_time}"
    assert bfd_md.timezone == bflw_md.timezone, f"{file_name}:{row} <MarketDefinition> timezone {bfd_md.timezone} != {bflw_md.timezone}"
    assert bfd_md.turn_in_play_enabled == bflw_md.turn_in_play_enabled, f"{file_name}:{row} <MarketDefinition> turn_in_play_enabled {bfd_md.turn_in_play_enabled} != {bflw_md.turn_in_play_enabled}"
    assert bfd_md.venue == bflw_md.venue, f"{file_name}:{row} <MarketDefinition> venue {bfd_md.venue} != {bflw_md.venue}"
    assert bfd_md.version == bflw_md.version, f"{file_name}:{row} <MarketDefinition> version {bfd_md.version} != {bflw_md.version}"
    for bfd_rd, bflw_rd in zip(bfd_md.runners, bflw_md.runners):
        test_MarketDefinitionRunner(bfd_rd, bflw_rd, file_name, row) 

def test_MarketDefinitionRunner(bfd_rd: bfd.MarketDefinitionRunner, bflw_rd: bflws.MarketDefinitionRunner, file_name: str, row: int):
    assert bfd_rd.selection_id == bflw_rd.selection_id, f"{file_name}:{row} <MarketDefinitionRunner> selection_id {bfd_rd.selection_id} != {bflw_rd.selection_id}"
    assert test_float(bfd_rd.adjustment_factor, bflw_rd.adjustment_factor), f"{file_name}:{row} <MarketDefinitionRunner> adjustment_factor {bfd_rd.adjustment_factor} != {bflw_rd.adjustment_factor}"
    assert bfd_rd.removal_date == bflw_rd.removal_date, f"{file_name}:{row} <MarketDefinitionRunner> removal_date {bfd_rd.removal_date} != {bflw_rd.removal_date}"
    assert bfd_rd.sort_priority == bflw_rd.sort_priority, f"{file_name}:{row} <MarketDefinitionRunner> sort_priority {bfd_rd.sort_priority} != {bflw_rd.sort_priority}"
    assert bfd_rd.status == bflw_rd.status, f"{file_name}:{row} <MarketDefinitionRunner> status {bfd_rd.status} != {bflw_rd.status}"
    assert test_float(bfd_rd.bsp, bflw_rd.bsp), f"{file_name}:{row} <MarketDefinitionRunner> bsp {bfd_rd.bsp} != {bflw_rd.bsp}"
    assert test_float(bfd_rd.handicap, bflw_rd.handicap), f"{file_name}:{row} <MarketDefinitionRunner> handicap {bfd_rd.handicap} != {bflw_rd.handicap}"
    assert bfd_rd.name == bflw_rd.name, f"{file_name}:{row} <MarketDefinitionRunner> name {bfd_rd.name} != {bflw_rd.name}"
    
def test_RunnerBook(bfd_rb: bfd.RunnerBook, bflw_rb: bflw.RunnerBook, file_name: str, row: int):
    assert bfd_rb.selection_id == bflw_rb.selection_id, f"{file_name}:{row} <RunnerBook> selection_id {bfd_rb.selection_id} != {bflw_rb.selection_id}"
    assert bfd_rb.adjustment_factor == bflw_rb.adjustment_factor, f"{file_name}:{row} <RunnerBook> adjustment_factor {bfd_rb.adjustment_factor} != {bflw_rb.adjustment_factor}"
    assert bfd_rb.handicap == bflw_rb.handicap, f"{file_name}:{row} <RunnerBook> handicap {bfd_rb.handicap} != {bflw_rb.handicap}"
    assert bfd_rb.last_price_traded == bflw_rb.last_price_traded, f"{file_name}:{row} <RunnerBook> last_price_traded {bfd_rb.last_price_traded} != {bflw_rb.last_price_traded}"
    assert bfd_rb.removal_date == bflw_rb.removal_date, f"{file_name}:{row} <RunnerBook> removal_date {bfd_rb.removal_date} != {bflw_rb.removal_date}"
    assert bfd_rb.status == bflw_rb.status, f"{file_name}:{row} <RunnerBook> status {bfd_rb.status} != {bflw_rb.status}"
    assert bfd_rb.total_matched == bflw_rb.total_matched, f"{file_name}:{row} <RunnerBook> total_matched {bfd_rb.total_matched} != {bflw_rb.total_matched}"
    assert bfd_rb.matches == bflw_rb.matches, f"{file_name}:{row} <RunnerBook> matches {bfd_rb.matches} != {bflw_rb.matches}"
    assert bfd_rb.orders == bflw_rb.orders, f"{file_name}:{row} <RunnerBook> orders {bfd_rb.orders} != {bflw_rb.orders}"
    test_RunnerBookEX(bfd_rb.ex, bflw_rb.ex, bfd_rb.selection_id, file_name, row)
    test_RunnerBookSP(bfd_rb.sp, bflw_rb.sp, bfd_rb.selection_id, file_name, row)
    
def test_RunnerBookEX(bfd_ex: betfair_data.RunnerBookEX, bflw_ex: bflw.RunnerBookEX, sid: int, file_name: str, row: int):
    test_ListPriceSize(bfd_ex.available_to_back, bflw_ex.available_to_back, f"{file_name}:{row} <RunnerBookEX[{sid}].available_to_back>")
    test_ListPriceSize(bfd_ex.available_to_lay, bflw_ex.available_to_lay, f"{file_name}:{row} <RunnerBookEX[{sid}].available_to_lay>")
    test_ListPriceSize(bfd_ex.traded_volume, bflw_ex.traded_volume, f"{file_name}:{row} <RunnerBookEX[{sid}].traded_volume>")

def test_RunnerBookSP(bfd_sp: betfair_data.RunnerBookSP, bflw_sp: bflw.RunnerBookSP, sid: int, file_name: str, row: int):
    assert test_float(bfd_sp.far_price, bflw_sp.far_price), f"{file_name}:{row} <RunnerBookSP> far_price {bfd_sp.far_price} != {bflw_sp.far_price}"
    assert test_float(bfd_sp.near_price, bflw_sp.near_price), f"{file_name}:{row} <RunnerBookSP> near_price {bfd_sp.near_price} != {bflw_sp.near_price}"
    assert test_float(bfd_sp.actual_sp, bflw_sp.actual_sp), f"{file_name}:{row} <RunnerBookSP> actual_sp {bfd_sp.actual_sp} != {bflw_sp.actual_sp}"
    test_ListPriceSize(bfd_sp.back_stake_taken, bflw_sp.back_stake_taken, f"{file_name}:{row} <RunnerBookSP[{sid}].back_stake_taken>")
    test_ListPriceSize(bfd_sp.lay_liability_taken, bflw_sp.lay_liability_taken, f"{file_name}:{row} <RunnerBookSP[{sid}].lay_liability_taken>")

def test_ListPriceSize(bfd_ps: List[betfair_data.PriceSize], bflw_ps: List[bflw.PriceSize], context: str):
    assert len(bfd_ps) == len(bflw_ps), f"{context} <List[PriceSize]> different lengths {print_ladder(bfd_ps)} != {print_ladder(bflw_ps)}"
    for i, (bfd, bflw) in enumerate(zip(bfd_ps, bflw_ps)):
        assert test_float(bfd.price, bflw.price), f"{context} <List[PriceSize][{i}]> PriceError {print_ladder(bfd_ps)} != {print_ladder(bflw_ps)}"
        assert test_float(bfd.size, bflw.size), f"{context} <List[PriceSize][{i}]> SizeError {print_ladder(bfd_ps)} != {print_ladder(bflw_ps)}"
    
def test_float(f1: float|str|NoneType, f2: float|str|int|NoneType) -> bool:
    if isinstance(f2, int):
        f2 = float(f2)
    if isinstance(f1, float):   
        return ((f1 == f2) or math.isclose(f1, f2, abs_tol=0.001) or (math.isnan(f1) and math.isnan(f2)) or (math.isinf(f1) and math.isinf(f2)))

    return f1 == f2

def print_ladder(ladder: List[bflw.PriceSize]) -> str:
    return ' '.join(list(f"[{ps.price}, {ps.size}]" for ps in ladder))


run_test()
