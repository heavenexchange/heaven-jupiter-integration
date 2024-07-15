use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize, Result};

use crate::utils::error::AmmErrorCode;


#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum SwapDirection {
    #[default]
    Quote2Base = 1u8,
    Base2Quote = 2u8,
}

impl SwapDirection {
    pub fn parse(
        source_mint: &Pubkey,
        destination_mint: &Pubkey,
        base_mint: &Pubkey,
        quote_mint: &Pubkey,
    ) -> Result<SwapDirection> {
        match (*source_mint, *destination_mint) {
            x if x == (*base_mint, *quote_mint) => Ok(SwapDirection::Base2Quote),
            x if x == (*quote_mint, *base_mint) => Ok(SwapDirection::Quote2Base),
            _ => Err(AmmErrorCode::InvalidUserToken.into()),
        }
    }
}
