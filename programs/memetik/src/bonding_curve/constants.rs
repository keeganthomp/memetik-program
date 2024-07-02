use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub const DEFAULT_TOKEN_DECIMALS: u8 = 9;
pub const SECONDS_IN_A_HOUR: i64 = 60 * 60;
pub const SECONDS_IN_A_DAY: i64 = SECONDS_IN_A_HOUR * 24;

pub const REQUIRED_ESCROW_AMOUNT: u64 = 0 * LAMPORTS_PER_SOL; // in lamports
pub const DAYS_TO_MATURITY: i64 = 2; // number of days the pool has to reach milestone

pub const REQUIRED_MARKET_CAP_TO_MATURE: f64 = 50_000.0; // is USD (i.e 1000.0 is $1000.0)
