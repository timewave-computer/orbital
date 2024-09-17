use orbital_core::msg::InstantiateMsg;

pub struct OrbitalCoreInstantiate {
    pub msg: InstantiateMsg,
}

impl Default for OrbitalCoreInstantiate {
    fn default() -> Self {
        OrbitalCoreInstantiate {
            msg: InstantiateMsg {
                owner: "TODO".to_string(),
            },
        }
    }
}
