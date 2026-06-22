use rpc_error::{Error, validate_unique_reasons};

use crate::support::DuplicateDelegateParentError;

#[test]
fn reason_catalogs_include_delegates() {
    let catalogs = DuplicateDelegateParentError::reason_catalogs();

    assert_eq!(catalogs.len(), 2);
    assert!(matches!(
        validate_unique_reasons(&catalogs),
        Err(Error::DuplicateReasonCode { reason }) if reason == "delegated-duplicate"
    ));
}
