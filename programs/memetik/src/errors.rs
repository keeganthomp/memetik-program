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
}
