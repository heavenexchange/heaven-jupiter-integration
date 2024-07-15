
use anchor_lang::{
    accounts::interface_account::InterfaceAccount, prelude::*,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    calculator::swap_direction::SwapDirection, seeds, AmmInstructions, GlobalUserStats, Heaven, LiquidityPoolState, LiquidityPoolUserStats
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct SwapInParams {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
    pub swap_direction: SwapDirection,
    pub encoded_user_defined_event_data: String,
}

impl AmmInstructions<Heaven> {
    pub fn swap_in(ctx: Context<SwapInAccounts>, params: SwapInParams) -> Result<()> {
        Ok(())
    }
}

#[event]
pub struct SwapInEvent {
    #[index]
    pub liquidity_pool_id: Pubkey,
    pub user: Pubkey,
    pub swap_direction: SwapDirection,
    pub swap_amount_in: u64,
    pub swap_amount_out: u64,
}

#[derive(Accounts)]
pub struct SwapInAccounts<'info> {
    pub token_program: Program<'info, Token>,
    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub liquidity_pool_state: AccountLoader<'info, LiquidityPoolState>,

    /// create pool fee account
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
        address = liquidity_pool_state.load()?.base_token_mint
    )]
    pub base_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        address = liquidity_pool_state.load()?.quote_token_mint
    )]
    pub quote_token_mint: Box<InterfaceAccount<'info, Mint>>,

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

    #[account(
        mut,
        address = liquidity_pool_state.load()?.base_token_vault
    )]
    /// CHECK
    pub base_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        address = liquidity_pool_state.load()?.quote_token_vault
    )]
    /// CHECK
    pub quote_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        address = liquidity_pool_state.load()?.base_token_swap_tax_vault
    )]
    /// CHECK
    pub base_token_swap_tax_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        address = liquidity_pool_state.load()?.quote_token_swap_tax_vault
    )]
    /// CHECK
    pub quote_token_swap_tax_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
        address = liquidity_pool_state.load()?.protocol_base_token_swap_fee_vault
    )]
    pub protocol_base_token_swap_fee_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, 
        address = liquidity_pool_state.load()?.protocol_quote_token_swap_fee_vault
    )]
    pub protocol_quote_token_swap_fee_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        seeds = [
            seeds::USER_AMM_STATS.as_bytes(),
            user.key().as_ref(),
            liquidity_pool_state.key().as_ref(),
        ],
        bump,
        payer = user,
        space = std::mem::size_of::<LiquidityPoolUserStats>() + 8
    )]
    pub user_amm_stats: Box<Account<'info, LiquidityPoolUserStats>>,

    #[account(
        init_if_needed,
        seeds = [
            seeds::USER_GLOBAL_STATS.as_bytes(),
            user.key().as_ref(),
        ],
        bump,
        payer = user,
        space = std::mem::size_of::<GlobalUserStats>() + 8
    )]
    pub user_global_stats: Box<Account<'info, GlobalUserStats>>,
}

#[cfg(test)]
mod test {}
