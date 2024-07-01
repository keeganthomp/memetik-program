use anchor_lang::prelude::*;

#[account]
pub struct BondingPool {
    // bonding pool
    pub creator: Pubkey,
    pub ticker: String,
    pub mint: Pubkey,
    pub maturity_time: i64,
    pub last_token_price: u64,
}

#[account]
pub struct AMMPool {
    // bonding pool
    pub mint: Pubkey,
    pub ticker: String,
    pub sol_balance: u64,
    pub token_balance: u64,
    pub lp_balance: u64,
}

#[account]

pub struct PoolEscrow {
    pub depositor: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}

#[account]

pub struct PoolSolVault {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}
