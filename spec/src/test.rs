#![cfg(test)]

#[derive(Deserialize, Debug)]
pub struct RuntimeValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "invoke")]
    Invoke {
        module: Option<String>,
        field: String,
        args: Vec<RuntimeValue>,
    },
    #[serde(rename = "get")]
    Get {
        module: Option<String>,
        field: String,
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "module")]
    Module {
        line: u64,
        name: Option<String>,
        filename: String
    },
    #[serde(rename = "assert_return")]
    AssertReturn { 
        line: u64, 
        action: Action,
        expected: Vec<RuntimeValue>,
    },
    #[serde(rename = "assert_return_canonical_nan")]
    AssertReturnCanonicalNan {
        line: u64,
        action: Action,
    },
    #[serde(rename = "assert_return_arithmetic_nan")]
    AssertReturnArithmeticNan {
        line: u64,
        action: Action,
    },
    #[serde(rename = "assert_trap")]
    AssertTrapKind {
        line: u64,
        action: Action,
        text: String,
    },
    #[serde(rename = "assert_invalid")]
    AssertInvalid {
        line: u64,
        filename: String,
        text: String,
    },
    #[serde(rename = "assert_malformed")]
    AssertMalformed {
        line: u64,
        filename: String,
        text: String,
    },
    #[serde(rename = "assert_uninstantiable")]
    AssertUninstantiable {
        line: u64,
        filename: String,
        text: String,
    },
    #[serde(rename = "assert_exhaustion")]
    AssertExhaustion {
        line: u64,
        action: Action,
    },
    #[serde(rename = "assert_unlinkable")]
    AssertUnlinkable {
        line: u64,
        filename: String,
        text: String,
    },
    #[serde(rename = "register")]
    Register {
        line: u64,
        name: Option<String>,
        #[serde(rename = "as")]
        as_name: String,
    },
    #[serde(rename = "action")]
    Action {
        line: u64,
        action: Action,
    },
}

#[derive(Deserialize, Debug)]
pub struct Spec {
    pub source_filename: String,
    pub commands: Vec<Command>,
}