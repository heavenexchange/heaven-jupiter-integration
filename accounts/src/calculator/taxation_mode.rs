use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize, Result};

use crate::utils::error::AmmErrorCode;

use super::StableCoin;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum TaxationMode {
    #[default]
    None = 0,
    Base = 1,
    Quote = 2,
}

impl TaxationMode {
    pub fn from_u8(mode: u8) -> Result<Self> {
        match mode {
            0 => Ok(TaxationMode::None),
            1 => Ok(TaxationMode::Base),
            2 => Ok(TaxationMode::Quote),
            _ => Err(AmmErrorCode::InvalidTaxationMode.into()),
        }
    }

    pub fn from_mints(base_mint: &Pubkey, quote_mint: &Pubkey) -> Self {
        let base_coin = StableCoin::from_mint(base_mint);
        let quote_coin = StableCoin::from_mint(quote_mint);

        let direction = match (base_coin, quote_coin) {
            (Some(base), Some(quote)) => {
                if quote == StableCoin::WSOL {
                    Self::Quote
                } else if base == StableCoin::WSOL {
                    Self::Base
                } else {
                    Self::Quote
                }
            },
            (Some(_base), None) => Self::Base,
            (None, Some(_quote)) => Self::Quote,
            (None, None) => Self::None,
        };
        direction
    }

    pub fn into_u8(&self) -> u8 {
        match self {
            TaxationMode::None => 0,
            TaxationMode::Base => 1,
            TaxationMode::Quote => 2,
        }
    }
}
