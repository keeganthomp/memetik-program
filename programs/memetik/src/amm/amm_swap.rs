use crate::amm::constants::*;

pub fn calculate_swap(
    amount_in: u128,
    current_token_balance: u128,
    current_sol_balance: u128,
    is_sol_to_token: bool,
) -> (u128, u128, u128) {
    let scale = 1_000_000_000; // Scale factor for precision

    // Calculate the fee
    let swap_fee = (amount_in * AMM_SWAP_FEE_PERCENT) / 1000; // 0.3% fee
    let actual_amount_in = amount_in - swap_fee;

    // Calculate the amount to swap using the constant product formula (x * y = k)
    if is_sol_to_token {
        // Swapping SOL for tokens
        let numerator = actual_amount_in * current_token_balance;
        let denominator = current_sol_balance + actual_amount_in;
        let amount_out = (numerator + denominator / 2) / denominator; // Rounding up

        let new_sol_reserve = current_sol_balance + actual_amount_in;
        let new_token_reserve = current_token_balance - amount_out;
        (amount_out, new_sol_reserve, new_token_reserve)
    } else {
        // Swapping tokens for SOL
        let numerator = actual_amount_in * current_sol_balance;
        let denominator = current_token_balance + actual_amount_in;
        let amount_out = (numerator + denominator / 2) / denominator; // Rounding up

        let new_sol_reserve = current_sol_balance - amount_out;
        let new_token_reserve = current_token_balance + actual_amount_in;
        (amount_out, new_sol_reserve, new_token_reserve)
    }
}
