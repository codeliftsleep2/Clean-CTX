//! Serving-boundary validation, predicate reconciliation and bounded owned serialization.
use super::{Failure, Prepared, Query, WorkResult, claims, dto};
use crate::ir::source_observations::{
    bounded,
    request_budget::{Cost, Permit, Request, Stop},
};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

pub struct Output {
    dto: dto::OwnedDto,
    bytes: Vec<u8>,
    measurements: Value,
    _reservations: Vec<Permit>,
}
impl Output {
    pub fn dto(&self) -> &dto::OwnedDto {
        &self.dto
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn measurements(&self) -> &Value {
        &self.measurements
    }
}
fn current(
    value: crate::ir::source_observations::request_budget::Reserved<bool>,
) -> Result<bool, Failure> {
    if let Some(stop) = value.completed_after_stop {
        return Err(Failure::Budget(stop));
    }
    Ok(*value.value())
}
pub(super) fn empty(
    request: &Request,
    query: &Query,
    roots: &[PathBuf],
    stop: Stop,
    reservation: Permit,
) -> Result<Output, Failure> {
    serialize(
        request,
        dto::base(request, query, roots, Some(stop)),
        vec![reservation],
        json!({"inventory_stopped":true}),
    )
}
pub(super) fn finish(
    request: &Request,
    query: &Query,
    work: WorkResult,
) -> Result<Output, Failure> {
    let WorkResult {
        mut inventory,
        sources,
        failures,
        stopped,
        mut timings,
        base_reservation,
    } = work;
    let validation_time = Instant::now();
    let mut boundary = bounded::Boundary::new(request);
    let mut reservations = vec![base_reservation];
    reservations.push(
        inventory
            .dto_reservation
            .take()
            .expect("reserved inventory DTO"),
    );
    let mut source_current = Vec::with_capacity(sources.len());
    for source in &sources {
        source_current.push(super::super::validated(
            boundary.pre_reserved(bounded::witnesses(&source.source)),
        ));
    }
    let inventory_current = match boundary.pre_reserved(&inventory.witnesses) {
        Ok(result) => current(result)?,
        Err(_) => false,
    };
    request.checkpoint_final()?;
    let mut value = dto::base(request, query, &inventory.roots, stopped);
    if !inventory_current {
        value["failures"]
            .as_array_mut()
            .expect("array")
            .push(dto::issue(
            "inventory_incomplete",
            "inventory_validation",
            "request",
            "inventory currentness could not be established; source claims validate independently",
        ));
    }
    value["examined_scope"]["enumerated"] = json!(inventory.files);
    value["examined_scope"]["excluded"] = json!(
        inventory
            .excluded
            .iter()
            .map(|(p, r)| json!({"path":p,"reason":r}))
            .collect::<Vec<_>>()
    );
    value["examined_scope"]["unexamined_count_known"] = json!(inventory.complete);
    for failure in failures {
        value["failures"]
            .as_array_mut()
            .expect("array")
            .push(dto::issue(
                "inventory_incomplete",
                "source_acquisition",
                "request",
                failure,
            ));
    }
    let mut complete = inventory.complete
        && inventory_current
        && sources.len() == inventory.files.len()
        && !sources.is_empty();
    // Validate every independently owned source before optional proof work can
    // consume remaining validation capacity. Credits were reserved before expansion.
    for (source_index, source) in sources.into_iter().enumerate() {
        reservations.push(source.dto_reservation);
        let source_id = dto::source_id(source.source.source(), source.source.source_hash());
        let identity = dto::source(source.source.source(), source.source.source_hash());
        value["examined_scope"]["read"]
            .as_array_mut()
            .expect("array")
            .push(identity.clone());
        let state = source_current[source_index];
        let valid = state == super::super::Validation::Current;
        value["currentness"]["components"]
            .as_array_mut()
            .expect("array")
            .push(dto::validation(&source_id, state));
        if !valid {
            complete = false;
            value["failures"]
                .as_array_mut()
                .expect("array")
                .push(dto::issue(
                    if state == super::super::Validation::Stale {
                        "source_changed"
                    } else {
                        "unvalidated"
                    },
                    "serving_boundary",
                    &source_id,
                    format!("source currentness: {state:?}"),
                ));
            continue;
        }
        value["examined_scope"]["admitted"]
            .as_array_mut()
            .expect("array")
            .push(identity);
        if !bounded::scan_complete(&source.source) {
            complete = false;
            if let Some(stop) = bounded::scan_stop(&source.source) {
                value["coverage"]["supported_written_source"]["omissions"]
                    .as_array_mut()
                    .expect("array")
                    .push(dto::issue(
                        dto::stop_code(stop),
                        "source_traversal",
                        &source_id,
                        "source AST examination stopped; written observations remain lower bound",
                    ));
            }
        }
        if !source.source.omissions().is_empty() {
            complete = false;
            for omission in source.source.omissions() {
                value["coverage"]["supported_written_source"]["omissions"]
                    .as_array_mut()
                    .expect("array")
                    .push(dto::issue(
                        "unsupported_container",
                        "written_source_scan",
                        &source_id,
                        format!(
                            "{:?} at {}..{}",
                            omission.reason, omission.span.start, omission.span.end
                        ),
                    ));
            }
        }
        for support in source.support {
            let mut bytes = 0u64;
            for proof in [
                support.direct.as_ref().ok(),
                support.role.as_ref().ok().map(|r| r.evidence()),
            ]
            .into_iter()
            .flatten()
            {
                bytes = bytes
                    .checked_add(
                        proof
                            .retained_estimate()
                            .checked_mul(16)
                            .ok_or(Stop::RetainedBytes)?,
                    )
                    .ok_or(Stop::RetainedBytes)?;
            }
            let credit = if bytes == 0 {
                Ok(None)
            } else {
                request
                    .reserve_nested(
                        Cost {
                            retained_bytes: bytes,
                            ..Cost::default()
                        },
                        false,
                    )
                    .map(Some)
            };
            let (direct, direct_error) = match support.direct {
                Ok(e) => (Some(e), None),
                Err(e) => (None, Some(e)),
            };
            let (role, role_error) = match support.role {
                Ok(e) => (Some(e), None),
                Err(e) => (None, Some(e)),
            };
            let (direct, role, direct_error, role_error) = match credit {
                Ok(permit) => {
                    if let Some(permit) = permit {
                        reservations.push(permit);
                    }
                    (direct, role, direct_error, role_error)
                }
                Err(stop) => (
                    None,
                    None,
                    Some(crate::ir::source_observations::direct_identity::Obligation::Budget(stop)),
                    Some(crate::ir::source_observations::direct_identity::Obligation::Budget(stop)),
                ),
            };
            let prepared =
                Prepared::admit_at(&mut boundary, &source.source, support.index, direct, role);
            let mut errors = Vec::new();
            if let Some(e) = &direct_error {
                errors.push(("direct_support", e));
            }
            if let Some(e) = &role_error {
                errors.push(("angular_role", e));
            }
            claims::append(
                &mut value,
                &prepared,
                &inventory.roots,
                source_index,
                support.index,
                &errors,
            )?;
        }
    }
    for file in &inventory.files {
        if !value["examined_scope"]["read"]
            .as_array()
            .expect("array")
            .iter()
            .any(|s| s["path"].as_str() == file.to_str())
        {
            value["examined_scope"]["unexamined"]
                .as_array_mut()
                .expect("array")
                .push(json!(file));
        }
    }
    let identities = value["examined_scope"]["admitted"].clone();
    let refs = identities
        .as_array()
        .expect("array")
        .iter()
        .map(|s| {
            json!(format!(
                "source:{}@{}",
                s["path"].as_str().expect("path"),
                s["sha256"].as_str().expect("hash")
            ))
        })
        .collect::<Vec<_>>();
    for (domain, closed) in [
        ("inventory", inventory.complete && inventory_current),
        ("supported_written_source", complete),
    ] {
        value["coverage"][domain]["state"] = json!(if closed { "closed" } else { "open" });
        value["coverage"][domain]["predicate"]["sources"] = identities.clone();
        value["coverage"][domain]["evidence"] = json!(refs);
    }
    value["response_completeness"]["closure_support_communicated"] = json!(complete);
    let count = value["observations"].as_array().expect("array").len() as u64;
    value["matching_records"]["established"] = json!(count);
    value["matching_records"]["returned"] = json!(count);
    value["search_conclusion"]["state"] = json!(if count > 0 {
        "observed"
    } else if complete {
        "absent_within_closed_predicate"
    } else {
        "not_observed_unknown"
    });
    value["search_conclusion"]["predicate"] =
        dto::predicate(query, identities, "supported_written_mapping");
    value["search_conclusion"]["evidence"] = json!(refs);
    timings.insert(
        "serving_validation_and_dto_us".into(),
        json!(validation_time.elapsed().as_micros()),
    );
    timings.insert(
        "serving_validation_us".into(),
        json!(boundary.elapsed().as_micros()),
    );
    timings.insert(
        "dto_construction_and_reconciliation_us".into(),
        json!(
            validation_time
                .elapsed()
                .saturating_sub(boundary.elapsed())
                .as_micros()
        ),
    );
    drop(boundary);
    drop(inventory);
    serialize(request, value, reservations, Value::Object(timings))
}

struct Counter<'a> {
    request: &'a Request,
    bytes: u64,
}
impl Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.request
            .checkpoint_final()
            .map_err(|e| io::Error::other(format!("serialization: {e:?}")))?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| io::Error::other("response overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn count(request: &Request, value: &Value) -> Result<u64, Failure> {
    let mut writer = Counter { request, bytes: 0 };
    serde_json::to_writer(&mut writer, value).map_err(|_| {
        Failure::Budget(request.snapshot().last_stop.unwrap_or(Stop::ResponseBytes))
    })?;
    Ok(writer.bytes)
}
pub(super) fn preflight(
    request: &Request,
    query: &Query,
    roots: &[PathBuf],
) -> Result<(), Failure> {
    let base = dto::base(request, query, roots, None);
    if count(request, &base)?
        .checked_add(512)
        .ok_or(Stop::ResponseBytes)?
        > request.response_available()
    {
        return Err(Failure::Budget(request.stop(Stop::ResponseBytes)));
    }
    Ok(())
}
fn serialize(
    request: &Request,
    mut value: Value,
    mut reservations: Vec<Permit>,
    mut measurements: Value,
) -> Result<Output, Failure> {
    let started = Instant::now();
    request.checkpoint_final()?;
    let _quantum = request.reserve_finalization(Cost::default())?;
    value["execution"] = dto::execution(request, request.snapshot().last_stop);
    if measurements["source_completed_after_stop"]
        .as_u64()
        .unwrap_or(0)
        > 0
    {
        value["execution"]["completed_after_stop"].as_array_mut().expect("array").push(json!({"stage":"source_capture","elapsed_ms":request.snapshot().elapsed.as_millis(),"charged_bytes":request.snapshot().used.source_bytes}));
    }
    let available = request.response_available();
    let required = count(request, &value)?
        .checked_add(512)
        .ok_or(Stop::ResponseBytes)?;
    if required > available {
        let mut removed = 0u64;
        let mut saved = 0u64;
        while saved < required.saturating_sub(available).saturating_add(2048) {
            let Some(record) = value["observations"].as_array_mut().expect("array").pop() else {
                break;
            };
            saved = saved
                .checked_add(count(request, &record)?)
                .ok_or(Stop::ResponseBytes)?;
            let id = record["id"].as_str().expect("id");
            let mut removed_ids = vec![id.to_string()];
            for f in value["failures"].as_array().expect("array") {
                if f["affected"]
                    .as_array()
                    .is_some_and(|a| a.iter().any(|x| x == id))
                {
                    removed_ids.extend(
                        f["affected"]
                            .as_array()
                            .expect("array")
                            .iter()
                            .filter_map(|x| x.as_str())
                            .filter(|s| s.starts_with("attempt:"))
                            .map(str::to_owned),
                    );
                }
            }
            for field in ["direct_evidence", "interpretations"] {
                let mut keep = Vec::new();
                for child in value[field].as_array_mut().expect("array").drain(..) {
                    if child["source_claim"] == id {
                        removed_ids.push(child["id"].as_str().expect("id").to_string());
                        saved = saved
                            .checked_add(count(request, &child)?)
                            .ok_or(Stop::ResponseBytes)?;
                    } else {
                        keep.push(child);
                    }
                }
                value[field] = Value::Array(keep);
            }
            value["currentness"]["components"]
                .as_array_mut()
                .expect("array")
                .retain(|c| !removed_ids.iter().any(|id| c["component"] == id.as_str()));
            value["failures"]
                .as_array_mut()
                .expect("array")
                .retain(|f| {
                    !f["affected"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|item| item.as_str() == Some(id)))
                });
            removed += 1;
        }
        if removed > 0 {
            let returned = value["observations"].as_array().expect("array").len();
            value["matching_records"]["returned"] = json!(returned);
            value["matching_records"]["withheld"] = json!(removed);
            value["response_completeness"]["state"] = json!("partial");
            value["response_omissions"] = json!([{"reason":"response capacity; complete matching records withheld","count":removed,"affected":["request"]}]);
            if returned == 0 {
                value["search_conclusion"]["state"] = json!("observed_not_returned");
            }
        }
    }
    let size = count(request, &value)?
        .checked_add(512)
        .ok_or(Stop::ResponseBytes)?;
    if size > available {
        return Err(Failure::Budget(request.stop(Stop::ResponseBytes)));
    }
    // Base/source/support DTO envelopes already own tree, lookup and encoded
    // buffer capacity. Keep those permits through Output; do not double charge.
    let permit = request.reserve_nested(
        Cost {
            response_bytes: size,
            ..Cost::default()
        },
        true,
    )?;
    let completed = value["execution"]["completed_after_stop"].clone();
    value["execution"] = dto::execution(request, request.snapshot().last_stop);
    value["execution"]["completed_after_stop"] = completed;
    if value["matching_records"]["withheld"].as_u64().unwrap_or(0) > 0 {
        value["execution"]["state"] = json!("exhausted");
        value["execution"]["stops"]
            .as_array_mut()
            .expect("array")
            .push(dto::issue(
                "response_bytes",
                "response_preparation",
                "request",
                "complete records withheld",
            ));
    }
    let dto = dto::OwnedDto::checked(value)?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(size).map_err(|_| Failure::Budget(Stop::ResponseBytes))?,
    );
    serde_json::to_writer(&mut bytes, dto.value())
        .map_err(|e| Failure::Integrity(e.to_string()))?;
    if bytes.len() as u64 > size {
        return Err(Failure::Budget(request.stop(Stop::ResponseBytes)));
    }
    request.checkpoint_final()?;
    measurements["serialization_us"] = json!(started.elapsed().as_micros());
    measurements["response_bytes"] = json!(bytes.len());
    measurements["deadline_overshoot_us"] =
        json!(request.snapshot().deadline_overshoot.as_micros());
    reservations.push(permit);
    Ok(Output {
        dto,
        bytes,
        measurements,
        _reservations: reservations,
    })
}
