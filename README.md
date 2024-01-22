# Swiss E-Government Standard UID

This library contains an implementation of the Swiss government's UID number and
validation of its check digit based on the E-Government Standard
[eCH-0098:2021 5.2.0](https://www.ech.ch/de/ech/ech-0097/5.2.0)

## Example

```rust
let my_uid = SwissUid::new("CHE-109.322.551");
assert_eq!(my_uid.is_ok(), true);
let uid = my_uid.unwrap();
assert_eq!(uid.to_string(), "CHE-109.322.551"); // Normal format
assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR"); // Handelsregister Nummer format
assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST"); // MWST-Nummer format
assert_eq!(format!("{}", uid), "CHE-109.322.551"); // Normal format
assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]"); // Debug format
assert_eq!(uid.checkdigit(), Some(1)); // Prüfziffer
```
