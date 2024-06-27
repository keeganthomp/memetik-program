pub struct AMMConfig {
    pub protocol_fee: u64,
    pub min_sol_balance: u64,
}

pub const AMM_CONFIG: AMMConfig = AMMConfig {
    protocol_fee: 10,
    min_sol_balance: 1000000000,
};
