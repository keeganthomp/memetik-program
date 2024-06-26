use crate::amm::context::create_pool::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Transfer};

pub fn initialize_pool(ctx: Context<CreatePool>, initial_token_amount: u64) -> Result<()> {
    let open_time = Clock::get()?.unix_timestamp as u64;
    let mut pool_state = ctx.accounts.pool_state.load_init()?;
    // who is initializing the pool
    let initializer = ctx.accounts.signer.clone();
    let pool_vault = ctx.accounts.vault.clone();
    pool_state.initialize(
        ctx.bumps.pool_state,
        open_time,
        initializer.key(),
        pool_vault.key(),
        &ctx.accounts.token_mint,
        &ctx.accounts.lp_mint,
    );

    // Transfer SPL tokens from user to pool vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.creator_token.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, initial_token_amount)?;

    // Transfer SOL from user to pool vault
    let transfer_instruction = system_instruction::transfer(
        &ctx.accounts.creator_token.key(),
        &ctx.accounts.vault.key(),
        initial_token_amount,
    );
    let system_program = ctx.accounts.system_program.as_ref();
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_instruction,
        &[
            ctx.accounts.creator_token.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            system_program.to_account_info(),
        ],
        &[],
    )?;
    Ok(())
}
