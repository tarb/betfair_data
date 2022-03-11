// use log::warn;
// use pyo3::{exceptions, prelude::*, PyIterProtocol};
// use serde::de::DeserializeSeed;
// use std::borrow::Borrow;
// use std::collections::VecDeque;
// use std::path::PathBuf;

// // use super::config::Config;
// use crate::deser::DeserializerWithData;
// use crate::immutable::container::SyncObj;
// use crate::market_source::{SourceConfig, SourceItem};
// use crate::mutable::config::Config;
// use crate::mutable::market::{PyMarketMut, PyMarketToken};

// #[pyclass(name = "MutIter")]
// pub struct MutIter {
//     inner: Iter<PyMarketMut>,
// }

// #[pymethods]
// impl MutIter {
//     #[new]
//     #[args(cumulative_runner_tv = "true")]
//     fn __new__(file: PathBuf, bytes: &[u8], cumulative_runner_tv: bool) -> PyResult<Self> {
//         let config = SourceConfig {
//             cumulative_runner_tv,
//         };

//         let deser = DeserializerWithData::build(bytes.to_owned())
//             .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

//         Ok(MutIter {
//             inner: Iter {
//                 file_name: SyncObj::new(file),
//                 deser: Some(deser),
//                 books: Vec::new(),
//                 iter_stack: VecDeque::new(),
//                 config,
//             }
//         })
//     }
// }




// pub trait MarketID {
//     fn id(&self) -> &str;
// }

// impl MarketID for PyMarketMut {
//     fn id(&self) -> &str {
//         self.market_id.as_str()
//     }
// }

// pub trait FileIter<'a, 'py> {
//     type Market;
//     type Value<'de>: DeserializeSeed<'de, Value = VecDeque<Py<Self::Market>>>;

//     fn new<'de>(books: &'a [Py<Self::Market>], py: Python<'py>, config: Config) -> Self::Value<'de>;
// }

// impl<'a, 'py> FileIter<'a, 'py> for PyMarketToken<'a, 'py> {
//     type Market = PyMarketMut;
//     type Value<'de> = PyMarketToken<'a, 'py>;

//     fn new<'de>(books: &'a [Py<PyMarketMut>], py: Python<'py>, config: Config) -> Self::Value<'de> {
//         PyMarketToken {
//             markets: books,
//             py,
//             config,
//         }
//     }
// }

// struct Iter<T: pyo3::PyClass + MarketID> {
//     file_name: SyncObj<PathBuf>,
//     config: SourceConfig,
//     deser: Option<DeserializerWithData>,
//     books: Vec<Py<T>>,
//     iter_stack: VecDeque<Py<T>>,
// }

// impl<T: pyo3::PyClass + MarketID> Iter<T> {
//     pub fn new(item: SourceItem, config: SourceConfig) -> Self {
//         Iter {
//             file_name: SyncObj::new(item.file),
//             deser: Some(item.deser),
//             books: Vec::new(),
//             iter_stack: VecDeque::new(),
//             config,
//         }
//     }

//     fn drive_deserialize<'a, 'py, 'de, D>(
//         deser: &'a mut DeserializerWithData,
//         books: &'a [Py<D::Market>],
//         config: SourceConfig,
//         py: Python<'py>,
//     ) -> Result<VecDeque<Py<D::Market>>, serde_json::Error>
//     where
//         D: FileIter<'a, 'py> + DeserializeSeed<'de, Value = VecDeque<Py<D::Market>>>,
//     {
//         deser.with_dependent_mut(|_, deser| {
//             let config = Config {
//                 cumulative_runner_tv: true,
//                 stable_runner_index: true,
//             };

//             D::new(books, py, config).deserialize(&mut deser.0)
//         })
//     }

//     fn next<'a, 'py, 'de, D>(&mut self, py: Python) -> Option<PyObject>
//     where
//     D: FileIter<'a, 'py> + DeserializeSeed<'de, Value = VecDeque<Py<D::Market>>>, {
//         if let Some(m) = self.iter_stack.pop_front() {
//             let index = {
//                 let market = m.borrow(py);
//                 self.books.iter().position(|m2| {
//                     market.id() == (*m2).borrow(py).id()
//                 })
//             };

//             let mc = m.clone_ref(py);
//             match index {
//                 Some(i) => self.books[i] = mc,
//                 None => self.books.push(mc),
//             }

//             Some(m.into_py(py))
//         } else {
//             let next_books = {
//                 let config = self.config;
//                 let mut deser = self.deser.take().expect("Iter without deser");

//                 let books = &self.books;

//                 let next_books = match Self::drive_deserialize::<'a, 'py, 'de, D>(&mut deser, books, config, py)
//                 {
//                     Ok(bs) => Some(bs),
//                     Err(err) => {
//                         if !err.is_eof() {
//                             warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", self.file_name.to_string_lossy(), err);
//                         }

//                         None
//                     }
//                 };

//                 self.deser = Some(deser);

//                 next_books
//             };

//             next_books.and_then(|mut next_books| {
//                 next_books.pop_front().map(|m| {

//                     let index = {
//                         let market = &(*m.borrow(py));
//                         self.books.iter().position(|m2| {
//                             market.id() == (*m2).borrow(py).id()
//                         })
//                     };

//                     let mc = m.clone_ref(py);
//                     match index {
//                         Some(i) => self.books[i] = mc,
//                         None => self.books.push(mc),
//                     }

//                     self.iter_stack = next_books;
//                     m.into_py(py)
//                 })
//             })
//         }
//     }

// }


// // #[pyproto]
// // impl<'p> PyIterProtocol for MutIter {
// //     fn __iter__(slf: PyRef<'p, Self>) -> PyRef<'p, Self> {
// //         slf
// //     }

// //     fn __next__(mut slf: PyRefMut<'p, Self>) -> Option<PyObject> {
// //         if let Some(m) = slf.iter_stack.pop_front() {
// //             let index = {
// //                 let market = m.borrow(slf.py());
// //                 slf.books.iter().position(|m2| {
// //                     market.market_id.as_str() == (*m2).borrow(slf.py()).market_id.as_str()
// //                 })
// //             };

// //             let mc = m.clone_ref(slf.py());
// //             match index {
// //                 Some(i) => slf.books[i] = mc,
// //                 None => slf.books.push(mc),
// //             }

// //             Some(m.into_py(slf.py()))
// //         } else {
// //             let next_books = {
// //                 let config = slf.config;
// //                 let mut deser = slf.deser.take().expect("Iter without deser");

// //                 let books = &slf.books;

// //                 let next_books = match Self::drive_deserialize(&mut deser, books, config, slf.py())
// //                 {
// //                     Ok(bs) => Some(bs),
// //                     Err(err) => {
// //                         if !err.is_eof() {
// //                             warn!(target: "betfair_data", "file: {} err: (JSON Parse Error) {}", slf.file_name.to_string_lossy(), err);
// //                         }

// //                         None
// //                     }
// //                 };

// //                 slf.deser = Some(deser);

// //                 next_books
// //             };

// //             next_books.and_then(|mut next_books| {
// //                 next_books.pop_front().map(|m| {
// //                     let index = {
// //                         let market = m.borrow(slf.py());
// //                         slf.books.iter().position(|m2| {
// //                             market.market_id.as_str() == (*m2).borrow(slf.py()).market_id.as_str()
// //                         })
// //                     };

// //                     let mc = m.clone_ref(slf.py());
// //                     match index {
// //                         Some(i) => slf.books[i] = mc,
// //                         None => slf.books.push(mc),
// //                     }

// //                     slf.iter_stack = next_books;
// //                     m.into_py(slf.py())
// //                 })
// //             })
// //         }
// //     }
// // }
