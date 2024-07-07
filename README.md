# Swiss E-Government Standard eCH-0097: Datenstandard Unternehmensidentifikation

This library implements functionalities and data structures in accordance to the
data standard [eCH-0098:2021 5.2.0](https://www.ech.ch/de/ech/ech-0097/5.2.0)

## Example usage

```rust
use swiss_uid::uid::SwissUid;

// Using the new function:
let uid = SwissUid::new("CHE-109.322.551").unwrap();
assert_eq!(format!("{:?}", uid), "CHE-109.322.55[1]".to_owned()); // Debug output
assert_eq!(format!("{}", uid), "CHE-109.322.551".to_owned()); // Display output
assert_eq!(uid.to_string(), "CHE-109.322.551".to_owned()); // Display output
assert_eq!(uid.to_string_mwst(), "CHE-109.322.551 MWST".to_owned());
assert_eq!(uid.to_string_hr(), "CHE-109.322.551 HR".to_owned());

// Parse a string directly:
let uid2: SwissUid = "CHE-109.322.551".parse().unwrap();
assert_eq!(uid2.to_string().len(), 15);
```
