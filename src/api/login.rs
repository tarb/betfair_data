use pyo3::PyErr;
use pyo3::{exceptions, pyclass, pyfunction, types::PyString, Py, PyAny, PyResult, Python};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::lazy::SyncOnceCell;
use strum_macros::{AsRefStr, IntoStaticStr};

static CLIENT: SyncOnceCell<Client> = SyncOnceCell::new();

#[pyclass]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr)]
pub enum LoginStatus {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "LIMITED_ACCESS")]
    Limited,
    #[serde(rename = "LOGIN_RESTRICTED")]
    Restricted,
    #[serde(rename = "FAIL")]
    Fail,
}
impl LoginStatus {
    fn __repr__(&self) -> &'static str {
        match self {
            LoginStatus::Success => "SUCCESS",
            LoginStatus::Limited => "LIMITED_ACCESS",
            LoginStatus::Restricted => "LOGIN_RESTRICTED",
            LoginStatus::Fail => "FAIL",
        }
    }
}

#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginError {
    status: LoginStatus,
    detail: String,
}

/// login request to Betfair api. Returns session token on success
/// # Arguments
/// * `username` - Betfair account username
/// * `password` - Betfair account passord
/// * `password` - Betfair AppKey
#[pyfunction]
pub fn login<'py>(
    py: Python<'py>,
    username: &str,
    password: &str,
    app_key: &str,
) -> PyResult<&'py PyAny> {
    #[derive(Deserialize)]
    pub struct LoginResult<'a> {
        #[serde(borrow)]
        token: &'a str,
        #[serde(borrow)]
        error: &'a str,
        // product: &'a str,
        status: LoginStatus,
    }

    let client = CLIENT.get_or_init(Client::new).clone();
    let resp = client
        .post("https://identitysso.betfair.com/api/login")
        .form(&[("username", username), ("password", password)])
        .header("X-Application", app_key)
        .header("Accept", "application/json")
        .header("Connection", "keep-alive")
        .send();

    pyo3_asyncio::tokio::future_into_py(py, async {
        let resp = resp
            .await
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?
            .error_for_status()
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;

        let bs = resp
            .bytes()
            .await
            .map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string()))?;
        let lr = serde_json::from_slice::<LoginResult>(&bs[..]);

        match lr {
            Ok(LoginResult {
                token,
                error: _,
                status: LoginStatus::Success,
            }) => Python::with_gil(|py| {
                let str: Py<PyString> = PyString::new(py, token).into();
                Ok(str)
            }),
            Ok(LoginResult {
                token: _,
                error,
                status: _,
            }) => {
                let e = error.to_string();
                Err(PyErr::new::<exceptions::PyRuntimeError, _>(e))
            }
            Err(err) => Err(PyErr::new::<exceptions::PyRuntimeError, _>(err.to_string())),
        }
    })
}
