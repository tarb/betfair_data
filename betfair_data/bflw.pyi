from datetime import datetime
from typing import Iterator, Optional, Sequence, List, str
from betfair_data import RunnerBookEX as RunnerBookEX, RunnerBookSP as RunnerBookSP, PriceSize as PriceSize

class File(Iterator[Sequence[MarketBook]]):
    file_name: str
    stream_unique_id: Optional[int]
    
    def __init__(self, path: str, bytes: bytes, cumulative_runner_tv: bool = True, streaming_unique_id: Optional[int] = None) -> None: ...

class Files(Iterator[File]):
    """"""
    def __init__(self, paths: Sequence[str], cumulative_runner_tv: bool = True, streaming_unique_id: Optional[int] = None) -> None: ...

class MarketBook:
    bet_delay: int
    bsp_reconciled: bool
    complete: bool
    cross_matching: bool
    inplay: bool
    is_market_data_delayed: bool
    key_line_description: Optional[MarketDefinitionKeyLine]
    last_match_time: datetime
    market_definition: MarketDefinition
    market_id: str
    number_of_active_runners: int
    number_of_runners: int
    number_of_winners: int
    price_ladder_definition: Optional[PriceLadderDescription]
    publish_time_epoch: int
    publish_time: datetime
    runners_voidable: bool
    runners: List[RunnerBook]
    status: str
    total_available: float
    total_matched: float
    version: int

class RunnerBook:
    adjustment_factor: float
    ex: RunnerBookEX
    handicap: float
    last_price_traded: float
    removal_date: datetime
    selection_id: int
    sp: RunnerBookSP
    status: str
    total_matched: float
    matches: None
    orders: None

class MarketDefinition:
    bet_delay: int
    betting_type: str
    bsp_market: bool
    bsp_reconciled: bool
    complete: bool
    country_code: str
    cross_matching: bool
    discount_allowed: bool
    event_id: str
    event_name: Optional[str]
    event_type_id: str
    in_play: bool
    key_line_definitions: Optional[MarketDefinitionKeyLine]
    line_interval: Optional[float]
    line_max_unit: Optional[float]
    line_min_unit: Optional[float]
    market_base_rate: float
    market_time: datetime
    market_type: str
    name: Optional[str]
    number_of_active_runners: int
    number_of_winners: int
    open_date: datetime
    persistence_enabled: bool
    price_ladder_definition: Optional[PriceLadderDescription]
    race_type: Optional[str]
    regulators: str
    runners_voidable: bool
    runners: List[MarketDefinitionRunner]
    settled_time: Optional[datetime]
    status: str
    suspend_time: Optional[datetime]
    timezone: str
    turn_in_play_enabled: bool
    venue: Optional[str]
    version: int

class MarketDefinitionRunner:
    adjustment_factor: Optional[float]
    bsp: Optional[float]
    handicap: float
    name: Optional[str]
    removal_date: Optional[datetime]
    selection_id: int
    sort_priority: int
    status: str

class PriceLadderDescription:
    type: str

class MarketDefinitionKeyLine:
    key_line: List[MarketDefinitionKeyLineSelection]

class MarketDefinitionKeyLineSelection:
    handicap: float
    selection_id: int
