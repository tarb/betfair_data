use pyo3::prelude::*;
use serde::{
    de::{self, DeserializeSeed, Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::borrow::Cow;
use std::fmt;

use crate::config::Config;
use crate::datetime::DateTimeString;
use crate::enums::{MarketBettingType, MarketStatus};
use crate::ids::{EventID, EventTypeID};
use crate::mutable::runner::{Runner, RunnerDefSeqDeser};
use crate::strings::{FixedSizeString, StringSetExtNeq};

#[derive(Default, Clone)]
pub struct MarketDefinition {
    pub bet_delay: u16,
    pub bsp_market: bool,
    pub bsp_reconciled: bool,
    pub complete: bool,
    pub cross_matching: bool,
    pub discount_allowed: bool,
    pub each_way_divisor: Option<f64>,
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
    pub country_code: FixedSizeString<2>,
    pub regulators: Vec<String>,
}

// Used for serializing in place over the mc marketDefinition object
pub struct MarketDefinitionDeser<'a, 'py> {
    pub def: &'a mut MarketDefinition,
    pub runners: &'a mut Vec<Py<Runner>>,
    pub config: Config,
    pub py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketDefinitionDeser<'a, 'py> {
    type Value = ();

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
            def: &'a mut MarketDefinition,
            runners: &'a mut Vec<Py<Runner>>,
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for MarketDefinitionVisitorDeser<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                // we need to make sure we see these fields - every field not marked optional
                // should be present every time (including these). But some of these fields
                // are sometimes missing. We should error on that case
                let mut country_code = false;
                let mut market_type = false;
                let mut market_time = false;
                let mut open_date = false;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::BspMarket => self.def.bsp_market = map.next_value()?,
                        Field::TurnInPlayEnabled => {
                            self.def.turn_in_play_enabled = map.next_value()?
                        }
                        Field::InPlay => self.def.in_play = map.next_value()?,
                        Field::PersistenceEnabled => {
                            self.def.persistence_enabled = map.next_value()?
                        }
                        Field::BspReconciled => self.def.bsp_reconciled = map.next_value()?,
                        Field::Complete => self.def.complete = map.next_value()?,
                        Field::CrossMatching => self.def.cross_matching = map.next_value()?,
                        Field::RunnersVoidable => self.def.runners_voidable = map.next_value()?,
                        Field::DiscountAllowed => self.def.discount_allowed = map.next_value()?,
                        Field::Timezone => {
                            self.def.timezone.set_if_ne(map.next_value::<&str>()?);
                        }
                        Field::Name => {
                            self.def
                                .market_name
                                .set_if_ne(map.next_value::<Cow<str>>()?);
                        }
                        Field::EventName => {
                            self.def.event_name.set_if_ne(map.next_value::<Cow<str>>()?);
                        }
                        Field::CountryCode => {
                            self.def.country_code = map.next_value::<FixedSizeString<2>>()?;
                            country_code = true;
                        }
                        Field::Venue => {
                            self.def.venue.set_if_ne(map.next_value::<Cow<str>>()?);
                        }
                        Field::Status => self.def.status = map.next_value()?,
                        Field::MarketBaseRate => {
                            self.def.market_base_rate = map.next_value::<f32>()?
                        }
                        Field::NumberOfWinners => {
                            self.def.number_of_winners = map.next_value::<f32>()? as u8
                        }
                        Field::NumberOfActiveRunners => {
                            self.def.number_of_active_runners = map.next_value()?
                        }
                        Field::BetDelay => self.def.bet_delay = map.next_value()?,
                        Field::EventId => {
                            self.def.event_id = map
                                .next_value::<&str>()?
                                .parse()
                                .map_err(de::Error::custom)?;
                        }
                        Field::EventTypeId => {
                            self.def.event_type_id = map
                                .next_value::<&str>()?
                                .parse()
                                .map_err(de::Error::custom)?;
                        }
                        Field::Version => self.def.version = map.next_value()?,
                        Field::Runners => map.next_value_seed(RunnerDefSeqDeser {
                            runners: self.runners,
                            config: self.config,
                            py: self.py,
                        })?,
                        Field::MarketType => {
                            self.def.market_type.set_if_ne(map.next_value::<&str>()?);
                            market_type = true;
                        }
                        Field::BettingType => self.def.betting_type = map.next_value()?,
                        Field::EachWayDivisor => {
                            self.def.each_way_divisor = Some(map.next_value::<f64>()?)
                        }
                        Field::MarketTime => {
                            let s = map.next_value::<&str>()?;
                            if &self.def.market_time != s {
                                let dt = DateTimeString::new(s).map_err(de::Error::custom)?;

                                self.def.market_time = dt;
                            }
                            market_time = true;
                        }
                        Field::SuspendTime => {
                            let s = map.next_value::<&str>()?;
                            if !self.def.suspend_time.contains(&s) {
                                let dt = DateTimeString::new(s).map_err(de::Error::custom)?;
                                self.def.suspend_time = Some(dt);
                            }
                        }
                        Field::SettledTime => {
                            let s = map.next_value::<&str>()?;
                            if !self.def.settled_time.contains(&s) {
                                let dt = DateTimeString::new(s).map_err(de::Error::custom)?;
                                self.def.settled_time = Some(dt);
                            }
                        }
                        Field::OpenDate => {
                            let s = map.next_value::<&str>()?;
                            if &self.def.open_date != s {
                                let dt = DateTimeString::new(s).map_err(de::Error::custom)?;
                                self.def.open_date = dt;
                            }
                            open_date = true;
                        }
                        Field::Regulators => {
                            if self.def.regulators.is_empty() {
                                self.def.regulators = map.next_value::<Vec<String>>()?;
                            } else {
                                let regs = map.next_value::<Vec<&str>>()?;

                                if self.def.regulators.iter().ne(regs.iter()) {
                                    self.def.regulators =
                                        regs.iter().map(|s| s.to_string()).collect();
                                }
                            }
                        }

                        // after searching over 200k markets, I cant find these values in any data sets :/
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

                // these are required fields that should always be present on a market - but sometimes
                // are missing. We should error on this case
                if !country_code {
                    Err(Error::custom("missing required field <countryCode>"))
                } else if !market_type {
                    Err(Error::custom("missing required field <marketType>"))
                } else if !market_time {
                    Err(Error::custom("missing required field <marketTime>"))
                } else if !open_date {
                    Err(Error::custom("missing required field <openDate>"))
                } else {
                    Ok(())
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
