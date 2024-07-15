use anchor_lang::{
    accounts::interface_account::InterfaceAccount, prelude::*, solana_program::clock::Epoch,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    calculator::taxation_mode::TaxationMode,
    protocol_account_config, seeds,
    utils::{epoch::EpochUtils, error::AmmErrorCode},
    AmmInstructions, Heaven,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct CreateLiquidityPoolParams {
    pub input_base_token_amount: u64,
    pub expected_base_token_balance_after_transfer_fee: u64,
    pub input_quote_token_amount: u64,
    pub expected_quote_token_balance_after_transfer_fee: u64,
    pub open_at: u64,
    pub lock_liquidity_provider_token_until: u64,
    pub buy_tax: u64,
    pub sell_tax: u64,
    pub encoded_user_defined_event_data: String,
    pub burn_lp_tokens: bool,
    pub disable_non_creator_add_liquidity: bool,
}

impl AmmInstructions<Heaven> {
    pub fn create_liquidity_pool<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreateLiquidityPoolAccounts<'info>>,
        protocol_config_version: u16,
        params: CreateLiquidityPoolParams,
    ) -> Result<()> {
        Ok(())
    }
}

#[event]
struct CreateLiquidityPoolEvent {
    #[index]
    pub liquidity_pool_id: Pubkey,
    pub user: Pubkey,
    pub base_token_input_transfer_fee_amount: u64,
    pub quote_token_input_transfer_fee_amount: u64,
    pub base_token_input_amount: u64,
    pub quote_token_input_amount: u64,
    pub lp_token_output_amount: u64,
    pub locked_lp: bool,
}

#[derive(Accounts)]
#[instruction(protocol_config_version: u16)]
pub struct CreateLiquidityPoolAccounts<'info> {
    pub token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// create pool fee account
    #[account(
            mut,
            address= protocol_account_config::pool_creation_fee_wallet::id(),
        )]
    /// CHECK: this is a normal wallet
    pub pool_creation_fee_wallet: UncheckedAccount<'info>,

    #[account(
            seeds = [
                seeds::AUTHORITY.as_bytes(),
            ],
            bump,
        )]
    /// CHECK: This is a pda
    pub authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
            mint::token_program = base_token_program,
        )]
    pub base_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
            mint::token_program = quote_token_program,
        )]
    pub quote_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        seeds = [
            seeds::LIQUIDITY_PROVIDER_TOKEN_MINT.as_bytes(),
            liquidity_pool_state.key().as_ref(),
        ],
        bump,
        mint::decimals = if base_token_mint.decimals >= quote_token_mint.decimals{
            base_token_mint.decimals
        }else{
            quote_token_mint.decimals
        },
        mint::authority = authority,
        payer = user,
        mint::token_program = token_program,
    )]
    pub lp_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
            mut,
            token::mint = base_token_mint,
            token::authority = user,
        )]
    pub user_base_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
            mut,
            token::mint = quote_token_mint,
            token::authority = user,
        )]
    pub user_quote_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // #[account(
    //         init,
    //         associated_token::mint = liquidity_provider_token_mint,
    //         associated_token::authority = user,
    //         payer = user,
    //         token::token_program = token_program,
    //     )]
    // pub user_liquidity_provider_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
            mut,
            seeds = [
                seeds::LIQUIDITY_POOL_TOKEN_VAULT.as_bytes(),
                liquidity_pool_state.key().as_ref(),
                base_token_mint.key().as_ref()
            ],
            bump,
        )]
    /// CHECK
    pub base_token_vault: UncheckedAccount<'info>,

    #[account(
            mut,
            seeds = [
                seeds::LIQUIDITY_POOL_TOKEN_VAULT.as_bytes(),
                liquidity_pool_state.key().as_ref(),
                quote_token_mint.key().as_ref()
            ],
            bump,
        )]
    /// CHECK
    pub quote_token_vault: UncheckedAccount<'info>,

    #[account(
            mut,
            seeds = [
                seeds::LIQUIDITY_POOL_SWAP_TAX_TOKEN_VAULT.as_bytes(),
                liquidity_pool_state.key().as_ref(),
                base_token_mint.key().as_ref()
            ],
            bump,
        )]
    /// CHECK
    pub base_token_swap_tax_vault: UncheckedAccount<'info>,

    #[account(
            mut,
            seeds = [
                seeds::LIQUIDITY_POOL_SWAP_TAX_TOKEN_VAULT.as_bytes(),
                liquidity_pool_state.key().as_ref(),
                quote_token_mint.key().as_ref()
            ],
            bump,
        )]
    /// CHECK
    pub quote_token_swap_tax_vault: UncheckedAccount<'info>,

    #[account(
        init,
        seeds = [
            seeds::LIQUIDITY_POOL_STATE.as_bytes(),
            user.key().as_ref(),
            base_token_mint.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        bump,
        payer = user,
        space = std::mem::size_of::<LiquidityPoolState>() + 8
    )]
    pub liquidity_pool_state: AccountLoader<'info, LiquidityPoolState>,

    #[account(mut)]
    /// CHECK: needs to be checked in the code
    pub lp_token_lock_vault: UncheckedAccount<'info>,

    #[account(
        seeds = [
            seeds::PROTOCOL_CONFIG_STATE.as_bytes(),
            &protocol_config_version.to_be_bytes()
        ],
        bump,
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,
}

#[account]
#[derive(Default, Debug)]
pub struct LiquidityPoolUserStats {
    // Number of swaps
    pub base_swap_in_count: u64,
    pub quote_swap_in_count: u64,
    pub base_swap_out_count: u64,
    pub quote_swap_out_count: u64,

    // Buy/Sell Volume
    pub base_in_amount: u128,
    pub base_out_amount: u128,
    pub quote_in_amount: u128,
    pub quote_out_amount: u128,

    // Timestamp
    pub first_swap_out_timestamp: u64,
    pub latest_swap_out_timestamp: u64,
    pub first_swap_in_timestamp: u64,
    pub latest_swap_in_timestamp: u64,

    // Liquidity
    pub base_token_added: u128,
    pub base_token_removed: u128,
    pub quote_token_added: u128,
    pub quote_token_removed: u128,
    pub add_liquidity_count: u64,
    pub remove_liquidity_count: u64,
    pub initial_base_token_added: u64,
    pub initial_quote_token_added: u64,
    pub initial_liquidity_added_timestamp: u64,

    // Protocol swap fee paid
    pub base_protocol_swap_fee_paid: u128,
    pub quote_protocol_swap_fee_paid: u128,

    // Pool swap fee paid
    pub base_pool_swap_fee_paid: u128,
    pub quote_pool_swap_fee_paid: u128,

    // Pool swap tax paid
    pub base_pool_swap_tax_paid: u128,
    pub quote_pool_swap_tax_paid: u128,

    // Protocol tax on taxation paid
    pub base_protocol_tax_paid: u128,
    pub quote_protocol_tax_paid: u128,
}

#[account]
#[derive(Default, Debug)]
pub struct GlobalUserStats {
    // Buy/Sell Volume
    pub sell_volume_wsol: u128,
    pub sell_volume_usdc: u128,
    pub sell_volume_usdt: u128,
    pub buy_volume_wsol: u128,
    pub buy_volume_usdc: u128,
    pub buy_volume_usdt: u128,

    // Timestamp of first and latest swap
    pub first_sell_timestamp: u64,
    pub latest_sell_timestamp: u64,
    pub first_buy_timestamp: u64,
    pub latest_buy_timestamp: u64,

    // Amount of first/last sell/buy
    pub first_sell_amount_wsol: u128,
    pub first_sell_amount_usdc: u128,
    pub first_sell_amount_usdt: u128,
    pub first_buy_amount_wsol: u128,
    pub first_buy_amount_usdc: u128,
    pub first_buy_amount_usdt: u128,

    // Number of Buy/Sell
    pub sell_count: u64,
    pub buy_count: u64,

    // Liquidity
    pub liquidity_added_wsol: u128,
    pub liquidity_added_usdc: u128,
    pub liquidity_added_usdt: u128,
    pub liquidity_removed_wsol: u128,
    pub liquidity_removed_usdc: u128,
    pub liquidity_removed_usdt: u128,

    pub add_liquidity_count: u64,
    pub remove_liquidity_count: u64,

    // Protocol swap fee paid
    pub protocol_swap_fee_paid_wsol: u128,
    pub protocol_swap_fee_paid_usdc: u128,
    pub protocol_swap_fee_paid_usdt: u128,

    // Pool swap fee paid
    pub pool_swap_fee_paid_wsol: u128,
    pub pool_swap_fee_paid_usdc: u128,
    pub pool_swap_fee_paid_usdt: u128,

    // Pool swap tax paid
    pub pool_swap_tax_paid_wsol: u128,
    pub pool_swap_tax_paid_usdc: u128,
    pub pool_swap_tax_paid_usdt: u128,

    // Pool swap tax paid
    pub protocol_swap_tax_paid_wsol: u128,
    pub protocol_swap_tax_paid_usdc: u128,
    pub protocol_swap_tax_paid_usdt: u128,
}

#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Default, Debug)]
pub struct LiquidityPoolState {
    pub base_token_mint: Pubkey,
    pub base_token_mint_decimals: u8,
    pub base_token_vault: Pubkey,
    pub base_token_swap_tax_vault: Pubkey,

    pub quote_token_mint: Pubkey,
    pub quote_token_mint_decimals: u8,
    pub quote_token_vault: Pubkey,
    pub quote_token_swap_tax_vault: Pubkey,

    pub protocol_base_token_swap_fee_vault: Pubkey,
    pub protocol_quote_token_swap_fee_vault: Pubkey,

    pub lp_token_mint: Pubkey,
    pub lp_token_mint_decimals: u8,
    pub lp_token_current_supply: u64,
    // swap fee is intended to left in the pool to be claimed by the liquidity providers
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    // protocol swap fee is intended to be sent to the protocol fee account
    pub protocol_swap_fee_numerator: u64,
    pub protocol_swap_fee_denominator: u64,
    // swap tax is user-defined and can be claimed by the liquidity pool creator
    pub buy_tax: u64,
    pub sell_tax: u64,

    // protocol tax is intended to be sent to the protocol fee account
    pub protocol_tax_numerator: u64,
    pub protocol_tax_denominator: u64,

    pub creator: Pubkey,
    pub authority_bump: u8,
    pub allow_swap: bool,
    pub allow_remove_liquidity: bool,
    pub allow_add_liquidity: bool,
    pub open_at: u64,
    pub created_at: u64,
    pub lock_until: u64,
    pub protocol_config_version: u16,
    pub taxation_mode: TaxationMode,

    // Buy/Sell Volume base on pair
    pub swap_base_in_amount: u128,
    pub swap_quote_in_amount: u128,
    pub swap_base_out_amount: u128,
    pub swap_quote_out_amount: u128,

    // Swap fees that is left in the pool
    pub swap_base_fee: u128,
    pub swap_quote_fee: u128,

    // Number of Buy/Sell
    pub swap_base_to_quote_count: u128,
    pub swap_quote_to_base_count: u128,

    // Number of Swap in/out
    pub swap_in_count: u128,
    pub swap_out_count: u128,

    // Tax fees that is transferred out to the tax wallet
    pub base_swap_tax_amount: u128,
    pub quote_swap_tax_amount: u128,

    // Timestamp
    pub first_swap_out_timestamp: u64,
    pub latest_swap_out_timestamp: u64,
    pub first_swap_in_timestamp: u64,
    pub latest_swap_in_timestamp: u64,

    pub first_base_to_quote_amount: u128,
    pub latest_base_to_quote_amount: u128,
    pub first_quote_to_base_amount: u128,
    pub latest_quote_to_base_amount: u128,

    // Liquidity
    pub locked_lp: u64,
    pub initial_lp: u64,
    pub liquidity_added: u128,
    pub liquidity_removed: u128,
    pub is_initial_lp_burned: bool,

    // Tokens
    pub base_token_added: u128,
    pub base_token_removed: u128,
    pub quote_token_added: u128,
    pub quote_token_removed: u128,

    // Protocol tax
    pub base_protocol_tax: u128,
    pub quote_protocol_tax: u128,

    // Protocol fee
    pub base_protocol_fee: u128,
    pub quote_protocol_fee: u128,

    // Vault balances
    pub base_token_vault_balance: u64,
    pub quote_token_vault_balance: u64,

    // Prices
    #[cfg(feature = "pool-price-stats")]
    pub min_price: f64,
    #[cfg(feature = "pool-price-stats")]
    pub max_price: f64,
    #[cfg(feature = "pool-price-stats")]
    pub curr_price: f64,
    #[cfg(feature = "pool-price-stats")]
    pub curr_mc: f64,
    #[cfg(feature = "pool-price-stats")]
    pub min_mc: f64,
    #[cfg(feature = "pool-price-stats")]
    pub max_mc: f64,
    pub locked_taxation: bool,
    pub disable_non_creator_add_liquidity: bool,
    pub allow_creator_claim_swap_fee: bool,
    pub extras: [u8; 8],
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

impl LiquidityPoolState {
    pub fn is_locked(&self) -> bool {
        self.lock_until > Epoch::current_epoch()
    }

    pub fn checked_is_open(&self) -> Result<()> {
        if self.open_at > Epoch::current_epoch() {
            return Err(AmmErrorCode::LiquidityPoolIsNotOpenYet.into());
        }
        Ok(())
    }

    pub fn checked_allow_add_liquidity(&self) -> Result<()> {
        if !self.allow_add_liquidity {
            return Err(AmmErrorCode::AddLiquidityDisabled.into());
        }
        Ok(())
    }

    pub fn checked_allow_remove_liquidity(&self) -> Result<()> {
        if !self.allow_remove_liquidity {
            return Err(AmmErrorCode::RemoveLiquidityDisabled.into());
        }
        Ok(())
    }

    pub fn checked_allow_swap(&self) -> Result<()> {
        if !self.allow_swap {
            return Err(AmmErrorCode::SwapDisabled.into());
        }
        Ok(())
    }
}

#[account]
#[derive(Default, Debug)]
pub struct ProtocolConfig {
    pub protocol_config_state_bump: u8,
    pub allow_create_pool: bool,
    pub protocol_swap_fee_numerator: u64,
    pub protocol_swap_fee_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub protocol_tax_numerator: u64,
    pub protocol_tax_denominator: u64,
    pub create_pool_fee: u64,
    pub protocol_owner: Pubkey,
    pub version: u16,
}

#[cfg(test)]
mod test {
    use crate::calculator::number::U128;

    #[test]
    fn test_create_liquidity_pool() {
        let base_decimals = 9u32;
        let quote_decimals = 6u32;
        let base_token_amount = 10u64.pow(base_decimals + 9);
        let quote_token_amount = 10u64.pow(quote_decimals + 3);
        let liquidity = U128::from(base_token_amount)
            .checked_mul((quote_token_amount).into())
            .unwrap()
            .integer_sqrt()
            .as_u64();
        let lock_lp_amount = (10u64).checked_pow(base_decimals).unwrap();
        println!(
            "liquidity:{}, lock_lp_amount:{}, base_token_amount:{},quote_token_amount:{}, lock_percentage:{}%",
            liquidity, lock_lp_amount, base_token_amount, quote_token_amount, (lock_lp_amount as f64) / (liquidity as f64) * 100.0
        );
    }
}

pub mod chainlink_program {
    use anchor_lang::prelude::declare_id;
    declare_id!("HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny");
}

pub mod chainlink_feed_account {
    use anchor_lang::prelude::declare_id;

    #[cfg(feature = "devnet")]
    declare_id!("99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR");

    // not devnet
    #[cfg(not(feature = "devnet"))]
    declare_id!("CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");
}

pub mod chainlink_feed_account_devnet {
    use anchor_lang::prelude::declare_id;

    declare_id!("99B2bTijsU6f1GCT73HmdR7HCFFjGMBcPZY6jZ96ynrR");
}

pub mod chainlink_feed_account_mainnet {
    use anchor_lang::prelude::declare_id;

    declare_id!("CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");
}

pub fn get_chainlink_feed_account(is_devnet: bool) -> Pubkey {
    if is_devnet {
        chainlink_feed_account_devnet::ID
    } else {
        chainlink_feed_account_mainnet::ID
    }
}
