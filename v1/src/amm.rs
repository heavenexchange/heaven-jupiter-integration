use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anchor_lang::{prelude::Pubkey, pubkey, system_program::System, AccountDeserialize, Id};
use anchor_lang::{solana_program::instruction::AccountMeta, ToAccountMetas};
use anchor_spl::{
    associated_token::AssociatedToken, token::Token,
    token_2022::spl_token_2022::extension::transfer_fee::TransferFee,
};
use anyhow::{Ok, Result};
use heaven_exchange::{
    calculator::{
        swap_direction::SwapDirection, taxation_mode::TaxationMode, ProtocolSwapFeeDirection,
    },
    get_transfer_fee_config,
    instructions::{chainlink_feed_account, chainlink_program, LiquidityPoolState},
    quote::{quote_exact_in, quote_exact_out},
};
use jupiter_amm_interface::{
    Amm, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapMode, SwapParams,
};

pub struct HeavenAmm {
    pub key: Pubkey,
    pub authority: Pubkey,
    pub state: LiquidityPoolState,
    pub base_transfer_fee: TransferFee,
    pub quote_transfer_fee: TransferFee,
    pub epoch: Arc<AtomicU64>,
}

pub const PROGRAM_ID: Pubkey = pubkey!("HEAVEnMX7RoaYCucpyFterLWzFJR8Ah26oNSnqBs5Jtn");
pub const AUTHORITY: Pubkey = pubkey!("GBrN2zZCrhzn1ouVxT3RNkBmwFAjLNt2p2MSXwMLMX7");

pub fn derive_user_global_stats(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            heaven_exchange::instructions::seeds::USER_GLOBAL_STATS.as_bytes(),
            user.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

pub fn derive_user_amm_stats(user: &Pubkey, pool_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            heaven_exchange::instructions::seeds::USER_AMM_STATS.as_bytes(),
            user.as_ref(),
            pool_id.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

pub fn derive_extras_account(creator: &Pubkey, base_mint: &Pubkey, quote_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            heaven_exchange::instructions::seeds::EXTRAS_ACCOUNT.as_bytes(),
            creator.as_ref(),
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

impl Amm for HeavenAmm {
    fn label(&self) -> String {
        return String::from("Heaven");
    }

    fn key(&self) -> Pubkey {
        return self.key;
    }

    fn from_keyed_account(
        keyed_account: &KeyedAccount,
        amm_context: &jupiter_amm_interface::AmmContext,
    ) -> Result<Self>
    where
        Self: Sized,
    {
        let mut info = keyed_account.account.data.clone();
        let size = std::mem::size_of::<LiquidityPoolState>() + 8;
        let extra = size - info.len();
        if extra > 0 {
            info.extend(vec![0u8; extra]);
        }
        let state = LiquidityPoolState::try_deserialize(&mut info.as_ref())?;

        Ok(HeavenAmm {
            key: keyed_account.key,
            state,
            authority: AUTHORITY,
            base_transfer_fee: TransferFee::default(),
            quote_transfer_fee: TransferFee::default(),
            epoch: amm_context.clock_ref.epoch.clone(),
        })
    }

    fn program_id(&self) -> Pubkey {
        PROGRAM_ID
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.state.base_token_mint, self.state.quote_token_mint]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.key,
            self.state.base_token_mint,
            self.state.quote_token_mint,
        ]
    }

    fn update(&mut self, account_map: &jupiter_amm_interface::AccountMap) -> Result<()> {
        let mut info = account_map
            .get(&self.key)
            .ok_or_else(|| anyhow::anyhow!("Could not find liquidity pool state account"))?
            .data.clone();
        let size = std::mem::size_of::<LiquidityPoolState>() + 8;
        let extra = size - info.len();
        if extra > 0 {
            info.extend(vec![0u8; extra]);
        }
        let new_state = LiquidityPoolState::try_deserialize(&mut info.as_ref())?;
        let base_mint = account_map
            .get(&self.state.base_token_mint)
            .ok_or_else(|| anyhow::anyhow!("Could not find base token mint account"))?;

        let quote_mint = account_map
            .get(&self.state.quote_token_mint)
            .ok_or_else(|| anyhow::anyhow!("Could not find quote token mint account"))?;

        let epoch = self.epoch.load(Ordering::Relaxed);
        let base_transfer_fee =
            get_transfer_fee_config(&(base_mint.data[0..].to_vec(), base_mint.owner), epoch)?;
        let quote_transfer_fee =
            get_transfer_fee_config(&(quote_mint.data[0..].to_vec(), quote_mint.owner), epoch)?;

        self.state = new_state;
        self.base_transfer_fee = base_transfer_fee;
        self.quote_transfer_fee = quote_transfer_fee;
        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let protocol_swap_fee_direction = match self.state.taxation_mode {
            TaxationMode::Base => ProtocolSwapFeeDirection::Base,
            TaxationMode::Quote => ProtocolSwapFeeDirection::Quote,
            TaxationMode::None => ProtocolSwapFeeDirection::None,
        };

        let swap_direction = if quote_params.input_mint.eq(&self.state.base_token_mint) {
            SwapDirection::Base2Quote
        } else {
            SwapDirection::Quote2Base
        };
        match quote_params.swap_mode {
            SwapMode::ExactIn => {
                let (_minimum_out, amount_out, total_fee) = quote_exact_in(
                    quote_params.amount,
                    swap_direction,
                    protocol_swap_fee_direction,
                    self.state.taxation_mode,
                    self.state.base_token_vault_balance,
                    self.state.quote_token_vault_balance,
                    self.state.swap_fee_numerator,
                    self.state.swap_fee_denominator,
                    self.state.protocol_swap_fee_numerator,
                    self.state.protocol_swap_fee_denominator,
                    self.state.buy_tax,
                    self.state.sell_tax,
                    self.base_transfer_fee,
                    self.quote_transfer_fee,
                    0,
                )?;
                Ok(Quote {
                    fee_amount: total_fee,
                    fee_mint: match self.state.taxation_mode {
                        TaxationMode::Base => self.state.base_token_mint,
                        TaxationMode::Quote => self.state.quote_token_mint,
                        TaxationMode::None => quote_params.input_mint,
                    },
                    in_amount: quote_params.amount,
                    out_amount: amount_out,
                    ..Default::default()
                })
            }
            SwapMode::ExactOut => {
                let (_maximum_amount_in, amount_in, total_fee) = quote_exact_out(
                    quote_params.amount,
                    swap_direction,
                    protocol_swap_fee_direction,
                    self.state.taxation_mode,
                    self.state.base_token_vault_balance,
                    self.state.quote_token_vault_balance,
                    self.state.swap_fee_numerator,
                    self.state.swap_fee_denominator,
                    self.state.protocol_swap_fee_numerator,
                    self.state.protocol_swap_fee_denominator,
                    self.state.buy_tax,
                    self.state.sell_tax,
                    self.base_transfer_fee,
                    self.quote_transfer_fee,
                    0,
                )?;
                Ok(Quote {
                    fee_amount: total_fee,
                    fee_mint: match self.state.taxation_mode {
                        TaxationMode::Base => self.state.base_token_mint,
                        TaxationMode::Quote => self.state.quote_token_mint,
                        TaxationMode::None => quote_params.input_mint,
                    },
                    in_amount: amount_in,
                    out_amount: quote_params.amount,
                    ..Default::default()
                })
            }
        }
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let (user_base_token_account, user_quote_token_account) =
            if swap_params.source_mint.eq(&self.state.base_token_mint) {
                (
                    swap_params.source_token_account,
                    swap_params.destination_token_account,
                )
            } else {
                (
                    swap_params.destination_token_account,
                    swap_params.source_token_account,
                )
            };

        // SwapIn and SwapOut instructions have the same account input
        let mut accounts = heaven_exchange::accounts::SwapInAccounts {
            associated_token_program: AssociatedToken::id(),
            authority: self.authority,
            base_token_program: if self.state.base_token_program.eq(&Pubkey::default()) {
                Token::id()
            } else {
                self.state.base_token_program
            },
            base_token_mint: self.state.base_token_mint,
            base_token_swap_tax_vault: self.state.base_token_swap_tax_vault,
            base_token_vault: self.state.base_token_vault,
            liquidity_pool_state: self.key,
            protocol_base_token_swap_fee_vault: self.state.protocol_base_token_swap_fee_vault,
            protocol_quote_token_swap_fee_vault: self.state.protocol_quote_token_swap_fee_vault,
            quote_token_mint: self.state.quote_token_mint,
            quote_token_program: if self.state.quote_token_program.eq(&Pubkey::default()) {
                Token::id()
            } else {
                self.state.quote_token_program
            },
            quote_token_swap_tax_vault: self.state.quote_token_swap_tax_vault,
            quote_token_vault: self.state.quote_token_vault,
            system_program: System::id(),
            token_program: Token::id(),
            user: swap_params.token_transfer_authority,
            user_base_token_vault: user_base_token_account,
            user_quote_token_vault: user_quote_token_account,
            user_amm_stats: derive_user_amm_stats(&swap_params.token_transfer_authority, &self.key)
                .0,
            user_global_stats: derive_user_global_stats(&swap_params.token_transfer_authority).0,
        }
        .to_account_metas(None);

        let remaining_accounts = vec![
            AccountMeta {
                pubkey: chainlink_feed_account::ID,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: chainlink_program::ID,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: derive_extras_account(&self.state.creator, &self.state.base_token_mint, &self.state.quote_token_mint).0,
                is_signer: false,
                is_writable: false,
            },
        ];

        accounts.extend(remaining_accounts);

        Ok(SwapAndAccountMetas {
            // Do we have to add `Heaven` to the Swap enum on the jupiter_amm_interface?
            swap: Swap::TokenSwap,
            account_metas: accounts,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(HeavenAmm {
            key: self.key,
            state: self.state.clone(),
            authority: self.authority,
            base_transfer_fee: self.base_transfer_fee.clone(),
            quote_transfer_fee: self.quote_transfer_fee.clone(),
            epoch: self.epoch.clone(),
        })
    }
}
