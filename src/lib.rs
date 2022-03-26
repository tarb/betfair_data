#![feature(
    bool_to_option,
    derive_default_enum,
    try_blocks,
    toowned_clone_into,
    once_cell,
    variant_count,
    option_result_contains,
    is_some_with,
    min_specialization,
    let_chains,
    generic_associated_types
)]

mod bflw;
mod config;
mod datetime;
mod deser;
mod enums;
mod errors;
mod file;
mod files;
mod file_iter;
mod ids;
mod immutable;
mod market_source;
mod mutable;
mod price_size;
mod py_rep;
mod strings;

use crate::bflw::file_iter::BflwFile;
use crate::file::File;
use crate::files::Files;
use crate::price_size::PriceSize;

use bflw::market_book::MarketBook;
use bflw::market_definition::MarketDefinition;
use bflw::market_definition_runner::MarketDefinitionRunner;
use bflw::runner_book::RunnerBook;
use immutable::market::Market;
use immutable::runner::Runner;
use immutable::runner_book_ex::RunnerBookEX;
use immutable::runner_book_sp::RunnerBookSP;
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
    
    m.add_class::<File>()?;
    m.add_class::<PriceSize>()?;
    m.add_class::<Market>()?;
    m.add_class::<Runner>()?;
    m.add_class::<RunnerBookEX>()?;
    m.add_class::<RunnerBookSP>()?;

    let bflw = PyModule::new(py, "bflw")?;
    bflw.add_class::<BflwFile>()?;
    bflw.add_class::<MarketBook>()?;
    bflw.add_class::<MarketDefinitionRunner>()?;
    bflw.add_class::<MarketDefinition>()?;
    bflw.add_class::<RunnerBook>()?;
    m.add_submodule(bflw)?;

    Ok(())
}
