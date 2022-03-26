use pyo3::prelude::*;
use serde::{
    de::{self, DeserializeSeed, Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::borrow::Cow;
use std::fmt;

use crate::datetime::DateTimeString;
use crate::enums::{MarketBettingType, MarketStatus};
use crate::ids::{EventID, EventTypeID};
use crate::mutable::runner::{Runner, RunnerDefSeqDeser};
use crate::strings::{FixedSizeString, StringSetExtNeq};
use crate::{config::Config, errors::DataError};

#[derive(Clone)]
pub struct MarketDefinition {
    pub bet_delay: u16,
    pub bsp_market: bool,
    pub bsp_reconciled: bool,
    pub complete: bool,
    pub cross_matching: bool,
    pub discount_allowed: bool,
    pub event_id: EventID,
    pub event_name: Option<String>,
    pub event_type_id: EventTypeID,
    pub in_play: bool,
    pub market_base_rate: f32,
    pub market_type: String,
    pub market_name: Option<String>,
    pub number_of_active_runners: u16,
    pub number_of_winners: u8,
    pub persistence_enabled: bool,
    pub runners_voidable: bool,
    pub timezone: String,
    pub turn_in_play_enabled: bool,
    pub venue: Option<String>,
    pub version: u64,
    pub status: MarketStatus,
    pub betting_type: MarketBettingType,
    pub market_time: DateTimeString,
    pub open_date: DateTimeString,
    pub suspend_time: Option<DateTimeString>,
    pub settled_time: Option<DateTimeString>,
    pub country_code: Option<FixedSizeString<2>>,
    pub regulators: Vec<String>,
    pub race_type: Option<String>,
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
    race_type: Option<&'a str>,
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
                .map(|v| v.iter().map(|s| s.to_string()).collect())
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
                .map(|s| DateTimeString::try_from(s).unwrap())
                .ok_or(DataError {
                    missing_field: "marketTime",
                })?,
            market_type: self.market_type.map(|s| s.to_owned()).ok_or(DataError {
                missing_field: "marketType",
            })?,
            timezone: self.timezone.map(|s| s.to_owned()).ok_or(DataError {
                missing_field: "timezone",
            })?,
            venue: self.venue.map(|s| s.to_owned()),
            country_code: self
                .country_code
                .map(|s| FixedSizeString::try_from(s).unwrap()), // too
            open_date: self
                .open_date
                .map(|s| DateTimeString::try_from(s).unwrap())
                .ok_or(DataError {
                    missing_field: "openDate",
                })?,
            settled_time: self.settled_time.map(|s| DateTimeString::try_from(s).unwrap()),
            suspend_time: self.suspend_time.map(|s| DateTimeString::try_from(s).unwrap()),
            market_name: self.market_name.map(|s| s.into_owned()),
            event_name: self.event_name.map(|s| s.into_owned()),
            race_type: self.race_type.map(|s| s.to_string()),
            each_way_divisor: self.each_way_divisor,
        })
    }

    fn update(self, market: &mut MarketDefinition) -> Result<(), DataError> {
        market.bet_delay = self.bet_delay.ok_or(DataError {
            missing_field: "betDelay",
        })?;
        market.betting_type = self.betting_type.ok_or(DataError {
            missing_field: "bettingType",
        })?;
        market.bsp_market = self.bsp_market.ok_or(DataError {
            missing_field: "bspMarket",
        })?;
        market.bsp_reconciled = self.bsp_reconciled.ok_or(DataError {
            missing_field: "bspReconciled",
        })?;
        market.complete = self.complete.ok_or(DataError {
            missing_field: "complete",
        })?;
        market.cross_matching = self.cross_matching.ok_or(DataError {
            missing_field: "crossMatching",
        })?;
        market.discount_allowed = self.discount_allowed.ok_or(DataError {
            missing_field: "discountAllowed",
        })?;
        market.event_id = self.event_id.ok_or(DataError {
            missing_field: "eventId",
        })?;
        market.event_type_id = self.event_type_id.ok_or(DataError {
            missing_field: "eventTypeId",
        })?;
        market.in_play = self.in_play.ok_or(DataError {
            missing_field: "inPlay",
        })?;
        market.market_base_rate = self.market_base_rate.ok_or(DataError {
            missing_field: "marketBaseRate",
        })?;
        market.number_of_winners = self.number_of_winners.ok_or(DataError {
            missing_field: "numberOfWinners",
        })?;
        market.persistence_enabled = self.persistence_enabled.ok_or(DataError {
            missing_field: "persistenceEnabled",
        })?;
        market.runners_voidable = self.runners_voidable.ok_or(DataError {
            missing_field: "runnersVoidable",
        })?;
        market.version = self.version.ok_or(DataError {
            missing_field: "version",
        })?;
        market.status = self.status.ok_or(DataError {
            missing_field: "status",
        })?;
        market.turn_in_play_enabled = self.turn_in_play_enabled.ok_or(DataError {
            missing_field: "turnInPlayEnabled",
        })?;
        market.number_of_active_runners = self.number_of_active_runners.ok_or(DataError {
            missing_field: "numberOfActiveRunners",
        })?;
        market.timezone.set_if_ne(self.timezone.ok_or(DataError {
            missing_field: "timezone",
        })?);
        market
            .market_type
            .set_if_ne(self.market_type.ok_or(DataError {
                missing_field: "marketType",
            })?);
        market.market_time = self
            .market_time
            .map(|s| {
                if market.market_time.as_str() != s {
                    DateTimeString::try_from(s).unwrap()
                } else {
                    market.market_time
                }
            })
            .ok_or(DataError {
                missing_field: "marketTime",
            })?;

        market.open_date = self
            .open_date
            .map(|s| {
                if market.open_date.as_str() != s {
                    DateTimeString::try_from(s).unwrap()
                } else {
                    market.open_date
                }
            })
            .ok_or(DataError {
                missing_field: "openDate",
            })?;
        let regs = self.regulators.ok_or(DataError {
            missing_field: "regulators",
        })?;
        if regs.iter().ne(market.regulators.iter()) {
            market.regulators = regs.iter().map(|s| s.to_string()).collect();
        }

        // below fields are optionally required

        // country code should be requried, but is missing ofter :/
        market.country_code = self
            .country_code
            .map(|cc| FixedSizeString::try_from(cc).unwrap());

        market.settled_time = self.settled_time.and_then(|s| match market.settled_time {
            Some(dts) if s != dts => Some(DateTimeString::try_from(s).unwrap()),
            None => Some(DateTimeString::try_from(s).unwrap()),
            _ => market.settled_time,
        });

        market.suspend_time = self.suspend_time.and_then(|s| match market.suspend_time {
            Some(dts) if s != dts => Some(DateTimeString::try_from(s).unwrap()),
            None => Some(DateTimeString::try_from(s).unwrap()),
            _ => market.suspend_time,
        });

        market.each_way_divisor = self.each_way_divisor;

        if let Some(race_type) = self.race_type {
            market.race_type.set_if_ne(race_type);
        } else {
            market.race_type = None;
        }
        if let Some(venue) = self.venue {
            market.venue.set_if_ne(venue);
        } else {
            market.race_type = None;
        }
        if let Some(market_name) = self.market_name {
            market.market_name.set_if_ne(market_name);
        } else {
            market.race_type = None;
        }
        if let Some(event_name) = self.event_name {
            market.event_name.set_if_ne(event_name);
        } else {
            market.race_type = None;
        }

        Ok(())
    }
}

// Used for serializing in place over the mc marketDefinition object
pub struct MarketDefinitionDeser<'a, 'py> {
    pub def: Option<&'a mut MarketDefinition>,
    pub runners: &'a mut Vec<Py<Runner>>,
    pub config: Config,
    pub py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketDefinitionDeser<'a, 'py> {
    type Value = Option<MarketDefinition>;

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

        struct MarketDefinitionVisitorDeser<'a, 'py> {
            def: Option<&'a mut MarketDefinition>,
            runners: &'a mut Vec<Py<Runner>>,
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketDefinitionVisitorDeser<'a, 'py> {
            type Value = Option<MarketDefinition>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut upt: MarketDefinitionUpdate = MarketDefinitionUpdate::default();

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Runners => map.next_value_seed(RunnerDefSeqDeser {
                            runners: self.runners,
                            config: self.config,
                            py: self.py,
                        })?,
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
                            upt.race_type = Some(map.next_value::<&str>()?);
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

                match self.def {
                    Some(def) => {
                        upt.update(def).map_err(|err| {
                            Error::custom(format!("missing required field <{}>", err.missing_field))
                        })?;

                        Ok(None)
                    }
                    None => {
                        let def = upt.create().map_err(|err| {
                            Error::custom(format!("missing required field <{}>", err.missing_field))
                        })?;

                        Ok(Some(def))
                    }
                }
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
            MarketDefinitionVisitorDeser {
                def: self.def,
                runners: self.runners,
                config: self.config,
                py: self.py,
            },
        )
    }
}
