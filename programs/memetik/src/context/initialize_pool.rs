use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
};

use crate::bonding_curve::constants::DEFAULT_TOKEN_DECIMALS;
use crate::state::pool::*;

pub const POOL_BONDING_SEED: &str = "pool";
pub const POOL_AMM_SEED: &str = "pool_amm";
pub const POOL_MINT_SEED: &str = "pool_mint";
pub const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
pub const POOL_ESCROW_SEED: &str = "pool_escrow";
pub const POOL_SOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_AUTH_SEED: &str = "pool_auth";

#[account]
pub struct EmptyAccount {}

#[derive(Accounts)]
#[instruction(symbol: String, name: String, uri: String)]
pub struct InitializePool<'info> {
    #[account(
        init,
        seeds = [POOL_MINT_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        payer = signer,
        mint::decimals = 9,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [POOL_BONDING_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        payer = signer,
        space = 8 + std::mem::size_of::<BondingPool>(),
    )]
    pub pool: Account<'info, BondingPool>,

    #[account(
        init,
        payer = signer,
        seeds = [POOL_SOL_VAULT_SEED.as_bytes(), symbol.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<PoolSolVault>(),
    )]
    pub sol_vault: Account<'info, PoolSolVault>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
}
