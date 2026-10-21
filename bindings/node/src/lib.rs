use napi::Env;
use napi::bindgen_prelude::{JsObjectValue, JsValue};
use napi_derive::napi;
use packvium_core::PackError;

/// A JavaScript `Error` carrying the core's message unchanged and, as properties, its closed
/// `code` and, for a named refusal, `reason`, `field` and `detail` -- what @packvium/engine's
/// `index.js` turns into the same error classes its JavaScript fallback throws. The
/// same shape the WebAssembly binding's `to_js_error` raises.
///
/// A property that cannot be set is reported instead, as the WebAssembly binding does; on a
/// fresh `Error` that does not happen.
fn to_js_error(env: &Env, error: PackError) -> napi::Error {
    structured_error(env, &error).unwrap_or_else(|failure| failure)
}

fn structured_error(env: &Env, error: &PackError) -> napi::Result<napi::Error> {
    let mut raised = env.create_error(napi::Error::from_reason(error.to_string()))?;
    let request_error = error.request_error();
    let properties = [
        ("code", Some(error.code())),
        ("reason", request_error.map(|refusal| refusal.reason())),
        ("field", request_error.map(|refusal| refusal.field())),
        ("detail", request_error.map(|refusal| refusal.detail())),
    ];
    let present = properties
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| (name, value)));
    for (name, value) in present {
        raised.set_named_property(name, value)?;
    }
    // An error built from a JS value keeps a reference to it, so the object thrown is this
    // one, properties included, rather than one rebuilt from the message.
    Ok(napi::Error::from(raised.to_unknown()))
}

#[napi]
pub fn pack_json(env: Env, request: String) -> napi::Result<String> {
    packvium_core::pack_json(&request).map_err(|error| to_js_error(&env, error))
}

#[napi]
pub fn rebalance_json(
    env: Env,
    request: String,
    result: String,
    max_moves: u32,
) -> napi::Result<String> {
    packvium_core::rebalance_json(&request, &result, max_moves as usize)
        .map_err(|error| to_js_error(&env, error))
}

/// The exported commercial and control-plane API, over the same
/// `{"document": ..., "request": ...}` JSON every binding uses (docs/COMMERCE-API.md).
///
/// A rejection the commercial model is entitled to make comes back as a normal result
/// with `"status":"rejected"`; only a malformed input becomes a thrown error.
#[napi]
pub fn commerce_quote_json(call: String) -> napi::Result<String> {
    packvium_core::commerce::quote_json(&call)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

#[napi]
pub fn commerce_evaluate_policy_json(call: String) -> napi::Result<String> {
    packvium_core::commerce::evaluate_policy_json(&call)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

#[napi]
pub fn commerce_catalog_version_info_json(call: String) -> napi::Result<String> {
    packvium_core::commerce::catalog_version_info_json(&call)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

/// The reference validator's verdict on a result (docs/VALIDATION-CONTRACT.md), as one JSON
/// document: `{"issues": [...]}`, or `{"refusal": {"kind", "message"}}` for documents it cannot
/// judge. A refusal is returned rather than thrown so its `kind` survives the boundary;
/// @packvium/engine turns it into a `MalformedResult`.
#[napi]
pub fn validate_json(request: String, result: String) -> String {
    packvium_core::validate_json_outcome(&request, &result)
}

#[napi]
pub fn version() -> String {
    packvium_core::VERSION.to_owned()
}
