//! Lossless structured/model channels from one qualified DTO, with metered wire bytes.
use super::{Cost, Error, Output, Request, Stop};
use crate::ir::source_observations::request_budget::Permit;
use serde::Serialize;
use serde_json::Value;
use std::io::{self, Write};
pub(super) struct Frame {
    pub wire: String,
    _retained: Vec<Permit>,
    _outputs: Vec<Output>,
}
#[derive(Serialize)]
struct Text<'a> {
    r#type: &'static str,
    text: &'a str,
}
#[derive(Serialize)]
struct ResultBody<'a> {
    content: [Text<'a>; 1],
    #[serde(rename = "structuredContent")]
    structured_content: &'a Value,
}
#[derive(Serialize)]
struct Rpc<'a> {
    jsonrpc: &'static str,
    id: &'a Value,
    result: ResultBody<'a>,
}
struct Counter<'a> {
    request: &'a Request,
    bytes: u64,
}
impl Write for Counter<'_> {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.request
            .checkpoint_final()
            .map_err(|s| io::Error::other(format!("{s:?}")))?;
        self.bytes = self
            .bytes
            .checked_add(b.len() as u64)
            .ok_or_else(|| io::Error::other("wire overflow"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) fn framing_capacity(id: &Value, args: &Value, items: usize) -> Result<u64, Error> {
    // Retain a complete fixed error for every later item, plus both root scopes
    // and IDs. The untrusted ingress already obeys the existing MCP line cap.
    let bytes=serde_json::to_vec(&serde_json::json!({"id":id,"workspaceRoot":args["workspaceRoot"],"withinPath":args["withinPath"],"ids":args["queries"].as_array().map(|a|a.iter().map(|v|v["id"].clone()).collect::<Vec<_>>())})).map_err(|e|Error::invalid(e.to_string()))?.len()as u64;
    bytes
        .checked_mul(16)
        .and_then(|n| n.checked_add((items as u64).checked_mul(2048)?))
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| Error::resource("Public framing overflow"))
}
fn rpc<'a>(id: &'a Value, dto: &'a Value, text: &'a str) -> Rpc<'a> {
    Rpc {
        jsonrpc: "2.0",
        id,
        result: ResultBody {
            content: [Text {
                r#type: "text",
                text,
            }],
            structured_content: dto,
        },
    }
}
pub(super) fn prepare(
    id: &Value,
    mut dto: Value,
    request: &Request,
    credit: crate::ir::source_observations::request_budget::response_capacity::ResponseCredit,
    capacity: u64,
    mut retained: Vec<Permit>,
    outputs: Vec<Output>,
) -> Result<Frame, Error> {
    let quantum = request.reserve_finalization(Cost::default())?;
    let text = serde_json::to_string(&dto).map_err(|e| Error::invalid(e.to_string()))?;
    let mut counter = Counter { request, bytes: 1 }; // newline framing
    serde_json::to_writer(&mut counter, &rpc(id, &dto, &text))
        .map_err(|_| Error::resource("Deadline/overflow during transport counting"))?;
    let charged = counter.bytes.checked_add(512).ok_or(Stop::ResponseBytes)?;
    if charged > capacity {
        return Err(Error::resource(
            "Minimum truthful structured/text response exceeds public capacity",
        ));
    }
    // The adapter reserved all DTO-copy and wire capacity before optional work.
    retained.push(quantum);
    credit.commit(charged)?;
    // Refresh aggregate transport accounting without changing any claim, scope,
    // currentness or qualification. A conservative framing margin is charged.
    let snapshot = super::execution::execution_snapshot(request);
    for key in [
        "used",
        "reserved",
        "peak_retained_bytes",
        "released_retained_bytes",
    ] {
        dto["execution"][key] = snapshot[key].clone();
    }
    let text = serde_json::to_string(&dto).map_err(|e| Error::invalid(e.to_string()))?;
    let wire =
        serde_json::to_string(&rpc(id, &dto, &text)).map_err(|e| Error::invalid(e.to_string()))?;
    if wire.len() as u64 + 1 > charged {
        return Err(Error::resource(
            "Transport framing exceeded its reservation",
        ));
    }
    request.checkpoint_final()?;
    eprintln!(
        "[resolver-evidence-v2] wire_bytes={} charged_wire_bytes={} retained_peak={} active_peak={}",
        wire.len() + 1,
        charged,
        request.snapshot().retained_peak,
        request.snapshot().active_peak
    );
    Ok(Frame {
        wire,
        _retained: retained,
        _outputs: outputs,
    })
}
