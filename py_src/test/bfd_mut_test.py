import math
from types import NoneType
from typing import List
import betfair_data as bfd
import logging
import glob

logging.basicConfig(level=logging.INFO, format='%(levelname)s %(name)s %(message)s')

# paths = [
#     "data/2021_10_OctRacingAUPro.tar",
#     "data/2021_11_NovRacingAUPro.tar",
#     "data/2021_12_DecRacingAUPro.tar",
# ]

paths = glob.glob("data/*OtherSports*")

mut = bfd.Files(paths, cumulative_runner_tv=True).iter(mutable=True)
imm = bfd.Files(paths, cumulative_runner_tv=True).iter(mutable=False)

def start_test():
    market_count = 0
    update_count = 0
    
    for (mut_file, imm_file) in zip(mut, imm):
        market_count += 1 

        for (m, i) in zip(mut_file, imm_file):
            assert m.market_id == i.market_id, f"<Market> market_id {m.market_id} != {i.market_id}"
            assert m.publish_time == i.publish_time, f"<Market> publish_time {m.publish_time} != {i.publish_time}"
            assert m.clk == i.clk, f"<Market> clk {m.clk} != {i.clk}"
            assert m.total_matched == i.total_matched, f"<Market> total_matched {m.total_matched} != {i.total_matched}"
            assert m.bet_delay == i.bet_delay, f"<Market> bet_delay {m.bet_delay} != {i.bet_delay}"
            assert m.bsp_market == i.bsp_market, f"<Market> bsp_market {m.bsp_market} != {i.bsp_market}"
            assert m.bsp_reconciled == i.bsp_reconciled, f"<Market> bsp_reconciled {m.bsp_reconciled} != {i.bsp_reconciled}"
            assert m.complete == i.complete, f"<Market> complete {m.complete} != {i.complete}"
            assert m.cross_matching == i.cross_matching, f"<Market> cross_matching {m.cross_matching} != {i.cross_matching}"
            assert m.discount_allowed == i.discount_allowed, f"<Market> discount_allowed {m.discount_allowed} != {i.discount_allowed}"
            assert m.each_way_divisor == i.each_way_divisor, f"<Market> each_way_divisor {m.each_way_divisor} != {i.each_way_divisor}"
            assert m.event_id == i.event_id, f"<Market> event_id {m.event_id} != {i.event_id}"
            assert m.event_name == i.event_name, f"<Market> event_name {m.event_name} != {i.event_name}"
            assert m.event_type_id == i.event_type_id, f"<Market> event_type_id {m.event_type_id} != {i.event_type_id}"
            assert m.in_play == i.in_play, f"<Market> in_play {m.in_play} != {i.in_play}"
            assert m.market_base_rate == i.market_base_rate, f"<Market> market_base_rate {m.market_base_rate} != {i.market_base_rate}"
            assert m.market_type == i.market_type, f"<Market> market_type {m.market_type} != {i.market_type}"
            assert m.race_type == i.race_type, f"<Market> race_type {m.race_type} != {i.race_type}"
            assert m.market_name == i.market_name, f"<Market> market_name {m.market_name} != {i.market_name}"
            assert m.number_of_active_runners == i.number_of_active_runners, f"<Market> number_of_active_runners {m.number_of_active_runners} != {i.number_of_active_runners}"
            assert m.number_of_winners == i.number_of_winners, f"<Market> number_of_winners {m.number_of_winners} != {i.number_of_winners}"
            assert m.persistence_enabled == i.persistence_enabled, f"<Market> persistence_enabled {m.persistence_enabled} != {i.persistence_enabled}"
            assert m.runners_voidable == i.runners_voidable, f"<Market> runners_voidable {m.runners_voidable} != {i.runners_voidable}"
            assert m.timezone == i.timezone, f"<Market> timezone {m.timezone} != {i.timezone}"
            assert m.turn_in_play_enabled == i.turn_in_play_enabled, f"<Market> turn_in_play_enabled {m.turn_in_play_enabled} != {i.turn_in_play_enabled}"
            assert m.venue == i.venue, f"<Market> venue {m.venue} != {i.venue}"
            assert m.version == i.version, f"<Market> version {m.version} != {i.version}"
            assert m.status == i.status, f"<Market> status {m.status} != {i.status}"
            assert m.betting_type == i.betting_type, f"<Market> betting_type {m.betting_type} != {i.betting_type}"
            assert m.market_time == i.market_time, f"<Market> market_time {m.market_time} != {i.market_time}"
            assert m.open_date == i.open_date, f"<Market> open_date {m.open_date} != {i.open_date}"
            assert m.suspend_time == i.suspend_time, f"<Market> suspend_time {m.suspend_time} != {i.suspend_time}"
            assert m.settled_time == i.settled_time, f"<Market> settled_time {m.settled_time} != {i.settled_time}"
            assert m.country_code == i.country_code, f"<Market> country_code {m.country_code} != {i.country_code}"
            for (mr, ir) in zip(m.regulators, i.regulators):
                assert mr == ir, f"<Regulator> {mr} != {ir}"
            assert len(m.runners) == len(i.runners), f"<Runner> len {len(m.runners)} != {len(i.runners)}" 
            for (mr, ir) in zip(m.runners, i.runners):
                assert mr.selection_id == ir.selection_id, f"<Runner> selection_id {mr.selection_id} != {ir.selection_id}"
                assert mr.adjustment_factor == ir.adjustment_factor, f"<Runner> adjustment_factor {mr.adjustment_factor} != {ir.adjustment_factor}"
                assert mr.handicap == ir.handicap, f"<Runner> handicap {mr.handicap} != {ir.handicap}"
                assert mr.last_price_traded == ir.last_price_traded, f"<Runner> last_price_traded {mr.last_price_traded} != {ir.last_price_traded}"
                assert mr.name == ir.name, f"<Runner> name {mr.name} != {ir.name}"
                assert mr.removal_date == ir.removal_date, f"<Runner> removal_date {mr.removal_date} != {ir.removal_date}"
                assert mr.status == ir.status, f"<Runner> status {mr.status} != {ir.status}"
                assert mr.sort_priority == ir.sort_priority, f"<Runner> sort_priority {mr.sort_priority} != {ir.sort_priority}"
                assert mr.total_matched == ir.total_matched, f"<Runner> total_matched {mr.total_matched} != {ir.total_matched}"

                test_RunnerBookEX(mr.ex, ir.ex, mr.selection_id)
                test_RunnerBookSP(mr.sp, ir.sp, mr.selection_id)

            update_count += 1
        print(f"Market {market_count} Update {update_count}", end='\r')

def test_RunnerBookSP(sp1: bfd.RunnerBookSP, sp2: bfd.RunnerBookSP, sid: int):
    assert test_float(sp1.far_price, sp2.far_price), f"<RunnerBookSP> far_price {sp1.far_price} != {sp2.far_price}"
    assert test_float(sp1.near_price, sp2.near_price), f"<RunnerBookSP> near_price {sp1.near_price} != {sp2.near_price}"
    assert test_float(sp1.actual_sp, sp2.actual_sp), f"<RunnerBookSP> actual_sp {sp1.actual_sp} != {sp2.actual_sp}"
    test_ListPriceSize(sp1.back_stake_taken, sp2.back_stake_taken, f"<RunnerBookSP[{sid}].back_stake_taken>")
    test_ListPriceSize(sp1.lay_liability_taken, sp2.lay_liability_taken, f"<RunnerBookSP[{sid}].lay_liability_taken>")

def test_RunnerBookEX(ex1: bfd.RunnerBookEX, ex2: bfd.RunnerBookEX, sid: int):
    test_ListPriceSize(ex1.available_to_back, ex2.available_to_back, f"<RunnerBookEX[{sid}].available_to_back> {print_ladder(ex1.available_to_back)} != {print_ladder(ex2.available_to_back)}")
    test_ListPriceSize(ex1.available_to_lay, ex2.available_to_lay, f"<RunnerBookEX[{sid}].available_to_lay>")
    test_ListPriceSize(ex1.traded_volume, ex2.traded_volume, f"<RunnerBookEX[{sid}].traded_volume>")

def test_ListPriceSize(ps1: List[bfd.PriceSize], ps2: List[bfd.PriceSize], context: str):
    assert len(ps1) == len(ps2), f"{context} <List[PriceSize]> different lengths {len(ps1)} != {len(ps2)}"
    for i, (a, b) in enumerate(zip(ps1, ps2)):
        assert test_float(a.price, b.price), f"{context} <List[PriceSize][{i}]> Price  {a.price} != {b.price}"
        assert test_float(a.size, b.size), f"{context} <List[PriceSize][{i}]> Size  {a.size} != {b.size}"
    
def test_float(f1: float|NoneType, f2: float|NoneType) -> bool:
    return ((f1 == f2) or math.isclose(f1, f2, abs_tol=0.0001) or (math.isnan(f1) and math.isnan(f2)) or (math.isinf(f1) and math.isinf(f2)))

def print_ladder(ladder: List[bfd.PriceSize]) -> str:
    return ' '.join(list(f"[{ps.price}, {ps.size}]" for ps in ladder))


start_test()
