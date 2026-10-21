use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

/// Never null in practice -- `VERSION` has no interior NUL byte -- but callers should
/// still check before dereferencing, since the return type is the same fallible
/// `into_raw` every other function here uses. The returned pointer is owned by the
/// caller and must be released with [`packvium_free_string`] exactly once.
#[unsafe(no_mangle)]
pub extern "C" fn packvium_version() -> *mut c_char {
    into_raw(packvium_core::VERSION.to_owned())
}

/// # Safety
///
/// `request` must be either null or point to a valid, null-terminated C string that
/// remains valid (not freed, not mutated by another thread) for the duration of this
/// call. The returned pointer, when non-null, is owned by the caller and must be
/// released with [`packvium_free_string`] exactly once, never with `free` or any
/// other deallocator. A panic inside the solver is caught at this boundary and
/// reported as an error string rather than unwinding into the caller's frames --
/// unwinding across `extern "C"` is not permitted without `extern "C-unwind"`, and the
/// default landing pad rustc inserts here would otherwise abort the host process
///.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_solve_json(request: *const c_char) -> *mut c_char {
    if request.is_null() {
        return error_json("null request");
    }
    let input = match unsafe { CStr::from_ptr(request) }.to_str() {
        Ok(value) => value,
        Err(_) => return error_json("request is not UTF-8"),
    };
    match catch_unwind(AssertUnwindSafe(|| packvium_core::pack_json(input))) {
        Ok(Ok(value)) => into_raw(value),
        Ok(Err(error)) => pack_error_json(&error),
        Err(_) => error_json("internal panic while solving (caught at the FFI boundary)"),
    }
}

/// Quote a shipment through the deterministic commercial API.
///
/// # Safety
///
/// `call` follows the same input and returned-string ownership contract as
/// [`packvium_solve_json`]. It is a JSON object containing `document` and `request`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_commerce_quote(call: *const c_char) -> *mut c_char {
    unsafe {
        call_json(call, "quoting", |input| {
            packvium_core::commerce::quote_json(input).map_err(|error| error.to_string())
        })
    }
}

/// Evaluate policy through the deterministic commercial API.
///
/// # Safety
///
/// `call` follows the same input and returned-string ownership contract as
/// [`packvium_solve_json`]. It is a JSON object containing `document` and `request`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_commerce_evaluate_policy(call: *const c_char) -> *mut c_char {
    unsafe {
        call_json(call, "evaluating commerce policy", |input| {
            packvium_core::commerce::evaluate_policy_json(input).map_err(|error| error.to_string())
        })
    }
}

/// Resolve catalog metadata through the deterministic commercial API.
///
/// # Safety
///
/// `call` follows the same input and returned-string ownership contract as
/// [`packvium_solve_json`]. It is a JSON object containing `document` and `request`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_commerce_catalog_version_info(
    call: *const c_char,
) -> *mut c_char {
    unsafe {
        call_json(call, "resolving commerce catalog metadata", |input| {
            packvium_core::commerce::catalog_version_info_json(input)
                .map_err(|error| error.to_string())
        })
    }
}

/// Rebalance a canonical result through the Rust core.
///
/// # Safety
///
/// `request` and `result` follow the same valid, immutable, NUL-terminated UTF-8
/// pointer contract as [`packvium_solve_json`]. The returned string is owned by
/// the caller and must be released exactly once with [`packvium_free_string`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_rebalance_json(
    request: *const c_char,
    result: *const c_char,
    max_moves: usize,
) -> *mut c_char {
    if request.is_null() || result.is_null() {
        return error_json("null request or result");
    }
    let request = match unsafe { CStr::from_ptr(request) }.to_str() {
        Ok(value) => value,
        Err(_) => return error_json("request is not UTF-8"),
    };
    let result = match unsafe { CStr::from_ptr(result) }.to_str() {
        Ok(value) => value,
        Err(_) => return error_json("result is not UTF-8"),
    };
    match catch_unwind(AssertUnwindSafe(|| {
        packvium_core::rebalance_json(request, result, max_moves)
    })) {
        Ok(Ok(value)) => into_raw(value),
        Ok(Err(error)) => pack_error_json(&error),
        Err(_) => error_json("internal panic while rebalancing (caught at the FFI boundary)"),
    }
}

/// # Safety
///
/// `value` must be either null or a pointer previously returned by
/// [`packvium_solve_json`], [`packvium_rebalance_json`] or
/// [`packvium_version`] that has not already been freed. Calling this more than
/// once on the same non-null pointer (a double free) or on a pointer this library did
/// not allocate is undefined behaviour.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn packvium_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

fn into_raw(value: String) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

fn error_json(message: &str) -> *mut c_char {
    into_raw(serde_json::json!({"status":"error","error":message}).to_string())
}

fn pack_error_json(error: &packvium_core::PackError) -> *mut c_char {
    let mut payload = serde_json::json!({
        "status": "error",
        "error": error.to_string(),
        "code": error.code(),
    });
    if let Some(req) = error.request_error() {
        payload["reason"] = serde_json::Value::String(req.reason().to_string());
        payload["field"] = serde_json::Value::String(req.field().to_string());
        payload["detail"] = serde_json::Value::String(req.detail().to_string());
    }
    into_raw(payload.to_string())
}

unsafe fn call_json(
    input: *const c_char,
    operation: &str,
    call: impl FnOnce(&str) -> Result<String, String>,
) -> *mut c_char {
    if input.is_null() {
        return error_json("null request");
    }
    let input = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(value) => value,
        Err(_) => return error_json("request is not UTF-8"),
    };
    match catch_unwind(AssertUnwindSafe(|| call(input))) {
        Ok(Ok(value)) => into_raw(value),
        Ok(Err(error)) => error_json(&error),
        Err(_) => error_json(&format!(
            "internal panic while {operation} (caught at the FFI boundary)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_string(ptr: *mut c_char) -> String {
        assert!(!ptr.is_null());
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { packvium_free_string(ptr) };
        value
    }

    #[test]
    fn version_is_non_empty() {
        let ptr = packvium_version();
        assert!(!to_string(ptr).is_empty());
    }

    #[test]
    fn null_request_is_reported_as_an_error_not_a_null_pointer() {
        let ptr = unsafe { packvium_solve_json(ptr::null()) };
        assert!(to_string(ptr).contains("null request"));
    }

    #[test]
    fn invalid_utf8_is_reported_as_an_error() {
        let bytes = [0x66u8, 0xff, 0x00]; // 'f', an invalid byte, then the terminator
        let ptr = unsafe { packvium_solve_json(bytes.as_ptr().cast()) };
        assert!(to_string(ptr).contains("not UTF-8"));
    }

    #[test]
    fn malformed_json_is_a_clean_error_not_a_crash() {
        let input = CString::new("{not json").unwrap();
        let ptr = unsafe { packvium_solve_json(input.as_ptr()) };
        assert!(to_string(ptr).contains("\"status\":\"error\""));
    }

    #[test]
    fn refused_request_returns_structured_error_code_reason_and_field() {
        let input = CString::new(
            r#"{"items":[{"id":"a","dimensions":{"length":"-10","width":"10","height":"10"}}],"containers":[]}"#,
        )
        .unwrap();
        let ptr = unsafe { packvium_solve_json(input.as_ptr()) };
        let str_val = to_string(ptr);
        let parsed: serde_json::Value = serde_json::from_str(&str_val).unwrap();
        assert_eq!(parsed["status"], "error");
        assert_eq!(parsed["code"], "invalid_request");
        assert_eq!(parsed["reason"], "negative_measure");
        assert_eq!(parsed["field"], "/items/0/dimensions/length");
    }

    #[test]
    fn commerce_quote_round_trips_through_the_c_abi() {
        let call = CString::new(
            r#"{"document":{"tariffs":[{"carrier_id":"acme","service_id":"ground","versions":[{"effective_at":0,"dimensional_weight_divisor":5000,"cost_per_dimensional_kg_minor":{"zone-a":450},"minimum_charge_minor":900,"fuel_surcharge_permille":120,"accessorials":[]}]}]},"request":{"carrier_id":"acme","service_id":"ground","tariff_version":1,"zone":"zone-a","actual_weight_g":1200,"volume_mm3":6000000}}"#,
        )
        .unwrap();

        let pointer = unsafe { packvium_commerce_quote(call.as_ptr()) };
        let result: serde_json::Value = serde_json::from_str(&to_string(pointer)).unwrap();

        assert_eq!(result["status"], "ok");
        assert_eq!(result["quote"]["total_minor"], 1008);
    }

    #[test]
    fn every_commerce_entry_point_rejects_a_null_pointer_safely() {
        for call in [
            packvium_commerce_quote,
            packvium_commerce_evaluate_policy,
            packvium_commerce_catalog_version_info,
        ] {
            let pointer = unsafe { call(ptr::null()) };
            assert!(to_string(pointer).contains("null request"));
        }
    }

    #[test]
    fn rebalance_round_trips_through_the_c_abi_and_keeps_all_items() {
        // `time_limit_ms` is a wall-clock ceiling, not an effort budget, and this scene's
        // three items solve instantly on any real host. Under `cargo miri test` the
        // interpreter is roughly three orders of magnitude slower, so a tight deadline
        // here made the solve step itself time out mid-search and hand rebalance an
        // incomplete packing to work from -- the assertion below failed on a completely
        // different input, not on rebalance's own logic. 300000ms is the same generous
        // ceiling /established for the conformance corpus, chosen so no
        // realistic host (including Miri) hits it on a scene this small.
        let request = CString::new(
            r#"{"items":[{"id":"heavy","priority":3,"weight":"500","dimensions":{"length":"10","width":"10","height":"10"}},{"id":"light","priority":2,"weight":"100","dimensions":{"length":"10","width":"10","height":"10"}},{"id":"alone","priority":1,"weight":"100","dimensions":{"length":"10","width":"10","height":"10"}}],"containers":[{"id":"box","quantity":2,"max_items":2,"max_payload":"600","inner_dimensions":{"length":"30","width":"20","height":"20"}}],"configuration":{"solvers":["extreme_points"],"time_limit_ms":300000}}"#,
        )
        .unwrap();
        let packed = unsafe { packvium_solve_json(request.as_ptr()) };
        let packed_string = to_string(packed);
        let packed_c = CString::new(packed_string).unwrap();

        let balanced = unsafe { packvium_rebalance_json(request.as_ptr(), packed_c.as_ptr(), 64) };
        let value: serde_json::Value = serde_json::from_str(&to_string(balanced)).unwrap();
        assert_eq!(value["improved"], true);
        let ids = value["containers"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|container| container["placements"].as_array().unwrap())
            .map(|placement| placement["item_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            ids,
            std::collections::BTreeSet::from(["alone#1", "heavy#1", "light#1"])
        );
    }

    #[test]
    fn a_panic_inside_the_wrapped_call_is_caught_rather_than_unwinding() {
        // Exercises the same catch_unwind(AssertUnwindSafe(..)) shape
        // `packvium_solve_json` wraps its call to `packvium_core::pack_json`
        // with; a real panic inside that crate is out of this test's control, so this
        // proves the boundary mechanism itself, not that specific caller.
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<String, ()> {
            panic!("simulated solver panic")
        }));
        assert!(result.is_err());
    }

    #[test]
    fn free_string_on_null_is_a_safe_no_op() {
        unsafe { packvium_free_string(ptr::null_mut()) };
    }

    /// audit: a fixed-seed byte-mutation smoke sweep across the FFI boundary
    /// this crate actually exposes to malformed input (`packvium_solve_json`).
    /// Deterministic (no external fuzzer, no OS randomness) so failures reproduce
    /// exactly; the seed and mutation count are fixed for the same reason. This
    /// complements, not replaces, the handful of hand-picked malformed-input cases
    /// above -- it widens coverage over the input-byte space without any new tooling
    /// dependency (`cargo-fuzz` is unavailable in this environment).
    #[test]
    fn a_byte_mutation_sweep_never_crashes_or_panics_across_the_ffi_boundary() {
        let mut state: u64 = 0x5eed_f11e_babe_cafe;
        let mut next_u8 = || {
            // xorshift64 -- deterministic, dependency-free.
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state & 0xff) as u8
        };

        let seeds: &[&[u8]] = &[
            b"{}",
            b"{\"containers\":[],\"items\":[]}",
            b"null",
            b"[]",
            b"",
            b"\"\xff\xfe\"",
        ];

        for seed in seeds {
            for mutation_count in [1usize, 4, 16] {
                let mut bytes = seed.to_vec();
                for _ in 0..mutation_count {
                    if bytes.is_empty() {
                        bytes.push(next_u8());
                        continue;
                    }
                    let index = (next_u8() as usize) % bytes.len();
                    bytes[index] = next_u8();
                }
                bytes.push(0); // NUL-terminate for CStr::from_ptr.

                let ptr = bytes.as_ptr().cast::<c_char>();
                let result_ptr = unsafe { packvium_solve_json(ptr) };
                assert!(
                    !result_ptr.is_null(),
                    "packvium_solve_json must never return null for a non-null request"
                );
                let response = unsafe { CStr::from_ptr(result_ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { packvium_free_string(result_ptr) };
                // Whatever the response is, it must be a value packvium_free_string
                // safely owns -- the crash/panic freedom is what to_string_lossy() and
                // the surrounding unsafe block not aborting already proves per-case.
                let _ = response;
            }
        }
    }
}
