use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::Metadata as Metaplex,
    token::{Mint, Token},
};

use crate::bonding_curve::constants::DEFAULT_TOKEN_DECIMALS;
use crate::amm::constants::*;
use crate::state::pool::*;

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
