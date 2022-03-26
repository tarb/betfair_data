use core::fmt;
use pyo3::prelude::*;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::borrow::Cow;
use std::sync::Arc;

use super::runner_book_ex::RunnerBookEX;
use super::runner_book_sp::RunnerBookSP;
use crate::config::Config;
use crate::datetime::DateTimeString;
use crate::enums::{MarketBettingType, MarketStatus, SelectionStatus};
use crate::errors::DataError;
use crate::ids::{EventID, EventTypeID, SelectionID};
use crate::immutable::container::SyncObj;
use crate::immutable::runner::Runner;
use crate::price_size::F64OrStr;
use crate::strings::FixedSizeString;

pub struct MarketDefinition {
    pub bet_delay: u16,
    pub bsp_market: bool,
    pub bsp_reconciled: bool,
    pub complete: bool,
    pub cross_matching: bool,
    pub discount_allowed: bool,
    pub event_id: EventID,
    pub event_name: Option<SyncObj<Arc<str>>>,
    pub event_type_id: EventTypeID,
    pub in_play: bool,
    pub market_base_rate: f32,
    pub market_type: SyncObj<Arc<str>>,
    pub race_type: Option<SyncObj<Arc<str>>>,
    pub market_name: Option<SyncObj<Arc<str>>>,
    pub number_of_active_runners: u16,
    pub number_of_winners: u8,
    pub persistence_enabled: bool,
    pub runners_voidable: bool,
    pub timezone: SyncObj<Arc<str>>,
    pub turn_in_play_enabled: bool,
    pub venue: Option<SyncObj<Arc<str>>>,
    pub version: u64,
    pub status: MarketStatus,
    pub betting_type: MarketBettingType,
    pub market_time: SyncObj<DateTimeString>,
    pub open_date: SyncObj<DateTimeString>,
    pub suspend_time: Option<SyncObj<DateTimeString>>,
    pub settled_time: Option<SyncObj<DateTimeString>>,
    pub country_code: Option<SyncObj<FixedSizeString<2>>>,
    pub regulators: SyncObj<Arc<Vec<String>>>,
    pub each_way_divisor: Option<f64>,
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
    market_time: Option<FixedSizeString<24>>,
    market_type: Option<&'a str>,
    race_type: Option<&'a str>,
    number_of_active_runners: Option<u16>,
    number_of_winners: Option<u8>,
    open_date: Option<FixedSizeString<24>>,
    persistence_enabled: Option<bool>,
    regulators: Option<Vec<&'a str>>,
    runners_voidable: Option<bool>,
    settled_time: Option<FixedSizeString<24>>,
    status: Option<MarketStatus>,
    suspend_time: Option<FixedSizeString<24>>,
    timezone: Option<&'a str>,
    turn_in_play_enabled: Option<bool>,
    venue: Option<&'a str>,
    version: Option<u64>,
    country_code: Option<&'a str>,
    market_name: Option<Cow<'a, str>>,
    event_name: Option<Cow<'a, str>>,
    each_way_divisor: Option<f64>,
}

impl<'a, 'b> MarketDefinitionUpdate<'a> {
    fn create(self) -> Result<MarketDefinition, DataError> {
        Ok(MarketDefinition {
            bet_delay: self.bet_delay.ok_or(DataError {
                missing_field: "betDelay",
            })?,
            betting_type: self.betting_type.ok_or(DataError {
                missing_field: "bettingType",
            })?,
            regulators: self
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .ok_or(DataError {
                    missing_field: "regulators",
                })?,
            bsp_reconciled: self.bsp_reconciled.ok_or(DataError {
                missing_field: "bspReconciled",
            })?,
            bsp_market: self.bsp_market.ok_or(DataError {
                missing_field: "bspMarket",
            })?,
            complete: self.complete.ok_or(DataError {
                missing_field: "complete",
            })?,
            cross_matching: self.cross_matching.ok_or(DataError {
                missing_field: "crossMatching",
            })?,
            discount_allowed: self.discount_allowed.ok_or(DataError {
                missing_field: "discountAllowed",
            })?,
            event_id: self.event_id.ok_or(DataError {
                missing_field: "eventId",
            })?,
            event_type_id: self.event_type_id.ok_or(DataError {
                missing_field: "eventTypeId",
            })?,
            in_play: self.in_play.ok_or(DataError {
                missing_field: "inPlay",
            })?,
            market_base_rate: self.market_base_rate.ok_or(DataError {
                missing_field: "marketBaseRate",
            })?,
            number_of_winners: self.number_of_winners.ok_or(DataError {
                missing_field: "numberOfWinners",
            })?,
            persistence_enabled: self.persistence_enabled.ok_or(DataError {
                missing_field: "persistenceEnabled",
            })?,
            runners_voidable: self.runners_voidable.ok_or(DataError {
                missing_field: "runnersVoidable",
            })?,
            version: self.version.ok_or(DataError {
                missing_field: "version",
            })?,
            status: self.status.ok_or(DataError {
                missing_field: "status",
            })?,
            turn_in_play_enabled: self.turn_in_play_enabled.ok_or(DataError {
                missing_field: "turnInPlayEnabled",
            })?,
            number_of_active_runners: self.number_of_active_runners.ok_or(DataError {
                missing_field: "numberOfActiveRunners",
            })?,
            market_time: self
                .market_time
                .map(|s| SyncObj::new(DateTimeString::try_from(s).unwrap()))
                .ok_or(DataError {
                    missing_field: "marketTime",
                })?,
            market_type: self
                .market_type
                .map(|s| SyncObj::new(Arc::from(s)))
                .ok_or(DataError {
                    missing_field: "marketType",
                })?,
            timezone: self
                .timezone
                .map(|s| SyncObj::new(Arc::from(s)))
                .ok_or(DataError {
                    missing_field: "timezone",
                })?,
            venue: self.venue.map(|s| SyncObj::new(Arc::from(s))),
            country_code: self
                .country_code
                .map(|s| SyncObj::new(FixedSizeString::try_from(s).unwrap())), // todo
            open_date: self
                .open_date
                .map(|s| SyncObj::new(DateTimeString::try_from(s).unwrap()))
                .ok_or(DataError {
                    missing_field: "openDate",
                })?,
            settled_time: self
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::try_from(s).unwrap())),
            suspend_time: self
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::try_from(s).unwrap())),
            market_name: self.market_name.map(|s| SyncObj::new(Arc::from(s.as_ref()))),
            event_name: self
                .event_name
                .map(|s| SyncObj::new(Arc::from(s.as_ref()))),
            each_way_divisor: self.each_way_divisor,
            race_type: self.race_type.map(|s| SyncObj::new(Arc::from(s))),
        })
    }

    fn update(self, market: &MarketDefinition) -> Result<MarketDefinition, DataError> {
        Ok(MarketDefinition {
            bet_delay: self.bet_delay.ok_or(DataError {
                missing_field: "betDelay",
            })?,
            betting_type: self.betting_type.ok_or(DataError {
                missing_field: "bettingType",
            })?,
            bsp_market: self.bsp_market.ok_or(DataError {
                missing_field: "bspMarket",
            })?,
            bsp_reconciled: self.bsp_reconciled.ok_or(DataError {
                missing_field: "bspReconciled",
            })?,
            complete: self.complete.ok_or(DataError {
                missing_field: "complete",
            })?,
            cross_matching: self.cross_matching.ok_or(DataError {
                missing_field: "crossMatching",
            })?,
            discount_allowed: self.discount_allowed.ok_or(DataError {
                missing_field: "discountAllowed",
            })?,
            event_id: self.event_id.ok_or(DataError {
                missing_field: "eventId",
            })?,
            event_type_id: self.event_type_id.ok_or(DataError {
                missing_field: "eventTypeId",
            })?,
            in_play: self.in_play.ok_or(DataError {
                missing_field: "inPlay",
            })?,
            market_base_rate: self.market_base_rate.ok_or(DataError {
                missing_field: "marketBaseRate",
            })?,
            number_of_winners: self.number_of_winners.ok_or(DataError {
                missing_field: "numberOfWinners",
            })?,
            persistence_enabled: self.persistence_enabled.ok_or(DataError {
                missing_field: "persistenceEnabled",
            })?,
            runners_voidable: self.runners_voidable.ok_or(DataError {
                missing_field: "runnersVoidable",
            })?,
            version: self.version.ok_or(DataError {
                missing_field: "version",
            })?,
            status: self.status.ok_or(DataError {
                missing_field: "status",
            })?,
            turn_in_play_enabled: self.turn_in_play_enabled.ok_or(DataError {
                missing_field: "turnInPlayEnabled",
            })?,
            number_of_active_runners: self.number_of_active_runners.ok_or(DataError {
                missing_field: "numberOfActiveRunners",
            })?,
            market_time: self
                .market_time
                .map(|s| {
                    if **market.market_time == s {
                        market.market_time.clone()
                    } else {
                        SyncObj::new(DateTimeString::try_from(s).unwrap())
                    }
                })
                .ok_or(DataError {
                    missing_field: "marketTime",
                })?,
            market_type: self
                .market_type
                .map(|s| {
                    if market.market_type.as_ref() == s {
                        market.market_type.clone()
                    } else {
                        SyncObj::new(Arc::from(s))
                    }
                })
                .ok_or(DataError {
                    missing_field: "marketType",
                })?,
            regulators: self
                .regulators
                .map(|v| {
                    if v.iter().eq(market.regulators.iter()) {
                        market.regulators.clone()
                    } else {
                        SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect()))
                    }
                })
                .ok_or(DataError {
                    missing_field: "regulators",
                })?,
            timezone: self
                .timezone
                .map(|s| {
                    if market.timezone.as_ref() == s {
                        market.timezone.clone()
                    } else {
                        SyncObj::new(Arc::from(s))
                    }
                })
                .ok_or(DataError {
                    missing_field: "timezone",
                })?,
            venue: self.venue.and_then(|n| {
                if market.venue.is_some_and(|v| v.as_ref() == n) {
                    market.venue.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n)))
                }
            }),
            country_code: self.country_code.and_then(|s| {
                if market.country_code.is_some_and(|v| v.as_ref() == s) {
                    market.country_code.clone()
                } else {
                    Some(SyncObj::new(FixedSizeString::try_from(s).unwrap()))
                }
            }),
            open_date: self
                .open_date
                .map(|s| {
                    if **market.open_date == s {
                        market.open_date.clone()
                    } else {
                        SyncObj::new(DateTimeString::try_from(s).unwrap())
                    }
                })
                .ok_or(DataError {
                    missing_field: "openDate",
                })?,
            settled_time: self.settled_time.and_then(|s| {
                if market.settled_time.is_some_and(|st| s == **st) {
                    market.settled_time.clone()
                } else {
                    Some(SyncObj::new(DateTimeString::try_from(s).unwrap()))
                }
            }),
            suspend_time: self.suspend_time.and_then(|s| {
                if market.suspend_time.is_some_and(|st| s == **st) {
                    market.suspend_time.clone()
                } else {
                    Some(SyncObj::new(DateTimeString::try_from(s).unwrap()))
                }
            }),
            market_name: self.market_name.and_then(|n| {
                if market.market_name.is_some_and(|name| name.as_ref() == n.as_ref()) {
                    market.market_name.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n.as_ref())))
                }
            }),
            event_name: self.event_name.and_then(|n| {
                if market
                    .event_name
                    .is_some_and(|ename| ename.as_ref() == n.as_ref())
                {
                    market.event_name.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n.as_ref())))
                }
            }),
            race_type: self.race_type.and_then(|n| {
                if market.race_type.is_some_and(|rt| rt.as_ref() == n) {
                    market.race_type.clone()
                } else {
                    Some(SyncObj::new(Arc::from(n)))
                }
            }),
            each_way_divisor: self.each_way_divisor,
        })
    }
}

pub struct MarketDefinitionDeser<'a, 'py> {
    pub def: Option<&'a MarketDefinition>,
    pub runners: Option<&'a [Py<Runner>]>,
    pub next_runners: Option<Vec<Py<Runner>>>,
    pub py: Python<'py>,
    pub config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketDefinitionDeser<'a, 'py> {
    type Value = (Option<Arc<MarketDefinition>>, Option<Vec<Py<Runner>>>);

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
            def: Option<&'a MarketDefinition>,
            runners: Option<&'a [Py<Runner>]>,
            next_runners: Option<Vec<Py<Runner>>>,
            py: Python<'py>,
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketDefinitionVisitor<'a, 'py> {
            type Value = (Option<Arc<MarketDefinition>>, Option<Vec<Py<Runner>>>);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut upt: MarketDefinitionUpdate = MarketDefinitionUpdate::default();
                let mut next_runners: Option<Vec<Py<Runner>>> = self.next_runners;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Runners => {
                            next_runners = map.next_value_seed(RunnerDefSeq {
                                runners: self.runners,
                                next: next_runners,
                                py: self.py,
                                config: self.config,
                            })?;
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
                            upt.market_name = Some(map.next_value::<Cow<str>>()?);
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
                            upt.market_time = Some(map.next_value::<FixedSizeString<24>>()?);
                        }
                        Field::SuspendTime => {
                            upt.suspend_time = Some(map.next_value::<FixedSizeString<24>>()?);
                        }
                        Field::SettledTime => {
                            upt.settled_time = Some(map.next_value::<FixedSizeString<24>>()?);
                        }
                        Field::OpenDate => {
                            upt.open_date = Some(map.next_value::<FixedSizeString<24>>()?);
                        }
                        Field::EachWayDivisor => {
                            upt.each_way_divisor = Some(map.next_value::<f64>()?);
                        }
                        Field::RaceType => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                        Field::KeyLineDefiniton => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                        Field::PriceLadderDefinition => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                        Field::LineMaxUnit => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                        Field::LineMinUnit => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                        Field::LineInterval => {
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let next_def = match self.def {
                    Some(def) => upt.update(def),
                    None => upt.create(),
                }
                .map_err(|err| {
                    Error::custom(format!("missing required field <{}>", err.missing_field))
                })?;

                let next_def = Some(Arc::new(next_def));

                Ok((next_def, next_runners))
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
                next_runners: self.next_runners,
                py: self.py,
                config: self.config,
            },
        )
    }
}

pub struct RunnerDefSeq<'a, 'py> {
    pub runners: Option<&'a [Py<Runner>]>,
    pub next: Option<Vec<Py<Runner>>>,
    pub py: Python<'py>,
    pub config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for RunnerDefSeq<'a, 'py> {
    type Value = Option<Vec<Py<Runner>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RunnerSeqVisitor<'a, 'py> {
            runners: Option<&'a [Py<Runner>]>,
            next: Option<Vec<Py<Runner>>>,
            py: Python<'py>,
            #[allow(dead_code)]
            config: Config,
        }
        impl<'de, 'a, 'py> Visitor<'de> for RunnerSeqVisitor<'a, 'py> {
            type Value = Option<Vec<Py<Runner>>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                match self.next {
                    // if we already have an in progress array to mutate
                    // this code *may* never run as definitions appear to always come
                    // before changes - but its better to be robust just in case.
                    // does mean this code hasnt been properly vetted
                    Some(mut n) => {
                        let mut i: usize = 0;

                        while let Some(change) = seq.next_element::<RunnerDefUpdate>()? {
                            // find runner and index
                            enum Action {
                                Insert(Py<Runner>, usize),
                                Swap(usize, usize),
                                Nothing,
                            }

                            let sid = SelectionID::from((change.id, change.hc));

                            let action = {
                                let runner_index = n
                                    .get(i)
                                    .and_then(|r| {
                                        let r = r.borrow_mut(self.py);
                                        (r.selection_id == sid).then_some((r, i))
                                    })
                                    .or_else(|| {
                                        n.iter()
                                            .position(|r| r.borrow_mut(self.py).selection_id == sid)
                                            .and_then(|pos| {
                                                n.get(pos).map(|r| (r.borrow_mut(self.py), pos))
                                            })
                                    });

                                // if present mutate them inplace and check if theyre in the right order
                                match runner_index {
                                    Some((r, j)) => {
                                        change.update_mut(r, self.py);

                                        if i != j {
                                            Action::Swap(i, j)
                                        } else {
                                            Action::Nothing
                                        }
                                    }
                                    None => Action::Insert(
                                        Py::new(self.py, change.create(self.py)).unwrap(),
                                        i,
                                    ),
                                }
                            };

                            match action {
                                Action::Insert(r, i) => n.insert(i, r),
                                Action::Swap(a, b) => n.swap(a, b),
                                Action::Nothing => {}
                            }

                            i += 1; // update deserialize index
                        }

                        Ok(Some(n))
                    }

                    // no previous runner serialization this update, serialize in rd order
                    None => {
                        let mut next: Option<Vec<Py<Runner>>> = None;
                        let mut i = 0;

                        while let Some(change) = seq.next_element::<RunnerDefUpdate>()? {
                            let sid = SelectionID::from((change.id, change.hc));

                            let r = self.runners.and_then(|rs| {
                                rs.iter()
                                    .position(|r| r.borrow(self.py).selection_id == sid)
                                    .map(|i| (unsafe { rs.get_unchecked(i) }, i))
                            });

                            match (r, next.as_mut()) {
                                (Some((r, ri)), None) => {
                                    let sel = r.borrow(self.py);

                                    if change.diff(&sel, self.py) {
                                        let rs = self.runners.unwrap();
                                        let mut n: Vec<Py<Runner>> =
                                            Vec::with_capacity(rs.len() + 1);
                                        for r in &rs[0..i] {
                                            n.push(r.clone_ref(self.py));
                                        }

                                        let r =
                                            Py::new(self.py, change.update(&sel, self.py)).unwrap();
                                        n.push(r);

                                        next = Some(n);
                                    } else if ri != i {
                                        let rs = self.runners.unwrap();
                                        let mut n: Vec<Py<Runner>> =
                                            Vec::with_capacity(rs.len() + 1);
                                        for r in &rs[0..i] {
                                            n.push(r.clone_ref(self.py));
                                        }

                                        let r = r.clone();
                                        n.push(r);

                                        next = Some(n)
                                    }
                                }

                                (Some((r, _ri)), Some(n)) => {
                                    let sel = r.borrow(self.py);

                                    let nr = if change.diff(&sel, self.py) {
                                        Py::new(self.py, change.update(&sel, self.py)).unwrap()
                                    } else {
                                        r.clone_ref(self.py)
                                    };

                                    n.push(nr);
                                }

                                (None, None) => {
                                    let mut n: Vec<Py<Runner>> = Vec::with_capacity(
                                        self.runners.map(|n| n.len() + 1).unwrap_or(12),
                                    );
                                    if let Some(rs) = self.runners {
                                        for r in &rs[0..i] {
                                            n.push(r.clone_ref(self.py));
                                        }
                                    }
                                    let r = Py::new(self.py, change.create(self.py)).unwrap();
                                    n.push(r);

                                    next = Some(n);
                                }

                                (None, Some(n)) => {
                                    let r = Py::new(self.py, change.create(self.py)).unwrap();
                                    n.push(r);
                                }
                            }

                            i += 1;
                        }

                        Ok(next)
                    }
                }
            }
        }

        deserializer.deserialize_seq(RunnerSeqVisitor {
            runners: self.runners,
            next: self.next,
            py: self.py,
            config: self.config,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunnerDefUpdate<'a> {
    id: u32,
    adjustment_factor: Option<f64>,
    status: SelectionStatus,
    sort_priority: u16,
    name: Option<&'a str>,
    bsp: Option<F64OrStr>,
    removal_date: Option<FixedSizeString<24>>,
    hc: Option<f32>,
}

impl<'a> RunnerDefUpdate<'a> {
    fn create(&self, py: Python) -> Runner {
        let sp = RunnerBookSP {
            actual_sp: self.bsp.map(|f| *f),
            ..Default::default()
        };

        let sid = SelectionID::from((self.id, self.hc));

        Runner {
            selection_id: sid,
            status: self.status,
            adjustment_factor: self.adjustment_factor,
            sort_priority: self.sort_priority,
            name: self.name.map(|s| SyncObj::new(Arc::from(s))),
            removal_date: self
                .removal_date
                .map(|s| SyncObj::new(DateTimeString::try_from(s).unwrap())),
            sp: Py::new(py, sp).unwrap(),
            ex: Py::new(py, RunnerBookEX::default()).unwrap(),
            total_matched: 0.0,
            last_price_traded: None,
        }
    }

    fn diff(&self, runner: &Runner, py: Python) -> bool {
        runner.status != self.status
            || runner.adjustment_factor != self.adjustment_factor
            || runner.sort_priority != self.sort_priority
            || runner.sp.borrow(py).actual_sp != self.bsp.map(|f| *f)
            || ((runner.name.is_none() && self.name.is_some())
                || runner
                    .name
                    .is_some_and(|s| !self.name.contains(&s.as_ref())))
            || ((runner.removal_date.is_some() != self.removal_date.is_some())
                || runner
                    .removal_date
                    .is_some_and(|s| !self.removal_date.contains(&s.as_str())))
    }

    fn update(&self, runner: &Runner, py: Python) -> Runner {
        // need to update sp obj with bsp value if it's changed
        let sp = {
            let sp = runner.sp.borrow(py);
            if sp.actual_sp != self.bsp.map(|f| *f) {
                Py::new(
                    py,
                    RunnerBookSP {
                        actual_sp: self.bsp.map(|f| *f),
                        far_price: sp.far_price,
                        near_price: sp.near_price,
                        back_stake_taken: sp.back_stake_taken.clone(),
                        lay_liability_taken: sp.lay_liability_taken.clone(),
                    },
                )
                .unwrap()
            } else {
                runner.sp.clone_ref(py)
            }
        };

        Runner {
            selection_id: runner.selection_id,
            status: self.status,
            adjustment_factor: self.adjustment_factor,
            sort_priority:  self.sort_priority,
            name: self
                .name
                .and_then(|n| {
                    if runner.name.contains(&n) {
                        runner.name.clone()
                    } else {
                        Some(SyncObj::new(Arc::from(n)))
                    }
                })
                .or_else(|| runner.name.clone()),

            removal_date: self
                .removal_date
                .and_then(|n| {
                    if runner.removal_date.contains(&n.as_ref()) {
                        runner.removal_date.clone()
                    } else {
                        Some(SyncObj::new(DateTimeString::try_from(n).unwrap()))
                    }
                }),
            total_matched: runner.total_matched,
            last_price_traded: runner.last_price_traded,
            ex: runner.ex.clone(),
            sp,
        }
    }

    fn update_mut(&self, mut runner: PyRefMut<Runner>, py: Python) {
        let sp = {
            let sp = runner.sp.borrow(py);
            if sp.actual_sp != self.bsp.map(|f| *f) {
                Some(
                    Py::new(
                        py,
                        RunnerBookSP {
                            actual_sp: self.bsp.map(|f| *f),
                            far_price: sp.far_price,
                            near_price: sp.near_price,
                            back_stake_taken: sp.back_stake_taken.clone(),
                            lay_liability_taken: sp.lay_liability_taken.clone(),
                        },
                    )
                    .unwrap(),
                )
            } else {
                None
            }
        };

        if let Some(sp) = sp {
            runner.sp = sp;
        }
        if runner.status != self.status {
            runner.status = self.status;
        }
        if runner.adjustment_factor != self.adjustment_factor {
            runner.adjustment_factor = self.adjustment_factor;
        }
        if runner.sort_priority != self.sort_priority {
            runner.sort_priority = self.sort_priority
        }
        if let Some(n) = self.name && !runner.name.contains(&n) {
            runner.name = Some(SyncObj::new(Arc::from(n)));
        }
        if (self.removal_date.is_some() != runner.removal_date.is_some()) || self.removal_date.is_some_and(|s| !runner.removal_date.contains(&s.as_ref())) {
            match self.removal_date {
                Some(s) => { runner.removal_date = Some(SyncObj::new(DateTimeString::try_from(s).unwrap())); },
                None => { runner.removal_date = None; }
            }
        }

    }
}
