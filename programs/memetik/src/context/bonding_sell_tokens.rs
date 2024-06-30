use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
    token_interface::{Mint as SPLMint, TokenAccount},
};

use crate::context::initialize_pool::{
    POOL_AUTH_SEED, POOL_MINT_SEED, POOL_BONDING_SEED, POOL_SOL_VAULT_SEED,
};
use crate::state::pool::{BondingPool, PoolSolVault};

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct SellTokens<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_BONDING_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, BondingPool>,

    #[account(
        mut,
        seeds = [POOL_SOL_VAULT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub sol_vault: Account<'info, PoolSolVault>,

    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
    )]
    pub seller_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
