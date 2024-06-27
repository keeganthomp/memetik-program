use anchor_lang::prelude::*;

use crate::bonding_curve::constants::{MIN_TOK_PRICE, DEFAULT_TOKEN_DECIMALS};
use crate::bonding_curve::utils::{calculate_maturity_time, calculate_test_time};
use crate::utils::string_to_fixed_bytes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PoolStatus {
    Inactive = 0,
    Active = 1,
    Matured = 2,
}

#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Default, Debug)]
pub struct Pool {
    // bonding pool
    pub ticker: [u8; 32],         // Fixed-size array for the ticker
    pub bonding_curve_price: u64, // Store price in atomic units (lamports)
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub maturity_time: i64,
    pub status: u8,
    // amm pool
    pub vault: Pubkey,
    pub lp_mint: Pubkey,
    pub lp_mint_decimals: u8,
    pub lp_supply: u64,
    pub auth_bump: u8,
    pub mint_decimals: u8,
    pub fund_fees_token: u64,
    pub open_time: u64,
    pub padding: [u64; 32],
}

#[account]

pub struct PoolEscrow {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}

impl Pool {
    pub const LEN: usize = 8 + 10 * 32 + 1 * 5 + 8 * 6 + 8 * 32;
    pub fn initialize(&mut self, creator: &Pubkey, ticker: &str, mint: &Pubkey) {
        //////////////////////////////////////////////
        // The pool will intialize as a bonding pool
        //////////////////////////////////////////////

        // bonding pool
        self.creator = creator.key();
        self.status = PoolStatus::Active as u8;
        self.mint = mint.key();
        self.mint_decimals = DEFAULT_TOKEN_DECIMALS;
        self.ticker = string_to_fixed_bytes(ticker, 32);
        self.bonding_curve_price = MIN_TOK_PRICE as u64;
        self.maturity_time = calculate_test_time();

        // default amm settings - pool will not be an amm until the maturity requirements are met
        self.vault = Pubkey::default();
        self.lp_mint = Pubkey::default();
        self.lp_supply = 0;
        self.lp_mint_decimals = DEFAULT_TOKEN_DECIMALS;
        self.padding = [0u64; 32];
        self.fund_fees_token = 0;
        self.auth_bump = 0;
        self.open_time = 0;
    }
}
