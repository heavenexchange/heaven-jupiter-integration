#[cfg(feature = "debug")]
use solana_program::msg;

use crate::TEN_THOUSAND;

use super::{
    number::{CheckedCeilDiv, U128},
    swap_direction::SwapDirection,
    taxation_mode::TaxationMode,
    ConstantProduct, ProtocolSwapFeeDirection, RoundDirection, SwapInCalculationResult,
    SwapOutCalculationResult, TokenSwapCalculator,
};

use anyhow::Result;

impl TokenSwapCalculator<ConstantProduct> {
    pub fn swap_in(
        amount_in: u64,
        swap_direction: &SwapDirection,
        protocol_swap_fee_direction: &ProtocolSwapFeeDirection,
        taxation_mode: &TaxationMode,
        base_token_amount: u64,
        quote_token_amount: u64,
        swap_fee_numerator: u64,
        swap_fee_denominator: u64,
        protocol_swap_fee_numerator: u64,
        protocol_swap_fee_denominator: u64,
        buy_tax: u64,
        sell_tax: u64,
    ) -> Result<SwapInCalculationResult> {
        let swap_amount_in_before_fees = U128::from(amount_in);

        let swap_fee = swap_amount_in_before_fees
            .checked_mul(swap_fee_numerator.into())
            .ok_or_else(|| anyhow::anyhow!("Swap fee calculation overflow"))?
            .checked_ceil_div(swap_fee_denominator.into())
            .ok_or_else(|| anyhow::anyhow!("Swap fee calculation overflow"))?
            .0;
        let mut swap_tax_on_input_amount = U128::zero();
        let mut swap_tax_on_output_amount = U128::zero();
        let mut protocol_swap_fee_on_input_amount = U128::zero();
        let mut protocol_swap_fee_on_output_amount = U128::zero();

        match (taxation_mode, swap_direction) {
            // If `Base` is the stable/native coin and `Quote` is the custom coin, 
            // then swapping base for quote is considered buying
            (TaxationMode::Base, SwapDirection::Base2Quote)
            // If `Quote` is the stable/native coin and `Base` is the custom coin, 
            // then swapping `Quote` for `Base` is considered buying
            | (TaxationMode::Quote, SwapDirection::Quote2Base) => {
                swap_tax_on_input_amount = swap_amount_in_before_fees
                    .checked_mul(buy_tax.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .checked_ceil_div(TEN_THOUSAND.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .0;
            }
            _ => {}
        };

        match (protocol_swap_fee_direction, swap_direction) {
            (ProtocolSwapFeeDirection::Base, SwapDirection::Base2Quote)
            | (ProtocolSwapFeeDirection::Quote, SwapDirection::Quote2Base) => {
                protocol_swap_fee_on_input_amount = swap_amount_in_before_fees
                    .checked_mul(protocol_swap_fee_numerator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .checked_ceil_div(protocol_swap_fee_denominator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .0;
            }
            _ => {}
        };

        let swap_amount_in_after_fees = swap_amount_in_before_fees
            .checked_sub(swap_fee)
            .ok_or_else(|| anyhow::anyhow!("Swap fee deduction overflow"))?
            .checked_sub(swap_tax_on_input_amount)
            .ok_or_else(|| anyhow::anyhow!("Swap tax deduction overflow"))?
            .checked_sub(protocol_swap_fee_on_input_amount)
            .ok_or_else(|| anyhow::anyhow!("Protocol swap fee deduction overflow"))?;

        let swap_amount_out_before_fees = match swap_direction {
            SwapDirection::Base2Quote => {
                let denominator = U128::from(base_token_amount)
                    .checked_add(swap_amount_in_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?;
                U128::from(quote_token_amount)
                    .checked_mul(swap_amount_in_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?
                    .checked_div(denominator)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?
            }
            SwapDirection::Quote2Base => {
                let denominator = U128::from(quote_token_amount)
                    .checked_add(swap_amount_in_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?;
                U128::from(base_token_amount)
                    .checked_mul(swap_amount_in_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?
                    .checked_div(denominator)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount out calculation overflow"))?
            }
        };

        match (taxation_mode, swap_direction) {
            // If `Base` is the stable/native coin and `Quote` is the custom coin, 
            // then swapping `Quote` for `Base` is considered selling
            (TaxationMode::Base, SwapDirection::Quote2Base)
            // If `Quote` is the stable/native coin and `Base` is the custom coin, 
            // then swapping `Base` for `Quote` is considered selling
            | (TaxationMode::Quote, SwapDirection::Base2Quote) => {
                swap_tax_on_output_amount = swap_amount_out_before_fees
                    .checked_mul(sell_tax.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .checked_ceil_div(TEN_THOUSAND.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .0;
            }
            _ => {}
        };

        match (protocol_swap_fee_direction, swap_direction) {
            (ProtocolSwapFeeDirection::Base, SwapDirection::Quote2Base)
            | (ProtocolSwapFeeDirection::Quote, SwapDirection::Base2Quote) => {
                protocol_swap_fee_on_output_amount = swap_amount_out_before_fees
                    .checked_mul(protocol_swap_fee_numerator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .checked_ceil_div(protocol_swap_fee_denominator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .0;
            }
            _ => {}
        };

        let swap_amount_out_after_fees = swap_amount_out_before_fees
            .checked_sub(swap_tax_on_output_amount)
            .ok_or_else(|| anyhow::anyhow!("Swap tax deduction overflow"))?
            .checked_sub(protocol_swap_fee_on_output_amount)
            .ok_or_else(|| anyhow::anyhow!("Protocol swap fee deduction overflow"))?;

        Ok(SwapInCalculationResult {
            swap_amount_in_before_fees,
            swap_amount_in_after_fees,
            swap_amount_out_after_fees,
            swap_amount_out_before_fees,
            swap_fee,
            swap_tax_on_input_amount,
            swap_tax_on_output_amount,
            protocol_swap_fee_on_input_amount,
            protocol_swap_fee_on_output_amount,
        })
    }

    pub fn swap_out(
        amount_out: u64,
        swap_direction: &SwapDirection,
        external_fee_direction: &ProtocolSwapFeeDirection,
        taxation_mode: &TaxationMode,
        base_token_amount: u64,
        quote_token_amount: u64,
        swap_fee_numerator: u64,
        swap_fee_denominator: u64,
        protocol_swap_fee_numerator: u64,
        protocol_swap_fee_denominator: u64,
        buy_tax: u64,
        sell_tax: u64,
    ) -> Result<SwapOutCalculationResult> {
        let swap_amount_out_before_fees = U128::from(amount_out);
        let total_base_token_amount = U128::from(base_token_amount);
        let total_quote_token_amount = U128::from(quote_token_amount);

        #[cfg(feature = "debug")]
        msg!(
            "swap_amount_out_before_fees: {}, total_base_token_amount: {}, total_quote_token_amount: {}",
            swap_amount_out_before_fees,
            total_base_token_amount,
            total_quote_token_amount
        );

        let protocol_swap_fee_on_output_amount;

        // Base on the directions apply external fee on the amount out
        match (external_fee_direction, swap_direction) {
            (ProtocolSwapFeeDirection::Quote, SwapDirection::Base2Quote)
            | (ProtocolSwapFeeDirection::Base, SwapDirection::Quote2Base) => {
                protocol_swap_fee_on_output_amount = swap_amount_out_before_fees
                    .checked_mul(protocol_swap_fee_numerator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .checked_ceil_div(protocol_swap_fee_denominator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .0;
            }
            _ => {
                protocol_swap_fee_on_output_amount = U128::zero();
            }
        }

        #[cfg(feature = "debug")]
        msg!(
            "protocol_swap_fee_on_output_amount: {}",
            protocol_swap_fee_on_output_amount
        );

        let swap_tax_on_output_amount;
        match (taxation_mode, swap_direction) {
            // If `Base` is the stable/native coin and `Quote` is the custom coin,
            // then swapping `Quote` for `Base` is considered selling
            (TaxationMode::Base, SwapDirection::Quote2Base)
            // If `Quote` is the stable/native coin and `Base` is the custom coin,
            // then swapping `Base` for `Quote` is considered selling
            | (TaxationMode::Quote, SwapDirection::Base2Quote) => {
                swap_tax_on_output_amount = swap_amount_out_before_fees
                    .checked_mul(sell_tax.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .checked_ceil_div(TEN_THOUSAND.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .0;
            }
            _ => {
                swap_tax_on_output_amount = U128::zero();
            }
        }

        #[cfg(feature = "debug")]
        msg!("swap_tax_on_output_amount: {}", swap_tax_on_output_amount);

        let swap_amount_out_after_fees = swap_amount_out_before_fees
            .checked_add(protocol_swap_fee_on_output_amount)
            .ok_or_else(|| anyhow::anyhow!("Protocol swap fee addition overflow"))?
            .checked_add(swap_tax_on_output_amount)
            .ok_or_else(|| anyhow::anyhow!("Swap tax addition overflow"))?;

        #[cfg(feature = "debug")]
        msg!("swap_amount_out_after_fees: {}", swap_amount_out_after_fees);

        let swap_amount_in_before_fees;
        match swap_direction {
            SwapDirection::Base2Quote => {
                let denominator = total_quote_token_amount
                    .checked_sub(swap_amount_out_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?;
                swap_amount_in_before_fees = total_base_token_amount
                    .checked_mul(swap_amount_out_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?
                    .checked_ceil_div(denominator)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?
                    .0;
            }
            SwapDirection::Quote2Base => {
                let denominator = total_base_token_amount
                    .checked_sub(swap_amount_out_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?;
                swap_amount_in_before_fees = total_quote_token_amount
                    .checked_mul(swap_amount_out_after_fees)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?
                    .checked_ceil_div(denominator)
                    .ok_or_else(|| anyhow::anyhow!("Swap amount in calculation overflow"))?
                    .0;
            }
        };

        #[cfg(feature = "debug")]
        msg!("swap_amount_in_before_fees: {}", swap_amount_in_before_fees);

        let swap_fee = swap_amount_in_before_fees
            .checked_mul(swap_fee_numerator.into())
            .ok_or_else(|| anyhow::anyhow!("Swap fee calculation overflow"))?
            .checked_ceil_div(swap_fee_denominator.into())
            .ok_or_else(|| anyhow::anyhow!("Swap fee calculation overflow"))?
            .0;

        #[cfg(feature = "debug")]
        msg!("swap_fee: {}", swap_fee);

        let protocol_swap_fee_on_input_amount;
        // Base on the directions apply external fee on the amount in
        match (external_fee_direction, swap_direction) {
            (ProtocolSwapFeeDirection::Base, SwapDirection::Base2Quote)
            | (ProtocolSwapFeeDirection::Quote, SwapDirection::Quote2Base) => {
                protocol_swap_fee_on_input_amount = swap_amount_in_before_fees
                    .checked_mul(protocol_swap_fee_numerator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .checked_ceil_div(protocol_swap_fee_denominator.into())
                    .ok_or_else(|| anyhow::anyhow!("Protocol swap fee calculation overflow"))?
                    .0;
            }
            _ => {
                protocol_swap_fee_on_input_amount = U128::zero();
            }
        }

        #[cfg(feature = "debug")]
        msg!(
            "protocol_swap_fee_on_input_amount: {}",
            protocol_swap_fee_on_input_amount
        );

        let swap_tax_on_input_amount;
        match (taxation_mode, swap_direction) {
            // If `Quote` is the stable/native coin and `Base` is the custom coin,
            // then swapping `Quote` for `Base` is considered buying
            (TaxationMode::Quote, SwapDirection::Quote2Base)
            // If `Base` is the stable/native coin and `Quote` is the custom coin,
            // then swapping `Base` for `Quote` is considered buying
            | (TaxationMode::Base, SwapDirection::Base2Quote) => {
                swap_tax_on_input_amount = swap_amount_in_before_fees
                    .checked_mul(buy_tax.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .checked_ceil_div(TEN_THOUSAND.into())
                    .ok_or_else(|| anyhow::anyhow!("Swap tax calculation overflow"))?
                    .0;
            }
            _ => {
                swap_tax_on_input_amount = U128::zero();
            }
        }

        #[cfg(feature = "debug")]
        msg!("swap_tax_on_input_amount: {}", swap_tax_on_input_amount);

        let swap_amount_in_after_fees = swap_amount_in_before_fees
            .checked_add(protocol_swap_fee_on_input_amount)
            .ok_or_else(|| anyhow::anyhow!("Protocol swap fee addition overflow"))?
            .checked_add(swap_tax_on_input_amount)
            .ok_or_else(|| anyhow::anyhow!("Swap tax addition overflow"))?
            .checked_add(swap_fee)
            .ok_or_else(|| anyhow::anyhow!("Swap fee addition overflow"))?;

        #[cfg(feature = "debug")]
        msg!("swap_amount_in_after_fees: {}", swap_amount_in_after_fees);

        Ok(SwapOutCalculationResult {
            swap_amount_in_after_fees,
            swap_amount_out_after_fees,
            swap_amount_in_before_fees,
            swap_amount_out_before_fees,
            swap_fee,
            swap_tax_on_input_amount,
            swap_tax_on_output_amount,
            protocol_swap_fee_on_input_amount,
            protocol_swap_fee_on_output_amount,
        })
    }
}
