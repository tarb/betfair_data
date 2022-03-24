use core::fmt;
use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::borrow::Cow;
use std::sync::Arc;

use super::market_definition_runner::MarketDefinitionRunner;
use super::runner_book::RunnerBook;
use crate::bflw::market_definition_runner::RunnerDefSeq;
use crate::datetime::DateTimeString;
use crate::enums::{MarketBettingType, MarketStatus};
use crate::errors::DataError;
use crate::ids::{EventID, EventTypeID};
use crate::immutable::container::SyncObj;
use crate::market_source::SourceConfig;
use crate::strings::FixedSizeString;

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
    pub market_type: SyncObj<Arc<str>>,
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
    pub timezone: SyncObj<Arc<str>>,
    #[pyo3(get)]
    pub turn_in_play_enabled: bool,
    #[pyo3(get)]
    pub venue: Option<SyncObj<Arc<str>>>,
    #[pyo3(get)]
    pub version: u64,
    #[pyo3(get)]
    pub country_code: Option<SyncObj<FixedSizeString<2>>>,
    #[pyo3(get)]
    pub name: Option<SyncObj<Arc<str>>>,
    #[pyo3(get)]
    pub event_name: Option<SyncObj<Arc<str>>>,
    #[pyo3(get)]
    pub race_type: Option<SyncObj<Arc<str>>>,

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
    race_type: Option<&'a str>,
    number_of_active_runners: Option<u16>,
    number_of_winners: Option<u8>,
    open_date: Option<&'a str>,
    persistence_enabled: Option<bool>,
    regulators: Option<Vec<&'a str>>,
    runners: Option<Vec<Py<MarketDefinitionRunner>>>,
    runners_voidable: Option<bool>,
    settled_time: Option<&'a str>,
    suspend_time: Option<&'a str>,
    status: Option<MarketStatus>,
    timezone: Option<&'a str>,
    turn_in_play_enabled: Option<bool>,
    venue: Option<&'a str>,
    version: Option<u64>,
    country_code: Option<&'a str>,
    name: Option<Cow<'a, str>>,
    event_name: Option<Cow<'a, str>>,
}

impl MarketDefinition {
    fn new(change: MarketDefinitionUpdate) -> Result<Self, DataError> {
        Ok(Self {
            bet_delay: change.bet_delay.ok_or(DataError {
                missing_field: "betDelay",
            })?,
            betting_type: change.betting_type.ok_or(DataError {
                missing_field: "bettingType",
            })?,
            regulators: change
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .ok_or(DataError {
                    missing_field: "regulators",
                })?,
            bsp_reconciled: change.bsp_reconciled.ok_or(DataError {
                missing_field: "bspReconciled",
            })?,
            bsp_market: change.bsp_market.ok_or(DataError {
                missing_field: "bspMarket",
            })?,
            complete: change.complete.ok_or(DataError {
                missing_field: "complete",
            })?,
            cross_matching: change.cross_matching.ok_or(DataError {
                missing_field: "crossMatching",
            })?,
            discount_allowed: change.discount_allowed.ok_or(DataError {
                missing_field: "discountAllowed",
            })?,
            event_id: change.event_id.ok_or(DataError {
                missing_field: "eventId",
            })?,
            event_type_id: change.event_type_id.ok_or(DataError {
                missing_field: "eventTypeId",
            })?,
            in_play: change.in_play.ok_or(DataError {
                missing_field: "inPlay",
            })?,
            market_base_rate: change.market_base_rate.ok_or(DataError {
                missing_field: "marketBaseRate",
            })?,
            number_of_winners: change.number_of_winners.ok_or(DataError {
                missing_field: "numberOfWinners",
            })?,
            persistence_enabled: change.persistence_enabled.ok_or(DataError {
                missing_field: "persistenceEnabled",
            })?,
            runners_voidable: change.runners_voidable.ok_or(DataError {
                missing_field: "runnersVoidable",
            })?,
            version: change.version.ok_or(DataError {
                missing_field: "version",
            })?,
            status: change.status.ok_or(DataError {
                missing_field: "status",
            })?,
            turn_in_play_enabled: change.turn_in_play_enabled.ok_or(DataError {
                missing_field: "turnInPlayEnabled",
            })?,
            number_of_active_runners: change.number_of_active_runners.ok_or(DataError {
                missing_field: "numberOfActiveRunners",
            })?,
            runners: change
                .runners
                .map(|r| SyncObj::new(Arc::new(r)))
                .ok_or(DataError {
                    missing_field: "runners",
                })?,
            market_time: change
                .market_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .ok_or(DataError {
                    missing_field: "marketTime",
                })?,
            market_type: change
                .market_type
                .map(|s| SyncObj::new(Arc::from(s)))
                .ok_or(DataError {
                    missing_field: "marketType",
                })?,
            timezone: change
                .timezone
                .map(|s| SyncObj::new(Arc::from(s)))
                .ok_or(DataError {
                    missing_field: "timezone",
                })?,
            venue: change.venue.map(|s| SyncObj::new(Arc::from(s))),
            country_code: change
                .country_code
                .map(|s| SyncObj::new(FixedSizeString::try_from(s).unwrap())), // todo
            open_date: change
                .open_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .ok_or(DataError {
                    missing_field: "openDate",
                })?,
            settled_time: change
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            suspend_time: change
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            name: change.name.map(|s| SyncObj::new(Arc::from(s.as_ref()))),
            race_type: change.race_type.map(|s| SyncObj::new(Arc::from(s))),
            event_name: change
                .event_name
                .map(|s| SyncObj::new(Arc::from(s.as_ref()))),
        })
    }

    fn update_from_change(&self, change: MarketDefinitionUpdate) -> Result<Self, DataError> {
        Ok(Self {
            bet_delay: change.bet_delay.ok_or(DataError {
                missing_field: "betDelay",
            })?,
            betting_type: change.betting_type.ok_or(DataError {
                missing_field: "bettingType",
            })?,
            bsp_market: change.bsp_market.ok_or(DataError {
                missing_field: "bspMarket",
            })?,
            bsp_reconciled: change.bsp_reconciled.ok_or(DataError {
                missing_field: "bspReconciled",
            })?,
            complete: change.complete.ok_or(DataError {
                missing_field: "complete",
            })?,
            cross_matching: change.cross_matching.ok_or(DataError {
                missing_field: "crossMatching",
            })?,
            discount_allowed: change.discount_allowed.ok_or(DataError {
                missing_field: "discountAllowed",
            })?,
            event_id: change.event_id.ok_or(DataError {
                missing_field: "eventId",
            })?,
            event_type_id: change.event_type_id.ok_or(DataError {
                missing_field: "eventTypeId",
            })?,
            in_play: change.in_play.ok_or(DataError {
                missing_field: "inPlay",
            })?,
            market_base_rate: change.market_base_rate.ok_or(DataError {
                missing_field: "marketBaseRate",
            })?,
            number_of_winners: change.number_of_winners.ok_or(DataError {
                missing_field: "numberOfWinners",
            })?,
            persistence_enabled: change.persistence_enabled.ok_or(DataError {
                missing_field: "persistenceEnabled",
            })?,
            runners_voidable: change.runners_voidable.ok_or(DataError {
                missing_field: "runnersVoidable",
            })?,
            version: change.version.ok_or(DataError {
                missing_field: "version",
            })?,
            status: change.status.ok_or(DataError {
                missing_field: "status",
            })?,
            turn_in_play_enabled: change.turn_in_play_enabled.ok_or(DataError {
                missing_field: "turnInPlayEnabled",
            })?,
            number_of_active_runners: change.number_of_active_runners.ok_or(DataError {
                missing_field: "numberOfActiveRunners",
            })?,
            market_time: change
                .market_time
                .map(|s| {
                    if self.market_time.as_str() == s {
                        self.market_time.clone()
                    } else {
                        SyncObj::new(DateTimeString::new(s).unwrap())
                    }
                })
                .ok_or(DataError {
                    missing_field: "marketTime",
                })?,
            market_type: change
                .market_type
                .map(|s| {
                    if self.market_type.as_ref() == s {
                        self.market_type.clone()
                    } else {
                        SyncObj::new(Arc::from(s))
                    }
                })
                .ok_or(DataError {
                    missing_field: "marketType",
                })?,
            regulators: change
                .regulators
                .map(|v| {
                    if v.iter().eq(self.regulators.iter()) {
                        self.regulators.clone()
                    } else {
                        SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect()))
                    }
                })
                .ok_or(DataError {
                    missing_field: "regulators",
                })?,
            timezone: change
                .timezone
                .map(|s| {
                    if self.timezone.as_ref() == s {
                        self.timezone.clone()
                    } else {
                        SyncObj::new(Arc::from(s))
                    }
                })
                .ok_or(DataError {
                    missing_field: "timezone",
                })?,
            venue: change.venue.and_then(|n| {
                if self.venue.is_some_and(|v| v.as_ref() == n) {
                    self.venue.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n)))
                }
            }),
            country_code: change.country_code.and_then(|s| {
                if self.country_code.is_some_and(|v| v.as_ref() == s) {
                    self.country_code.clone()
                } else {
                    Some(SyncObj::new(FixedSizeString::try_from(s).unwrap()))
                }
            }),
            open_date: change
                .open_date
                .map(|s| {
                    if self.open_date.as_str() == s {
                        self.open_date.clone()
                    } else {
                        SyncObj::new(DateTimeString::new(s).unwrap())
                    }
                })
                .ok_or(DataError {
                    missing_field: "openDate",
                })?,
            settled_time: change.settled_time.and_then(|s| {
                if self.settled_time.is_some_and(|st| st.as_str() == s) {
                    self.settled_time.clone()
                } else {
                    Some(SyncObj::new(DateTimeString::new(s).unwrap()))
                }
            }),
            suspend_time: change.suspend_time.and_then(|s| {
                if self.suspend_time.is_some_and(|st| st.as_str() == s) {
                    self.suspend_time.clone()
                } else {
                    Some(SyncObj::new(DateTimeString::new(s).unwrap()))
                }
            }),
            name: change.name.and_then(|n| {
                if self.name.is_some_and(|name| name.as_ref() == n.as_ref()) {
                    self.name.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n.as_ref())))
                }
            }),
            event_name: change.event_name.and_then(|n| {
                if self
                    .event_name
                    .is_some_and(|ename| ename.as_ref() == n.as_ref())
                {
                    self.event_name.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n.as_ref())))
                }
            }),
            race_type: change.race_type.and_then(|s| {
                if self
                    .race_type
                    .is_some_and(|rt| rt.as_ref() == s)
                {
                    self.race_type.clone()
                } else {
                    Some(SyncObj::new(Arc::from(s)))
                }
            }),
            runners: change
                .runners
                .map(|r| SyncObj::new(Arc::new(r)))
                .unwrap_or_else(|| self.runners.clone()),
        })
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

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Runners => {
                            let s1 = self.def.as_ref().map(|def| def.runners.as_ref());
                            let s2 = self.runners;

                            let (d, b) = map.next_value_seed(RunnerDefSeq {
                                defs: s1,
                                books: s2,
                                py: self.py,
                                config: self.config,
                            })?;

                            if d.is_some() {
                                upt.runners = d;
                            }

                            books = b;
                        }
                        Field::Regulators => {
                            upt.regulators = Some(map.next_value::<Vec<&str>>()?);
                        }
                        Field::BspMarket => {
                            upt.bsp_market = Some(map.next_value::<bool>()?);
                        }
                        Field::TurnInPlayEnabled => {
                            upt.turn_in_play_enabled = Some(map.next_value::<bool>()?);
                        }
                        Field::InPlay => {
                            upt.in_play = Some(map.next_value::<bool>()?);
                        }
                        Field::PersistenceEnabled => {
                            upt.persistence_enabled = Some(map.next_value::<bool>()?);
                        }
                        Field::BspReconciled => {
                            upt.bsp_reconciled = Some(map.next_value::<bool>()?);
                        }
                        Field::Complete => {
                            upt.complete = Some(map.next_value::<bool>()?);
                        }
                        Field::CrossMatching => {
                            upt.cross_matching = Some(map.next_value::<bool>()?);
                        }
                        Field::RunnersVoidable => {
                            upt.runners_voidable = Some(map.next_value::<bool>()?);
                        }
                        Field::DiscountAllowed => {
                            upt.discount_allowed = Some(map.next_value::<bool>()?);
                        }
                        Field::Timezone => {
                            upt.timezone = Some(map.next_value::<&str>()?);
                        }
                        Field::Name => {
                            upt.name = Some(map.next_value::<Cow<str>>()?);
                        }
                        Field::EventName => {
                            upt.event_name = Some(map.next_value::<Cow<str>>()?);
                        }
                        Field::CountryCode => {
                            upt.country_code = Some(map.next_value::<&str>()?);
                        }
                        Field::Venue => {
                            upt.venue = Some(map.next_value::<&str>()?);
                        }
                        Field::Status => {
                            upt.status = Some(map.next_value::<MarketStatus>()?);
                        }
                        Field::MarketBaseRate => {
                            upt.market_base_rate = Some(map.next_value::<f32>()?);
                        }
                        Field::NumberOfWinners => {
                            upt.number_of_winners = Some(map.next_value::<f32>()? as u8);
                        }
                        Field::NumberOfActiveRunners => {
                            upt.number_of_active_runners = Some(map.next_value::<u16>()?);
                        }
                        Field::BetDelay => {
                            upt.bet_delay = Some(map.next_value::<u16>()?);
                        }
                        Field::EventId => {
                            upt.event_id = Some(
                                map.next_value::<&str>()?
                                    .parse()
                                    .map_err(de::Error::custom)?,
                            );
                        }
                        Field::EventTypeId => {
                            upt.event_type_id = Some(
                                map.next_value::<&str>()?
                                    .parse()
                                    .map_err(de::Error::custom)?,
                            );
                        }
                        Field::Version => {
                            upt.version = Some(map.next_value::<u64>()?);
                        }
                        Field::MarketType => {
                            upt.market_type = Some(map.next_value::<&str>()?);
                        }
                        Field::BettingType => {
                            upt.betting_type = Some(map.next_value::<MarketBettingType>()?);
                        }
                        Field::MarketTime => {
                            upt.market_time = Some(map.next_value::<&str>()?);
                        }
                        Field::SuspendTime => {
                            upt.suspend_time = Some(map.next_value::<&str>()?);
                        }
                        Field::SettledTime => {
                            upt.settled_time = Some(map.next_value::<&str>()?);
                        }
                        Field::OpenDate => {
                            upt.open_date = Some(map.next_value::<&str>()?);
                        }
                        // after searching over 200k markets, I cant find these values in any data sets :/
                        Field::EachWayDivisor => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                            // let each_way_divisor = Some(map.next_value::<f64>()?);
                        }
                        Field::RaceType => {
                            upt.race_type = Some(map.next_value::<&str>()?);
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
                    Some(def) => def.update_from_change(upt),
                    None => MarketDefinition::new(upt),
                }
                .map_err(|err| {
                    Error::custom(format!("missing required field <{}>", err.missing_field))
                })?;

                Ok((Some(def), books))
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
