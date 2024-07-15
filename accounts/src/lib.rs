use anchor_lang::prelude::*;

pub mod calculator;
pub mod instructions;
pub mod quote;
pub mod utils;
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{transfer_fee::{TransferFee, TransferFeeConfig}, BaseStateWithExtensions, StateWithExtensions},
    },
};
use instructions::*;
use protocol_account_config;

declare_id!("HEAVEnMX7RoaYCucpyFterLWzFJR8Ah26oNSnqBs5Jtn");

#[program]
pub mod heaven_anchor_amm {
    use super::*;

    pub fn swap_in(ctx: Context<SwapInAccounts>, params: SwapInParams) -> Result<()> {
        AmmInstructions::<Heaven>::swap_in(ctx, params)
    }

    pub fn swap_out(ctx: Context<SwapOutAccounts>, params: SwapOutParams) -> Result<()> {
        AmmInstructions::<Heaven>::swap_out(ctx, params)
    }
}

pub fn get_transfer_fee_config(mint_info: &(Vec<u8>, Pubkey), epoch: u64) -> Result<TransferFee> {
    if mint_info.1 == Token::id() {
        return Ok(TransferFee::default());
    }

    let mint_data = &mint_info.0;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

    let transfer_fee_config = mint.get_extension::<TransferFeeConfig>()?;
    Ok(*transfer_fee_config.get_epoch_fee(epoch))
}
