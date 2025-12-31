#[derive(Debug, Clone)]
pub struct ParamMeta {
    pub credential_type: &'static str, // "JWT" or "mDL"
    pub signature_algo: &'static str,  // "RS256", "ES256", ...
    pub claim_count: Option<u32>,
    pub public_inputs: Option<u32>,
    pub public_outputs: Option<u32>,
}

pub fn lookup(param: &str) -> Option<ParamMeta> {
    match param {
        "rs256" => Some(ParamMeta {
            credential_type: "JWT",
            signature_algo: "RS256",
            claim_count: Some(2),
            public_inputs: Some(19),
            public_outputs: Some(0),
        }),
        "rs256-db" => Some(ParamMeta {
            credential_type: "JWT (device-bound)",
            signature_algo: "RS256",
            claim_count: Some(10),
            public_inputs: Some(25),
            public_outputs: Some(2),
        }),
        "mdl1" => Some(ParamMeta {
            credential_type: "mDL",
            signature_algo: "ES256",
            // Circuit reveals 5 fields in our log (3 revealed, 2 reveal_digest).
            claim_count: Some(5),
            public_inputs: Some(9),
            public_outputs: Some(2),
        }),
        // Fill this once we rerun ./run_setup.sh rs256-sd and copy the numbers.
        "rs256-sd" => Some(ParamMeta {
            credential_type: "JWT",
            signature_algo: "RS256",
            claim_count: None,
            public_inputs: None,
            public_outputs: None,
        }),
        _ => None,
    }
}
