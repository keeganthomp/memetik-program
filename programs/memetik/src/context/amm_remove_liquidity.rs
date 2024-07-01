use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::{
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
    token_interface::{Mint as SPLMint, TokenAccount},
};

use crate::state::pool::{AMMPool, PoolSolVault};
use crate::amm::constants::*;

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_LP_MINT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = user,
    )]
    pub user_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [POOL_MINT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [POOL_AMM_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub amm_pool: Account<'info, AMMPool>,

    /// CHECK
    #[account(
        mut,
        seeds = [POOL_SOL_VAULT_SEED.as_bytes(), ticker.as_bytes()],
        bump,
    )]
    pub sol_vault: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = amm_pool,
    )]
    pub token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
