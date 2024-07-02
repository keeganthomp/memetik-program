use crate::bonding_curve::constants::*;
use anchor_lang::solana_program::msg;
use anchor_lang::solana_program::sysvar::clock::Clock;
use anchor_lang::solana_program::sysvar::Sysvar;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub fn check_valid_ticker(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_ascii_uppercase())
}

pub fn to_whole_units(lamport_amount: u64) -> f64 {
    lamport_amount as f64 / LAMPORTS_PER_SOL as f64
}

pub fn to_atomic_units(amount: f64) -> u64 {
    (amount * TOKEN_SCALE).round() as u64
}

pub fn calculate_maturity_time() -> i64 {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let maturity_date = current_timestamp + DAYS_TO_MATURITY * SECONDS_IN_A_DAY;
    maturity_date
}

pub fn calculate_test_time() -> i64 {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let test_date = current_timestamp + 3; // x seconds from now
    test_date
}

pub fn check_if_maturity_time_passed(maturity_date: i64) -> bool {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    current_timestamp >= maturity_date
}

pub fn check_if_maturity_amount_reached(pool_balance_lamports: u64, latest_sol_price: f64) -> bool {
    let pool_balance_usd = to_whole_units(pool_balance_lamports) * latest_sol_price;
    msg!("pool bal lamports: {}", pool_balance_lamports);
    msg!("Whole units: {}", to_whole_units(pool_balance_lamports));
    msg!("Pool balance in USD: {}", pool_balance_usd);
    pool_balance_usd >= REQUIRED_MARKET_CAP_TO_MATURE
}
