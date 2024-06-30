use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::{
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
    token_interface::{Mint as SPLMint, TokenAccount},
};

use crate::context::initialize_pool::{
    POOL_AMM_SEED, POOL_AUTH_SEED, POOL_BONDING_SEED, POOL_LP_MINT_SEED, POOL_MINT_SEED,
    POOL_SOL_VAULT_SEED,
};
use crate::errors::Error;
use crate::state::pool::{AMMPool, PoolSolVault};

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [POOL_LP_MINT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
        mint::decimals = 9,
        mint::authority = pool,
        payer = user,
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_mint,
        associated_token::authority = user,
    )]
    pub user_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_mint,
        associated_token::authority = pool,
    )]
    pub lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
        mint::authority = token_mint,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [POOL_AMM_SEED.as_bytes(), ticker.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<AMMPool>(),
    )]
    pub pool: Account<'info, AMMPool>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [POOL_SOL_VAULT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<PoolSolVault>(),
    )]
    pub sol_vault: Account<'info, PoolSolVault>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = pool,
    )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}