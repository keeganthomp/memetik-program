use anchor_lang::prelude::*;

use crate::bonding_curve::constants::{DEFAULT_TOKEN_DECIMALS, MIN_TOK_PRICE};
use crate::bonding_curve::utils::{calculate_maturity_time, calculate_test_time};
use crate::utils::string_bytes::string_to_fixed_bytes;

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
    pub vault: Pubkey,
    pub escrow: Pubkey,
    pub creator: Pubkey,
    pub status: u8,
    // bonding pool
    pub ticker: [u8; 32],         // Fixed-size array for the ticker
    pub bonding_curve_price: u64, // Store price in atomic units (lamports)
    pub mint: Pubkey,
    pub mint_decimals: u8,
    pub mint_supply: u64,
    pub maturity_time: i64,
    // amm pool
    pub lp_mint: Pubkey,
    pub lp_mint_decimals: u8,
    pub lp_supply: u64,
    pub open_time: u64,
    pub padding: [u64; 32],
}

#[account]

pub struct PoolEscrow {
    pub depositor: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}

#[account]

pub struct PoolVault {
    pub creator: Pubkey,
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}

impl Pool {
    pub const LEN: usize = 8 + 10 * 32 + 1 * 5 + 8 * 6 + 8 * 32;
    pub fn initialize(
        &mut self,
        creator: &Pubkey,
        ticker: &str,
        mint: &Pubkey,
        vault: &Pubkey,
        escrow: &Pubkey,
        lp_mint: &Pubkey,
    ) {
        //////////////////////////////////////////////
        // The pool will intialize as a bonding pool
        //////////////////////////////////////////////
        self.vault = vault.key();
        self.escrow = escrow.key();
        // bonding pool
        self.creator = creator.key();
        self.status = PoolStatus::Active as u8;
        self.mint = mint.key();
        self.mint_decimals = DEFAULT_TOKEN_DECIMALS;
        self.mint_supply = 0;
        self.ticker = string_to_fixed_bytes(ticker, 32);
        self.bonding_curve_price = MIN_TOK_PRICE as u64;
        self.maturity_time = calculate_test_time();

        // default amm settings - pool will not be an amm until the maturity requirements are met
        self.lp_mint = lp_mint.key();
        self.lp_mint_decimals = DEFAULT_TOKEN_DECIMALS;
        self.lp_supply = 0;
        self.open_time = 0;
        self.padding = [0u64; 32];
    }
}
