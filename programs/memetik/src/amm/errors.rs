use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Pool amount too low")]
    PoolAmountTooLow,
}
