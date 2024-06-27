use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
};
use bonding_curve::constants::REQUIRED_ESCROW_AMOUNT;
use context::initialize_pool::*;

mod amm;
mod bonding_curve;
mod context;
mod state;
mod utils;

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
            MINT_PDA.as_bytes(),
            symbol.as_bytes(),
            &[ctx.bumps.mint_pda],
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
                update_authority: ctx.accounts.mint_pda.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint_pda.to_account_info(),
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

        // /////////////////////////////////
        // // Transfer SOL into pool escrow
        // /////////////////////////////////
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &ctx.accounts.escrow.key(),
            REQUIRED_ESCROW_AMOUNT,
        );
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.escrow.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("SOL transferred intoto escrow successfully");

        // /////////////////////////
        // // Initialize pool
        // /////////////////////////
        let mut pool = ctx.accounts.pool.load_init()?;
        pool.initialize(
            &ctx.accounts.signer.key(),
            &token_info.symbol,
            &ctx.accounts.mint.key(),
        );
        Ok(())
    }
}
