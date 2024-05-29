use std::str::FromStr;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;

#[cw_serde]
pub enum OrbitalDomain {
    Neutron,
    Gaia,
    Juno,
}

impl OrbitalDomain {
    pub fn value(&self) -> u8 {
        match self {
            OrbitalDomain::Neutron => 0,
            OrbitalDomain::Gaia => 1,
            OrbitalDomain::Juno => 2,
        }
    }
}

impl FromStr for OrbitalDomain {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "neutron" => Ok(OrbitalDomain::Neutron),
            "gaia" => Ok(OrbitalDomain::Gaia),
            "juno" => Ok(OrbitalDomain::Juno),
            _ => Err(StdError::generic_err("Invalid domain")),
        }
    }
    
}