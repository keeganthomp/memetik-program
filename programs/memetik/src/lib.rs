#![allow(unused)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    },
    token::{self, mint_to, MintTo},
};

pub mod amm;
pub mod bonding_curve;
pub mod context;
pub mod state;
pub mod utils;

pub use bonding_curve::{constants::REQUIRED_ESCROW_AMOUNT, price::*, utils::*};
pub use context::{
    bonding_buy_tokens::*, bonding_sell_tokens::*, close_pool::*, initialize_pool::*,
};
pub use state::pool::*;

declare_id!("14a3y3QApSRvxd8kgG9S4FTjQFeTQ92XpUxTvXkTrknR");

#[program]
pub mod memetik {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        symbol: String,
        token_info: TokenArgs,
    ) -> Result<()> {
        /////////////////////////////////
        // Create the token mint
        /////////////////////////////////
        let seeds = &[
            POOL_AUTH_SEED.as_bytes(),
            symbol.as_bytes(),
            &[ctx.bumps.authority],
        ];
        let signer = [&seeds[..]];

        let token_data: DataV2 = DataV2 {
            name: token_info.name,
            symbol: token_info.symbol.clone(),
            uri: token_info.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );
        let is_mutable = false;
        let update_authority_is_signer = false;
        let collection_details = None;
        create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            is_mutable,
            update_authority_is_signer,
            collection_details,
        )?;
        msg!("Token mint created successfully.");

        /////////////////////////////////////////////
        // Transfer initial SOL into vault as escrow
        /////////////////////////////////////////////
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &ctx.accounts.vault.key(),
            REQUIRED_ESCROW_AMOUNT,
        );
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;
        // vault will essentially start as escrow holdings while in "bonding curve mode"
        ctx.accounts.vault.creator = ctx.accounts.signer.key();
        ctx.accounts.vault.pool = ctx.accounts.pool.key();
        ctx.accounts.vault.mint = ctx.accounts.mint.key();
        ctx.accounts.vault.balance = REQUIRED_ESCROW_AMOUNT;
        msg!("SOL transferred into escrow successfully");

        /////////////////////////
        // Initialize pool
        /////////////////////////
        let mut pool = ctx.accounts.pool.load_init()?;
        pool.initialize(
            &ctx.accounts.signer.key(),
            &token_info.symbol,
            &ctx.accounts.mint.key(),
            &ctx.accounts.vault.key(),
            &ctx.accounts.lp_mint.key(),
        );
        Ok(())
    }

    pub fn buy(ctx: Context<BuyTokens>, ticker: String, amount: u64) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool.load_mut()?;
        let mint = &ctx.accounts.mint;

        let current_supply = ctx.accounts.mint.supply;
        let (total_cost, latest_price_per_unit) = calculate_price(current_supply, amount, false);

        /////////////////////////////////
        // Transfer SOL to the pool
        /////////////////////////////////
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.buyer.to_account_info().key(),
            &ctx.accounts.vault.to_account_info().key(),
            total_cost,
        );
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("SOL sent to pool successfully");

        /////////////////////////////////
        // Mint tokens to buyer
        /////////////////////////////////
        let auth_seeds = &[
            POOL_AUTH_SEED.as_bytes(),
            ticker.as_bytes(),
            &[ctx.bumps.authority],
        ];
        let signer = [&auth_seeds[..]];
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            amount,
        )?;
        msg!("Tokens minted to buyer successfully");

        /////////////////////////////////
        // Update pool state
        /////////////////////////////////
        pool_state.bonding_curve_price = latest_price_per_unit;

        // check if pool has matured
        let new_pool_vault_balance = ctx.accounts.vault.to_account_info().lamports();
        let has_reached_maturity_amount = check_if_maturity_amount_reached(new_pool_vault_balance);
        if has_reached_maturity_amount {
            pool_state.status = PoolStatus::Matured as u8;
        }

        Ok(())
    }
}
