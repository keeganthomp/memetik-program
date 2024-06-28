use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
    token_interface::{Mint as SPLMint, TokenAccount},
};

use crate::context::initialize_pool::{POOL_ESCROW_SEED, POOL_SEED};
use crate::state::pool::Pool;

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct Close<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub pool: AccountLoader<'info, Pool>,
    pub system_program: Program<'info, System>,
}
