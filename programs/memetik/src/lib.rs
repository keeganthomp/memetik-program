#![allow(unused)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    },
    token::{self, mint_to, Burn, MintTo, Transfer},
};

pub mod amm;
pub mod bonding_curve;
pub mod context;
pub mod errors;
pub mod state;
pub mod utils;

pub use bonding_curve::{constants::REQUIRED_ESCROW_AMOUNT, price::*, utils::*};
pub use context::{
    amm_add_liquidity::*, amm_swap_tokens::*, bonding_buy_tokens::*, bonding_sell_tokens::*,
    close_pool::*, initialize_pool::*,
};
pub use errors::Error;
pub use state::pool::*;

declare_id!("14a3y3QApSRvxd8kgG9S4FTjQFeTQ92XpUxTvXkTrknR");

#[program]
pub mod memetik {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        symbol: String,
        name: String,
        uri: String,
    ) -> Result<()> {
        // check ticker value
        require!(
            symbol.chars().all(char::is_alphanumeric),
            Error::InvalidTicker
        );

        let seeds = &[
            POOL_MINT_SEED.as_bytes(),
            symbol.as_bytes(),
            &[ctx.bumps.mint],
        ];
        let signer = [&seeds[..]];

        let token_data: DataV2 = DataV2 {
            name: name,
            symbol: symbol.clone(),
            uri: uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.signer.to_account_info(),
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
        create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            is_mutable,
            update_authority_is_signer,
            None,
        )?;
        msg!("Metadata account created successfully");

        // /////////////////////////
        // // Initialize pool
        // /////////////////////////
        let pool = &mut ctx.accounts.pool;
        pool.creator = ctx.accounts.signer.to_account_info().key();
        pool.ticker = symbol.to_uppercase().clone();
        pool.mint = ctx.accounts.mint.to_account_info().key();
        pool.maturity_time = calculate_test_time();

        msg!("Pool initialized successfully");
        Ok(())
    }

    pub fn buy(ctx: Context<BuyTokens>, ticker: String, amount: u64) -> Result<()> {
        require!(amount > 0, Error::NoTokensToBuy);

        let pool_state = &mut ctx.accounts.pool;
        let mint = &ctx.accounts.mint;

        let current_supply = ctx.accounts.mint.supply;
        let (total_cost, latest_price_per_unit) = calculate_price(current_supply, amount, false);

        /////////////////////////////////////
        // Transfer SOL to vault
        ////////////////////////////////////
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.buyer.to_account_info().key(),
            &ctx.accounts.sol_vault.to_account_info().key(),
            total_cost,
        );
        let system_program = ctx.accounts.system_program.as_ref();
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.sol_vault.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;
        msg!("SOL sent to pool successfully");

        /////////////////////////////////
        // Mint tokens to buyer
        /////////////////////////////////
        let auth_seeds = &[
            POOL_MINT_SEED.as_bytes(),
            ticker.as_bytes(),
            &[ctx.bumps.mint],
        ];
        let signer = [&auth_seeds[..]];
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

        /////////////////////////////////
        // Update pool state
        /////////////////////////////////

        // check if pool has matured
        let new_pool_vault_balance = ctx.accounts.sol_vault.to_account_info().lamports();
        let has_reached_maturity_amount = check_if_maturity_amount_reached(new_pool_vault_balance);
        if has_reached_maturity_amount {}

        Ok(())
    }

    pub fn sell(ctx: Context<SellTokens>, _ticker: String, amount: u64) -> Result<()> {
        require!(amount > 0, Error::NoTokensToSell);
        require!(
            ctx.accounts.seller_token_account.amount >= amount,
            Error::NoTokensToSell
        );

        let pool_state = &mut ctx.accounts.pool;
        let mint = &ctx.accounts.mint;

        let current_supply = ctx.accounts.mint.supply;
        let (sol_to_receive, latest_price_per_unit) = calculate_price(current_supply, amount, true);

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

        /////////////////////////////////
        // Transfer SOL to the seller
        /////////////////////////////////
        **ctx
            .accounts
            .sol_vault
            .to_account_info()
            .try_borrow_mut_lamports()? -= sol_to_receive;
        **ctx
            .accounts
            .seller
            .to_account_info()
            .try_borrow_mut_lamports()? += sol_to_receive;

        msg!("SOL transferred to seller successfully.");

        Ok(())
    }

    pub fn close(ctx: Context<ClosePool>, _symbol: String) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        require!(
            pool.creator == ctx.accounts.signer.to_account_info().key(),
            Error::Unauthorized
        );

        let mint = &ctx.accounts.mint;
        let has_passed_maturity_time = check_if_maturity_time_passed(pool.maturity_time);

        // can only close pool if it has matured and the maturity time has passed
        require!(has_passed_maturity_time, Error::PoolCannotBeClosed);

        Ok(())
    }

    pub fn swap(
        ctx: Context<Swap>,
        ticker: String,
        amount_in: u64,
        is_sol_to_token: bool,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &ctx.accounts.user;
        let user_token_account = &ctx.accounts.user_token_account;
        let pool_token_mint = &ctx.accounts.token_mint;
        let lp_mint = &ctx.accounts.lp_mint;
        let sol_vault = &ctx.accounts.sol_vault;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        let mint_supply = pool_token_mint.supply;
        let lp_supply = lp_mint.supply;

        // Ensure there is sufficient liquidity in the pool
        let liquidity_available = if is_sol_to_token {
            mint_supply
        } else {
            lp_supply
        };

        require!(
            liquidity_available >= amount_in,
            Error::InsufficientLiquidity
        );

        // Calculate the amount to swap using the constant product formula (x * y = k)
        let (amount_out, new_sol_reserve, new_token_reserve) = if is_sol_to_token {
            // Swapping SOL for tokens
            let amount_in_with_fee = amount_in * 997 / 1000; // 0.3% fee
            let amount_out = (amount_in_with_fee * mint_supply) / (lp_supply + amount_in_with_fee);
            let new_sol_reserve = lp_supply + amount_in;
            let new_token_reserve = mint_supply - amount_out;
            (amount_out, new_sol_reserve, new_token_reserve)
        } else {
            // Swapping tokens for SOL
            let amount_in_with_fee = amount_in * 997 / 1000; // 0.3% fee
            let amount_out = (amount_in_with_fee * lp_supply) / (mint_supply + amount_in_with_fee);
            let new_sol_reserve = lp_supply - amount_out;
            let new_token_reserve = mint_supply + amount_in;
            (amount_out, new_sol_reserve, new_token_reserve)
        };

        // Perform the swap
        // let auth_seeds = &[
        //     POOL_AUTH_SEED.as_bytes(),
        //     ticker.as_bytes(),
        //     &[ctx.bumps.authority],
        // ];
        // let signer = [&auth_seeds[..]];

        if is_sol_to_token {
            // Transfer SOL from user to pool SOL vault
            invoke(
                &system_instruction::transfer(&user.key(), &sol_vault.key(), amount_in),
                &[
                    user.to_account_info(),
                    sol_vault.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;

            // Transfer tokens from pool to user
            // token::transfer(
            //     CpiContext::new_with_signer(
            //         token_program.to_account_info(),
            //         Transfer {
            //             from: sol_vault.to_account_info(),
            //             to: user_token_account.to_account_info(),
            //             authority: pool_authority.to_account_info(),
            //         },
            //         &signer,
            //     ),
            //     amount_out,
            // )?;
        } else {
            // Transfer tokens from user to pool token vault
            token::transfer(
                CpiContext::new(
                    token_program.to_account_info(),
                    Transfer {
                        from: user_token_account.to_account_info(),
                        to: ctx.accounts.token_vault.to_account_info(),
                        authority: user.to_account_info(),
                    },
                ),
                amount_in,
            )?;

            // Transfer SOL from pool vault to user
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.sol_vault.key(),
                    &user.key(),
                    amount_out,
                ),
                &[
                    ctx.accounts.sol_vault.to_account_info(),
                    user.to_account_info(),
                    system_program.to_account_info(),
                ],
            )?;
        }

        // Update pool reserves
        pool.sol_balance = new_sol_reserve;
        pool.token_balance = new_token_reserve;

        Ok(())
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        ticker: String,
        sol_amount: u64,
        token_amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &ctx.accounts.user;
        let user_token_account = &ctx.accounts.user_token_account;
        let user_lp_token_account = &ctx.accounts.user_lp_token_account;
        let pool_lp_mint = &ctx.accounts.lp_mint;
        let pool_sol_vault = &ctx.accounts.sol_vault;
        let token_program = &ctx.accounts.token_program;
        let system_program = &ctx.accounts.system_program;

        msg!("Adding liquidity to pool");

        // // Transfer SOL from user to pool SOL vault
        invoke(
            &system_instruction::transfer(&user.key(), &pool_sol_vault.key(), sol_amount),
            &[
                user.to_account_info(),
                pool_sol_vault.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;

        msg!("SOL transferred to pool vault successfully");

        // // Transfer tokens from user to pool token vault
        token::transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: user_token_account.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: user.to_account_info(),
                },
            ),
            token_amount,
        )?;

        msg!("Tokens transferred to pool vault successfully");

        // Calculate the amount of LP tokens to mint
        let lp_supply = pool_lp_mint.supply;
        let mint_supply = pool.token_balance;
        let lp_amount = if lp_supply == 0 {
            // Initial liquidity, 1:1 ratio
            sol_amount
        } else {
            // Mint LP tokens proportional to the liquidity added
            let lp_sol_ratio = (sol_amount * lp_supply) / lp_supply;
            let lp_token_ratio = (token_amount * lp_supply) / mint_supply;
            std::cmp::min(lp_sol_ratio, lp_token_ratio)
        };

        // // Mint LP tokens to user
        let auth_seeds = &[
            POOL_AMM_SEED.as_bytes(),
            ticker.as_bytes(),
            &[ctx.bumps.pool],
        ];
        let signer = [&auth_seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: pool_lp_mint.to_account_info(),
                    to: user_lp_token_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                &signer,
            ),
            lp_amount,
        )?;

        msg!("LP tokens minted to user successfully");

        // Update pool reserves
        // pool.sol_balance += sol_amount;
        // pool.token_balance += token_amount;

        Ok(())
    }
}
