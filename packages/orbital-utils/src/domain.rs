use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum OrbitalDomain {
    Neutron,
    Gaia,
    Osmosis,
}

impl OrbitalDomain {
    pub fn value(&self) -> u8 {
        match self {
            OrbitalDomain::Neutron => 0,
            OrbitalDomain::Gaia => 1,
            OrbitalDomain::Osmosis => 2,
        }
    }
}
