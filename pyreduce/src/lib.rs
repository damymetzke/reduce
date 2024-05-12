use pyo3::{exceptions::PyRuntimeError, prelude::*};
use reduce_core::ServerConfig as CoreConfig;
use tokio::runtime::Runtime;

#[allow(dead_code)]
#[pyclass]
struct ServerRuntimeHandle(Runtime);

#[pyclass]
#[derive(Debug, Default)]
pub struct ServerConfig {
    database_url: Option<String>,
    server_bind_address: Option<String>,
    runtime: Option<Runtime>,
}

#[pymethods]
impl ServerConfig {
    #[new]
    fn new() -> ServerConfig {
        ServerConfig::default()
    }

    fn database_url(&mut self, value: String) {
        self.database_url = Some(value);
    }

    fn server_bind_address(&mut self, value: String) {
        self.server_bind_address = Some(value);
    }

    fn start_server(&mut self) -> PyResult<()> {
        println!("{:?}", self);
        reduce_core::setup_tracing()
            .map_err(|error| PyErr::new::<PyRuntimeError, _>(error.to_string()))?;
        let runtime = Runtime::new()?;
        let config = match self {
            ServerConfig {
                database_url: Some(database_url),
                server_bind_address: Some(server_bind_address),
                ..
            } => CoreConfig {
                db_url: database_url.as_str().into(),
                server_bind_address: server_bind_address.as_str().into(),
            },
            _ => {
                return Err(PyErr::new::<PyRuntimeError, _>(
                    "Could not create server because of missing configuration",
                ))
            }
        };
        runtime.spawn(async move {
            if let Err(e) = reduce_core::start_server(config).await {
                eprintln!("Error starting server: {}", e);
            }
        });

        self.runtime = Some(runtime);

        Ok(())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyreduce(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ServerConfig>()?;
    Ok(())
}
