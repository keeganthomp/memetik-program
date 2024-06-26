use anchor_lang::prelude::*;

use crate::amm::state::pool::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount},
};

pub const POOL_SEED: &str = "pool";
pub const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
pub const POOL_VAULT_SEED: &str = "pool_vault";

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        seeds = [POOL_SEED.as_bytes(), token_mint.key().as_ref()],
        bump,
        payer = signer,
        space = PoolState::LEN
    )]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(mint::token_program = token_program)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        seeds = [POOL_LP_MINT_SEED.as_bytes(), pool_state.key().as_ref()],
        bump,
        mint::decimals = 9,
        mint::authority = signer,
        payer = signer,
        mint::token_program = token_program,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(init_if_needed, payer = signer, token::mint = token_mint, token::authority = signer)]
    pub creator_token: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        associated_token::mint = lp_mint,
        associated_token::authority = signer,
        payer = signer,
        token::token_program = token_program,
    )]
    pub creator_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK - fine as we need the vault to hold both SOL and SPL tokens
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool_state.key().as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>, // Unchecked account to handle both SOL and SPL tokens

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
