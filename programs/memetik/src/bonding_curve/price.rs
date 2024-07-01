use anchor_lang::prelude::*;
use crate::bonding_curve::constants::*;
use crate::bonding_curve::utils::*;

const M: f64 = 1e-9; // Slope of the linear curve in atomic units
const TOKEN_SCALE: f64 = 1e9; // Scale factor for token price in atomic units

fn integral_function_linear(n: u64) -> f64 {
    ((M * TOKEN_SCALE) / 2.0) * (n as f64).powf(2.0)
}

fn price_function_linear(n: u64) -> f64 {
    M * TOKEN_SCALE * n as f64
}

pub fn calculate_price(current_supply: u64, amount: u64, is_selling: bool) -> (u64, u64) {
    msg!("========Calculating price========");

    // Check for valid operation
    if is_selling && amount > current_supply {
        panic!("Attempt to sell more tokens than available in supply");
    }

    let new_supply = if is_selling {
        current_supply.saturating_sub(amount) // Ensure we don't go negative
    } else {
        current_supply.saturating_add(amount)
    };

    // Calculate total cost based on the integral of the linear price function
    let total_cost_f64 = if is_selling {
        msg!("integral_function_linear(current_supply) {}", integral_function_linear(current_supply));
        msg!("integral_function_linear(new_supply) {}", integral_function_linear(new_supply));
        integral_function_linear(current_supply) - integral_function_linear(new_supply)
    } else {
        integral_function_linear(new_supply) - integral_function_linear(current_supply)
    };

    // Round the total cost to nearest atomic unit
    let total_cost = total_cost_f64.round() as u64;

    // Calculate the new price per unit using the price function
    let price_per_unit_f64 = price_function_linear(new_supply);
    let price_per_unit = price_per_unit_f64.round() as u64;

    // Ensure the total cost and price per unit are not zero
    let total_cost = if total_cost == 0 { 1 } else { total_cost };
    let price_per_unit = if price_per_unit == 0 { 1 } else { price_per_unit };

    msg!("Old supply: {}", current_supply);
    msg!("Amount: {}", amount);
    msg!("New supply: {}", new_supply);
    msg!("Total cost: {}", total_cost);
    msg!("Price per unit: {}", price_per_unit);

    (total_cost, price_per_unit)
}