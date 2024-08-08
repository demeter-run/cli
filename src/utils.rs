use std::{collections::BTreeMap, str::FromStr};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    CustomResourceDefinition, JSONSchemaProps,
};

pub fn get_spec_from_crd(
    crd: &CustomResourceDefinition,
) -> Option<BTreeMap<String, JSONSchemaProps>> {
    let version = crd.spec.versions.last()?;
    let schema = version.schema.clone()?.open_api_v3_schema?.properties?;
    schema.get("spec")?.properties.clone()
}

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
