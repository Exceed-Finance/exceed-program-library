use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create_idempotent, AssociatedToken, Create},
    token_interface::{Mint, TokenInterface},
};

use crate::instructions::update_pair_yield::MAX_INTERVAL_APR_RATE;
use crate::state::pair::PRECISION;
use crate::{
    error::StakingError,
    state::{AccessControl, Pair},
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePairParams {
    pub interval_apr_rate: u64,     // APR rate for yield calculations
    pub seconds_per_interval: i32,  // Time interval for yield updates
    pub initial_exchange_rate: u64, // Initial exchange rate between base and LST
    pub deposit_cap: u64,           // Maximum deposit amount allowed
    pub stake_fee_bps: u16,         // Fee for staking operations (basis points)
    pub swap_fee_bps: u16,          // Fee for swap operations (basis points)
    pub withdraw_fee_bps: u16,      // Fee for withdrawal operations (basis points)
    pub minimum_deposit: u64,
}

impl CreatePairParams {
    pub fn validate(&self) -> Result<()> {
        // Validate interval_apr_rate
        require!(
            self.interval_apr_rate as u128 >= PRECISION,
            StakingError::InvalidYieldRate
        );

        require!(
            self.interval_apr_rate <= MAX_INTERVAL_APR_RATE,
            StakingError::MaxYieldRateExceeded
        );

        // Validate initial_exchange_rate
        require!(
            self.initial_exchange_rate > 0,
            StakingError::InvalidQuantity
        );

        // Validate deposit_cap
        require!(self.deposit_cap > 0, StakingError::InvalidQuantity);

        // Validate minimum_deposit
        require!(
            self.minimum_deposit <= self.deposit_cap,
            StakingError::InvalidMinimumDeposit
        );

        Ok(())
    }
}

fn validate_symbol(symbol: &str) -> bool {
    // Length between 3-10 chars
    if symbol.len() < 3 || symbol.len() > 10 {
        return false;
    }

    // Must start with a letter
    if !symbol.chars().next().unwrap().is_ascii_alphabetic() {
        return false;
    }

    // Must contain only alphanumeric characters or spaces
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ' ')
    {
        return false;
    }

    true
}

#[derive(Accounts)]
#[instruction(symbol: String)]
pub struct CreatePair<'info> {
    #[account(
        mut,
        address = access_control.pair_authority
    )]
    pub pair_authority: Signer<'info>,

    /// The authority proof account
    pub access_control: Box<Account<'info, AccessControl>>,

    // Base token mint
    pub base_token_mint: Box<InterfaceAccount<'info, Mint>>,

    // The Pair account to be created first
    #[account(
        init,
        payer = pair_authority,
        space = 8 + std::mem::size_of::<Pair>(),
        seeds = [
            b"pair",
            base_token_mint.key().as_ref(),
            lst_mint.key().as_ref()
        ],
        bump,
    )]
    pub pair: Box<Account<'info, Pair>>,

    /// CHECK: empty account used to create the ATA
    #[account(mut)]
    pub pair_base_token_account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = pair_authority,
        seeds = [
            b"lst_mint",
            symbol.as_bytes()
        ],
        bump,
        mint::authority = lst_mint,
        mint::decimals = base_token_mint.decimals,
    )]
    pub lst_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: empty account used to create the ATA
    #[account(mut)]
    pub lst_fee_account: UncheckedAccount<'info>,

    // Programs
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

fn create_token_accounts(ctx: &Context<CreatePair>) -> Result<()> {
    let base_token_mint_info = ctx.accounts.base_token_mint.to_account_info();
    let lst_mint_info = ctx.accounts.lst_mint.to_account_info();
    let pair_authority_info = ctx.accounts.pair_authority.to_account_info();
    let token_program_info = ctx.accounts.token_program.to_account_info();
    let associated_token_info = ctx.accounts.associated_token_program.to_account_info();
    let system_program_info = ctx.accounts.system_program.to_account_info();
    let pair_info = ctx.accounts.pair.to_account_info();
    let base_token_account_info = ctx.accounts.pair_base_token_account.to_account_info();
    let lst_fee_token_account_info = ctx.accounts.lst_fee_account.to_account_info();

    // create base token account
    create_idempotent(CpiContext::new(
        associated_token_info.clone(),
        Create {
            payer: pair_authority_info.clone(),
            associated_token: base_token_account_info,
            authority: pair_info.clone(),
            mint: base_token_mint_info,
            system_program: system_program_info.clone(),
            token_program: token_program_info.clone(),
        },
    ))?;

    // create the lst fee token account
    create_idempotent(CpiContext::new(
        associated_token_info,
        Create {
            payer: pair_authority_info,
            associated_token: lst_fee_token_account_info,
            authority: pair_info,
            mint: lst_mint_info,
            system_program: system_program_info,
            token_program: token_program_info,
        },
    ))?;

    Ok(())
}

pub fn handler(ctx: Context<CreatePair>, symbol: &str, params: &CreatePairParams) -> Result<()> {
    ctx.accounts.access_control.verify_unsealed()?;

    // Validate symbol
    require!(validate_symbol(&symbol), StakingError::InvalidSymbol);

    // Validate params
    params.validate()?;

    // Initialize fee parameters
    require!(
        params.stake_fee_bps <= 2500,
        StakingError::InvalidFeePercentage
    );
    require!(
        params.swap_fee_bps <= 5000, // raised from 2500 (2026-10-07), see pair.rs calculate_fee
        StakingError::InvalidFeePercentage
    );
    require!(
        params.withdraw_fee_bps <= 2500,
        StakingError::InvalidFeePercentage
    );

    create_token_accounts(&ctx)?;

    let pair = &mut ctx.accounts.pair;
    // Initialize pair fields
    pair.pair_bump = ctx.bumps.pair;
    pair.lst_mint_bump = ctx.bumps.lst_mint;
    pair.base_token_mint = ctx.accounts.base_token_mint.key();
    pair.base_mint_decimals = ctx.accounts.base_token_mint.decimals;
    pair.lst_mint = ctx.accounts.lst_mint.key();
    pair.lst_mint_decimals = ctx.accounts.lst_mint.decimals;
    pair.lst_symbol = symbol.to_string();

    // Set yield parameters from params
    pair.interval_apr_rate = params.interval_apr_rate;
    pair.seconds_per_interval = params.seconds_per_interval;
    pair.initial_exchange_rate = params.initial_exchange_rate;
    pair.last_yield_change_exchange_rate = params.initial_exchange_rate;
    pair.last_yield_change_timestamp = Clock::get()?.unix_timestamp;
    pair.deposit_cap = params.deposit_cap;
    pair.minimum_deposit = params.minimum_deposit;
    pair.stake_fee_bps = params.stake_fee_bps;
    pair.swap_fee_bps = params.swap_fee_bps;
    pair.withdraw_fee_bps = params.withdraw_fee_bps;

    Ok(())
}
