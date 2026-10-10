use std::collections::HashSet;

use gameplay::achievements::Achievement;

use super::{decode, encode};

#[test]
fn decode_accepts_any_version_and_splits_known_from_unknown_ids() {
    let data = br#"{"version":99,"unlocked":["RESTORE_BOILER","FUTURE_ID"]}"#;
    let decoded = decode(data).unwrap();
    assert_eq!(decoded.known, HashSet::from([Achievement::RestoreBoiler]));
    assert_eq!(decoded.unknown, HashSet::from(["FUTURE_ID".to_string()]));
}

#[test]
fn encode_sorts_ids_and_round_trips_through_decode() {
    let known = HashSet::from([Achievement::RestoreBoiler, Achievement::EscapeUndetected]);
    let unknown = HashSet::from(["ZZZ_FUTURE".to_string(), "AAA_FUTURE".to_string()]);
    let data = encode(&known, &unknown);
    let text = String::from_utf8(data.clone()).unwrap();
    assert!(text.find("AAA_FUTURE").unwrap() < text.find("ZZZ_FUTURE").unwrap());

    let decoded = decode(&data).unwrap();
    assert_eq!(decoded.known, known);
    assert_eq!(decoded.unknown, unknown);
}

#[test]
fn decode_rejects_invalid_json() {
    assert!(decode(b"not json").is_err());
}
