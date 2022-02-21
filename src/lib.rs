#![feature(
    bool_to_option,
    derive_default_enum,
    try_blocks,
    toowned_clone_into,
    once_cell,
    variant_count,
    option_result_contains,
    is_some_with,
    // impl_specialization,
    // min_specialization,
    // specialization

)]

mod bflw;
mod deser;
mod enums;
mod errors;
mod files_source;
mod ids;
mod immutable;
mod market_source;
mod mutable;
mod price_size;
mod strings;
mod tarbz2_source;

use crate::mutable::market::{PyMarket, PyMarketBase};
use crate::mutable::runner::{PyRunner, PyRunnerBookEX, PyRunnerBookSP};
use crate::bflw::iter::BflwIter;
use crate::bflw::market_book::MarketBook;
use crate::bflw::market_definition::MarketDefinition;
use crate::bflw::market_definition_runner::MarketDefinitionRunner;
use crate::bflw::runner_book::RunnerBook;
use crate::files_source::Files;
use crate::price_size::PriceSize;
use crate::tarbz2_source::TarBz2;

use pyo3::prelude::*;

#[cfg(not(target_os = "linux"))]
use mimalloc::MiMalloc;

#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[pymodule]
fn betfair_data(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Files>()?;
    m.add_class::<TarBz2>()?;
    m.add_class::<PriceSize>()?;

    let mutable = PyModule::new(py, "mut")?;
    mutable.add_class::<PyMarket>()?;
    mutable.add_class::<PyMarketBase>()?;
    mutable.add_class::<PyRunner>()?;
    mutable.add_class::<PyRunnerBookEX>()?;
    mutable.add_class::<PyRunnerBookSP>()?;
    m.add_submodule(mutable)?;

    let bflw = PyModule::new(py, "bflw")?;
    bflw.add_class::<BflwIter>()?;
    bflw.add_class::<MarketBook>()?;
    bflw.add_class::<MarketDefinitionRunner>()?;
    bflw.add_class::<MarketDefinition>()?;
    bflw.add_class::<RunnerBook>()?;

    m.add_submodule(bflw)?;

    Ok(())
}
