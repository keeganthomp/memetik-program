use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{self, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};
use pool::{
    calculate_price_per_unit, calculate_total_cost, get_new_supply, get_starting_tok_price, Pool,
    TOKEN_DECIMALS,
};

mod pool;

declare_id!("AryGSmQ8KjdJCjTF6ta9DZXse4JnCJvohYnJ9btkm9gZ");

#[program]
pub mod memetik {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        pool_id: u64,
        token_info: TokenArgs,
    ) -> Result<Pool> {
        let creator = &ctx.accounts.signer;
        let pool = &mut ctx.accounts.pool;

        let seeds = &["mint".as_bytes(), &pool_id.to_le_bytes(), &[ctx.bumps.mint]];
        let signer = [&seeds[..]];

        let token_data: DataV2 = DataV2 {
            name: token_info.name,
            symbol: token_info.symbol,
            uri: token_info.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };
        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: creator.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );
        let is_mutable = false;
        let update_authority_is_signer = true;
        let collection_details = None;
        create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            is_mutable,
            update_authority_is_signer,
            collection_details,
        )?;

        msg!("Token mint created successfully.");

        pool.id = pool_id;
        pool.tok_price = get_starting_tok_price();
        pool.mint = *ctx.accounts.mint.to_account_info().key;

        Ok(pool.clone().into_inner())
    }

    pub fn buy(ctx: Context<Buy>, pool_id: u64, amount: u64) -> Result<Pool> {
        require!(amount > 0, Error::MustBuyAtLeastOneToken);

        let current_supply = ctx.accounts.mint.supply;
        let pool_balance = ctx.accounts.pool.to_account_info().lamports();
        let total_cost = calculate_total_cost(current_supply, pool_balance, amount);

        msg!("Current supply: {}", current_supply);
        msg!("Total SOL to pay: {}", total_cost);
        msg!("Amount of tokens to mint: {}", amount);

        // Transfer SOL to the pool
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.buyer.to_account_info().key(),
            &ctx.accounts.pool.to_account_info().key(),
            total_cost,
        );
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.pool.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;

        msg!("SOL transferred to pool successfully");
        msg!(
            "Pool balance: {}",
            ctx.accounts.pool.to_account_info().lamports()
        );

        let seeds = &["mint".as_bytes(), &pool_id.to_le_bytes(), &[ctx.bumps.mint]];
        let signer = [&seeds[..]];

        // Mint the tokens to the buyer's account in atomic units
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.buyer_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            amount,
        )?;

        // Update the token price based on the new supply
        let new_supply = get_new_supply(current_supply, amount, false);
        ctx.accounts.pool.tok_price = calculate_price_per_unit(new_supply);

        msg!("Tokens minted to buyer successfully");
        msg!(
            "New token supply: {}",
            current_supply.checked_add(amount).unwrap()
        );
        msg!("New token price: {}", ctx.accounts.pool.tok_price);

        Ok(ctx.accounts.pool.clone().into_inner())
    }

    pub fn sell(ctx: Context<Sell>, _pool_id: u64, amount: u64) -> Result<Pool> {
        require!(amount > 0, Error::NoTokensToSell);
        require!(
            ctx.accounts.seller_token_account.amount >= amount,
            Error::NoTokensToSell
        );

        let current_supply = ctx.accounts.mint.supply;
        let pool_balance = ctx.accounts.pool.to_account_info().lamports();
        let sol_to_receive = calculate_total_cost(current_supply, pool_balance, amount);

        msg!("Current supply: {}", current_supply);
        msg!("Pool balance before transaction: {}", pool_balance);
        msg!("SOL to receive: {}", sol_to_receive);

        // Burn the tokens from the seller's account
        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info().clone(),
            from: ctx.accounts.seller_token_account.to_account_info().clone(),
            authority: ctx.accounts.seller.to_account_info().clone(),
        };
        let cpi_context =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::burn(cpi_context, amount)?;

        msg!("Tokens burned successfully");

        // Transfer SOL from the pool to the seller
        **ctx
            .accounts
            .pool
            .to_account_info()
            .try_borrow_mut_lamports()? -= sol_to_receive;
        **ctx
            .accounts
            .seller
            .to_account_info()
            .try_borrow_mut_lamports()? += sol_to_receive;

        msg!("SOL transferred to seller successfully.");

        // Update the token price based on the new supply
        let new_supply = get_new_supply(current_supply, amount, true);
        ctx.accounts.pool.tok_price = calculate_price_per_unit(new_supply);

        msg!("New supply after transaction: {}", new_supply);
        msg!("New token price: {}", ctx.accounts.pool.tok_price);
        msg!(
            "Pool balance after transaction: {}",
            ctx.accounts.pool.to_account_info().lamports()
        );

        Ok(ctx.accounts.pool.clone().into_inner())
    }
}

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"pool", pool_id.to_le_bytes().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Pool>()
    )]
    pub pool: Account<'info, Pool>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [b"mint", pool_id.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
}

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool", pool_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"mint", pool_id.to_le_bytes().as_ref()],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool", pool_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"mint", pool_id.to_le_bytes().as_ref()],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[error_code]
pub enum Error {
    #[msg("Incorrect mint account provided")]
    IncorrectMintAccount,
    #[msg("Incorrect pool id")]
    IncorrectPoolId,
    #[msg("No tokens to sell")]
    NoTokensToSell,
    #[msg("Must buy at least one token")]
    MustBuyAtLeastOneToken,
    #[msg("Overflow")]
    Overflow,
    #[msg("Pool has insufficient funds")]
    PoolInsufficientFunds,
}
