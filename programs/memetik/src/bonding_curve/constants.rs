use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub const DEFAULT_TOKEN_DECIMALS: u8 = 9;
pub const SECONDS_IN_A_HOUR: i64 = 60 * 60;
pub const SECONDS_IN_A_DAY: i64 = SECONDS_IN_A_HOUR * 24;

pub const REQUIRED_ESCROW_AMOUNT: u64 = 0 * LAMPORTS_PER_SOL; // in lamports
pub const DAYS_TO_MATURITY: i64 = 2; // number of days the pool has to reach milestone
pub const REQUIRED_POOL_BALANCE_TO_MATURE: u64 = 1 * LAMPORTS_PER_SOL; // amount required for a pool to mature (convert to AMM) - effectively the market

pub const TOKEN_SCALE: f64 = 1e18;
