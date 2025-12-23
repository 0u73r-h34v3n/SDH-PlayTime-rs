mod db;
mod playtime;
mod user_manager;

pub use playtime::PlayTime;
use pyo3::prelude::*;
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen::derive::gen_stub_pyfunction;
pub use user_manager::UserManager;

#[gen_stub_pyfunction]
#[pyfunction]
fn clear_db_cache() {
    db::clear_cache();
}

#[pymodule]
fn playtime_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PlayTime>()?;
    m.add_class::<UserManager>()?;
    m.add_function(wrap_pyfunction!(clear_db_cache, m)?)?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
