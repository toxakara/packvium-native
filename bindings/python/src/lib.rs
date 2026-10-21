use packvium_core::PackError;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// A `ValueError` subclass, so callers that caught `ValueError` before keep working, carrying the
// core's closed `code` and, for a named refusal, its `reason`, JSON Pointer `field` and `detail`
// -- what `packvium-native` needs to raise `packvium`'s own classes.
create_exception!(_packvium_native_rust, PackviumError, PyValueError);
// `packvium.MalformedResult`'s counterpart: the result is not a document `validate_json` can
// judge. It carries `kind`, `contract` when the reference raises the same message.
create_exception!(_packvium_native_rust, MalformedResult, PyValueError);

fn to_python_error(py: Python<'_>, error: &PackError) -> PyErr {
    let raised = PackviumError::new_err(error.to_string());
    let request_error = error.request_error();
    let attributes = [
        ("code", Some(error.code())),
        ("reason", request_error.map(|refusal| refusal.reason())),
        ("field", request_error.map(|refusal| refusal.field())),
        ("detail", request_error.map(|refusal| refusal.detail())),
    ];
    for (name, value) in attributes {
        if let Err(failure) = raised.value(py).setattr(name, value) {
            return failure;
        }
    }
    raised
}

// The solve runs detached from the interpreter so a threaded host (a web server's worker pool)
// keeps serving while the core computes; the core touches no Python object.
#[pyfunction]
fn pack_json(py: Python<'_>, request: &str) -> PyResult<String> {
    py.detach(|| packvium_core::pack_json(request))
        .map_err(|error| to_python_error(py, &error))
}

#[pyfunction]
fn rebalance_json(
    py: Python<'_>,
    request: &str,
    result: &str,
    max_moves: usize,
) -> PyResult<String> {
    py.detach(|| packvium_core::rebalance_json(request, result, max_moves))
        .map_err(|error| to_python_error(py, &error))
}

/// The reference validator's issues for a request and a result, both JSON text, as a list of
/// `{"code": ..., "detail": ...}` dicts, empty when the result is valid.
#[pyfunction]
fn validate_json<'py>(
    py: Python<'py>,
    request: &str,
    result: &str,
) -> PyResult<Bound<'py, PyList>> {
    match py.detach(|| packvium_core::validate_json(request, result)) {
        Ok(issues) => {
            let listed = PyList::empty(py);
            for issue in issues {
                let entry = PyDict::new(py);
                entry.set_item("code", issue.code)?;
                entry.set_item("detail", issue.detail)?;
                listed.append(entry)?;
            }
            Ok(listed)
        }
        Err(refusal) => {
            let raised = MalformedResult::new_err(refusal.message.clone());
            let kind = match refusal.kind {
                packvium_core::RefusalKind::Contract => "contract",
                packvium_core::RefusalKind::Shape => "shape",
            };
            raised.value(py).setattr("kind", kind)?;
            Err(raised)
        }
    }
}

#[pyfunction]
fn version() -> &'static str {
    packvium_core::VERSION
}

#[pymodule]
fn _packvium_native_rust(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(pack_json, module)?)?;
    module.add_function(wrap_pyfunction!(rebalance_json, module)?)?;
    module.add_function(wrap_pyfunction!(validate_json, module)?)?;
    module.add_function(wrap_pyfunction!(version, module)?)?;
    module.add("PackviumError", module.py().get_type::<PackviumError>())?;
    module.add("MalformedResult", module.py().get_type::<MalformedResult>())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // The exported functions need a live interpreter; the Linux gates run `cargo test` without
    // one, so this pins the shared-core round trip the binding forwards to.
    const REQUEST: &str = r#"{"items":[{"id":"heavy","priority":3,"weight":"500","dimensions":{"length":"10","width":"10","height":"10"}},{"id":"light","priority":2,"weight":"100","dimensions":{"length":"10","width":"10","height":"10"}},{"id":"alone","priority":1,"weight":"100","dimensions":{"length":"10","width":"10","height":"10"}}],"containers":[{"id":"box","quantity":2,"max_items":2,"max_payload":"600","inner_dimensions":{"length":"30","width":"20","height":"20"}}],"configuration":{"solvers":["extreme_points"],"time_limit_ms":5000}}"#;

    #[test]
    fn pyo3_rebalance_export_round_trips_through_the_shared_core() {
        let packed = packvium_core::pack_json(REQUEST).unwrap();
        let balanced = packvium_core::rebalance_json(REQUEST, &packed, 64).unwrap();
        assert!(balanced.contains(r#""improved":true"#));
        assert!(balanced.contains(r#""item_id":"light#1""#));
    }

    #[test]
    fn pyo3_validate_export_round_trips_through_the_shared_core() {
        let packed = packvium_core::pack_json(REQUEST).unwrap();
        assert_eq!(
            packvium_core::validate_json(REQUEST, &packed).unwrap(),
            Vec::new()
        );
    }
}
