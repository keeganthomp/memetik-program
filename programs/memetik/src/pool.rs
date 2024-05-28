use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub id: u64,
    pub tok_price: u64, // Store price in atomic units
    pub mint: Pubkey,
}

pub const TOKEN_DECIMALS: u8 = 9;
pub const TOKEN_SCALE: f64 = 1_000_000_000.0; // 10^9, precomputed for TOKEN_DECIMALS = 9
pub const MIN_TOK_PRICE: f64 = 1.0 / TOKEN_SCALE; // 100 lamports in atomic units

// Quadratic bonding curve constants
pub const A: f64 = 1e-13; // Impact: High - Dominates at large supply values, causing exponential increase
pub const B: f64 = 1e-10; // Impact: Moderate - Influences both initial and ongoing price increases
pub const C: f64 = MIN_TOK_PRICE; // Impact: Low - Sets the minimum price and initial price floor

pub fn price_function(n: f64) -> f64 {
    A * n.powf(2.0) + B * n + C
}
pub fn integral_function(n: f64) -> f64 {
    (A / 3.0) * n.powf(3.0) + (B / 2.0) * n.powf(2.0) + C * n
}

pub fn to_whole_units(amount: u64) -> f64 {
    amount as f64 / TOKEN_SCALE
}
pub fn to_atomic_units(amount: f64) -> u64 {
    (amount * TOKEN_SCALE).round() as u64
}

pub fn calculate_price(current_supply: u64, amount: u64, is_selling: bool) -> (u64, u64) {
    msg!("========Calculating price========");
    // Convert atomic units to whole units
    let current_supply_units = to_whole_units(current_supply);
    let amount_units = to_whole_units(amount);
    let new_supply_units = if is_selling {
        current_supply_units - amount_units
    } else {
        current_supply_units + amount_units
    };

    let total_cost_f64 = if is_selling {
        integral_function(current_supply_units) - integral_function(new_supply_units)
    } else {
        integral_function(new_supply_units) - integral_function(current_supply_units)
    };

    let total_cost_lamports = to_atomic_units(total_cost_f64);

    // Calculate price per whole unit correctly
    let price_per_unit_f64 = price_function(new_supply_units);
    let price_per_unit = to_atomic_units(price_per_unit_f64);

    msg!("Old supply (units): {}", current_supply_units);
    msg!("Amount (units): {}", amount_units);
    msg!("New supply (units): {}", new_supply_units);
    msg!("Total cost: {}", total_cost_lamports);
    msg!("Price per whole unit: {}", price_per_unit);

    (total_cost_lamports, price_per_unit)
}
