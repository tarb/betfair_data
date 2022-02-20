use log::warn;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyUnicode};
use serde::de::{Error, IgnoredAny};
use serde::{
    de::{self, DeserializeSeed, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use staticvec::StaticString;
use std::borrow::Cow;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::bflw::runner_book::RunnerChangeSeq;
use crate::deser::DeserializerWithData;
use crate::ids::SelectionID;
use crate::immutable::container::SyncObj;
use crate::market_source::{SourceConfig, SourceItem};
use crate::strings::StringSetExtNeq;

use pyo3::{Py, PyAny};

use crate::{enums::MarketStatus, ids::MarketID};

use super::datetime::{DateTime, DateTimeString};
use super::market_definition::MarketDefinition;
use super::runner_book::RunnerBook;

/*
"""
:type bet_delay: int
:type bsp_reconciled: bool
:type complete: bool
:type cross_matching: bool
:type inplay: bool
:type is_market_data_delayed: bool
:type last_match_time: datetime.datetime
:type market_id: unicode
:type number_of_active_runners: int
:type number_of_runners: int
:type number_of_winners: int
:type publish_time: datetime.datetime
:type runners: list[RunnerBook]
:type runners_voidable: bool
:type status: unicode
:type total_available: float
:type total_matched: float
:type version: int
"""

+ market_definition
*/

// store the current state of the market mutably
#[pyclass]
pub struct MarketBook {
    publish_time: DateTime,
    bet_delay: u64,
    bsp_reconciled: bool,
    complete: bool,
    cross_matching: bool,
    inplay: bool,
    is_market_data_delayed: bool,
    number_of_active_runners: u64,
    number_of_runners: u64,
    number_of_winners: u64,
    runners_voidable: bool,
    status: MarketStatus,
    total_available: f64,
    total_matched: f64,
    version: u64,
    runners: SyncObj<Vec<Py<RunnerBook>>>,
    market_definition: Py<MarketDefinition>,

    market_id: SyncObj<MarketID>,
    #[pyo3(get)]
    last_match_time: SyncObj<DateTimeString>,
}

#[pymethods]
impl MarketBook {
    #[getter(last_match_time)]
    fn get_last_match_time(&self, py: Python) -> PyObject {
        self.last_match_time.to_object(py)
    }

    #[getter(runners)]
    fn get_runners(&self, py: Python) -> PyObject {
        self.runners.to_object(py)
    }
}

impl MarketBook {
    fn clone_ref<'py>(&self, py: Python<'py>) -> Self {
        Self {
            publish_time: self.publish_time,
            bet_delay: self.bet_delay,
            bsp_reconciled: self.bsp_reconciled,
            complete: self.complete,
            cross_matching: self.cross_matching,
            inplay: self.inplay,
            is_market_data_delayed: self.is_market_data_delayed,
            number_of_active_runners: self.number_of_active_runners,
            number_of_runners: self.number_of_runners,
            number_of_winners: self.number_of_winners,
            runners_voidable: self.runners_voidable,
            status: self.status,
            total_available: self.total_available,
            total_matched: self.total_matched,
            version: self.version,
            
            market_id: self.market_id.clone(),
            last_match_time: self.last_match_time.clone(),
            runners: self.runners.clone(),
            market_definition: self.market_definition.clone_ref(py),
        }
    }
}

struct MarketBookDeser<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketBookDeser<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Op,
            Clk,
            Pt,
            Mc,
        }

        struct MarketBookDeserVisitor<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
        impl<'de, 'a, 'py> Visitor<'de> for MarketBookDeserVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pt: Option<DateTime> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Op => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Pt => {
                            let ts = map.next_value::<u64>()?;
                            if self.0.publish_time != ts {
                                pt = Some(DateTime::new(ts));
                            }
                        }
                        Field::Mc => map.next_value_seed(MarketMcSeq(self.0, self.1, self.2))?,
                        Field::Clk => {
                            map.next_value::<IgnoredAny>()?;
                            // self.0.clk.set_if_ne(map.next_value::<&str>()?);
                        }
                    }
                }

                Ok(())
            }
        }

        const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
        deserializer.deserialize_struct(
            "MarketBook",
            FIELDS,
            MarketBookDeserVisitor(self.0, self.1, self.2),
        )
    }
}

// Used for serializing in place over the marketChange `mc` array
struct MarketMcSeq<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMcSeq<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MarketMcSeqVisitor<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
        impl<'de, 'a, 'py> Visitor<'de> for MarketMcSeqVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while seq
                    .next_element_seed(MarketMc(self.0, self.1, self.2))?
                    .is_some()
                {}
                Ok(())
            }
        }

        deserializer.deserialize_seq(MarketMcSeqVisitor(self.0, self.1, self.2))
    }
}

// Used for serializing in place over the marketChange `mc` objects
struct MarketMc<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMc<'a, 'py> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Id,
            MarketDefinition,
            Rc,
            Con,
            Img,
            Tv,

            // bflw recorded field
            #[serde(rename = "_stream_id")]
            StreamId,
        }

        struct MarketMcVisitor<'a, 'py>(&'a mut MarketBook, Python<'py>, SourceConfig);
        impl<'de, 'a, 'py> Visitor<'de> for MarketMcVisitor<'a, 'py> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {

                let mut mid: Option<&str> = None;
                let mut rb: Option<Vec<Py<RunnerBook>>> = None;


                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            let s = map.next_value::<&str>()?;
                            if self.0.market_id.value.as_str() != s {
                                mid = Some(s)
                            }
                        }
                        Field::MarketDefinition => {
                            // map.next_value_seed(PyMarketDefinition(self.0, self.1, self.2))?
                        }
                        Field::Rc => {
                            let v = rb.as_ref().unwrap_or(&self.0.runners.value);
                            rb = Some(map.next_value_seed(RunnerChangeSeq(v, self.1, self.2))?);
                        }
                        Field::Tv => {
                            if !self.2.cumulative_runner_tv {
                                self.0.total_matched += map.next_value::<f64>()?;
                            } else {
                                map.next_value::<IgnoredAny>()?;
                            }
                        }
                        Field::Con => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Img => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        _ => {
                            map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
                // to be derived from the sum of runner tv's. This happens when using the data provided
                // from betfair historical data service, not saved from the actual stream
                if self.2.cumulative_runner_tv {
                    // self.0.total_matched = self
                    //     .0
                    //     .runners
                    //     .iter()
                    //     .map(|r| r.borrow(self.1).total_matched)
                    //     .sum();
                }

                Ok(())
            }
        }

        const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
        deserializer.deserialize_struct(
            "MarketChange",
            FIELDS,
            MarketMcVisitor(self.0, self.1, self.2),
        )
    }
}

/*
class MarketBook(BaseResource):


def __init__(self, **kwargs):
    self.streaming_unique_id = kwargs.pop("streaming_unique_id", None)
    self.streaming_update = kwargs.pop("streaming_update", None)
    self.streaming_snap = kwargs.pop("streaming_snap", False)
    self.market_definition = kwargs.pop("market_definition", None)
    super(MarketBook, self).__init__(**kwargs)
    self.market_id = kwargs.get("marketId")
    self.bet_delay = kwargs.get("betDelay")
    self.bsp_reconciled = kwargs.get("bspReconciled")
    self.complete = kwargs.get("complete")
    self.cross_matching = kwargs.get("crossMatching")
    self.inplay = kwargs.get("inplay")
    self.is_market_data_delayed = kwargs.get("isMarketDataDelayed")
    self.last_match_time = self.strip_datetime(kwargs.get("lastMatchTime"))
    self.number_of_active_runners = kwargs.get("numberOfActiveRunners")
    self.number_of_runners = kwargs.get("numberOfRunners")
    self.number_of_winners = kwargs.get("numberOfWinners")
    self.runners_voidable = kwargs.get("runnersVoidable")
    self.status = kwargs.get("status")
    self.total_available = kwargs.get("totalAvailable")
    self.total_matched = kwargs.get("totalMatched")
    self.version = kwargs.get("version")
    self.runners = [RunnerBook(**i) for i in kwargs.get("runners")]
    self.publish_time = self.strip_datetime(kwargs.get("publishTime"))
    self.publish_time_epoch = kwargs.get("publishTime")
    self.key_line_description = (
        KeyLine(**kwargs.get("keyLineDescription"))
        if kwargs.get("keyLineDescription")
        else None
    )
    self.price_ladder_definition = (
        PriceLadderDescription(**kwargs.get("priceLadderDefinition"))
        if kwargs.get("priceLadderDefinition")
        else None
    )

*/

