use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub id: u64,
    pub tok_price: f64,
    pub mint: Pubkey,
}

pub const TOKEN_DECIMALS: u8 = 9;
pub const MIN_TOK_PRICE: f64 = 1.0; // Starting price in atomic units (lamports)
pub const LINEAR_TERM: f64 = 1e-9; // Smaller linear term for flatter increase
pub const PRICE_CONSTANT: f64 = 1.0; // Moderate value to balance the impact
pub const EXPONENT: f64 = 0.01; // Smaller exponent for more gradual increase

pub fn get_starting_tok_price() -> f64 {
    MIN_TOK_PRICE
}

pub fn calculate_price(current_supply: u64, amount: u64, is_selling: bool) -> (u64, f64) {
    msg!("========Calculating price========");
    let starting_price = get_starting_tok_price();

    if current_supply == 0 && !is_selling {
        return (amount * starting_price as u64, starting_price);
    }

    let current_supply_f64 = current_supply as f64;
    let amount_f64 = amount as f64;
    let new_supply = if is_selling {
        current_supply.saturating_sub(amount)
    } else {
        current_supply.saturating_add(amount)
    };
    let new_supply_f64 = new_supply as f64;

    // Define the integral of the price function with the adjusted terms
    let integral_part = |n: f64| -> f64 {
        (LINEAR_TERM / 2.0) * n.powf(2.0)
            + (PRICE_CONSTANT / (EXPONENT + PRICE_CONSTANT)) * n.powf(EXPONENT + PRICE_CONSTANT)
    };

    // Calculate the total cost by evaluating the integral at the start and end points
    let total_cost_f64 = if is_selling {
        integral_part(current_supply_f64) - integral_part(new_supply_f64)
    } else {
        integral_part(new_supply_f64) - integral_part(current_supply_f64)
    };

    // Round total_cost_f64 to no decimals
    let total_cost_rounded = total_cost_f64.round() as u64;

    // Calculate the price per unit based on the new supply
    let price_per_unit_f64 = if is_selling && new_supply == 0 {
        starting_price
    } else if is_selling {
        integral_part(new_supply_f64) / new_supply_f64
    } else {
        total_cost_f64 / amount_f64
    };

    msg!("Price per unit: {}", price_per_unit_f64);

    // Round to specified decimal places
    let round_to = 10u64.pow(TOKEN_DECIMALS as u32);
    let final_price_per_unit = (price_per_unit_f64 * round_to as f64).round() / round_to as f64;

    msg!(
        "Current supply: {} Amount: {} Is selling: {}",
        current_supply,
        amount,
        is_selling
    );
    msg!("Final supply: {}", new_supply);
    msg!(
        "Total cost: {} Final price per unit: {:.2}",
        total_cost_rounded,
        final_price_per_unit
    );

    (total_cost_rounded, final_price_per_unit)
}
