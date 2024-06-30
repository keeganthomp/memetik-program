use anchor_lang::prelude::*;

#[error_code]
pub enum Error {
    #[msg("Invalid ticker")]
    InvalidTicker,
    #[msg("No tokens to sell")]
    NoTokensToSell,
    #[msg("No tokens to buy")]
    NoTokensToBuy,
    #[msg("Pool cannot be closed")]
    PoolCannotBeClosed,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Division by zero")]
    DivideByZero,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid swap input")]
    InvalidSwapInput,
}
