use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub enum PoolStatusBitIndex {
    Deposit,
    Withdraw,
    Swap,
}

#[derive(PartialEq, Eq)]
pub enum PoolStatusBitFlag {
    Enable,
    Disable,
}

#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Default, Debug)]
pub struct PoolState {
    pub pool_creator: Pubkey,
    pub token_vault: Pubkey,
    pub lp_mint: Pubkey,
    pub token_mint: Pubkey,
    pub token_program: Pubkey,
    pub auth_bump: u8,
    pub status: u8,
    pub lp_mint_decimals: u8,
    pub mint_decimals: u8,
    pub lp_supply: u64,
    pub protocol_fees_token: u64,
    pub fund_fees_token: u64,
    pub open_time: u64,
    pub padding: [u64; 32],
}

impl PoolState {
    pub const LEN: usize = 8 + 10 * 32 + 1 * 5 + 8 * 6 + 8 * 32;
    pub fn initialize(
        &mut self,
        auth_bump: u8,
        open_time: u64,
        pool_creator: Pubkey,
        token_vault: Pubkey,
        token_mint: &InterfaceAccount<Mint>,
        lp_mint: &InterfaceAccount<Mint>,
    ) {
        self.pool_creator = pool_creator.key();
        self.token_vault = token_vault;
        self.token_mint = token_mint.key();
        self.mint_decimals = token_mint.decimals;
        self.lp_mint = lp_mint.key();
        self.lp_supply = 0;
        self.lp_mint_decimals = lp_mint.decimals;
        self.padding = [0u64; 32];
        self.fund_fees_token = 0;
        self.auth_bump = auth_bump;
        self.open_time = open_time;
        self.protocol_fees_token = 0;
        self.token_program = *token_mint.to_account_info().owner;
    }
}
