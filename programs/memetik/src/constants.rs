use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

pub const DEFAULT_TOKEN_DECIMALS: u8 = 9;
pub const TOKEN_SCALE: f64 = 1_000_000_000.0; // 10^9, precomputed for TOKEN_DECIMALS = 9
pub const SECONDS_IN_A_HOUR: i64 = 60 * 60;
pub const SECONDS_IN_A_DAY: i64 = SECONDS_IN_A_HOUR * 24;

pub const REQUIRED_ESCROW_AMOUNT: u64 = 1 * LAMPORTS_PER_SOL; // in lamports
pub const MIN_TOK_PRICE: f64 = 1.0 / TOKEN_SCALE; // 100 lamports in atomic units
pub const DAYS_TO_MATURITY: i64 = 2; // number of days the pool has to reach milestone
pub const REQUIRED_POOL_BALANCE_TO_MATURE: u64 = 1 * LAMPORTS_PER_SOL; // amount required for a pool to mature (convert to AMM) - effectively the market
