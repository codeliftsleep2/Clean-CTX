use super::{client, fixture};
use crate::cbm::client::CbmError;
use tempfile::TempDir;

#[test]
fn cbm_a3_exited_startup_refusal_preserves_structured_cause() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a3-startup-refusal-exit");
    fixture::wait_frontend_exit(&mut client);
    let error = fixture::admit(&mut client).unwrap_err();
    assert!(matches!(error, CbmError::AdmissionError { code: Some(-32001), ref message }
        if message == "provider admission refused"), "queued structured refusal was discarded: {error}");
    assert_eq!(fixture::owned_resources(&client), 0);
    assert!(!client.status().is_available());
}
