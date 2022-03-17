use core::fmt;
use pyo3::prelude::*;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::borrow::Cow;
use std::sync::Arc;

use super::runner_book_ex::RunnerBookEX;
use super::runner_book_sp::{RunnerBookSP, RunnerBookSPUpdate};
use crate::config::Config;
use crate::datetime::DateTimeString;
use crate::enums::{MarketBettingType, MarketStatus, SelectionStatus};
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
    pub each_way_divisor: Option<f64>,
    pub event_id: EventID,
    pub event_name: Option<SyncObj<Arc<str>>>,
    pub event_type_id: EventTypeID,
    pub in_play: bool,
    pub market_base_rate: f32,
    pub market_type: SyncObj<Arc<str>>,
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
    pub country_code: SyncObj<FixedSizeString<2>>,
    pub regulators: SyncObj<Arc<Vec<String>>>,
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
    runners_voidable: Option<bool>,
    settled_time: Option<&'a str>,
    status: Option<MarketStatus>,
    suspend_time: Option<&'a str>,
    timezone: Option<&'a str>,
    turn_in_play_enabled: Option<bool>,
    venue: Option<&'a str>,
    version: Option<u64>,
    country_code: Option<&'a str>,
    market_name: Option<Cow<'a, str>>,
    event_name: Option<Cow<'a, str>>,
}

impl<'a, 'b> MarketDefinitionUpdate<'a> {
    fn create(self) -> MarketDefinition {
        MarketDefinition {
            each_way_divisor: Default::default(),
            bet_delay: self.bet_delay.unwrap_or_default(),
            betting_type: self.betting_type.unwrap_or_default(),
            regulators: self
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .unwrap_or_default(),
            bsp_reconciled: self.bsp_reconciled.unwrap_or_default(),
            bsp_market: self.bsp_market.unwrap_or_default(),
            complete: self.complete.unwrap_or_default(),
            cross_matching: self.cross_matching.unwrap_or_default(),
            discount_allowed: self.discount_allowed.unwrap_or_default(),
            event_id: self.event_id.unwrap_or_default(),
            event_type_id: self.event_type_id.unwrap_or_default(),
            in_play: self.in_play.unwrap_or_default(),
            market_base_rate: self.market_base_rate.unwrap_or_default(),
            number_of_winners: self.number_of_winners.unwrap_or_default(),
            persistence_enabled: self.persistence_enabled.unwrap_or_default(),
            runners_voidable: self.runners_voidable.unwrap_or_default(),
            version: self.version.unwrap_or_default(),
            status: self.status.unwrap_or_default(),
            turn_in_play_enabled: self.turn_in_play_enabled.unwrap_or_default(),
            number_of_active_runners: self.number_of_active_runners.unwrap_or_default(),
            market_time: self
                .market_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap(),
            market_type: self
                .market_type
                .map(|s| SyncObj::new(Arc::from(s)))
                .unwrap(),
            timezone: self.timezone.map(|s| SyncObj::new(Arc::from(s))).unwrap(),
            venue: self.venue.map(|s| SyncObj::new(Arc::from(s))),
            country_code: self
                .country_code
                .map(|s| SyncObj::new(FixedSizeString::try_from(s).unwrap()))
                .unwrap(), // todo
            open_date: self
                .open_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap(),
            settled_time: self
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            suspend_time: self
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
            market_name: self
                .market_name
                .map(|s| SyncObj::new(Arc::from(s.into_owned()))),
            event_name: self
                .event_name
                .map(|s| SyncObj::new(Arc::from(s.into_owned()))),
        }
    }

    fn update(self, market: &MarketDefinition) -> MarketDefinition {
        MarketDefinition {
            each_way_divisor: market.each_way_divisor,
            bet_delay: self.bet_delay.unwrap_or(market.bet_delay),
            betting_type: self.betting_type.unwrap_or(market.betting_type),
            regulators: self
                .regulators
                .map(|v| SyncObj::new(Arc::new(v.iter().map(|s| s.to_string()).collect())))
                .unwrap_or_else(|| market.regulators.clone()),
            bsp_reconciled: self.bsp_reconciled.unwrap_or(market.bsp_reconciled),
            bsp_market: self.bsp_market.unwrap_or(market.bsp_market),
            complete: self.complete.unwrap_or(market.complete),
            cross_matching: self.cross_matching.unwrap_or(market.cross_matching),
            discount_allowed: self.discount_allowed.unwrap_or(market.discount_allowed),
            event_id: self.event_id.unwrap_or(market.event_id),
            event_type_id: self.event_type_id.unwrap_or(market.event_type_id),
            in_play: self.in_play.unwrap_or(market.in_play),
            market_base_rate: self.market_base_rate.unwrap_or(market.market_base_rate),
            number_of_winners: self.number_of_winners.unwrap_or(market.number_of_winners),
            persistence_enabled: self
                .persistence_enabled
                .unwrap_or(market.persistence_enabled),
            runners_voidable: self.runners_voidable.unwrap_or(market.runners_voidable),
            version: self.version.unwrap_or(market.version),
            status: self.status.unwrap_or(market.status),
            turn_in_play_enabled: self
                .turn_in_play_enabled
                .unwrap_or(market.turn_in_play_enabled),
            number_of_active_runners: self
                .number_of_active_runners
                .unwrap_or(market.number_of_active_runners),
            market_time: self
                .market_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap_or_else(|| market.market_time.clone()),
            market_type: self
                .market_type
                .map(|s| SyncObj::new(Arc::from(s)))
                .unwrap_or_else(|| market.market_type.clone()),
            timezone: self
                .timezone
                .map(|s| SyncObj::new(Arc::from(s)))
                .unwrap_or_else(|| market.timezone.clone()),
            venue: self
                .venue
                .map(|s| SyncObj::new(Arc::from(s)))
                .or_else(|| market.venue.clone()),
            country_code: self
                .country_code
                .map(|s| SyncObj::new(FixedSizeString::try_from(s).unwrap()))
                .unwrap_or_else(|| market.country_code.clone()), // todo
            open_date: self
                .open_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .unwrap_or_else(|| market.open_date.clone()),
            settled_time: self
                .settled_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .or_else(|| market.settled_time.clone()),
            suspend_time: self
                .suspend_time
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap()))
                .or_else(|| market.suspend_time.clone()),
            market_name: self
                .market_name
                .map(|s| SyncObj::new(Arc::from(s.into_owned())))
                .or_else(|| market.market_name.clone()),
            event_name: self
                .event_name
                .map(|s| SyncObj::new(Arc::from(s.into_owned())))
                .or_else(|| market.event_name.clone()),
        }
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
                let mut changed = false;

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
                        Field::BspMarket => {
                            let bsp_market = map.next_value()?;
                            if self.def.is_some_and(|def| def.bsp_market != bsp_market)
                                || self.def.is_none()
                            {
                                upt.bsp_market = Some(bsp_market);
                                changed = true;
                            }
                        }
                        Field::Name => {
                            let market_name = map.next_value::<Cow<str>>()?;
                            if self.def.is_some_and(|def| {
                                !def.market_name.contains(&market_name.as_ref())
                            }) || self.def.is_none()
                            {
                                upt.market_name = Some(market_name);
                                changed = true;
                            }
                        }
                        Field::TurnInPlayEnabled => {
                            let turn_in_play_enabled = map.next_value()?;
                            if self.def.is_some_and(|def| {
                                def.turn_in_play_enabled != turn_in_play_enabled
                            }) || self.def.is_none()
                            {
                                upt.turn_in_play_enabled = Some(turn_in_play_enabled);
                                changed = true;
                            }
                        }
                        Field::InPlay => {
                            let in_play = map.next_value()?;
                            if self.def.is_some_and(|def| def.in_play != in_play)
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
                                .is_some_and(|def| def.persistence_enabled != persistence_enabled)
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
                                .is_some_and(|def| def.bsp_reconciled != bsp_reconciled)
                                || self.def.is_none()
                            {
                                upt.bsp_reconciled = Some(bsp_reconciled);
                                changed = true;
                            }
                        }
                        Field::Complete => {
                            let complete = map.next_value()?;
                            if self.def.is_some_and(|def| def.complete != complete)
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
                                .is_some_and(|def| def.cross_matching != cross_matching)
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
                                .is_some_and(|def| def.runners_voidable != runners_voidable)
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
                                .is_some_and(|def| def.discount_allowed != discount_allowed)
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
                                .is_some_and(|def| def.timezone.as_ref() != timezone)
                                || self.def.is_none()
                            {
                                upt.timezone = Some(timezone);
                                changed = true;
                            }
                        }

                        Field::EventName => {
                            let event_name = map.next_value::<Cow<str>>()?;
                            if self
                                .def
                                .is_some_and(|def| !def.event_name.contains(&event_name.as_ref()))
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
                                .is_some_and(|def| !def.country_code.contains(&country_code))
                                || self.def.is_none()
                            {
                                upt.country_code = Some(country_code);
                                changed = true;
                            }
                        }
                        Field::Venue => {
                            let venue = map.next_value::<&str>()?;
                            if self.def.is_some_and(|def| !def.venue.contains(&venue))
                                || self.def.is_none()
                            {
                                upt.venue = Some(venue);
                                changed = true;
                            }
                        }
                        Field::Status => {
                            let status = map.next_value()?;
                            if self.def.is_some_and(|def| def.status != status)
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
                                .is_some_and(|def| def.market_base_rate != market_base_rate)
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
                                .is_some_and(|def| def.number_of_winners != number_of_winners)
                                || self.def.is_none()
                            {
                                upt.number_of_winners = Some(number_of_winners);
                                changed = true;
                            }
                        }
                        Field::NumberOfActiveRunners => {
                            let number_of_active_runners = map.next_value()?;
                            if self.def.is_some_and(|def| {
                                def.number_of_active_runners != number_of_active_runners
                            }) || self.def.is_none()
                            {
                                upt.number_of_active_runners = Some(number_of_active_runners);
                                changed = true;
                            }
                        }
                        Field::BetDelay => {
                            let bet_delay = map.next_value()?;
                            if self.def.is_some_and(|def| def.bet_delay != bet_delay)
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
                            if self.def.is_some_and(|def| def.event_id != event_id)
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
                                .is_some_and(|def| def.event_type_id != event_type_id)
                                || self.def.is_none()
                            {
                                upt.event_type_id = Some(event_type_id);
                                changed = true;
                            }
                        }
                        Field::Version => {
                            let version = map.next_value()?;
                            if self.def.is_some_and(|def| def.version != version)
                                || self.def.is_none()
                            {
                                upt.version = Some(version);
                                changed = true;
                            }
                        }
                        Field::MarketType => {
                            let market_type = map.next_value::<&str>()?;
                            if self
                                .def
                                .is_some_and(|def| def.market_type.as_ref() != market_type)
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
                                .is_some_and(|def| def.betting_type != betting_type)
                            {
                                upt.betting_type = Some(betting_type);
                                changed = true;
                            }
                        }

                        Field::MarketTime => {
                            let market_time = map.next_value()?;
                            if self
                                .def
                                .is_some_and(|def| def.market_time.as_str() != market_time)
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
                                .is_some_and(|def| def.suspend_time.contains(&suspend_time))
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
                                .is_some_and(|def| def.settled_time.contains(&settled_time))
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
                                .is_some_and(|def| def.open_date.as_str() != open_date)
                                || self.def.is_none()
                            {
                                upt.open_date = Some(open_date);
                                changed = true;
                            }
                        }

                        Field::Regulators => {
                            let v = map.next_value::<Vec<&str>>()?;

                            if self.def.is_some_and(|def| {
                                (def.regulators.is_empty() && !v.is_empty())
                                    || !def.regulators.iter().eq(v.iter())
                            }) || self.def.is_none()
                            {
                                upt.regulators = Some(v);
                                changed = true;
                            }
                        }

                        Field::EachWayDivisor => {
                            map.next_value::<serde::de::IgnoredAny>()?;
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
                    Some(def) if changed => Some(Arc::new(upt.update(def))),
                    None if changed => Some(Arc::new(upt.create())),
                    _ => None,
                };

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
                            let action = {
                                let runner_index = n
                                    .get(i)
                                    .and_then(|r| {
                                        let r = r.borrow_mut(self.py);
                                        (r.selection_id == change.id).then_some((r, i))
                                    })
                                    .or_else(|| {
                                        n.iter()
                                            .position(|r| {
                                                r.borrow_mut(self.py).selection_id == change.id
                                            })
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
                            let r = self.runners.and_then(|rs| {
                                rs.iter()
                                    .position(|r| r.borrow(self.py).selection_id == change.id)
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
    id: SelectionID,
    adjustment_factor: Option<f64>,
    status: SelectionStatus,
    sort_priority: u16,
    name: Option<&'a str>,
    bsp: Option<F64OrStr>,
    removal_date: Option<&'a str>,
    hc: Option<F64OrStr>,
}

impl<'a> RunnerDefUpdate<'a> {
    fn create(&self, py: Python) -> Runner {
        let sp = RunnerBookSP {
            actual_sp: self.bsp.map(|f| *f),
            ..Default::default()
        };

        Runner {
            selection_id: self.id,
            status: self.status,
            adjustment_factor: self.adjustment_factor,
            handicap: self.hc.map(|f| *f),
            sort_priority: self.sort_priority,
            name: self.name.map(|s| SyncObj::new(Arc::from(s))),
            removal_date: self
                .removal_date
                .map(|s| SyncObj::new(DateTimeString::new(s).unwrap())),
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
            || runner.handicap != self.hc.map(|f| *f)
            || ((runner.name.is_none() && self.name.is_some())
                || runner
                    .name
                    .is_some_and(|s| !self.name.contains(&s.as_ref())))
            || ((runner.removal_date.is_none() && self.removal_date.is_some())
                || runner
                    .removal_date
                    .is_some_and(|s| !self.removal_date.contains(&s.as_str())))
    }

    fn update(&self, runner: &Runner, py: Python) -> Runner {
        let (ex, sp) = if self.status == SelectionStatus::Removed
            || self.status == SelectionStatus::RemovedVacant
        {
            (
                Py::new(py, RunnerBookEX::default()).unwrap(),
                runner.sp.clone_ref(py),
            )
        } else if self.bsp.is_some() {
            // need to update sp obj with bsp value
            let sp = runner.sp.borrow(py);
            if sp.actual_sp != self.bsp.map(|f| *f) {
                let upt = RunnerBookSPUpdate {
                    actual_sp: self.bsp.map(|f| *f),
                    ..Default::default()
                };
                (runner.ex.clone_ref(py), sp.update(upt, py))
            } else {
                (runner.ex.clone_ref(py), runner.sp.clone_ref(py))
            }
        } else {
            (runner.ex.clone_ref(py), runner.sp.clone_ref(py))
        };

        Runner {
            selection_id: runner.selection_id,
            status: self.status,
            adjustment_factor: self.adjustment_factor.or(runner.adjustment_factor),
            handicap: self.hc.map(|f| *f).or(runner.handicap),
            sort_priority: if runner.sort_priority != self.sort_priority {
                self.sort_priority
            } else {
                runner.sort_priority
            },
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
                    if runner.removal_date.contains(&n) {
                        runner.removal_date.clone()
                    } else {
                        Some(SyncObj::new(DateTimeString::new(n).unwrap()))
                    }
                })
                .or_else(|| runner.removal_date.clone()),
            total_matched: runner.total_matched,
            last_price_traded: runner.last_price_traded,
            sp,
            ex,
        }
    }

    fn update_mut(&self, mut runner: PyRefMut<Runner>, py: Python) {
        // maybe wipe ex & sp when removed status detected?

        if self.bsp.is_some() {
            let sp = {
                let sp = runner.sp.borrow(py);
                if sp.actual_sp != self.bsp.map(|f| *f) {
                    let upt = RunnerBookSPUpdate {
                        actual_sp: self.bsp.map(|f| *f),
                        ..Default::default()
                    };
                    Some(sp.update(upt, py))
                } else {
                    None
                }
            };

            if let Some(sp) = sp {
                runner.sp = sp;
            }
        }
        if runner.status != self.status {
            runner.status = self.status;
        }
        if runner.handicap != self.hc.map(|f| *f) {
            runner.handicap = self.hc.map(|f| *f);
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
        if let Some(n) = self.removal_date &&!runner.removal_date.contains(&n) {
            runner.removal_date = Some(SyncObj::new(DateTimeString::new(n).unwrap()));
        }
    }
}
