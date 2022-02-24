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
from betfair_data import bflw as bfd
import betfairlightweight.resources.bettingresources as bflw
import betfairlightweight.resources.streamingresources as bflws

#  paths to test files
paths = glob("data/stream/*.gz")
# paths = [
#     "data/2021_12_DecRacingAUPro.tar",
#     "data/2021_10_OctRacingAUPro.tar",
#     "data/2021_11_NovRacingAUPro.tar",
# ]
# find way to garentee order - this is gross
bfd_source = betfair_data.Files(paths, cumulative_runner_tv=True).bflw()

def bflw_source(file_paths: Sequence[str]):
    trading = betfairlightweight.APIClient("username", "password", "appkey")
    listener = betfairlightweight.StreamListener(
        max_latency=None, update_clk=False, output_queue=None, cumulative_runner_tv=True, calculate_market_tv=True
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
    for (bfd_iter, bflw_gen) in zip(bfd_source, bflw_source(paths)): 
        files += 1       
        for (bfd_mbs, bflw_mbs) in zip(bfd_iter, bflw_gen()):
            for (bfd_m, bflw_m) in zip(bfd_mbs, bflw_mbs):
                bflw_m: bflw.MarketBook
                updates += 1
                print(f"Updates Tested {bfd_m.market_id} files:{files} updates:{updates}", end='\r')
                test_MarketBook(bfd_m, bflw_m)
    print(f"Updates Tested files:{files} updates:{updates}")

def test_MarketBook(bfd_m: bfd.MarketBook, bflw_m: bflw.MarketBook):
    assert bfd_m.bet_delay == bflw_m.bet_delay, f"<MarketBook> bet_delay {bfd_m.bet_delay} != {bflw_m.bet_delay}"
    assert bfd_m.bsp_reconciled == bflw_m.bsp_reconciled, f"<MarketBook> bsp_reconciled {bfd_m.bsp_reconciled} != {bflw_m.bsp_reconciled}"
    assert bfd_m.complete == bflw_m.complete, f"<MarketBook> complete {bfd_m.complete} != {bflw_m.complete}"
    assert bfd_m.cross_matching == bflw_m.cross_matching, f"<MarketBook> cross_matching {bfd_m.cross_matching} != {bflw_m.cross_matching}"
    assert bfd_m.inplay == bflw_m.inplay, f"<MarketBook> inplay {bfd_m.inplay} != {bflw_m.inplay}"
    assert bfd_m.is_market_data_delayed == bflw_m.is_market_data_delayed, f"<MarketBook> is_market_data_delayed {bfd_m.is_market_data_delayed} != {bflw_m.is_market_data_delayed}"
    assert bfd_m.last_match_time == bflw_m.last_match_time, f"<MarketBook> last_match_time {bfd_m.last_match_time} != {bflw_m.last_match_time}"
    assert bfd_m.market_id == bflw_m.market_id, f"<MarketBook> market_id {bfd_m.market_id} != {bflw_m.market_id}"
    assert bfd_m.number_of_active_runners == bflw_m.number_of_active_runners, f"<MarketBook> number_of_active_runners {bfd_m.number_of_active_runners} != {bflw_m.number_of_active_runners}"
    assert bfd_m.number_of_runners == bflw_m.number_of_runners, f"<MarketBook> number_of_runners {bfd_m.number_of_runners} != {bflw_m.number_of_runners}"
    assert bfd_m.number_of_winners == bflw_m.number_of_winners, f"<MarketBook> number_of_winners {bfd_m.number_of_winners} != {bflw_m.number_of_winners}"
    assert bfd_m.publish_time == bflw_m.publish_time, f"<MarketBook> publish_time {bfd_m.publish_time} != {bflw_m.publish_time}"
    assert bfd_m.publish_time_epoch == bflw_m.publish_time_epoch, f"<MarketBook> publish_time_epoch {bfd_m.publish_time_epoch} != {bflw_m.publish_time_epoch}"
    assert bfd_m.runners_voidable == bflw_m.runners_voidable, f"<MarketBook> runners_voidable {bfd_m.runners_voidable} != {bflw_m.runners_voidable}"
    assert bfd_m.status == bflw_m.status, f"<MarketBook> status {bfd_m.status} != {bflw_m.status}"
    assert bfd_m.total_available == bflw_m.total_available, f"<MarketBook> total_available {bfd_m.total_available} != {bflw_m.total_available}"
    assert bfd_m.total_matched == bflw_m.total_matched, f"<MarketBook> total_matched {bfd_m.total_matched} != {bflw_m.total_matched}"
    assert bfd_m.version == bflw_m.version, f"<MarketBook> version {bfd_m.version} != {bflw_m.version}"
    test_MarketDefinition(bfd_m.market_definition, bflw_m.market_definition)
    for bfd_rb, bflw_rb in zip(bfd_m.runners, bflw_m.runners):
        test_RunnerBook(bfd_rb, bflw_rb) 

def test_MarketDefinition(bfd_md: bfd.MarketDefinition, bflw_md: bflws.MarketDefinition):
    assert bfd_md.bet_delay == bflw_md.bet_delay, f"<MarketDefinition> bet_delay {bfd_md.bet_delay} != {bflw_md.bet_delay}"
    assert bfd_md.betting_type == bflw_md.betting_type, f"<MarketDefinition> betting_type {bfd_md.betting_type} != {bflw_md.betting_type}"
    assert bfd_md.bsp_market == bflw_md.bsp_market, f"<MarketDefinition> bsp_market {bfd_md.bsp_market} != {bflw_md.bsp_market}"
    assert bfd_md.bsp_reconciled == bflw_md.bsp_reconciled, f"<MarketDefinition> bsp_reconciled {bfd_md.bsp_reconciled} != {bflw_md.bsp_reconciled}"
    assert bfd_md.complete == bflw_md.complete, f"<MarketDefinition> complete {bfd_md.complete} != {bflw_md.complete}"
    assert bfd_md.country_code == bflw_md.country_code, f"<MarketDefinition> country_code {bfd_md.country_code} != {bflw_md.country_code}"
    assert bfd_md.cross_matching == bflw_md.cross_matching, f"<MarketDefinition> cross_matching {bfd_md.cross_matching} != {bflw_md.cross_matching}"
    assert bfd_md.discount_allowed == bflw_md.discount_allowed, f"<MarketDefinition> discount_allowed {bfd_md.discount_allowed} != {bflw_md.discount_allowed}"
    assert bfd_md.event_id == bflw_md.event_id, f"<MarketDefinition> event_id {bfd_md.event_id} != {bflw_md.event_id}"
    assert bfd_md.event_type_id == bflw_md.event_type_id, f"<MarketDefinition> event_type_id {bfd_md.event_type_id} != {bflw_md.event_type_id}"
    assert bfd_md.in_play == bflw_md.in_play, f"<MarketDefinition> in_play {bfd_md.in_play} != {bflw_md.in_play}"
    assert bfd_md.market_base_rate == bflw_md.market_base_rate, f"<MarketDefinition> market_base_rate {bfd_md.market_base_rate} != {bflw_md.market_base_rate}"
    assert bfd_md.market_time == bflw_md.market_time, f"<MarketDefinition> market_time {bfd_md.market_time} != {bflw_md.market_time}"
    assert bfd_md.market_type == bflw_md.market_type, f"<MarketDefinition> market_type {bfd_md.market_type} != {bflw_md.market_type}"
    assert bfd_md.number_of_active_runners == bflw_md.number_of_active_runners, f"<MarketDefinition> number_of_active_runners {bfd_md.number_of_active_runners} != {bflw_md.number_of_active_runners}"
    assert bfd_md.number_of_winners == bflw_md.number_of_winners, f"<MarketDefinition> number_of_winners {bfd_md.number_of_winners} != {bflw_md.number_of_winners}"
    assert bfd_md.open_date == bflw_md.open_date, f"<MarketDefinition> open_date {bfd_md.open_date} != {bflw_md.open_date}"
    assert bfd_md.persistence_enabled == bflw_md.persistence_enabled, f"<MarketDefinition> persistence_enabled {bfd_md.persistence_enabled} != {bflw_md.persistence_enabled}"
    assert bfd_md.regulators == bflw_md.regulators, f"<MarketDefinition> regulators {bfd_md.regulators} != {bflw_md.regulators}"
    assert bfd_md.runners_voidable == bflw_md.runners_voidable, f"<MarketDefinition> runners_voidable {bfd_md.runners_voidable} != {bflw_md.runners_voidable}"
    assert bfd_md.settled_time == bflw_md.settled_time, f"<MarketDefinition> settled_time {bfd_md.settled_time} != {bflw_md.settled_time}"
    assert bfd_md.status == bflw_md.status, f"<MarketDefinition> status {bfd_md.status} != {bflw_md.status}"
    assert bfd_md.suspend_time == bflw_md.suspend_time, f"<MarketDefinition> suspend_time {bfd_md.suspend_time} != {bflw_md.suspend_time}"
    assert bfd_md.timezone == bflw_md.timezone, f"<MarketDefinition> timezone {bfd_md.timezone} != {bflw_md.timezone}"
    assert bfd_md.turn_in_play_enabled == bflw_md.turn_in_play_enabled, f"<MarketDefinition> turn_in_play_enabled {bfd_md.turn_in_play_enabled} != {bflw_md.turn_in_play_enabled}"
    assert bfd_md.venue == bflw_md.venue, f"<MarketDefinition> venue {bfd_md.venue} != {bflw_md.venue}"
    assert bfd_md.version == bflw_md.version, f"<MarketDefinition> version {bfd_md.version} != {bflw_md.version}"
    for bfd_rd, bflw_rd in zip(bfd_md.runners, bflw_md.runners):
        test_MarketDefinitionRunner(bfd_rd, bflw_rd) 

def test_MarketDefinitionRunner(bfd_rd: bfd.MarketDefinitionRunner, bflw_rd: bflws.MarketDefinitionRunner):
    assert bfd_rd.selection_id == bflw_rd.selection_id, f"<MarketDefinitionRunner> selection_id {bfd_rd.selection_id} != {bflw_rd.selection_id}"
    assert test_float(bfd_rd.adjustment_factor, bflw_rd.adjustment_factor), f"<MarketDefinitionRunner> adjustment_factor {bfd_rd.adjustment_factor} != {bflw_rd.adjustment_factor}"
    assert bfd_rd.removal_date == bflw_rd.removal_date, f"<MarketDefinitionRunner> removal_date {bfd_rd.removal_date} != {bflw_rd.removal_date}"
    assert bfd_rd.sort_priority == bflw_rd.sort_priority, f"<MarketDefinitionRunner> sort_priority {bfd_rd.sort_priority} != {bflw_rd.sort_priority}"
    assert bfd_rd.status == bflw_rd.status, f"<MarketDefinitionRunner> status {bfd_rd.status} != {bflw_rd.status}"
    assert test_float(bfd_rd.bsp, bflw_rd.bsp), f"<MarketDefinitionRunner> bsp {bfd_rd.bsp} != {bflw_rd.bsp}"
    assert test_float(bfd_rd.handicap, bflw_rd.handicap), f"<MarketDefinitionRunner> handicap {bfd_rd.handicap} != {bflw_rd.handicap}"
    assert bfd_rd.name == bflw_rd.name, f"<MarketDefinitionRunner> name {bfd_rd.name} != {bflw_rd.name}"
    
def test_RunnerBook(bfd_rb: bfd.RunnerBook, bflw_rb: bflw.RunnerBook):
    assert bfd_rb.selection_id == bflw_rb.selection_id, f"<RunnerBook> selection_id {bfd_rb.selection_id} != {bflw_rb.selection_id}"
    assert bfd_rb.adjustment_factor == bflw_rb.adjustment_factor, f"<RunnerBook> adjustment_factor {bfd_rb.adjustment_factor} != {bflw_rb.adjustment_factor}"
    assert bfd_rb.handicap == bflw_rb.handicap, f"<RunnerBook> handicap {bfd_rb.handicap} != {bflw_rb.handicap}"
    assert bfd_rb.last_price_traded == bflw_rb.last_price_traded, f"<RunnerBook> last_price_traded {bfd_rb.last_price_traded} != {bflw_rb.last_price_traded}"
    assert bfd_rb.removal_date == bflw_rb.removal_date, f"<RunnerBook> removal_date {bfd_rb.removal_date} != {bflw_rb.removal_date}"
    assert bfd_rb.status == bflw_rb.status, f"<RunnerBook> status {bfd_rb.status} != {bflw_rb.status}"
    assert bfd_rb.total_matched == bflw_rb.total_matched, f"<RunnerBook> total_matched {bfd_rb.total_matched} != {bflw_rb.total_matched}"
    assert bfd_rb.matches == bflw_rb.matches, f"<RunnerBook> matches {bfd_rb.matches} != {bflw_rb.matches}"
    assert bfd_rb.orders == bflw_rb.orders, f"<RunnerBook> orders {bfd_rb.orders} != {bflw_rb.orders}"
    test_RunnerBookEX(bfd_rb.ex, bflw_rb.ex, bfd_rb.selection_id)
    test_RunnerBookSP(bfd_rb.sp, bflw_rb.sp, bfd_rb.selection_id)
    
def test_RunnerBookEX(bfd_ex: betfair_data.RunnerBookEX, bflw_ex: bflw.RunnerBookEX, sid: int):
    test_ListPriceSize(bfd_ex.available_to_back, bflw_ex.available_to_back, f"<RunnerBookEX[{sid}].available_to_back>")
    test_ListPriceSize(bfd_ex.available_to_lay, bflw_ex.available_to_lay, f"<RunnerBookEX[{sid}].available_to_lay>")
    test_ListPriceSize(bfd_ex.traded_volume, bflw_ex.traded_volume, f"<RunnerBookEX[{sid}].traded_volume>")

def test_RunnerBookSP(bfd_sp: betfair_data.RunnerBookSP, bflw_sp: bflw.RunnerBookSP, sid: int):
    assert test_float(bfd_sp.far_price, bflw_sp.far_price), f"<RunnerBookSP> far_price {bfd_sp.far_price} != {bflw_sp.far_price}"
    assert test_float(bfd_sp.near_price, bflw_sp.near_price), f"<RunnerBookSP> near_price {bfd_sp.near_price} != {bflw_sp.near_price}"
    assert test_float(bfd_sp.actual_sp, bflw_sp.actual_sp), f"<RunnerBookSP> actual_sp {bfd_sp.actual_sp} != {bflw_sp.actual_sp}"
    test_ListPriceSize(bfd_sp.back_stake_taken, bflw_sp.back_stake_taken, f"<RunnerBookSP[{sid}].back_stake_taken>")
    test_ListPriceSize(bfd_sp.lay_liability_taken, bflw_sp.lay_liability_taken, f"<RunnerBookSP[{sid}].lay_liability_taken>")

def test_ListPriceSize(bfd_ps: List[betfair_data.PriceSize], bflw_ps: List[bflw.PriceSize], context: str):
    assert len(bfd_ps) == len(bflw_ps), f"{context} <List[PriceSize]> different lengths {print_ladder(bfd_ps)} != {print_ladder(bflw_ps)}"
    for i, (bfd, bflw) in enumerate(zip(bfd_ps, bflw_ps)):
        assert test_float(bfd.price, bflw.price), f"{context} <List[PriceSize][{i}]> Price  {bfd.price} != {bflw.price}"
        assert test_float(bfd.size, bflw.size), f"{context} <List[PriceSize][{i}]> Size  {bfd.size} != {bflw.size}"
    
def test_float(f1: float|str|NoneType, f2: float|str|int|NoneType) -> bool:
    if isinstance(f2, int):
        f2 = float(f2)
    if isinstance(f1, float):   
        return ((f1 == f2) or math.isclose(f1, f2, abs_tol=0.001) or (math.isnan(f1) and math.isnan(f2)) or (math.isinf(f1) and math.isinf(f2)))

    return f1 == f2

def print_ladder(ladder: List[bflw.PriceSize]) -> str:
    return ' '.join(list(f"[{ps.price}, {ps.size}]" for ps in ladder))


run_test()
