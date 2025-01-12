use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsValue;
use web_sys::{window, Storage};

/// For interacting with the local storage and session storage APIs.
pub struct BrowserStorages {
    pub local: BrowserStorage,
    pub session: BrowserStorage,
}

impl BrowserStorages {
    pub fn new() -> Self {
        Self {
            local: BrowserStorage::new(window().unwrap().local_storage().ok().flatten()),
            session: BrowserStorage::new(window().unwrap().session_storage().ok().flatten()),
        }
    }
}

/// For interacting with the web storage API.
pub struct BrowserStorage {
    inner: Option<Storage>,
}

/// Errors that can occur with storages.
#[derive(Debug)]
pub enum Error {
    /// Javascript error.
    Js(JsValue),
    /// Serialization error.
    Serde(serde_json::Error),
    /// Could not un-quote.
    Unquote,
    /// Storage API is not available.
    Nonexistent,
}

impl BrowserStorage {
    /// If storage API is unavailable, future calls will return `Err(Error::Nonexistent)`.
    pub(crate) fn new(inner: Option<Storage>) -> Self {
        Self { inner }
    }

    /// Gets a key from storage, returning None if it doesn't exist or any error occurs.
    pub fn get<V: DeserializeOwned>(&self, key: &str, unquote: bool) -> Option<V> {
        self.try_get(key, unquote).ok().flatten()
    }

    /// Gets a key from storage, returning Ok(None) if it doesn't exist or Err if an error occurs.
    pub fn try_get<V: DeserializeOwned>(
        &self,
        key: &str,
        unquote: bool,
    ) -> Result<Option<V>, Error> {
        let inner = self.inner.as_ref().ok_or(Error::Nonexistent)?;

        let s: Option<String> = inner.get(key).map_err(Error::Js)?;

        match s {
            Some(s) => {
                let processed = if unquote { format!(r#""{}""#, s) } else { s };

                serde_json::from_str(&processed).map_err(Error::Serde)
            }
            None => Ok(None),
        }
    }

    /// Sets a key in storage to a value.
    pub fn set<V: Serialize>(
        &mut self,
        key: &str,
        value: Option<V>,
        unquote: bool,
    ) -> Result<(), Error> {
        let inner = self.inner.as_ref().ok_or(Error::Nonexistent)?;

        match value {
            Some(ref v) => {
                let s = serde_json::to_string(v).map_err(Error::Serde)?;

                let processed = if unquote {
                    if s.starts_with('"') && s.ends_with('"') {
                        String::from(&s[1..s.len() - 1])
                    } else {
                        return Err(Error::Unquote);
                    }
                } else {
                    s
                };

                inner.set(key, &processed).map_err(Error::Js)
            }
            None => inner.delete(key).map_err(Error::Js),
        }
    }
}
