use super::*;
use std::str;

#[test]
fn empty_enc() {
    const TEXT: &str = "";
    const EXPECTED: &str = "";
    let encoder = Base64::new();

    assert_eq!(str::from_utf8(&encoder.encode(TEXT.as_bytes())[..]).unwrap(), EXPECTED);
}

#[test]
fn encode1() {
    const TEXT: &str = "Man";
    const EXPECTED: &str = "TWFu";
    let encoder = Base64::new();

    assert_eq!(str::from_utf8(&encoder.encode(TEXT.as_bytes())[..]).unwrap(), EXPECTED);
}

#[test]
fn encode2() {
    const TEXT: &str = "Man is distinguished";
    const EXPECTED: &str = "TWFuIGlzIGRpc3Rpbmd1aXNoZWQ=";
    let encoder = Base64::new();

    assert_eq!(str::from_utf8(&encoder.encode(TEXT.as_bytes())[..]).unwrap(), EXPECTED);
}

#[test]
fn encode3() {
    const TEXT: &str =
        "Is there any availability of Rest service type for the Base64 Encoding?\r\n";
    const EXPECTED: &str = "SXMgdGhlcmUgYW55IGF2YWlsYWJpbGl0eSBvZiBSZXN0IHNlcnZpY2UgdHlwZSBmb3IgdGhlIEJhc2U2NCBFbmNvZGluZz8NCg==";
    let encoder = Base64::new();

    assert_eq!(str::from_utf8(&encoder.encode(TEXT.as_bytes())[..]).unwrap(), EXPECTED);
}

#[test]
fn empty_dec() {
    const TEXT: &str = "";
    const EXPECTED: &str = "";
    let encoder = Base64::new();

    assert_eq!(
        encoder.decode(TEXT.as_bytes()).expect("Decoding error"),
        EXPECTED.as_bytes()
    );
}

#[test]
fn encode_decode1() {
    const TEXT: &str = "Man";
    let encoder = Base64::new();
    let encoded = encoder.encode(TEXT.as_bytes());

    assert_eq!(
        encoder.decode(&encoded).expect("Decoding error"),
        TEXT.as_bytes()
    );
}

#[test]
fn encode_decode2() {
    const TEXT: &str = "Man is distinguished";
    let encoder = Base64::new();
    let encoded = encoder.encode(TEXT.as_bytes());

    assert_eq!(
        encoder.decode(&encoded).expect("Decoding error"),
        TEXT.as_bytes()
    );
}

#[test]
fn encode_decode3() {
    const TEXT: &str =
        "Is there any availability of Rest service type for the Base64 Encoding?\r\n";
    let encoder = Base64::new();
    let encoded = encoder.encode(TEXT.as_bytes());

    assert_eq!(
        encoder.decode(&encoded).expect("Decoding error"),
        TEXT.as_bytes()
    );
}
