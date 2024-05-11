use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tokio::runtime::Runtime;

#[allow(dead_code)]
#[pyclass]
struct ServerRuntimeHandle(Runtime);

#[pyfunction]
fn start_server() -> PyResult<PyObject> {
    reduce_core::setup_tracing()
        .map_err(|error| PyErr::new::<PyRuntimeError, _>(error.to_string()))?;
    let runtime = Runtime::new()?;
    runtime.spawn(async move {
        if let Err(e) = reduce_core::start_server().await {
            eprintln!("Error starting server: {}", e);
        }
    });

    Python::with_gil(|py| {
        // Convert the Arc to a PyCell, which allows it to be returned as a Python object
        let py_obj = PyCell::new(py, ServerRuntimeHandle(runtime))?;

        // Return the PyCell as a PyResult<PyObject>
        Ok(py_obj.into())
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyreduce(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_server, m)?)?;
    Ok(())
}
