use std::fmt;
use pyo3::{prelude::*}; 
// use pyo3::types::PyDateTime;
use serde::de::IgnoredAny;
use staticvec::StaticString;
use serde::{ Deserialize, Deserializer, de::{ DeserializeSeed, Visitor, MapAccess }};
use serde_json::{Error };

use crate::strings::StringSetExtNeq;
use crate::ids::{MarketID, EventID, EventTypeID};
use crate::enums::{MarketStatus, MarketBettingType};
use crate::DeserErr;
use crate::SourceItem;
use crate::runner::{PyRunner, PyRunnerChangeSeq, PyRunnerDefSeq};
use crate::deser::{Deser, DeserializerWithData};

#[derive(Clone)]
#[pyclass(name="MarketImage", subclass)]
pub struct PyMarketBase {    
    #[pyo3(get)]
    source: String,
    #[pyo3(get)]
    file: String,
    #[pyo3(get)]
    bet_delay: u16,
    #[pyo3(get)]
    bsp_market: bool,
    #[pyo3(get)]
    bsp_reconciled: bool,
    #[pyo3(get)]
    clk: String,
    #[pyo3(get)]
    complete: bool,
    #[pyo3(get)]
    cross_matching: bool,
    #[pyo3(get)]
    discount_allowed: bool,
    #[pyo3(get)]
    each_way_divisor: Option<f64>,
    #[pyo3(get)]
    event_id: EventID,
    #[pyo3(get)]
    event_name: Option<String>,
    #[pyo3(get)]
    event_type_id: EventTypeID,
    #[pyo3(get)]
    in_play: bool,
    #[pyo3(get)]
    market_base_rate: u8,
    #[pyo3(get)]
    market_type: String,
    #[pyo3(get)]
    market_name: Option<String>,
    #[pyo3(get)]
    number_of_active_runners: u16,
    #[pyo3(get)]
    number_of_winners: u8,
    #[pyo3(get)]
    persistence_enabled: bool,
    #[pyo3(get)]
    publish_time: u64,
    #[pyo3(get)]
    runners_voidable: bool,
    #[pyo3(get)]
    timezone: String,
    #[pyo3(get)]
    total_matched: f64,
    #[pyo3(get)]
    turn_in_play_enabled: bool,
    #[pyo3(get)]
    venue: Option<String>,
    #[pyo3(get)]
    version: u64,
    #[pyo3(get)]
    runners: Vec<Py<PyRunner>>,    
    #[pyo3(get)]
    market_time: i64,
    // market_time: Option<Py<PyDateTime>>,
    market_time_str: StaticString<24>,
    #[pyo3(get)]
    open_date: i64,
    // open_date: Option<Py<PyDateTime>>,
    open_date_str: StaticString<24>,
    #[pyo3(get)]
    suspend_time: Option<i64>,
    // suspend_time: Option<Py<PyDateTime>>,
    suspend_time_str: Option<StaticString<24>>,
    #[pyo3(get)]
    settled_time: Option<i64>,
    // settled_time: Option<Py<PyDateTime>>,
    settled_time_str: Option<StaticString<24>>,
    // require getter functions
    market_id: MarketID,
    status: MarketStatus,
    country_code: StaticString<2>,
    betting_type: MarketBettingType,
}

impl PyMarketBase {
    fn new(source: String, file: String) -> Self {
        Self { 
            source,
            file, 
            bsp_market: false,
            turn_in_play_enabled: false,
            in_play: false,
            persistence_enabled: false,
            bsp_reconciled: false,
            complete: false,
            cross_matching: false,
            runners_voidable: false,
            discount_allowed: false,
            publish_time: Default::default(), 
            clk: Default::default(),
            each_way_divisor: Default::default(),
            market_type: Default::default(),
            betting_type: Default::default(),
            market_id: Default::default(),
            timezone: Default::default(),
            market_name: Default::default(),
            event_name: Default::default(),
            country_code: Default::default(),
            market_time_str: Default::default(),
            open_date_str: Default::default(),
            suspend_time_str: Default::default(),
            settled_time_str: Default::default(),
            market_time: Default::default(),
            open_date: Default::default(),
            suspend_time: Default::default(),
            settled_time: Default::default(),
            status: Default::default(),
            venue: Default::default(),
            market_base_rate: Default::default(),
            number_of_winners: Default::default(),
            number_of_active_runners: Default::default(),
            bet_delay: Default::default(),
            event_id: Default::default(),
            event_type_id: Default::default(),
            version: Default::default(),
            total_matched: Default::default(),
            runners: Default::default(),
        }
    }

    fn clone(&self, py: Python) -> Self {
        let runners = self.runners.iter()
            .map(|r| Py::new(py, r.borrow(py).clone(py)).unwrap())
            .collect::<Vec<_>>();
        
        Self { 
            source: self.source.clone(),
            file: self.file.clone(),
            bsp_market: self.bsp_market,
            turn_in_play_enabled: self.turn_in_play_enabled,
            in_play: self.in_play,
            persistence_enabled: self.persistence_enabled,
            bsp_reconciled: self.bsp_reconciled,
            complete: self.complete,
            cross_matching: self.cross_matching,
            runners_voidable: self.runners_voidable,
            discount_allowed: self.discount_allowed,
            publish_time: self.publish_time, 
            clk: self.clk.clone(),
            each_way_divisor: self.each_way_divisor,
            market_type: self.market_type.clone(),
            betting_type: self.betting_type,
            market_id: self.market_id.clone(),
            timezone: self.timezone.clone(),
            market_name: self.market_name.clone(),
            event_name: self.event_name.clone(),
            country_code: self.country_code.clone(),
            market_time_str: self.market_time_str.clone(),
            open_date_str: self.open_date_str.clone(),
            suspend_time_str: self.suspend_time_str.clone(),
            settled_time_str: self.settled_time_str.clone(),
            market_time: self.market_time.clone(),
            open_date: self.open_date.clone(),
            suspend_time: self.suspend_time.clone(),
            settled_time: self.settled_time.clone(),
            status: self.status,
            venue: self.venue.clone(),
            market_base_rate: self.market_base_rate,
            number_of_winners: self.number_of_winners,
            number_of_active_runners: self.number_of_active_runners,
            bet_delay: self.bet_delay,
            event_id: self.event_id,
            event_type_id: self.event_type_id,
            version: self.version,
            total_matched: self.total_matched,
            runners: runners,
        }
    }
}

#[pymethods]
impl PyMarketBase {
    #[getter(market_id)]
    fn market_id(&self) -> &str {
        self.market_id.as_ref()
    } 
    #[getter(status)]
    fn status(&self) -> &'static str {
        self.status.into()
    }  
    #[getter(country_code)]
    fn country(&self) -> &str {
        self.country_code.as_str()
    } 
    #[getter(betting_type)]
    fn betting_type_fn(&self) -> &'static str {
        self.betting_type.into()
    } 
}

#[pyclass(name="Market", extends=PyMarketBase)]
pub struct PyMarket {
    deser: Option<DeserializerWithData>,
}

impl PyMarket {

    pub fn new_object(item: SourceItem, py: Python) -> Result<PyObject, DeserErr> {
        let mut deser = DeserializerWithData::new(
            item.bs, 
            |bs| Deser(serde_json::Deserializer::from_slice(&bs)),
        );

        let mut base = PyMarketBase::new(item.source, item.file);
        
        match Self::drive_deserialize(&mut deser, &mut base, py) {
            Ok(()) => {
                let market = PyMarket { deser: Some(deser) };
                Ok(Py::new(py, (market, base)).unwrap().into_py(py))
            },
            Err(err) => Err(DeserErr{ source: base.source, file: base.file, err: err }),
        }
    }

    fn drive_deserialize(deser: &mut DeserializerWithData, base: &mut PyMarketBase, py: Python) -> Result<(), Error> {
        deser.with_dependent_mut(|_, deser| 
            PyMarketToken(base, py).deserialize(&mut deser.0)
        )
    }
}

#[pymethods]
impl PyMarket {

    fn update(mut self_: PyRefMut<Self>, py: Python) -> PyResult<bool> { 
        let mut deser = self_.deser.take().expect("Market without deser");
        let base = self_.as_mut(); 

        let r = Self::drive_deserialize(&mut deser, base, py)
            .map(|_| true)
            .unwrap_or_else(|_err| {
                // TODO: log here
                false
            });

        self_.deser = Some(deser);
 
        Ok(r)
    }

    fn copy(self_: PyRef<Self>, py: Python) -> PyResult<Py<PyMarketBase>> { 
        Py::new(py, self_.as_ref().clone(py))
    }
}

struct PyMarketToken<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketToken<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug,Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Op, Clk, Pt, Mc }

        struct PyMarketOuterVisitor<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
        impl<'de, 'a, 'py> Visitor<'de> for PyMarketOuterVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Op => { map.next_value::<&[u8]>()?; }, //self.0.pt = map.next_value()?,
                        Field::Pt => self.0.publish_time = map.next_value()?,
                        Field::Mc => map.next_value_seed(PyMarketMcSeq(self.0, self.1))?,
                        Field::Clk => { self.0.clk.set_if_ne(map.next_value()?); },
                    }
                }

                Ok(())
            }
        }
        
        const FIELDS: &'static [&'static str] = &[ "op", "pt", "clk", "mc" ];
        deserializer.deserialize_struct("MarketBook", FIELDS, PyMarketOuterVisitor(self.0, self.1))
    }
}

// Used for serializing in place over the marketChange `mc` array
struct PyMarketMcSeq<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMcSeq<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de> 
    {
        struct PyMarketMcSeqVisitor<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
        impl<'de, 'a, 'py> Visitor<'de> for PyMarketMcSeqVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>, 
            {
                while let Some(_) = seq.next_element_seed(PyMarketMc(self.0, self.1))? {}
                Ok(())
            }
        }

        deserializer.deserialize_seq(PyMarketMcSeqVisitor(self.0, self.1))
    }
}

// Used for serializing in place over the marketChange `mc` objects
struct PyMarketMc<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMc<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de> {

        #[derive(Debug,Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field { Id, MarketDefinition, Rc, Con, Img, Tv }

        struct PyMarketMcVisitor<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
        impl<'de, 'a, 'py> Visitor<'de> for PyMarketMcVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => { self.0.market_id.set_if_ne(map.next_value()?); },
                        Field::MarketDefinition => map.next_value_seed(PyMarketDefinition(self.0, self.1))?,
                        Field::Rc => map.next_value_seed(PyRunnerChangeSeq(&mut self.0.runners, self.1))?,
                        Field::Tv => self.0.total_matched += map.next_value::<f64>()?,
                        Field::Con => { map.next_value::<IgnoredAny>()?; },
                        Field::Img => { map.next_value::<IgnoredAny>()?; },
                    }
                }
                Ok(())
            }    
        }

        const FIELDS: &'static [&'static str] = &[ "id", "marketDefinition", "rc", "con", "img", "tv" ];
        deserializer.deserialize_struct("MarketChange", FIELDS, PyMarketMcVisitor(self.0, self.1))
    }
}

// Used for serializing in place over the mc marketDefinition object
struct PyMarketDefinition<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketDefinition<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de> {

            #[derive(Debug,Deserialize)]
            #[serde(field_identifier, rename_all = "camelCase")]
            enum Field {
                BspMarket, TurnInPlayEnabled, PersistenceEnabled, MarketBaseRate, EventId, EventTypeId,
                NumberOfWinners, BettingType, MarketType, MarketTime, SuspendTime, BspReconciled, Complete,
                InPlay, CrossMatching, RunnersVoidable, NumberOfActiveRunners, BetDelay, Status, Runners,
                Regulators, CountryCode, DiscountAllowed, Timezone, OpenDate, Version, Name, EventName, Venue,
                SettledTime, EachWayDivisor
            }

            struct PyMarketDefinitionVisitor<'a, 'py>(&'a mut PyMarketBase, Python<'py>);
            impl<'de, 'a, 'py> Visitor<'de> for PyMarketDefinitionVisitor<'a, 'py> {
                type Value = ();
    
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("")
                }

                fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::BspMarket => self.0.bsp_market = map.next_value()?,
                            Field::TurnInPlayEnabled => self.0.turn_in_play_enabled = map.next_value()?,
                            Field::InPlay => self.0.in_play = map.next_value()?,
                            Field::PersistenceEnabled => self.0.persistence_enabled = map.next_value()?,
                            Field::BspReconciled => self.0.bsp_reconciled = map.next_value()?,
                            Field::Complete => self.0.complete = map.next_value()?,
                            Field::CrossMatching => self.0.cross_matching = map.next_value()?,
                            Field::RunnersVoidable => self.0.runners_voidable = map.next_value()?,
                            Field::DiscountAllowed => self.0.discount_allowed = map.next_value()?,
                            Field::Timezone => { self.0.timezone.set_if_ne(map.next_value()?); },
                            Field::Name => { self.0.market_name.set_if_ne(map.next_value()?); },
                            Field::EventName => { self.0.event_name.set_if_ne(map.next_value()?); },
                            Field::CountryCode => { self.0.country_code.set_if_ne(map.next_value()?); },
                            Field::Venue => { self.0.venue.set_if_ne(map.next_value()?); },
                            Field::Status => self.0.status = map.next_value()?,
                            Field::MarketBaseRate => self.0.market_base_rate = map.next_value::<f32>()? as u8, // TODO: why is this needed
                            Field::NumberOfWinners => self.0.number_of_winners = map.next_value::<f32>()? as u8, // TODO: why is this needed
                            Field::NumberOfActiveRunners => self.0.number_of_active_runners = map.next_value()?,
                            Field::BetDelay => self.0.bet_delay = map.next_value()?,
                            Field::EventId => self.0.event_id = map.next_value::<&str>()?.parse().unwrap(), // TODO: fix unwrap
                            Field::EventTypeId => self.0.event_type_id = map.next_value::<&str>()?.parse().unwrap(), // TODO: fix unwrap
                            Field::Version =>  self.0.version = map.next_value()?,
                            Field::Runners => map.next_value_seed(PyRunnerDefSeq(&mut self.0.runners, self.1))?,
                            Field::MarketType => { self.0.market_type.set_if_ne(map.next_value()?); },
                            Field::BettingType => self.0.betting_type = map.next_value()?,
                            Field::EachWayDivisor => self.0.each_way_divisor = Some(map.next_value::<f64>()?),
                            Field::MarketTime => {
                                let s = map.next_value()?;
                                if self.0.market_time_str.set_if_ne(s) {                                
                                    let ts = chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis() / 1000;
                                    // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
                                    // self.0.market_time = Some(d.into_py(self.1));
                                    self.0.market_time = ts;
                                }
                            }
                            Field::SuspendTime => {
                                let s = map.next_value()?;
                                if self.0.suspend_time_str.set_if_ne(s) {
                                    let ts = chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis() / 1000;
                                    // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
                                    // self.0.suspend_time = Some(d.into_py(self.1));
                                    self.0.suspend_time = Some(ts);
                                }
                            }
                            Field::SettledTime => {
                                let s = map.next_value()?;
                                if self.0.settled_time_str.set_if_ne(s) {
                                    let ts = chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis() / 1000;
                                    // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
                                    // self.0.settled_time = Some(d.into_py(self.1));
                                    self.0.settled_time = Some(ts);
                                }
                            }
                            Field::OpenDate => {
                                let s = map.next_value()?;
                                if self.0.open_date_str.set_if_ne(s) {
                                    let ts = chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis() / 1000;
                                    // let d = PyDateTime::from_timestamp(self.1, ts as f64, None).unwrap();
                                    // self.0.open_date = Some(d.into_py(self.1));
                                    self.0.open_date = ts;
                                }
                            }
                            Field::Regulators => { map.next_value::<serde::de::IgnoredAny>()?; },
                        }
                    }
                    Ok(())
                }    
            }

            const FIELDS: &'static [&'static str] = &[ 
                "bspMarket", "turnInPlayEnabled", "persistenceEnabled", "marketBaseRate", "eventId", "eventTypeId",
                "numberOfWinners", "bettingType", "marketType", "marketTime", "suspendTime", "bspReconciled", "complete",
                "inPlay", "crossMatching", "runnersVoidable", "numberOfActiveRunners", "betDelay", "status", "runners",
                "regulators", "countryCode", "discountAllowed", "timezone", "openDate", "version", "name", "eventName", 
                "venue", "settledTime", "eachWayDivisor",
            ];
            deserializer.deserialize_struct("MarketDefinition", FIELDS, PyMarketDefinitionVisitor(self.0, self.1))
        }
}


