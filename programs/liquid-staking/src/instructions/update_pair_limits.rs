use crate::{
    error::StakingError,
    state::{AccessControl, Pair},
    types::AuthorityType,
};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct UpdatePairLimitsParams {
    pub deposit_cap: Option<u64>,
    pub minimum_deposit: Option<u64>,
    pub stake_fee_bps: Option<u16>,
    pub swap_fee_bps: Option<u16>,
    pub withdraw_fee_bps: Option<u16>,
}

#[derive(Accounts)]
pub struct UpdatePairLimits<'info> {
    #[account(
        mut,
        seeds = [
            b"pair",
            pair.base_token_mint.key().as_ref(),
            pair.lst_mint.key().as_ref()
        ],
        bump = pair.pair_bump
    )]
    pub pair: Box<Account<'info, Pair>>,

    #[account(address = access_control.pair_authority)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"access_control"],
        bump = access_control.bump
    )]
    pub access_control: Box<Account<'info, AccessControl>>,
}

pub fn handler(ctx: Context<UpdatePairLimits>, params: UpdatePairLimitsParams) -> Result<()> {
    ctx.accounts.access_control.verify_unsealed()?;

    let pair = &mut ctx.accounts.pair;

    // Update deposit cap if provided
    if let Some(deposit_cap) = params.deposit_cap {
        pair.deposit_cap = deposit_cap;
    }

    // Update minimum deposit if provided
    if let Some(minimum_deposit) = params.minimum_deposit {
        pair.minimum_deposit = minimum_deposit;
    }

    // Update fees if provided
    if let Some(stake_fee_bps) = params.stake_fee_bps {
        require!(stake_fee_bps <= 2500, StakingError::InvalidFeePercentage);
        pair.stake_fee_bps = stake_fee_bps;
    }

    if let Some(swap_fee_bps) = params.swap_fee_bps {
        // swap fee cap raised to 5000 bps (2026-10-07): pikSOL -> exSOL at 50%
        require!(swap_fee_bps <= 5000, StakingError::InvalidFeePercentage);
        pair.swap_fee_bps = swap_fee_bps;
    }

    if let Some(withdraw_fee_bps) = params.withdraw_fee_bps {
        require!(withdraw_fee_bps <= 2500, StakingError::InvalidFeePercentage);
        pair.withdraw_fee_bps = withdraw_fee_bps;
    }

    Ok(())
}
