from typing import Iterator, List, Sequence, Optional
from betfair_data import bflw

class MarketImage():
    """
    A class representing a Betfair Market.
    """
    source: str
    file: str
    market_id: str
    """Market Id - the id of the market"""
    bet_delay: int
    betting_type: str
    bsp_market: bool
    """Is BSP betting available for this market
            >>> m.bsp_market
            True
    """
    bsp_reconciled: bool
    """Has the starting price been detirmined for this market
            >>> m.bsp_reconciled
            False
    """
    clk: str
    """Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to resume subscription (in case of disconnect)"""
    complete: bool
    """Is the market in a completed state
            >>> m.complete
            False
    """
    country_code: str
    """The country the market is taking place in. 2 digit string
            >>> m.country_code
            'AU'
    """
    cross_matching: bool
    discount_allowed: bool
    each_way_divisor: Optional[float]
    event_id: int
    event_name: Optional[str]
    event_type_id: int
    in_play: bool
    market_base_rate: int
    market_name: Optional[str]
    market_time: int
    market_type: str
    number_of_active_runners: int
    number_of_winners: int
    open_date: int
    persistence_enabled: bool
    publish_time: int
    """Publish Time (in millis since epoch) that the changes were generated"""
    runners_voidable: bool
    runners: List[Runner]   
    settled_time: Optional[int]
    status: str
    """The markets current status, value is one of ["INACTIVE", "OPEN", "SUSPENDED", "CLOSED"]
        >>> market.status
        'CLOSED'
    """
    suspend_time: Optional[int]
    timezone: str
    total_matched: float
    """The total amount matched across the market. This value is truncated at 2dp (or null if un-changed)
        >>> m.total_matched
        53212.45
    """
    turn_in_play_enabled: bool
    venue: Optional[str]
    version: int

class Market(MarketImage):
    """"""
    def __init__(self, path: str, bytes: bytes, cumulative_runner_tv: bool = True, stable_runner_index = True) -> None: ...

    def update(self) -> bool: 
        """ Update the market with the next delta

        Example:
            >>> market.update()
        """  
    def copy(self) -> MarketImage:
        """ Create an immutable copy of the market, including runners and prices that wont be updated

        Example:
            >>> c = market.copy()
            >>> market.update()
            >>> market.publish_time != c.publish_time
            True
        """  

class Runner():
    """
    A class representing a Betfair Runner.
    """
    selection_id: int
    name: Optional[str]
    status: str
    """The runners current status, value is one of ["ACTIVE", "REMOVED", "REMOVED_VACANT", "WINNER", "PLACED", "LOSER", "HIDDEN"]
        >>> market.runners[0].status
        'ACTIVE'
    """
    last_price_traded: Optional[float]
    total_matched: float
    adjustment_factor: Optional[float]
    handicap: Optional[float]
    sort_priority: int
    removal_date: Optional[int]
    ex: RunnerBookEX
    sp: RunnerBookSP

class RunnerBookEX():
    available_to_back: List[PriceSize]
    available_to_lay: List[PriceSize]
    traded_volume: List[PriceSize]

class RunnerBookSP(): 
    far_price: Optional[float]
    near_price: Optional[float]
    actual_sp: Optional[float]
    back_stake_taken: List[PriceSize]
    lay_liability_taken: List[PriceSize]

class PriceSize():
    price: float
    size: float

# sources
class MutableAdapter(Iterator[Market]): ...

class TarBz2():
    """"""
    def __init__(self, paths: Sequence[str], cumulative_runner_tv: bool = True) -> None: ...

    def mutable(self, stable_runner_index = True) -> MutableAdapter: ...
    def bflw(self) -> bflw.BflwAdapter: ...


class Files():
    """"""
    def __init__(self, paths: Sequence[str], cumulative_runner_tv: bool = True) -> None: ...

    def mutable(self, stable_runner_index = True) -> MutableAdapter: ...
    def bflw(self) -> bflw.BflwAdapter: ...
