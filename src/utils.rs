use std::str::FromStr;

pub enum KnownField {
    Network,
    OperatorVersion,
}

impl FromStr for KnownField {
    type Err = miette::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "network" => Ok(Self::Network),
            "operatorVersion" => Ok(Self::OperatorVersion),
            _ => Err(miette::Error::msg("field isnt known")),
        }
    }
}
