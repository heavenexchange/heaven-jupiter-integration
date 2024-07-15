use anchor_lang::prelude::*;

use self::{number::U128, swap_direction::SwapDirection};

use super::stable_coin;

pub mod constant_product_curve;
pub mod number;
pub mod swap_direction;
pub mod taxation_mode;

#[derive(PartialEq, Eq)]
pub enum StableCoin {
    WSOL,
    USDC,
    USDT,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundDirection {
    Floor,
    Ceiling,
}

impl StableCoin {
    pub fn to_u8(&self) -> u8 {
        match self {
            StableCoin::WSOL => 0,
            StableCoin::USDC => 1,
            StableCoin::USDT => 2,
        }
    }

    pub fn mint(&self) -> Pubkey {
        match self {
            StableCoin::WSOL => stable_coin::wsol::id(),
            StableCoin::USDC => stable_coin::usdc::id(),
            StableCoin::USDT => stable_coin::usdt::id(),
        }
    }

    pub fn from_mint(mint: &Pubkey) -> Option<Self> {
        match *mint {
            key if key.eq(&stable_coin::wsol::id()) => Some(StableCoin::WSOL),
            key if key.eq(&stable_coin::usdc::id()) => Some(StableCoin::USDC),
            key if key.eq(&stable_coin::usdt::id()) => Some(StableCoin::USDT),
            _ => None,
        }
    }
}

pub struct ConstantProduct;
pub struct SharedConstantProduct;
pub struct TokenSwapCalculator<Curve> {
    _marker: std::marker::PhantomData<Curve>,
}

#[derive(PartialEq, Eq, Copy, Clone, Default, Debug)]
pub enum ProtocolSwapFeeDirection {
    #[default]
    None,
    Base,
    Quote,
}

impl ProtocolSwapFeeDirection {
    pub fn from(
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
        swap_direction: &SwapDirection,
    ) -> Result<Self> {
        let base_coin = StableCoin::from_mint(base_mint);
        let quote_coin = StableCoin::from_mint(quote_mint);

        let direction = match (base_coin, quote_coin) {
            (Some(_), Some(_)) => match swap_direction {
                SwapDirection::Base2Quote => ProtocolSwapFeeDirection::Base,
                SwapDirection::Quote2Base => ProtocolSwapFeeDirection::Quote,
            },
            (Some(_), None) => ProtocolSwapFeeDirection::Base,
            (None, Some(_)) => ProtocolSwapFeeDirection::Quote,
            _ => ProtocolSwapFeeDirection::None,
        };

        Ok(direction)
    }
}

#[derive(Debug)]
pub struct SwapInCalculationResult {
    pub swap_amount_in_before_fees: U128,
    pub swap_amount_in_after_fees: U128,
    pub swap_amount_out_after_fees: U128,
    pub swap_amount_out_before_fees: U128,
    pub swap_fee: U128,
    pub swap_tax_on_input_amount: U128,
    pub swap_tax_on_output_amount: U128,
    pub protocol_swap_fee_on_input_amount: U128,
    pub protocol_swap_fee_on_output_amount: U128,
}

#[derive(Debug)]
pub struct SwapOutCalculationResult {
    pub swap_amount_in_after_fees: U128,
    pub swap_amount_out_after_fees: U128,
    pub swap_amount_in_before_fees: U128,
    pub swap_amount_out_before_fees: U128,
    pub swap_fee: U128,
    pub swap_tax_on_input_amount: U128,
    pub swap_tax_on_output_amount: U128,
    pub protocol_swap_fee_on_input_amount: U128,
    pub protocol_swap_fee_on_output_amount: U128,
}
