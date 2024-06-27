use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
    token_interface::{Mint as SPLMint, TokenAccount},
};

use crate::bonding_curve::constants::DEFAULT_TOKEN_DECIMALS;
use crate::state::pool::*;

pub const POOL_SEED: &str = "pool";
pub const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_ESCROW: &str = "pool_escrow";
pub const MINT: &str = "mint";
pub const MINT_PDA: &str = "mint_pda";

#[derive(AnchorSerialize, Debug, Clone)]
pub struct Empty {}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
#[instruction(symbol:String, token_info: TokenArgs)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [POOL_ESCROW.as_bytes(), symbol.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<PoolEscrow>()
    )]
    pub escrow: Account<'info, PoolEscrow>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK
    #[account(
        init,
        seeds = [MINT_PDA.as_bytes(), symbol.as_bytes()],
        bump,
        payer = signer,
        space = 0
    )]
    pub mint_pda: UncheckedAccount<'info>,

    /// CHECK
    #[account(
        init,
        seeds = [POOL_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        payer = signer,
        space = 8 + std::mem::size_of::<Pool>(),
    )]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        init,
        seeds = [MINT.as_bytes(), symbol.as_bytes()],
        bump,
        payer = signer,
        mint::decimals = 9,
        mint::authority = mint_pda,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [POOL_LP_MINT_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        mint::decimals = 9,
        mint::authority = mint_pda,
        payer = signer,
        token::token_program = token_program,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, SPLMint>>,

    #[account(
        init,
        associated_token::mint = lp_mint,
        associated_token::authority = mint_pda,
        payer = signer,
        token::token_program = token_program,
    )]
    pub lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK - fine as we need the vault to hold both SOL and SPL tokens
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED.as_bytes(), symbol.as_bytes()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>, // Unchecked account to handle both SOL and SPL tokens

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metaplex>,
}
