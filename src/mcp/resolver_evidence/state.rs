//! Session-pinned operator selection and cancellation for queued/active opt-in calls.
use super::{Error, Request, budget::Budget};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema: String,
    ceiling: Budget,
}
#[derive(Default)]
struct Slot {
    cancelled: bool,
    request: Option<Request>,
}
pub(crate) struct State {
    pub(super) ceiling: Result<Option<Budget>, String>,
    slots: Mutex<HashMap<String, Slot>>,
}
impl State {
    pub fn from_environment() -> Self {
        let ceiling = (|| {
            let Some(path) = std::env::var_os("CLEAN_CTX_RESOLVER_EVIDENCE_PROFILE") else {
                return Ok(None);
            };
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err("Operator profile must be absolute".into());
            }
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            let profile: Profile = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if profile.schema != "clean-ctx/resolver-evidence-operator-v1" {
                return Err("Unsupported operator profile schema".into());
            }
            profile.ceiling.limits().map_err(|e| e.message)?;
            Ok(Some(profile.ceiling))
        })();
        Self {
            ceiling,
            slots: Mutex::new(HashMap::new()),
        }
    }
    #[cfg(test)]
    pub(super) fn configured(ceiling: Budget) -> Self {
        Self {
            ceiling: Ok(Some(ceiling)),
            slots: Mutex::new(HashMap::new()),
        }
    }
    pub fn enabled(&self) -> bool {
        matches!(self.ceiling, Ok(Some(_)))
    }
    pub fn ceiling(&self) -> Option<&Budget> {
        self.ceiling.as_ref().ok().and_then(Option::as_ref)
    }
    pub(super) fn required(&self) -> Result<&Budget, Error> {
        self.ceiling().ok_or_else(|| {
            Error::invalid(format!(
                "Resolver evidence v2 disabled: {}",
                self.ceiling
                    .as_ref()
                    .err()
                    .map_or("no operator ceiling profile", String::as_str)
            ))
        })
    }
    pub(crate) fn queue(&self, req: &crate::protocol::JsonRpcRequest) -> Result<(), Error> {
        if !self.enabled() || req.method != "tools/call" {
            return Ok(());
        }
        let Some(params) = &req.params else {
            return Ok(());
        };
        let args = &params["arguments"];
        if params["name"] != "workspace_query" || !super::eligible(args) {
            return Ok(());
        }
        if let Some(id) = &req.id {
            let key = id.to_string();
            let mut slots = self.slots.lock().expect("cancellation slots");
            if slots.contains_key(&key) {
                return Err(Error::invalid("Duplicate in-flight v2 request ID"));
            }
            slots.insert(key, Slot::default());
        }
        Ok(())
    }
    pub(crate) fn cancel(&self, id: &Value) {
        if let Some(slot) = self
            .slots
            .lock()
            .expect("cancellation slots")
            .get_mut(&id.to_string())
        {
            slot.cancelled = true;
            if let Some(request) = &slot.request {
                request.cancel_optional();
            }
        }
    }
    pub(super) fn bind(&self, id: &Value, request: &Request) -> Result<(), Error> {
        let mut slots = self.slots.lock().expect("cancellation slots");
        let slot = slots.entry(id.to_string()).or_default();
        if slot.request.is_some() {
            return Err(Error::invalid("Duplicate executing v2 request ID"));
        }
        if slot.cancelled {
            request.cancel_optional();
        }
        slot.request = Some(request.clone());
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn request(&self, id: &Value) -> Request {
        self.slots.lock().unwrap()[&id.to_string()]
            .request
            .clone()
            .unwrap()
    }
    pub(crate) fn retire(&self, id: &Value) {
        self.slots
            .lock()
            .expect("cancellation slots")
            .remove(&id.to_string());
    }
}
