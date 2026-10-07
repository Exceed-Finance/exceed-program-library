use crate::error::StakingError;
use crate::types::ConversionDirection;
use crate::u64x64_math::{from_decimal, pow, to_decimal};
use anchor_lang::prelude::*;
use std::convert::TryInto;

#[account]
pub struct Pair {
    // Bumps
    pub pair_bump: u8,
    pub lst_mint_bump: u8,

    // Tokens
    pub base_token_mint: Pubkey,
    pub base_mint_decimals: u8,
    pub lst_mint: Pubkey,
    pub lst_mint_decimals: u8,
    pub lst_symbol: String,

    // Yield
    pub interval_apr_rate: u64,
    pub seconds_per_interval: i32,

    pub initial_exchange_rate: u64,
    pub last_yield_change_exchange_rate: u64,
    pub last_yield_change_timestamp: i64,

    // The max sum of deposits this pair will accept.
    pub deposit_cap: u64,
    pub minimum_deposit: u64,

    // Fees (in basis points)
    pub stake_fee_bps: u16,
    pub swap_fee_bps: u16,
    pub withdraw_fee_bps: u16,
}

pub const PRECISION: u128 = 1_000_000_000_000;

impl Pair {
    pub fn calculate_exchange_rate(&mut self, current_timestamp: i64) -> Option<u64> {
        if current_timestamp == self.last_yield_change_timestamp {
            return Some(self.last_yield_change_exchange_rate);
        }

        // Prevent timestamp manipulation by ensuring current_timestamp is greater than last_yield_change_timestamp
        if current_timestamp < self.last_yield_change_timestamp {
            return None;
        }

        let elapsed_time = current_timestamp.checked_sub(self.last_yield_change_timestamp)?;
        msg!("Elapsed time: {}", elapsed_time);

        // Convert i32 to i64 (infallible, so direct cast is fine)
        let seconds_per_interval_i64 = self.seconds_per_interval as i64;

        let interval_amounts = elapsed_time.checked_div(seconds_per_interval_i64)?;
        let remaining_seconds = elapsed_time.checked_rem(seconds_per_interval_i64)?;
        msg!("intervals: {}", interval_amounts);
        msg!("Remaining seconds: {}", remaining_seconds);

        // Convert u64 to u128 (infallible, so direct cast is fine)
        let interval_rate = self.interval_apr_rate as u128;
        msg!("Interval Rate: {}", interval_rate);

        // Convert interval_rate to fixed-point for the pow function
        let interval_rate_fp = from_decimal(interval_rate)?;

        // Use the pow function to calculate the compounded rate
        // Convert i64 to i32 with checked conversion
        let interval_amounts_i32 = i32::try_from(interval_amounts).ok()?;
        let compounded_rate_fp = pow(interval_rate_fp, interval_amounts_i32)?;

        // Convert back to decimal
        let compounded_rate = to_decimal(compounded_rate_fp)?;
        msg!("Compounded rate: {}", compounded_rate);

        // Calculate the linear yield for the remaining seconds
        // First subtract PRECISION to get just the yield portion
        let yield_portion = interval_rate.checked_sub(PRECISION)?;

        // Convert i64 to u128 with checked conversion
        let remaining_seconds_u128 = u128::try_from(remaining_seconds).ok()?;
        // Convert i32 to u128 with checked conversion
        let seconds_per_interval_u128 = u128::try_from(self.seconds_per_interval).ok()?;

        let linear_yield = yield_portion
            .checked_mul(remaining_seconds_u128)?
            .checked_div(seconds_per_interval_u128)?;
        msg!("Linear yield: {}", linear_yield);

        // Add the linear yield to the compounded rate
        let total_rate = compounded_rate.checked_add(linear_yield)?;
        msg!("Total rate: {}", total_rate);

        // Multiply the current exchange rate with the total rate
        // Convert u64 to u128 (infallible, so direct cast is fine)
        let last_yield_change_exchange_rate_u128 = self.last_yield_change_exchange_rate as u128;

        let new_exchange_rate = last_yield_change_exchange_rate_u128
            .checked_mul(total_rate)?
            .checked_div(PRECISION)?;

        // Scale the exchange rate appropriately
        // The exchange rate should be in the millions range (10^6)
        msg!("New exchange rate: {}", new_exchange_rate);

        // Use checked conversion instead of unchecked cast
        let new_exchange_rate_u64 = new_exchange_rate.try_into().ok()?;

        Some(new_exchange_rate_u64)
    }

    pub fn calculate_output_amount(
        &mut self,
        amount: u64,
        current_timestamp: i64,
        direction: ConversionDirection,
    ) -> Result<u64> {
        let (from_decimals, to_decimals) = match direction {
            ConversionDirection::BaseToLst => (self.base_mint_decimals, self.lst_mint_decimals),
            ConversionDirection::LstToBase => (self.lst_mint_decimals, self.base_mint_decimals),
        };

        let exchange_rate = self
            .calculate_exchange_rate(current_timestamp)
            .ok_or(StakingError::CalculationOverflow)?;

        // No need to normalize here - we'll handle decimal conversion directly in the calculations
        msg!("Input amount: {}", amount);
        msg!("Exchange rate: {}", exchange_rate);
        msg!(
            "From decimals: {}, To decimals: {}",
            from_decimals,
            to_decimals
        );

        // Calculate the output amount based on the direction of conversion
        let output_amount = match direction {
            ConversionDirection::BaseToLst => {
                // Converting from base_token to lst
                // Formula: (amount * PRECISION) / exchange_rate
                // Then adjust for decimal difference: * 10^(to_decimals - from_decimals)

                // First calculate the conversion based on exchange rate
                // Convert u64 to u128 (infallible, so direct cast is fine)
                let amount_u128 = amount as u128;
                // Convert u64 to u128 (infallible, so direct cast is fine)
                let exchange_rate_u128 = exchange_rate as u128;

                let base_conversion = amount_u128
                    .checked_mul(PRECISION)
                    .ok_or(StakingError::CalculationOverflow)?
                    .checked_div(exchange_rate_u128)
                    .ok_or(StakingError::CalculationOverflow)?;

                // Then adjust for decimal places difference
                if to_decimals > from_decimals {
                    // LST has more decimals than base, multiply by 10^difference
                    let decimal_adjustment = to_decimals
                        .checked_sub(from_decimals)
                        .ok_or(StakingError::CalculationOverflow)?;

                    base_conversion
                        .checked_mul(10u128.pow(decimal_adjustment.into()))
                        .ok_or(StakingError::CalculationOverflow)?
                } else if from_decimals > to_decimals {
                    // LST has fewer decimals than base, divide by 10^difference
                    let decimal_adjustment = from_decimals
                        .checked_sub(to_decimals)
                        .ok_or(StakingError::CalculationOverflow)?;

                    base_conversion
                        .checked_div(10u128.pow(decimal_adjustment.into()))
                        .ok_or(StakingError::CalculationOverflow)?
                } else {
                    // Same number of decimals, no adjustment needed
                    base_conversion
                }
            }
            ConversionDirection::LstToBase => {
                // Converting from lst to base_token
                // Formula: (amount * exchange_rate) / PRECISION
                // Then adjust for decimal difference: * 10^(to_decimals - from_decimals)

                // First calculate the conversion based on exchange rate
                // Convert u64 to u128 (infallible, so direct cast is fine)
                let amount_u128 = amount as u128;
                // Convert u64 to u128 (infallible, so direct cast is fine)
                let exchange_rate_u128 = exchange_rate as u128;

                let lst_conversion = amount_u128
                    .checked_mul(exchange_rate_u128)
                    .ok_or(StakingError::CalculationOverflow)?
                    .checked_div(PRECISION)
                    .ok_or(StakingError::CalculationOverflow)?;

                // Then adjust for decimal places difference
                if to_decimals > from_decimals {
                    // Base has more decimals than LST, multiply by 10^difference
                    let decimal_adjustment = to_decimals
                        .checked_sub(from_decimals)
                        .ok_or(StakingError::CalculationOverflow)?;

                    lst_conversion
                        .checked_mul(10u128.pow(decimal_adjustment.into()))
                        .ok_or(StakingError::CalculationOverflow)?
                } else if from_decimals > to_decimals {
                    // Base has fewer decimals than LST, divide by 10^difference
                    let decimal_adjustment = from_decimals
                        .checked_sub(to_decimals)
                        .ok_or(StakingError::CalculationOverflow)?;

                    lst_conversion
                        .checked_div(10u128.pow(decimal_adjustment.into()))
                        .ok_or(StakingError::CalculationOverflow)?
                } else {
                    // Same number of decimals, no adjustment needed
                    lst_conversion
                }
            }
        }
        .try_into()
        .map_err(|_| StakingError::CalculationOverflow)?;

        msg!("Output amount: {}", output_amount);

        Ok(output_amount)
    }

    pub fn check_excessive_deposit(&self, quote_amount: u64, base_token_amount: u64) -> Result<()> {
        // Convert u64 to u128 (infallible, so direct cast is fine)
        let quote_amount_u128 = quote_amount as u128;
        let base_token_amount_u128 = base_token_amount as u128;

        let new_base_token_amount = base_token_amount_u128
            .checked_add(quote_amount_u128)
            .ok_or(StakingError::CalculationOverflow)?;

        msg!("quote_amount: {}", quote_amount);
        msg!("base_token_amount: {}", base_token_amount);
        msg!("deposit_cap: {}", self.deposit_cap);

        // Convert u64 to u128 (infallible, so direct cast is fine)
        let deposit_cap_u128 = self.deposit_cap as u128;

        if new_base_token_amount > deposit_cap_u128 {
            return err!(StakingError::DepositCapExceeded);
        }

        Ok(())
    }

    pub fn check_minimum_deposit(&self, amount: u64) -> Result<()> {
        if amount >= self.minimum_deposit.into() {
            return Ok(());
        }
        return Err(StakingError::InvalidQuantity.into());
    }

    pub fn calculate_fee(&self, amount: u64, fee_bps: u16) -> Result<u64> {
        // Cap raised 2500 -> 5000 bps (2026-10-07) so a 50% swap haircut can be
        // expressed on one side (pikSOL -> exSOL conversion). Stake/withdraw fees
        // keep their own 2500 bps caps in the instruction handlers.
        if fee_bps > 5000 {
            return Err(StakingError::InvalidFeePercentage.into());
        }

        // Convert u64 to u128 (infallible, so direct cast is fine)
        let amount_u128 = amount as u128;
        // Convert u16 to u128 (infallible, so direct cast is fine)
        let fee_bps_u128 = fee_bps as u128;

        let fee_amount = amount_u128
            .checked_mul(fee_bps_u128)
            .ok_or(StakingError::CalculationOverflow)?
            .checked_div(10000)
            .ok_or(StakingError::CalculationOverflow)?;

        // Use checked conversion instead of unchecked cast
        let fee_amount_u64 = fee_amount
            .try_into()
            .map_err(|_| StakingError::CalculationOverflow)?;

        Ok(fee_amount_u64)
    }

    pub fn calculate_stake_fee_lst(&self, lst_amount: u64) -> Result<u64> {
        self.calculate_fee(lst_amount, self.stake_fee_bps)
    }

    pub fn calculate_swap_fee_lst(&self, lst_amount: u64) -> Result<u64> {
        self.calculate_fee(lst_amount, self.swap_fee_bps)
    }

    pub fn calculate_withdraw_fee_lst(&self, lst_amount: u64) -> Result<u64> {
        self.calculate_fee(lst_amount, self.withdraw_fee_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_pair() -> Pair {
        Pair {
            pair_bump: 255,
            lst_mint_bump: 255,
            base_token_mint: Pubkey::default(),
            lst_mint: Pubkey::default(),
            base_mint_decimals: 6,
            lst_mint_decimals: 6,
            lst_symbol: String::from("TEST"),
            interval_apr_rate: 1000166517567, // Interval APR rate without considering zeros
            seconds_per_interval: 8 * 60 * 60, // 8 hours
            initial_exchange_rate: 1_000_000,
            last_yield_change_exchange_rate: 1_000_000,
            last_yield_change_timestamp: 0,
            deposit_cap: 500_000,
            minimum_deposit: 0,
            stake_fee_bps: 0,
            swap_fee_bps: 0,
            withdraw_fee_bps: 0,
        }
    }

    #[test]
    fn test_calculate_exchange_rate() {
        let mut pair = default_pair();

        // Test case where current_timestamp equals last_yield_change_timestamp
        pair.last_yield_change_timestamp = 1_000_000;
        pair.last_yield_change_exchange_rate = 1_000_000;
        let result = pair.calculate_exchange_rate(1_000_000).unwrap();
        assert_eq!(result, 1_000_000);

        // Test case where one year has passed
        pair.last_yield_change_timestamp = 0;
        let current_timestamp = 31_536_000; // One year in seconds
        let result = pair.calculate_exchange_rate(current_timestamp).unwrap();
        println!("One year has passed: {}", result);
        assert!(result > 1_000_000); // Exchange rate should increase

        // Test case where half a year has passed
        pair.last_yield_change_timestamp = 0;
        let result = pair.calculate_exchange_rate(current_timestamp / 2).unwrap();
        assert!(result > 1_000_000); // Exchange rate should increase but less than one year
        assert!(result < pair.calculate_exchange_rate(current_timestamp).unwrap());
        // Should be less than one year rate
    }

    #[test]
    fn test_calculate_output_amount() {
        let mut pair = default_pair();
        let current_timestamp = 31_536_000; // One year in seconds

        // Test conversion from base_token to lst
        let base_quantity = 1_000_000_000; // 1000 base tokens
        let lst_amount = pair
            .calculate_output_amount(
                base_quantity,
                current_timestamp,
                ConversionDirection::BaseToLst,
            )
            .unwrap();
        assert!(lst_amount > 0); // Should get some LST tokens back

        // Test conversion from lst to base_token
        let lst_quantity = 100_000_000; // 100 LST tokens
        let base_amount = pair
            .calculate_output_amount(
                lst_quantity,
                current_timestamp,
                ConversionDirection::LstToBase,
            )
            .unwrap();
        assert!(base_amount > lst_quantity); // Should get more base tokens due to yield
    }

    #[test]
    fn test_calculate_fee_cap_5000() {
        let pair = create_test_pair();
        // 50% is now a legal fee (one-sided pikSOL -> exSOL haircut)
        assert_eq!(pair.calculate_fee(1_000_000, 5000).unwrap(), 500_000);
        // the previous cap still works
        assert_eq!(pair.calculate_fee(1_000_000, 2500).unwrap(), 250_000);
        // one basis point above the new cap is rejected
        assert!(pair.calculate_fee(1_000_000, 5001).is_err());
        // zero fee unchanged
        assert_eq!(pair.calculate_fee(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn test_check_excessive_deposit() {
        let pair = default_pair();

        // Test case where deposit is within limit
        let result = pair.check_excessive_deposit(200_000, 200_000);
        assert!(result.is_ok());

        // Test case where deposit exceeds limit
        let result = pair.check_excessive_deposit(300_000, 300_000);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().to_string(), s if s.contains("Deposit cap exceeded")));
    }
}
