use anchor_lang::prelude::*;
use crate::bonding_curve::constants::*;
use crate::bonding_curve::utils::*;

// Linear bonding curve constants
const M: f64 = 1e-9; // Slope of the linear curve in atomic units

fn price_function_linear(n: u64) -> f64 {
    M * n as f64
}

fn integral_function_linear(n: u64) -> f64 {
    (M / 2.0) * (n as f64).powf(2.0)
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
        integral_function_linear(current_supply) - integral_function_linear(new_supply)
    } else {
        integral_function_linear(new_supply) - integral_function_linear(current_supply)
    };

    // Round the total cost to nearest atomic unit
    let total_cost = total_cost_f64.round() as u64;

    // Calculate the new price per unit
    let price_per_unit_f64 = price_function_linear(new_supply);
    let price_per_unit = price_per_unit_f64.round() as u64;

    msg!("Old supply: {}", current_supply);
    msg!("Amount: {}", amount);
    msg!("New supply: {}", new_supply);
    msg!("Total cost: {}", total_cost);
    msg!("Price per unit: {}", price_per_unit);

    (total_cost, price_per_unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_and_sell() {
        let initial_supply: u64 = 1000;
        let amount: u64 = 100;
        let (buy_cost, _) = calculate_price(initial_supply, amount, false);
        let (sell_revenue, _) = calculate_price(initial_supply + amount, amount, true);

        assert_eq!(buy_cost, sell_revenue);
    }

    #[test]
    fn test_linear_increase() {
        let initial_supply: u64 = 1000;
        let amount: u64 = 100;
        let (_, price_before) = calculate_price(initial_supply, 0, false);
        let (_, price_after) = calculate_price(initial_supply, amount, false);

        assert!(price_after > price_before);
    }

    #[test]
    fn test_linear_decrease() {
        let initial_supply: u64 = 1000;
        let amount: u64 = 100;
        let (_, price_before) = calculate_price(initial_supply, 0, false);
        let (_, price_after) = calculate_price(initial_supply, amount, true);

        assert!(price_after < price_before);
    }
}