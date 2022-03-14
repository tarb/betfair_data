use pyo3::prelude::*;
use serde::de::{Error, IgnoredAny};
use serde::{
    de::{DeserializeSeed, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;
use std::collections::VecDeque;
use std::fmt;

use super::definition::{MarketDefinition, MarketDefinitionDeser};
use crate::config::Config;
use crate::datetime::DateTime;
use crate::ids::MarketID;
use crate::immutable::container::SyncObj;
use crate::mutable::runner::{Runner, RunnerChangeSeqDeser};
use crate::py_rep::PyRep;
use crate::strings::FixedSizeString;

#[derive(Default)]
#[pyclass(name = "MarketMut")]
pub struct MarketMut {
    #[pyo3(get)]
    pub market_id: SyncObj<MarketID>,
    #[pyo3(get)]
    pub clk: SyncObj<FixedSizeString<10>>,
    #[pyo3(get)]
    pub publish_time: DateTime,
    #[pyo3(get)]
    pub total_matched: f64,
    #[pyo3(get)]
    pub runners: Vec<Py<Runner>>,
    def: MarketDefinition,
}

impl MarketMut {
    fn new(market_id: MarketID) -> Self {
        Self {
            market_id: SyncObj::new(market_id),
            clk: Default::default(),
            publish_time: Default::default(),
            total_matched: Default::default(),
            runners: Default::default(),
            def: Default::default(),
        }
    }

    fn clone(&self, py: Python) -> Self {
        let runners = self
            .runners
            .iter()
            .map(|r| Py::new(py, r.borrow(py).clone(py)).unwrap())
            .collect::<Vec<_>>();

        Self {
            market_id: self.market_id.clone(),
            publish_time: self.publish_time,
            total_matched: self.total_matched,
            clk: self.clk.clone(),
            def: self.def.clone(),
            runners,
        }
    }
}

#[pymethods]
impl MarketMut {
    fn copy(&self, py: Python) -> PyObject {
        self.clone(py).into_py(py)
    }

    #[getter(market_id)]
    fn get_market_id(&self) -> &str {
        self.market_id.as_ref()
    }
    #[getter(clk)]
    fn get_clk(&self) -> &str {
        self.clk.as_ref()
    }
    #[getter(country_code)]
    fn get_country_code(&self) -> &str {
        self.def.country_code.as_str()
    }
    #[getter(event_id)]
    fn get_event_id(&self) -> u32 {
        self.def.event_id
    }
    #[getter(event_type_id)]
    fn get_event_type_id(&self, py: Python) -> PyObject {
        self.def.event_type_id.into_py(py)
    }
    #[getter(bet_delay)]
    fn get_bet_delay(&self, py: Python) -> PyObject {
        self.def.bet_delay.into_py(py)
    }
    #[getter(bsp_market)]
    fn get_bsp_market(&self, py: Python) -> PyObject {
        self.def.bsp_market.into_py(py)
    }
    #[getter(bsp_reconciled)]
    fn get_bsp_reconciled(&self, py: Python) -> PyObject {
        self.def.bsp_reconciled.into_py(py)
    }
    #[getter(complete)]
    fn get_complete(&self, py: Python) -> PyObject {
        self.def.complete.into_py(py)
    }
    #[getter(cross_matching)]
    fn get_cross_matching(&self, py: Python) -> PyObject {
        self.def.cross_matching.into_py(py)
    }
    #[getter(discount_allowed)]
    fn get_discount_allowed(&self, py: Python) -> PyObject {
        self.def.discount_allowed.into_py(py)
    }
    #[getter(each_way_divisor)]
    fn get_each_way_divisor(&self, py: Python) -> PyObject {
        self.def.each_way_divisor.into_py(py)
    }
    #[getter(event_name)]
    fn get_event_name(&self, py: Python) -> PyObject {
        self.def.event_name.to_object(py)
    }
    #[getter(in_play)]
    fn get_in_play(&self, py: Python) -> PyObject {
        self.def.in_play.into_py(py)
    }
    #[getter(market_base_rate)]
    fn get_market_base_rate(&self, py: Python) -> PyObject {
        self.def.market_base_rate.into_py(py)
    }
    #[getter(market_type)]
    fn get_market_type(&self, py: Python) -> PyObject {
        self.def.market_type.to_object(py)
    }
    #[getter(market_name)]
    fn get_market_name(&self, py: Python) -> PyObject {
        self.def.market_name.to_object(py)
    }
    #[getter(number_of_active_runners)]
    fn get_number_of_active_runners(&self, py: Python) -> PyObject {
        self.def.number_of_active_runners.into_py(py)
    }
    #[getter(number_of_winners)]
    fn get_number_of_winners(&self, py: Python) -> PyObject {
        self.def.number_of_winners.into_py(py)
    }
    #[getter(persistence_enabled)]
    fn get_persistence_enabled(&self, py: Python) -> PyObject {
        self.def.persistence_enabled.into_py(py)
    }
    #[getter(runners_voidable)]
    fn get_runners_voidable(&self, py: Python) -> PyObject {
        self.def.runners_voidable.into_py(py)
    }
    #[getter(timezone)]
    fn get_timezone(&self, py: Python) -> PyObject {
        self.def.timezone.to_object(py)
    }
    #[getter(turn_in_play_enabled)]
    fn get_turn_in_play_enabled(&self, py: Python) -> PyObject {
        self.def.turn_in_play_enabled.into_py(py)
    }
    #[getter(venue)]
    fn get_venue(&self, py: Python) -> PyObject {
        self.def.venue.to_object(py)
    }
    #[getter(version)]
    fn get_version(&self, py: Python) -> PyObject {
        self.def.version.into_py(py)
    }
    #[getter(status)]
    fn get_status(&self, py: Python) -> PyObject {
        self.def.status.into_py(py)
    }
    #[getter(betting_type)]
    fn get_betting_type(&self, py: Python) -> PyObject {
        self.def.betting_type.into_py(py)
    }
    #[getter(market_time)]
    fn get_market_time(&self, py: Python) -> PyObject {
        self.def.market_time.py_rep(py)
    }
    #[getter(open_date)]
    fn get_open_date(&self, py: Python) -> PyObject {
        self.def.open_date.py_rep(py)
    }
    #[getter(suspend_time)]
    fn get_suspend_time(&self, py: Python) -> PyObject {
        self.def
            .suspend_time
            .map(|st| st.py_rep(py))
            .unwrap_or_else(|| py.None())
    }
    #[getter(settled_time)]
    fn get_settled_time(&self, py: Python) -> PyObject {
        self.def
            .settled_time
            .map(|st| st.py_rep(py))
            .unwrap_or_else(|| py.None())
    }
    #[getter(regulators)]
    fn get_regulators(&self, py: Python) -> PyObject {
        self.def.regulators.py_rep(py)
    }
}

pub struct MarketMutDeser<'a, 'py> {
    pub markets: &'a [Py<MarketMut>],
    pub py: Python<'py>,
    pub config: Config,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for MarketMutDeser<'a, 'py> {
    type Value = VecDeque<Py<MarketMut>>;

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

        struct PyMarketOuterVisitor<'a, 'py> {
            markets: &'a [Py<MarketMut>],
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for PyMarketOuterVisitor<'a, 'py> {
            type Value = VecDeque<Py<MarketMut>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pt: Option<DateTime> = None;
                let mut clk: Option<FixedSizeString<10>> = None;
                let mut books: VecDeque<Py<MarketMut>> = VecDeque::new();

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Op => {
                            map.next_value::<IgnoredAny>()?;
                        }
                        Field::Pt => {
                            pt = Some(DateTime::new(map.next_value::<u64>()?));
                        }
                        Field::Mc => {
                            books = map.next_value_seed(PyMarketMcSeqDeser {
                                markets: self.markets,
                                config: self.config,
                                py: self.py,
                            })?;
                        }
                        Field::Clk => {
                            clk = Some(map.next_value::<FixedSizeString<10>>()?);
                        }
                    }
                }

                if let (Some(pt), Some(clk)) = (pt, clk) {
                    books.iter_mut().for_each(|mb| {
                        let mut m = mb.borrow_mut(self.py);
                        m.publish_time = pt;
                        m.clk = SyncObj::new(clk);
                    });
                }

                Ok(books)
            }
        }

        const FIELDS: &[&str] = &["op", "pt", "clk", "mc"];
        deserializer.deserialize_struct(
            "MarketBook",
            FIELDS,
            PyMarketOuterVisitor {
                markets: self.markets,
                config: self.config,
                py: self.py,
            },
        )
    }
}

// Used for serializing in place over the marketChange `mc` array
struct PyMarketMcSeqDeser<'a, 'py> {
    markets: &'a [Py<MarketMut>],
    config: Config,
    py: Python<'py>,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMcSeqDeser<'a, 'py> {
    type Value = VecDeque<Py<MarketMut>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PyMarketMcSeqDeserVisitor<'a, 'py> {
            markets: &'a [Py<MarketMut>],
            config: Config,
            py: Python<'py>,
        }
        impl<'de, 'a, 'py> Visitor<'de> for PyMarketMcSeqDeserVisitor<'a, 'py> {
            type Value = VecDeque<Py<MarketMut>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                #[derive(Deserialize)]
                struct IdImg {
                    id: MarketID,
                    img: Option<bool>,
                }

                // TODO what should we do if a market appears twice in a mc
                let mut next_books: VecDeque<Py<MarketMut>> = VecDeque::new();

                while let Some(raw) = seq.next_element::<&RawValue>()? {
                    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
                    let idimg: IdImg = serde_json::from_str(raw.get()).map_err(Error::custom)?;

                    let market = self
                        .markets
                        .iter()
                        .find(|m| (*m).borrow(self.py).market_id.as_str() == idimg.id)
                        .map(|m| m.clone_ref(self.py));

                    let market = PyMarketMc {
                        mid: idimg.id,
                        market,
                        config: self.config,
                        py: self.py,
                        img: idimg.img.unwrap_or(false),
                    }
                    .deserialize(&mut deserializer)
                    .map_err(Error::custom)?;

                    if let Some(m) = market {
                        next_books.push_back(m);
                    }
                }

                Ok(next_books)
            }
        }

        deserializer.deserialize_seq(PyMarketMcSeqDeserVisitor {
            markets: self.markets,
            config: self.config,
            py: self.py,
        })
    }
}

// Used for serializing in place over the marketChange `mc` objects
struct PyMarketMc<'py> {
    mid: MarketID,
    market: Option<Py<MarketMut>>,
    config: Config,
    py: Python<'py>,
    img: bool,
}
impl<'de, 'a, 'py> DeserializeSeed<'de> for PyMarketMc<'py> {
    type Value = Option<Py<MarketMut>>;

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

        struct PyMarketMcVisitor<'py> {
            mid: MarketID,
            market: Option<Py<MarketMut>>,
            config: Config,
            img: bool,
            py: Python<'py>,
        }
        impl<'de, 'py> Visitor<'de> for PyMarketMcVisitor<'py> {
            type Value = Option<Py<MarketMut>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let market = self
                    .market
                    .unwrap_or_else(|| Py::new(self.py, MarketMut::new(self.mid)).unwrap());

                {
                    let m = &mut *market.borrow_mut(self.py);

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::MarketDefinition => {
                                map.next_value_seed(MarketDefinitionDeser {
                                    def: &mut m.def,
                                    runners: &mut m.runners,
                                    config: self.config,
                                    img: self.img,
                                    py: self.py,
                                })?;
                            }
                            Field::Rc => {
                                map.next_value_seed(RunnerChangeSeqDeser {
                                    runners: &mut m.runners,
                                    img: self.img,
                                    config: self.config,
                                    py: self.py,
                                })?;

                                // if cumulative_runner_tv is on, then tv shouldnt be sent at a market level and will have
                                // to be derived from the sum of runner tv's. This happens when using the data provided
                                // from betfair historical data service, not saved from the actual stream
                                if self.config.cumulative_runner_tv {
                                    m.total_matched = m
                                        .runners
                                        .iter()
                                        .map(|r| r.borrow(self.py).total_matched)
                                        .sum();
                                }
                            }
                            Field::Tv => {
                                if !self.config.cumulative_runner_tv {
                                    m.total_matched += map.next_value::<f64>()?;
                                } else {
                                    map.next_value::<IgnoredAny>()?;
                                }
                            }
                            _ => {
                                map.next_value::<IgnoredAny>()?;
                            }
                        }
                    }
                }

                Ok(Some(market))
            }
        }

        const FIELDS: &[&str] = &["id", "marketDefinition", "rc", "con", "img", "tv"];
        deserializer.deserialize_struct(
            "MarketChange",
            FIELDS,
            PyMarketMcVisitor {
                mid: self.mid,
                market: self.market,
                config: self.config,
                img: self.img,
                py: self.py,
            },
        )
    }
}

#[cfg(test)]
mod tests {

    // test disabled awaiting merge which fixes cargo test
    // https://github.com/PyO3/pyo3/pull/2135
    /*
    use super::*;

    #[test]
    fn test_multiple_markets() {
        let mut m = MarketMut::new("".to_owned(), "".to_owned());
        let py = unsafe { Python::assume_gil_acquired() };

        let config = Config{cumulative_runner_tv: true, stable_runner_index: false};

        let mut deser = serde_json::Deserializer::from_str(r#"{"id": "1.123456789"}{"id":"1.987654321"}"#);

        PyMarketMc(&mut m, py, config).deserialize(&mut deser).expect("1st market_id deser ok");
        PyMarketMc(&mut m, py, config).deserialize(&mut deser).expect_err("2nd market_id deser error");
    }
    */
}
