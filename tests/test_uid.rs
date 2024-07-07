use swiss_uid::uid::*;

#[test]
fn test_valid_uid_che() {
    let uid = SwissUid::new("CHE-109.322.551");
    assert_eq!(uid.is_ok(), true);
    let uid = uid.unwrap();
    assert_eq!(uid.to_string(), "CHE-109.322.551");
    assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR");
    assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST");
    assert_eq!(format!("{}", uid), "CHE-109.322.551");
    assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]");
}
