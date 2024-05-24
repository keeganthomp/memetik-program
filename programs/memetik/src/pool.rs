use anchor_lang::prelude::*;
use std::cmp;

#[account]
pub struct Pool {
    pub id: u64,
    pub tok_price: u64,
    pub mint: Pubkey,
}

pub const TOKEN_DECIMALS: u8 = 9;
pub const STARTING_TOK_SOL_PRICE: u64 = 100; // Starting price in atomic units (lamports)
pub const MIN_TOK_PRICE: u64 = 1; // Minimum price in atomic units

pub fn get_starting_tok_price() -> u64 {
    return STARTING_TOK_SOL_PRICE;
}

// make sure mulitplier is at least 0.01
pub fn normalize_multiplier(multiplier: f64) -> f64 {
    if multiplier < 0.01 {
        0.01
    } else {
        multiplier
    }
}

pub const PRICE_MULTIPLIER: f64 = 20.0; // Adjustment factor for price increase. increases rate of which price changes per purchase and sell
pub fn calculate_price_per_unit(supply: u64) -> u64 {
    let starting_tok_price = get_starting_tok_price();
    let multiplier = normalize_multiplier(PRICE_MULTIPLIER);
    let calculated_price = ((supply as f64).sqrt() * multiplier) as u64 + starting_tok_price;
    cmp::max(calculated_price, MIN_TOK_PRICE)
}

pub fn calculate_total_cost(current_supply: u64, sol_available: u64, amount: u64) -> u64 {
    let price_per_token = calculate_price_per_unit(current_supply);
    msg!("Price per token: {}", price_per_token);
    let max_tokens_buyable = sol_available
        .checked_mul(10u64.pow(TOKEN_DECIMALS as u32))
        .unwrap_or(0)
        .checked_div(price_per_token)
        .unwrap_or(0);
    msg!("Max tokens buyable: {}", max_tokens_buyable);
    msg!("Amount: {}", amount);
    std::cmp::min(max_tokens_buyable, amount)
}

pub fn get_new_supply(current_supply: u64, amount: u64, deduct: bool) -> u64 {
    if deduct {
        current_supply.checked_sub(amount).unwrap()
    } else {
        current_supply.checked_add(amount).unwrap()
    }
}
