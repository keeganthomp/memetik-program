use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{self, mint_to, Burn, Mint, MintTo, Token, TokenAccount},
};

pub const OBSERVATION_SEED: &str = "observation";
pub const POOL_SEED: &str = "pool";
pub const POOL_LP_MINT_SEED: &str = "pool_lp_mint";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const AUTH_SEED: &str = "vault_and_lp_mint_auth_seed";
pub const RAYDIUM_SP_SWAP_DEVNET: &str = "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW";

pub mod create_pool_fee_reveiver {
    use anchor_lang::prelude::declare_id;
    #[cfg(feature = "devnet")]
    declare_id!("G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2");
    #[cfg(not(feature = "devnet"))]
    declare_id!("DNXgeM9EiiaAbaWvwjHj9fQQLAX5ZsfHyvmYUNRAdNC8");
}

use bonding_curve::*;
use constants::*;
use utils::*;

mod bonding_curve;
mod constants;
mod utils;

// CPI HELPERS
fn get_function_hash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()[..8],
    );
    sighash
}
fn serialize_cpi_instruction_data(namespace: &str, name: &str, args: CreateAMMArgs) -> Vec<u8> {
    let hash = get_function_hash(namespace, name);
    let mut buf: Vec<u8> = vec![];
    buf.extend_from_slice(&hash);
    args.serialize(&mut buf).unwrap();
    buf
}
fn create_cpi_instruction(
    program_id: Pubkey,
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
) -> Instruction {
    Instruction {
        program_id,
        accounts,
        data,
    }
}
fn invoke_cpi_instruction(instruction: Instruction, account_infos: &[AccountInfo]) -> Result<()> {
    invoke(&instruction, account_infos)?;
    Ok(())
}

fn invoke_signed_cpi_instruction(
    instruction: Instruction,
    account_infos: &[AccountInfo],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    invoke_signed(&instruction, account_infos, signer_seeds)?;
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize)]
struct CreateAMMArgs {
    init_amount_0: u64,
    init_amount_1: u64,
    open_time: u64,
}

declare_id!("14a3y3QApSRvxd8kgG9S4FTjQFeTQ92XpUxTvXkTrknR");

#[program]
pub mod memetik {
    use std::str::FromStr;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, token_info: TokenArgs) -> Result<Pool> {
        let creator = &ctx.accounts.signer;
        let pool = &mut ctx.accounts.pool;
        let escrow = &mut ctx.accounts.escrow;

        require!(
            check_valid_ticker(&token_info.symbol),
            Error::InvalidTickerFormat
        );

        /////////////////////////////////
        // Create the token mint
        /////////////////////////////////
        let seeds = &[
            "mint".as_bytes(),
            &token_info.symbol.as_bytes(),
            &[ctx.bumps.mint],
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

        /////////////////////////////////
        // Transfer SOL into pool escrow
        /////////////////////////////////
        let transfer_instruction =
            system_instruction::transfer(&creator.key(), &escrow.key(), REQUIRED_ESCROW_AMOUNT);
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                creator.to_account_info(),
                escrow.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("SOL transferred intoto escrow successfully");

        // init escrow
        escrow.pool = *pool.to_account_info().key;
        escrow.mint = *ctx.accounts.mint.to_account_info().key;
        escrow.owner = *creator.to_account_info().key;
        escrow.balance = REQUIRED_ESCROW_AMOUNT;

        // init pool
        let maturity_time_timestamp = calculate_test_time(); // time the pool has to reach milestone (maturity)
        pool.creator = *creator.to_account_info().key;
        pool.mint = *ctx.accounts.mint.to_account_info().key;
        pool.ticker = token_info.symbol;
        pool.tok_price = MIN_TOK_PRICE as u64;
        pool.maturity_time = maturity_time_timestamp;
        pool.has_matured = false;

        Ok(pool.clone().into_inner())
    }

    pub fn close(ctx: Context<Close>, ticker: String) -> Result<()> {
        let closer = &ctx.accounts.creator;
        let pool = &ctx.accounts.pool;
        let escrow = &ctx.accounts.escrow;
        let escrow_sol_balance = escrow.get_lamports();

        require!(pool.ticker == ticker, Error::InvalidPoolTicker);
        require!(
            pool.creator == *closer.to_account_info().key,
            Error::NotPoolCreator
        );
        require!(
            escrow.pool == *pool.to_account_info().key,
            Error::InvalidEscrowAccount
        );
        require!(
            escrow.owner == *closer.to_account_info().key,
            Error::NotEscrowOwner
        );
        msg!("pool maturity time: {}", pool.maturity_time);
        msg!("current timestamp: {}", Clock::get()?.unix_timestamp);
        require!(
            check_if_maturity_time_passed(pool.maturity_time),
            Error::PoolNotMatured
        );

        require!(
            escrow_sol_balance > 0 && escrow.balance > 0 && escrow_sol_balance >= escrow.balance,
            Error::InsufficientFundsInEscrow
        );

        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, ticker: String, amount: u64) -> Result<Pool> {
        require!(amount > 0, Error::MustBuyAtLeastOneToken);

        let current_supply = ctx.accounts.mint.supply;
        let (total_cost, latest_price_per_unit) = calculate_price(current_supply, amount, false);

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
        msg!("SOL sent to pool successfully");

        let seeds = &["mint".as_bytes(), ticker.as_bytes(), &[ctx.bumps.mint]];
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
        msg!("Tokens minted to buyer successfully");

        // Update the token price based on the new supply
        ctx.accounts.pool.tok_price = latest_price_per_unit;

        // check if pool has matured
        let new_pool_balance = ctx.accounts.pool.to_account_info().lamports();
        let has_reached_maturity_amount = check_if_maturity_amount_reached(new_pool_balance);

        // https://github.com/raydium-io/raydium-amm
        if has_reached_maturity_amount {
            ctx.accounts.pool.has_matured = true;
            /////////////////////////////////
            // CPI TO RAYDIUM CP-SWAP
            //////////////////////////////////
            // let raydium_program_id: Pubkey = Pubkey::from_str(RAYDIUM_SP_SWAP_DEVNET).unwrap();
            // let accounts_meta = vec![
            //     AccountMeta::new(*ctx.accounts.pool_state.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.authority.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_0_mint.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_1_mint.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.lp_mint.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.creator_token_0.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.creator_token_1.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.creator_lp_token.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_0_vault.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_1_vault.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.create_pool_fee.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.observation_state.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_program.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_0_program.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.token_1_program.to_account_info().key, false),
            //     AccountMeta::new(
            //         *ctx.accounts.associated_token_program.to_account_info().key,
            //         false,
            //     ),
            //     AccountMeta::new(*ctx.accounts.system_program.to_account_info().key, false),
            //     AccountMeta::new(*ctx.accounts.rent.to_account_info().key, false),
            // ];
            // let now = Clock::get()?.unix_timestamp;
            // let create_amm_args = CreateAMMArgs {
            //     init_amount_0: current_supply,
            //     init_amount_1: new_pool_balance,
            //     open_time: now as u64,
            // };
            // let data = serialize_cpi_instruction_data("global", "initialize", create_amm_args);
            // let cpi_ix = create_cpi_instruction(raydium_program_id, accounts_meta, data);
            // let account_infos = vec![
            //     ctx.accounts.pool_state.to_account_info(),
            //     ctx.accounts.authority.to_account_info(),
            //     ctx.accounts.token_0_mint.to_account_info(),
            //     ctx.accounts.token_1_mint.to_account_info(),
            //     ctx.accounts.lp_mint.to_account_info(),
            //     ctx.accounts.creator_token_0.to_account_info(),
            //     ctx.accounts.creator_token_1.to_account_info(),
            //     ctx.accounts.creator_lp_token.to_account_info(),
            //     ctx.accounts.token_0_vault.to_account_info(),
            //     ctx.accounts.token_1_vault.to_account_info(),
            //     ctx.accounts.create_pool_fee.to_account_info(),
            //     ctx.accounts.observation_state.to_account_info(),
            //     ctx.accounts.token_program.to_account_info(),
            //     ctx.accounts.token_0_program.to_account_info(),
            //     ctx.accounts.token_1_program.to_account_info(),
            //     ctx.accounts.associated_token_program.to_account_info(),
            //     ctx.accounts.system_program.to_account_info(),
            //     ctx.accounts.rent.to_account_info(),
            // ];
            // invoke_cpi_instruction(cpi_ix, &account_infos)?;
        }

        Ok(ctx.accounts.pool.clone().into_inner())
    }

    pub fn sell(ctx: Context<Sell>, _ticker: String, amount: u64) -> Result<Pool> {
        require!(amount > 0, Error::NoTokensToSell);
        require!(
            ctx.accounts.seller_token_account.amount >= amount,
            Error::NoTokensToSell
        );

        let current_supply = ctx.accounts.mint.supply;
        let (sol_to_receive, latest_price_per_unit) = calculate_price(current_supply, amount, true);

        // check if pool has enough funds to buy token from seller
        let min_pool_rent = 8 + std::mem::size_of::<Pool>() as u64;
        require!(
            ctx.accounts.pool.to_account_info().lamports() >= (sol_to_receive + min_pool_rent),
            Error::PoolInsufficientFunds
        );

        msg!(
            "Pool bal {}",
            ctx.accounts.pool.to_account_info().lamports()
        );

        // Burn the tokens from the seller's token account
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

        msg!(
            "Pool balance after sell: {}",
            ctx.accounts.pool.to_account_info().lamports()
        );

        // Update the token price based on the new supply
        ctx.accounts.pool.tok_price = latest_price_per_unit;

        Ok(ctx.accounts.pool.clone().into_inner())
    }
    pub fn get_pool(ctx: Context<GetPool>, ticker: String) -> Result<Pool> {
        let pool = &ctx.accounts.pool;
        require!(pool.ticker == ticker, Error::InvalidPoolTicker);
        Ok(pool.clone().into_inner())
    }
}

#[derive(Accounts)]
#[instruction(token_info: TokenArgs)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"pool", token_info.symbol.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<Pool>()
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = signer,
        seeds = [b"pool-escrow", token_info.symbol.as_bytes()],
        bump,
        space = 8 + std::mem::size_of::<PoolEscrow>()
    )]
    pub escrow: Account<'info, PoolEscrow>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [b"mint", token_info.symbol.as_bytes()],
        bump,
        payer = signer,
        mint::decimals = DEFAULT_TOKEN_DECIMALS,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
}

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct Close<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool", ticker.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"pool-escrow", ticker.as_bytes()],
        bump,
        close = creator
    )]
    pub escrow: Account<'info, PoolEscrow>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool", ticker.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"mint", ticker.as_bytes()],
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

    // /////////////////////////////////
    // // AMM CREATION ACCOUNTS
    // /////////////////////////////////
    /// Which config the pool belongs to.
    /// CHECK fwer
    pub cp_swap_program: UncheckedAccount<'info>,
    /// CHECK fine
    pub amm_config: UncheckedAccount<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            raydium_cp_swap::AUTH_SEED.as_bytes(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Initialize an account to store the pool state, init by cp-swap
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        seeds::program = cp_swap_program.key(),
        bump,
    )]
    pub pool_state: UncheckedAccount<'info>,

    // // /// Token_0 mint, the key must smaller then token_1 mint.
    // #[account(
    //     constraint = token_0_mint.key() < token_1_mint.key(),
    //     mint::token_program = token_0_program,
    // )]
    // pub token_0_mint: Account<'info, Mint>,

    // /// Token_1 mint, the key must grater then token_0 mint.
    #[account(
        mint::token_program = token_program,
    )]
    pub token_1_mint: Account<'info, Mint>,

    /// CHECK: pool lp mint, init by cp-swap
    // #[account(
    //     mut,
    //     seeds = [
    //         POOL_LP_MINT_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //     ],
    //     seeds::program = cp_swap_program.key(),
    //     bump,
    // )]
    // pub lp_mint: UncheckedAccount<'info>,

    // /// payer token0 account
    // #[account(
    //     init_if_needed,
    //     token::mint = token_0_mint,
    //     token::authority = buyer,
    // )]
    // pub creator_token_0: Account<'info, TokenAccount>,

    // /// creator token1 account
    #[account(
        mut,
        token::mint = token_1_mint,
        token::authority = buyer,
    )]
    pub creator_token_1: Account<'info, TokenAccount>,

    // /// CHECK: creator lp ATA token account, init by cp-swap
    // #[account(mut)]
    // pub creator_lp_token: UncheckedAccount<'info>,

    // /// CHECK: Token_0 vault for the pool, init by cp-swap
    // #[account(
    //     mut,
    //     seeds = [
    //         POOL_VAULT_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //         token_0_mint.key().as_ref()
    //     ],
    //     seeds::program = cp_swap_program.key(),
    //     bump,
    // )]
    // pub token_0_vault: UncheckedAccount<'info>,

    // /// CHECK: Token_1 vault for the pool, init by cp-swap
    // #[account(
    //     mut,
    //     seeds = [
    //         POOL_VAULT_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //         token_1_mint.key().as_ref()
    //     ],
    //     seeds::program = cp_swap_program.key(),
    //     bump,
    // )]
    // pub token_1_vault: UncheckedAccount<'info>,

    // /// create pool fee account
    // #[account(
    //     mut,
    //     address= create_pool_fee_reveiver::id(),
    // )]
    // pub create_pool_fee: Account<'info, TokenAccount>,

    // /// CHECK: an account to store oracle observations, init by cp-swap
    // #[account(
    //     mut,
    //     seeds = [
    //         OBSERVATION_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //     ],
    //     seeds::program = cp_swap_program.key(),
    //     bump,
    // )]
    // pub observation_state: UncheckedAccount<'info>,
    // /// Spl token program or token program 2022
    // pub token_0_program: Program<'info, Token>,
    // /// Spl token program or token program 2022
    // pub token_1_program: Program<'info, Token>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        seeds = [b"pool", ticker.as_bytes()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"mint", ticker.as_bytes()],
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

#[derive(Accounts)]
#[instruction(ticker: String)]
pub struct GetPool<'info> {
    #[account(seeds = [b"pool", ticker.as_bytes()], bump)]
    pub pool: Account<'info, Pool>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TokenArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[account]
pub struct Pool {
    pub ticker: String,
    pub tok_price: u64, // Store price in atomic units (lamports)
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub maturity_time: i64,
    pub has_matured: bool,
}
#[account]

pub struct PoolEscrow {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub balance: u64, // in atomic units (lamports)
}

#[error_code]
pub enum Error {
    #[msg("Invalid pool ticker")]
    InvalidPoolTicker,
    #[msg("Not pool creator")]
    NotPoolCreator,
    #[msg("Not escrow owner")]
    NotEscrowOwner,
    #[msg("Invalid escrow account")]
    InvalidEscrowAccount,
    #[msg("Insufficient funds in escrow")]
    InsufficientFundsInEscrow,
    #[msg("Pool has not matured")]
    PoolNotMatured,
    #[msg("No tokens to sell")]
    NoTokensToSell,
    #[msg("Must buy at least one token")]
    MustBuyAtLeastOneToken,
    #[msg("Pool has insufficient funds")]
    PoolInsufficientFunds,
    #[msg("Invalid ticker format")]
    InvalidTickerFormat,
}
