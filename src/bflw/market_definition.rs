use core::fmt;
use pyo3::prelude::*;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::borrow::Cow;
use std::sync::Arc;

use crate::bflw::market_definition_runner::RunnerDefSeq;
use crate::enums::{MarketBettingType, MarketStatus};
use crate::ids::{EventID, EventTypeID};
use crate::immutable::container::SyncObj;
use crate::market_source::SourceConfig;
use crate::strings::FixedSizeString;
use crate::datetime::DateTimeString;
use super::market_definition_runner::MarketDefinitionRunner;
use super::runner_book::RunnerBook;

#[derive(Debug)]
#[pyclass]
pub struct MarketDefinition {
    #[pyo3(get)]
    pub bet_delay: u16,
    #[pyo3(get)]
    pub betting_type: MarketBettingType,
    #[pyo3(get)]
    pub bsp_market: bool,
    #[pyo3(get)]
    pub bsp_reconciled: bool,
    #[pyo3(get)]
    pub complete: bool,
    #[pyo3(get)]
    pub cross_matching: bool,
    #[pyo3(get)]
    pub discount_allowed: bool,
    #[pyo3(get)]
    pub in_play: bool,
    #[pyo3(get)]
    pub market_base_rate: f32,
    #[pyo3(get)]
    pub market_time: SyncObj<DateTimeString>,
    #[pyo3(get)]
    pub market_type: SyncObj<Arc<String>>,
    #[pyo3(get)]
    pub number_of_active_runners: u16,
    #[pyo3(get)]
    pub number_of_winners: u8,
    #[pyo3(get)]
    pub open_date: SyncObj<DateTimeString>,
    #[pyo3(get)]
    pub persistence_enabled: bool,
    #[pyo3(get)]
    pub regulators: SyncObj<Arc<Vec<String>>>,
    #[pyo3(get)]
    pub runners: SyncObj<Arc<Vec<Py<MarketDefinitionRunner>>>>,
    #[pyo3(get)]
    pub runners_voidable: bool,
    #[pyo3(get)]
    pub settled_time: Option<SyncObj<DateTimeString>>,
    #[pyo3(get)]
    pub status: MarketStatus,
    #[pyo3(get)]
    pub suspend_time: Option<SyncObj<DateTimeString>>,
    #[pyo3(get)]
    pub timezone: SyncObj<Arc<String>>,
    #[pyo3(get)]
    pub turn_in_play_enabled: bool,
    #[pyo3(get)]
    pub venue: Option<SyncObj<Arc<String>>>,
    #[pyo3(get)]
    pub version: u64,
    #[pyo3(get)]
    pub country_code: Option<SyncObj<FixedSizeString<2>>>,
    #[pyo3(get)]
    pub name: Option<SyncObj<Arc<String>>>,
    #[pyo3(get)]
    pub event_name: Option<SyncObj<Arc<String>>>,

    // use getters to turn these into strings
    pub event_id: EventID,
    pub event_type_id: EventTypeID,
    // lineMaxUnit: float = None,
    // lineMinUnit: float = None,
    // lineInterval: float = None,
    // priceLadderDefinition: dict = None,
    // keyLineDefinition: dict = None,
    // raceType: str = None,
}

#[pymethods]
impl MarketDefinition {
    #[getter(event_id)]
    fn get_event_id(&self, py: Python) -> PyObject {
        self.event_id.to_string().into_py(py)
    }

    #[getter(event_type_id)]
    fn get_event_type_id(&self, py: Python) -> PyObject {
        self.event_type_id.to_string().into_py(py)
    }
}

#[derive(Debug, Default)]
struct MarketDefinitionUpdate<'a> {
    bet_delay: Option<u16>,
    betting_type: Option<MarketBettingType>,
    bsp_market: Option<bool>,
    bsp_reconciled: Option<bool>,
    complete: Option<bool>,
    cross_matching: Option<bool>,
    discount_allowed: Option<bool>,
    event_id: Option<EventID>,
    event_type_id: Option<EventTypeID>,
    in_play: Option<bool>,
    market_base_rate: Option<f32>,
    market_time: Option<&'a str>,
    market_type: Option<&'a str>,
    number_of_active_runners: Option<u16>,
    number_of_winners: Option<u8>,
    open_date: Option<&'a str>,
    persistence_enabled: Option<bool>,
    regulators: Option<Vec<&'a str>>,
    runners: Option<Vec<Py<MarketDefinitionRunner>>>,
    runners_voidable: Option<bool>,
    settled_time: Option<&'a str>,
    status: Option<MarketStatus>,
    suspend_time: Option<&'a str>,
    timezone: Option<&'a str>,
    turn_in_play_enabled: Option<bool>,
    venue: Option<&'a str>,
    version: Option<u64>,
    country_code: Option<&'a str>,
    name: Option<Cow<'a, str>>,
    event_name: Option<Cow<'a, str>>,
}

impl MarketDefinition {
    fn new(change: MarketDefinitionUpdate) -> Self {
        Self {
            bet_delay: change.bet_delay.unwrap_or_default(),
            betting_type: change.betting_type.unwrap_or_default(),
            regulators: change
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .unwrap_or_default(),
            bsp_reconciled: change.bsp_reconciled.unwrap_or_default(),
            bsp_market: change.bsp_market.unwrap_or_default(),
            complete: change.complete.unwrap_or_default(),
            cross_matching: change.cross_matching.unwrap_or_default(),
            discount_allowed: change.discount_allowed.unwrap_or_default(),
            event_id: change.event_id.unwrap_or_default(),
            event_type_id: change.event_type_id.unwrap_or_default(),
            in_play: change.in_play.unwrap_or_default(),
            market_base_rate: change.market_base_rate.unwrap_or_default(),
            number_of_winners: change.number_of_winners.unwrap_or_default(),
            persistence_enabled: change.persistence_enabled.unwrap_or_default(),
            runners_voidable: change.runners_voidable.unwrap_or_default(),
            version: change.version.unwrap_or_default(),
            status: change.status.unwrap_or_default(),
            turn_in_play_enabled: change.turn_in_play_enabled.unwrap_or_default(),
            number_of_active_runners: change.number_of_active_runners.unwrap_or_default(),
            runners: change
                .runners
                .map(|r| SyncObj::new(Arc::new(r)))
                .unwrap_or_default(),
            market_time: change
                .market_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap(),
            market_type: change
                .market_type
                .map(|s| SyncObj::new(Arc::new(String::from(s))))
                .unwrap(),
            timezone: change
                .timezone
                .map(|s| SyncObj::new(Arc::new(String::from(s))))
                .unwrap(),
            venue: change
                .venue
                .map(|s| SyncObj::new(Arc::new(String::from(s)))),
            country_code: change
                .country_code
                .map(|s| SyncObj::new(FixedSizeString::try_from(s).unwrap())), // todo
            open_date: change
                .open_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap(),
            settled_time: change
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            suspend_time: change
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            name: change.name.map(|s| SyncObj::new(Arc::new(s.into_owned()))),
            event_name: change
                .event_name
                .map(|s| SyncObj::new(Arc::new(s.into_owned()))),
        }
    }
    fn update_from_change(&self, change: MarketDefinitionUpdate) -> Self {
        Self {
            bet_delay: change.bet_delay.unwrap_or(self.bet_delay),
            betting_type: change.betting_type.unwrap_or(self.betting_type),
            bsp_market: change.bsp_market.unwrap_or(self.bsp_market),
            bsp_reconciled: change.bsp_reconciled.unwrap_or(self.bsp_reconciled),
            complete: change.complete.unwrap_or(self.complete),
            cross_matching: change.cross_matching.unwrap_or(self.cross_matching),
            discount_allowed: change.discount_allowed.unwrap_or(self.discount_allowed),
            event_id: change.event_id.unwrap_or(self.event_id),
            event_type_id: change.event_type_id.unwrap_or(self.event_type_id),
            in_play: change.in_play.unwrap_or(self.in_play),
            market_base_rate: change.market_base_rate.unwrap_or(self.market_base_rate),
            number_of_winners: change.number_of_winners.unwrap_or(self.number_of_winners),
            persistence_enabled: change
                .persistence_enabled
                .unwrap_or(self.persistence_enabled),
            runners_voidable: change.runners_voidable.unwrap_or(self.runners_voidable),
            version: change.version.unwrap_or(self.version),
            status: change.status.unwrap_or(self.status),
            turn_in_play_enabled: change
                .turn_in_play_enabled
                .unwrap_or(self.turn_in_play_enabled),
            number_of_active_runners: change
                .number_of_active_runners
                .unwrap_or(self.number_of_active_runners),
            market_time: change
                .market_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap_or_else(|| self.market_time.clone()),
            market_type: change
                .market_type
                .map(|s| SyncObj::new(Arc::new(String::from(s))))
                .unwrap_or_else(|| self.market_type.clone()),
            regulators: change
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .unwrap_or_else(|| self.regulators.clone()),
            timezone: change
                .timezone
                .map(|s| SyncObj::new(Arc::new(String::from(s))))
                .unwrap_or_else(|| self.timezone.clone()),
            venue: change
                .venue
                .map(|s| Some(SyncObj::new(Arc::new(String::from(s)))))
                .unwrap_or_else(|| self.venue.clone()),
            country_code: change
                .country_code
                .map(|s| Some(SyncObj::new(FixedSizeString::try_from(s).unwrap()))) // todo
                .unwrap_or_else(|| self.country_code.clone()),
            open_date: change
                .open_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap_or_else(|| self.open_date.clone()),
            settled_time: change
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .or_else(|| self.settled_time.clone()),
            suspend_time: change
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .or_else(|| self.suspend_time.clone()),
            name: change
                .name
                .map(|s| SyncObj::new(Arc::new(s.into_owned())))
                .or_else(|| self.name.clone()),
            event_name: change
                .event_name
                .map(|s| SyncObj::new(Arc::new(s.into_owned())))
                .or_else(|| self.event_name.clone()),
            runners: change
                .runners
                .map(|r| SyncObj::new(Arc::new(r)))
                .unwrap_or_else(|| self.runners.clone()),
        }
    }
}

// Used for serializing in place over the mc marketDefinition object
pub struct MarketDefinitionDeser<'a, 'py> {
    pub def: Option<PyRef<'py, MarketDefinition>>,
    pub runners: Option<&'a Vec<Py<RunnerBook>>>,
    pub py: Python<'py>,
    pub config: SourceConfig,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketDefinitionDeser<'a, 'py> {
    type Value = (Option<MarketDefinition>, Option<Vec<Py<RunnerBook>>>);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            BetDelay,
            BettingType,
            BspMarket,
            BspReconciled,
            Complete,
            CountryCode,
            CrossMatching,
            DiscountAllowed,
            EachWayDivisor,
            EventId,
            EventName,
            EventTypeId,
            InPlay,
            KeyLineDefiniton,
            LineMaxUnit,
            LineMinUnit,
            LineInterval,
            MarketBaseRate,
            MarketTime,
            MarketType,
            Name,
            NumberOfActiveRunners,
            NumberOfWinners,
            OpenDate,
            PersistenceEnabled,
            PriceLadderDefinition,
            RaceType,
            Regulators,
            Runners,
            RunnersVoidable,
            SettledTime,
            Status,
            SuspendTime,
            Timezone,
            TurnInPlayEnabled,
            Venue,
            Version,
        }

        struct MarketDefinitionVisitor<'a, 'py> {
            pub def: Option<PyRef<'py, MarketDefinition>>,
            pub runners: Option<&'a Vec<Py<RunnerBook>>>,
            pub py: Python<'py>,
            pub config: SourceConfig,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketDefinitionVisitor<'a, 'py> {
            type Value = (Option<MarketDefinition>, Option<Vec<Py<RunnerBook>>>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut books: Option<Vec<Py<RunnerBook>>> = None;
                let mut upt: MarketDefinitionUpdate = MarketDefinitionUpdate::default();
                let mut changed = false;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::BspMarket => {
                            let bsp_market = map.next_value()?;
                            if self.def.is_some_with(|def| def.bsp_market != bsp_market)
                                || self.def.is_none()
                            {
                                upt.bsp_market = Some(bsp_market);
                                changed = true;
                            }
                        }
                        Field::TurnInPlayEnabled => {
                            let turn_in_play_enabled = map.next_value()?;
                            if self.def.is_some_with(|def| {
                                def.turn_in_play_enabled != turn_in_play_enabled
                            }) || self.def.is_none()
                            {
                                upt.turn_in_play_enabled = Some(turn_in_play_enabled);
                                changed = true;
                            }
                        }
                        Field::InPlay => {
                            let in_play = map.next_value()?;
                            if self.def.is_some_with(|def| def.in_play != in_play)
                                || self.def.is_none()
                            {
                                upt.in_play = Some(in_play);
                                changed = true;
                            }
                        }
                        Field::PersistenceEnabled => {
                            let persistence_enabled = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.persistence_enabled != persistence_enabled)
                                || self.def.is_none()
                            {
                                upt.persistence_enabled = Some(persistence_enabled);
                                changed = true;
                            }
                        }
                        Field::BspReconciled => {
                            let bsp_reconciled = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.bsp_reconciled != bsp_reconciled)
                                || self.def.is_none()
                            {
                                upt.bsp_reconciled = Some(bsp_reconciled);
                                changed = true;
                            }
                        }
                        Field::Complete => {
                            let complete = map.next_value()?;
                            if self.def.is_some_with(|def| def.complete != complete)
                                || self.def.is_none()
                            {
                                upt.complete = Some(complete);
                                changed = true;
                            }
                        }
                        Field::CrossMatching => {
                            let cross_matching = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.cross_matching != cross_matching)
                                || self.def.is_none()
                            {
                                upt.cross_matching = Some(cross_matching);
                                changed = true;
                            }
                        }
                        Field::RunnersVoidable => {
                            let runners_voidable = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.runners_voidable != runners_voidable)
                                || self.def.is_none()
                            {
                                upt.runners_voidable = Some(runners_voidable);
                                changed = true;
                            }
                        }
                        Field::DiscountAllowed => {
                            let discount_allowed = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.discount_allowed != discount_allowed)
                                || self.def.is_none()
                            {
                                upt.discount_allowed = Some(discount_allowed);
                                changed = true;
                            }
                        }
                        Field::Timezone => {
                            let timezone = map.next_value::<&str>()?;
                            if self
                                .def
                                .is_some_with(|def| def.timezone.as_str() != timezone)
                                || self.def.is_none()
                            {
                                upt.timezone = Some(timezone);
                                changed = true;
                            }
                        }
                        Field::Name => {
                            let market_name = map.next_value::<Cow<str>>()?;
                            if self
                                .def
                                .is_some_with(|def| !def.name.contains(&market_name.as_ref()))
                                || self.def.is_none()
                            {
                                upt.name = Some(market_name);
                                changed = true;
                            }
                        }
                        Field::EventName => {
                            let event_name = map.next_value::<Cow<str>>()?;
                            if self
                                .def
                                .is_some_with(|def| !def.event_name.contains(&event_name.as_ref()))
                                || self.def.is_none()
                            {
                                upt.event_name = Some(event_name);
                                changed = true;
                            }
                        }
                        Field::CountryCode => {
                            let country_code = map.next_value::<&str>()?;
                            if self
                                .def
                                .is_some_with(|def| !def.country_code.contains(&country_code))
                                || self.def.is_none()
                            {
                                upt.country_code = Some(country_code);
                                changed = true;
                            }
                        }
                        Field::Venue => {
                            let venue = map.next_value::<&str>()?;
                            if self.def.is_some_with(|def| !def.venue.contains(&venue))
                                || self.def.is_none()
                            {
                                upt.venue = Some(venue);
                                changed = true;
                            }
                        }
                        Field::Status => {
                            let status = map.next_value()?;
                            if self.def.is_some_with(|def| def.status != status)
                                || self.def.is_none()
                            {
                                upt.status = Some(status);
                                changed = true;
                            }
                        }
                        Field::MarketBaseRate => {
                            let market_base_rate = map.next_value::<f32>()?;
                            if self
                                .def
                                .is_some_with(|def| def.market_base_rate != market_base_rate)
                                || self.def.is_none()
                            {
                                upt.market_base_rate = Some(market_base_rate);
                                changed = true;
                            }
                        }
                        Field::NumberOfWinners => {
                            let number_of_winners = map.next_value::<f32>()? as u8;
                            if self
                                .def
                                .is_some_with(|def| def.number_of_winners != number_of_winners)
                                || self.def.is_none()
                            {
                                upt.number_of_winners = Some(number_of_winners);
                                changed = true;
                            }
                        }
                        Field::NumberOfActiveRunners => {
                            let number_of_active_runners = map.next_value()?;
                            if self.def.is_some_with(|def| {
                                def.number_of_active_runners != number_of_active_runners
                            }) || self.def.is_none()
                            {
                                upt.number_of_active_runners = Some(number_of_active_runners);
                                changed = true;
                            }
                        }
                        Field::BetDelay => {
                            let bet_delay = map.next_value()?;
                            if self.def.is_some_with(|def| def.bet_delay != bet_delay)
                                || self.def.is_none()
                            {
                                upt.bet_delay = Some(bet_delay);
                                changed = true;
                            }
                        }
                        Field::EventId => {
                            let event_id = map
                                .next_value::<&str>()?
                                .parse()
                                .map_err(de::Error::custom)?;
                            if self.def.is_some_with(|def| def.event_id != event_id)
                                || self.def.is_none()
                            {
                                upt.event_id = Some(event_id);
                                changed = true;
                            }
                        }
                        Field::EventTypeId => {
                            let event_type_id = map
                                .next_value::<&str>()?
                                .parse()
                                .map_err(de::Error::custom)?;
                            if self
                                .def
                                .is_some_with(|def| def.event_type_id != event_type_id)
                                || self.def.is_none()
                            {
                                upt.event_type_id = Some(event_type_id);
                                changed = true;
                            }
                        }
                        Field::Version => {
                            let version = map.next_value()?;
                            if self.def.is_some_with(|def| def.version != version)
                                || self.def.is_none()
                            {
                                upt.version = Some(version);
                                changed = true;
                            }
                        }
                        Field::Runners => {
                            let s1 = self.def.as_ref().map(|def| def.runners.as_ref());
                            let s2 = self.runners;

                            let (d, b) = map.next_value_seed(RunnerDefSeq {
                                defs: s1,
                                books: s2,
                                py: self.py,
                                config: self.config,
                            })?;

                            changed = d.is_some();
                            upt.runners = d;
                            books = b;
                        }
                        Field::MarketType => {
                            let market_type = map.next_value::<&str>()?;
                            if self
                                .def
                                .is_some_with(|def| def.market_type.as_str() != market_type)
                                || self.def.is_none()
                            {
                                upt.market_type = Some(market_type);
                                changed = true;
                            }
                        }
                        Field::BettingType => {
                            let betting_type = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.betting_type != betting_type)
                            {
                                upt.betting_type = Some(betting_type);
                                changed = true;
                            }
                        }

                        Field::MarketTime => {
                            let market_time = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.market_time.as_str() != market_time)
                                || self.def.is_none()
                            {
                                upt.market_time = Some(market_time);
                                changed = true;
                            }
                        }
                        Field::SuspendTime => {
                            let suspend_time = map.next_value::<&str>()?;
                            if !self
                                .def
                                .is_some_with(|def| def.suspend_time.contains(&suspend_time))
                                || self.def.is_none()
                            {
                                upt.suspend_time = Some(suspend_time);
                                changed = true;
                            }
                        }
                        Field::SettledTime => {
                            let settled_time = map.next_value::<&str>()?;
                            if !self
                                .def
                                .is_some_with(|def| def.settled_time.contains(&settled_time))
                                || self.def.is_none()
                            {
                                upt.settled_time = Some(settled_time);
                                changed = true;
                            }
                        }
                        Field::OpenDate => {
                            let open_date = map.next_value()?;
                            if self
                                .def
                                .is_some_with(|def| def.open_date.as_str() != open_date)
                                || self.def.is_none()
                            {
                                upt.open_date = Some(open_date);
                                changed = true;
                            }
                        }

                        Field::Regulators => {
                            let v = map.next_value::<Vec<&str>>()?;

                            if self.def.is_some_with(|def| {
                                (def.regulators.is_empty() && !v.is_empty())
                                    || !def.regulators.iter().eq(v.iter())
                            }) || self.def.is_none()
                            {
                                upt.regulators = Some(v);
                                changed = true;
                            }
                        }
                        // after searching over 200k markets, I cant find these values in any data sets :/
                        Field::EachWayDivisor => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // let each_way_divisor = Some(map.next_value::<f64>()?);
                        }
                        Field::RaceType => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                        Field::KeyLineDefiniton => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                        Field::PriceLadderDefinition => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                        Field::LineMaxUnit => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                        Field::LineMinUnit => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                        Field::LineInterval => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // panic!("{} {}", self.def.source, self.def.file);
                        }
                    }
                }

                let def = match self.def {
                    Some(def) => changed.then(|| def.update_from_change(upt)),
                    None => Some(MarketDefinition::new(upt)),
                };

                Ok((def, books))
            }
        }

        const FIELDS: &[&str] = &[
            "keyLineDefiniton",
            "priceLadderDefinition",
            "raceType",
            "lineMaxUnit",
            "lineMinUnit",
            "lineInterval",
            "bspMarket",
            "turnInPlayEnabled",
            "persistenceEnabled",
            "marketBaseRate",
            "eventId",
            "eventTypeId",
            "numberOfWinners",
            "bettingType",
            "marketType",
            "marketTime",
            "suspendTime",
            "bspReconciled",
            "complete",
            "inPlay",
            "crossMatching",
            "runnersVoidable",
            "numberOfActiveRunners",
            "betDelay",
            "status",
            "runners",
            "regulators",
            "countryCode",
            "discountAllowed",
            "timezone",
            "openDate",
            "version",
            "name",
            "eventName",
            "venue",
            "settledTime",
            "eachWayDivisor",
        ];

        deserializer.deserialize_struct(
            "MarketDefinition",
            FIELDS,
            MarketDefinitionVisitor {
                def: self.def,
                runners: self.runners,
                py: self.py,
                config: self.config,
            },
        )
    }
}
