use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
};

use crate::context::initialize_pool::{
    POOL_AUTH_SEED, POOL_ESCROW_SEED, POOL_MINT_SEED, POOL_SEED,
};
use crate::state::pool::{Pool, PoolEscrow};

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct ClosePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), symbol.as_bytes()],
        bump,
    )]
    pub pool: AccountLoader<'info, Pool>,

    /// CHECK
    #[account(
        mut,
        seeds = [POOL_AUTH_SEED.as_bytes(), symbol.as_bytes()],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        mint::authority = authority,
        // close = signer,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [POOL_ESCROW_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        close = signer,
    )]
    pub escrow: Account<'info, PoolEscrow>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
