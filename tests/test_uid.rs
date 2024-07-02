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
    assert_eq!(uid.checkdigit(), Ok(1));
}

#[test]
fn test_valid_uid_adm() {
    let uid = SwissUid::new("CHE-109.322.551");
    assert_eq!(uid.is_ok(), true);
    let uid = uid.unwrap();
    assert_eq!(uid.to_string(), "CHE-109.322.551");
    assert_eq!(uid.checkdigit(), Ok(1));
}

#[test]
fn test_invalid_format() {
    let uid = SwissUid::new("CH-109.322.552");
    assert_eq!(uid.is_err(), true);
    let uid = uid.unwrap_err();
    assert_eq!(format!("{}", uid), "Invalid format: 'CH-109.322.552'");
}

#[test]
fn test_invalid_prefix() {
    let uid = SwissUid::new("ABC-109.322.551");
    assert_eq!(uid.is_err(), true);
    let uid = uid.unwrap_err();
    assert_eq!(
        format!("{}", uid),
        "Invalid format: 'ABC' prefix must be 'CHE' or 'ADM'"
    );
}

#[test]
fn test_invalid_checkdigit() {
    let uid = SwissUid::new("CHE-000.002.000");
    assert_eq!(uid.is_err(), true);
    let uid = uid.unwrap_err();
    assert_eq!(format!("{:?}", uid), "InvalidCheckDigit(\"10\")");
}

#[test]
fn test_mismatched_checkdigit() {
    let uid = SwissUid::new("CHE-109.322.552");
    assert_eq!(uid.is_err(), true);
    let uid = uid.unwrap_err();
    assert_eq!(
        format!("{}", uid),
        "Mismatched check digit: 'CHE-109.322.55[2]' should have the check digit [1]"
    );
}

#[test]
fn test_eq_uid() {
    let uid1 = SwissUid::new("CHE-109.322.551");
    assert_eq!(uid1.is_ok(), true);
    let uid1 = uid1.unwrap();
    let uid2 = SwissUid::new("CHE-109.322.551");
    assert_eq!(uid2.is_ok(), true);
    let uid2 = uid2.unwrap();
    assert_eq!(uid1, uid2);
}

#[test]
fn test_clone_uid() {
    let uid1 = SwissUid::new("CHE-109.322.551");
    assert_eq!(uid1.is_ok(), true);
    let uid1 = uid1.unwrap();
    let uid2 = uid1.clone();
    assert_eq!(uid1, uid2);
}
