use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
};

use crate::amm::constants::*;
use crate::state::pool::{BondingPool, PoolEscrow};

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct ClosePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_BONDING_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        close = signer,
    )]
    pub pool: Account<'info, BondingPool>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
