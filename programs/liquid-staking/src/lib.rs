#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use instructions::*;
use solana_security_txt::security_txt;
use types::AuthorityType;

pub mod error;
pub mod instructions;
pub mod state;
pub mod types;
pub mod u64x64_math;

declare_id!("GTmavJc15q4TowiFmdzFU7VFsfkfahxiMXT9xfS9eeCo"); // DEVNET fee-cap test build, never deploy to mainnet

security_txt! {
    name: "liquid_staking",
    project_url: "https://exceed.finance",
    contacts: "email:hi@exceed.finance",
    policy: "https://exceed.finance/security-policy",
    source_code: "https://github.com/Exceed-Finance/exceed-program-library",
    auditors: "https://quantstamp.com/"
}

#[program]
pub mod liquid_staking {

    use super::*;

    pub fn create_access_control(
        ctx: Context<CreateAccessControl>,
        vault_authority: Pubkey,
        window_authority: Pubkey,
        deposit_authority: Pubkey,
        pair_authority: Pubkey,
        unseal_authority: Pubkey,
        access_authority: Pubkey,
        nav_authority: Pubkey,
    ) -> Result<()> {
        instructions::create_access_control::handler(
            ctx,
            vault_authority,
            window_authority,
            deposit_authority,
            pair_authority,
            unseal_authority,
            access_authority,
            nav_authority,
        )
    }

    pub fn create_pair(
        ctx: Context<CreatePair>,
        symbol: String,
        params: CreatePairParams,
    ) -> Result<()> {
        instructions::create_pair::handler(ctx, &symbol, &params)
    }

    pub fn stake(
        ctx: Context<Stake>,
        quantity: u64,
        merkle_proof: Option<Vec<[u8; 32]>>,
    ) -> Result<()> {
        instructions::stake::handler(ctx, quantity, merkle_proof)
    }

    pub fn create_withdrawal_window(
        ctx: Context<CreateWithdrawalWindow>,
        params: CreateWithdrawalWindowParams,
    ) -> Result<()> {
        instructions::create_withdrawal_window::handler(ctx, params)
    }

    pub fn request_withdraw(
        ctx: Context<RequestWithdraw>,
        amount: u64,
        merkle_proof: Option<Vec<[u8; 32]>>,
    ) -> Result<()> {
        instructions::request_withdraw::handler(ctx, amount, merkle_proof)
    }

    pub fn execute_withdraw(ctx: Context<ExecuteWithdraw>) -> Result<()> {
        instructions::execute_withdraw::handler(ctx)
    }

    pub fn restake_expired_withdraw(ctx: Context<RestakeExpiredWithdraw>) -> Result<()> {
        instructions::restake_expired_withdrawal::handler(ctx)
    }

    pub fn fund_withdrawal_window(ctx: Context<FundWithdrawalWindow>) -> Result<()> {
        instructions::fund_withdrawal_window::handler(ctx)
    }

    pub fn vault_withdraw(ctx: Context<VaultWithdraw>, amount: u64) -> Result<()> {
        instructions::vault_withdraw::handler(ctx, amount)
    }

    pub fn update_pair_yield(
        ctx: Context<UpdatePairYield>,
        params: UpdatePairYieldParams,
    ) -> Result<()> {
        instructions::update_pair_yield::handler(ctx, params)
    }

    pub fn initiate_authority_transfer(
        ctx: Context<InitiateAuthorityTransfer>,
        authority_type: AuthorityType,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::initiate_authority_transfer::handler(ctx, authority_type, new_authority)
    }

    pub fn accept_authority_transfer(
        ctx: Context<AcceptAuthorityTransfer>,
        authority_type: AuthorityType,
    ) -> Result<()> {
        instructions::accept_authority_transfer::handler(ctx, authority_type)
    }

    pub fn cancel_authority_transfer(
        ctx: Context<CancelAuthorityTransfer>,
        authority_type: AuthorityType,
    ) -> Result<()> {
        instructions::cancel_authority_transfer::handler(ctx, authority_type)
    }

    pub fn close_withdrawal_window(ctx: Context<CloseWithdrawalWindow>) -> Result<()> {
        instructions::close_withdrawal_window::handler(ctx)
    }

    pub fn cancel_withdrawal_request(ctx: Context<CancelWithdrawalRequest>) -> Result<()> {
        instructions::cancel_withdrawal_request::handler(ctx)
    }

    pub fn swap_lst(
        ctx: Context<SwapLst>,
        quantity: u64,
        merkle_proof: Option<Vec<[u8; 32]>>,
    ) -> Result<()> {
        instructions::swap_lst::handler(ctx, quantity, merkle_proof)
    }

    pub fn seal_program(ctx: Context<SealProgram>) -> Result<()> {
        instructions::seal_program::handler(ctx)
    }

    pub fn unseal_program(ctx: Context<UnsealProgram>) -> Result<()> {
        instructions::unseal_program::handler(ctx)
    }

    pub fn manage_guardian(
        ctx: Context<ManageGuardian>,
        guardian: Pubkey,
        operation: GuardianOperation,
    ) -> Result<()> {
        instructions::manage_guardian::handler(ctx, guardian, operation)
    }

    pub fn update_pair_limits(
        ctx: Context<UpdatePairLimits>,
        params: UpdatePairLimitsParams,
    ) -> Result<()> {
        instructions::update_pair_limits::handler(ctx, params)
    }

    pub fn update_sol_usdc_feed(
        ctx: Context<UpdateSolUsdcFeed>,
        new_feed_id: [u8; 32],
    ) -> Result<()> {
        instructions::update_sol_usdc_feed::handler(ctx, new_feed_id)
    }

    pub fn update_whitelist(
        ctx: Context<UpdateWhitelist>,
        merkle_root: Option<[u8; 32]>,
        enable_whitelist: Option<bool>,
    ) -> Result<()> {
        instructions::update_whitelist::handler(ctx, merkle_root, enable_whitelist)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
        instructions::withdraw_fees::handler(ctx, amount)
    }

    pub fn create_token_metadata(
        ctx: Context<CreateTokenMetadata>,
        params: CreateTokenMetadataParams,
    ) -> Result<()> {
        instructions::create_token_metadata::handler(ctx, params)
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        instructions::migrate::handler(ctx)
    }

    pub fn migrate_pair_v2(ctx: Context<MigratePairV2>) -> Result<()> {
        instructions::migrate_v2::migrate_pair_handler(ctx)
    }

    pub fn migrate_access_control_v2(
        ctx: Context<MigrateAccessControlV2>,
        nav_authority: Pubkey,
    ) -> Result<()> {
        instructions::migrate_v2::migrate_access_control_handler(ctx, nav_authority)
    }

    pub fn update_nav(ctx: Context<UpdateNav>, total_equity: u64) -> Result<()> {
        instructions::update_nav::handler(ctx, total_equity)
    }

    pub fn update_pair_type(ctx: Context<UpdatePairType>, new_pair_type: u8) -> Result<()> {
        instructions::update_pair_type::handler(ctx, new_pair_type)
    }
}
