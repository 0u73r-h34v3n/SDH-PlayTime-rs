mod db;
mod playtime;
mod user_manager;

pub use playtime::PlayTime;
use pyo3::prelude::*;
pub use user_manager::UserManager;

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
