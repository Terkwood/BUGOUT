use base64;
pub fn header() -> Option<String> {
    let cleartext = &*crate::env::AUTHORIZATION;
    cleartext
        .as_ref()
        .map(|a| format!("Basic {}", base64::encode(a)))
}
