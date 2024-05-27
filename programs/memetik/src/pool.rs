use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub id: u64,
    pub tok_price: u64, // Store price in atomic units
    pub mint: Pubkey,
}

pub const TOKEN_DECIMALS: u8 = 9;
pub const MIN_TOK_PRICE: f64 = 1.0 / 1_000_000_000.0; // 1000 lamports in atomic units

// Quadratic bonding curve constants
pub const A: f64 = 1e-40; // Impact: High - Dominates at large supply values, causing exponential increase
pub const B: f64 = 1e-18; // Impact: Moderate - Influences both initial and ongoing price increases
pub const C: f64 = MIN_TOK_PRICE; // Impact: Low - Sets the minimum price and initial price floor

pub fn get_starting_tok_price() -> f64 {
    MIN_TOK_PRICE
}

pub fn price_function(n: f64) -> f64 {
    A * n.powf(2.0) + B * n + C
}

pub fn integral_function(n: f64) -> f64 {
    (A / 3.0) * n.powf(3.0) + (B / 2.0) * n.powf(2.0) + C * n
}

pub fn calculate_price(current_supply: u64, amount: u64, is_selling: bool) -> (u64, u64) {
    msg!("========Calculating price========");
    let current_supply_f64 = current_supply as f64;
    let new_supply = if is_selling {
        current_supply.saturating_sub(amount)
    } else {
        current_supply.saturating_add(amount)
    };
    let new_supply_f64 = new_supply as f64;

    let total_cost_f64 = if is_selling {
        integral_function(current_supply_f64) - integral_function(new_supply_f64)
    } else {
        integral_function(new_supply_f64) - integral_function(current_supply_f64)
    };

    let total_cost_lamports = (total_cost_f64 * 1_000_000_000.0).round() as u64;

    let price_per_unit_f64 = price_function(new_supply_f64);
    let price_per_unit_lamports = (price_per_unit_f64 * 1_000_000_000.0).round() as u64;

    msg!("Old supply: {}", current_supply);
    msg!("Amount: {}", amount);
    msg!("New supply: {}", new_supply);
    msg!("Total cost: {}", total_cost_lamports);
    msg!("Price per unit: {}", price_per_unit_lamports);

    (total_cost_lamports, price_per_unit_lamports)
}
