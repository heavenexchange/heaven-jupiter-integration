use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::TransferFee;

use crate::calculator::{
    number::{CheckedCeilDiv, U128},
    swap_direction::SwapDirection,
    taxation_mode::TaxationMode,
    ConstantProduct, ProtocolSwapFeeDirection, TokenSwapCalculator,
};
use anyhow::Result;

pub fn quote_exact_out(
    amount_out: u64,
    swap_direction: SwapDirection,
    external_fee_direction: ProtocolSwapFeeDirection,
    taxation_mode: TaxationMode,
    base_token_amount: u64,
    quote_token_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    protocol_swap_fee_numerator: u64,
    protocol_swap_fee_denominator: u64,
    buy_tax: u64,
    sell_tax: u64,
    base_token_transfer_fee: TransferFee,
    quote_token_transfer_fee: TransferFee,
    slippage_numerator: u64,
) -> Result<(u64, u64, u64)> {
    let (input_transfer_fee, output_transfer_fee) = match swap_direction {
        SwapDirection::Base2Quote => (base_token_transfer_fee, quote_token_transfer_fee),
        SwapDirection::Quote2Base => (quote_token_transfer_fee, base_token_transfer_fee),
    };

    let amount_out_transfer_fee = output_transfer_fee
        .calculate_inverse_fee(amount_out)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to calculate transfer fee for amount_out: {}",
                amount_out
            )
        })?;

    let amount_out_after_transfer_fee = amount_out
        .checked_add(amount_out_transfer_fee)
        .ok_or_else(|| {
            anyhow::anyhow!("Failed to add transfer fee to amount_out: {}", amount_out)
        })?;

    let result = TokenSwapCalculator::<ConstantProduct>::swap_out(
        amount_out_after_transfer_fee,
        &swap_direction,
        &external_fee_direction,
        &taxation_mode,
        base_token_amount,
        quote_token_amount,
        swap_fee_numerator,
        swap_fee_denominator,
        protocol_swap_fee_numerator,
        protocol_swap_fee_denominator,
        buy_tax,
        sell_tax,
    )?;

    let amount_in_transfer_fee = input_transfer_fee
        .calculate_inverse_fee(result.swap_amount_in_after_fees.as_u64())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to calculate transfer fee for amount_in: {}",
                result.swap_amount_in_after_fees.as_u64()
            )
        })?;

    let amount_in_after_transfer_fee = result
        .swap_amount_in_after_fees
        .checked_add(amount_in_transfer_fee.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to add transfer fee to amount_in"))?;

    let slippage_amount = amount_in_after_transfer_fee
        .checked_mul(slippage_numerator.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate slippage amount"))?
        .checked_ceil_div(10000.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate slippage amount"))?
        .0;

    let maximum_amount_in = amount_in_after_transfer_fee
        .checked_add(slippage_amount)
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate maximum amount in"))?
        .as_u64();

    let total_fees = match (taxation_mode, swap_direction) {
        (TaxationMode::Base, SwapDirection::Base2Quote)
        | (TaxationMode::Quote, SwapDirection::Quote2Base) => {
            let total = result.protocol_swap_fee_on_input_amount
                + result.swap_tax_on_input_amount
                + result.swap_fee;
            total
        }
        (TaxationMode::Quote, SwapDirection::Base2Quote)
        | (TaxationMode::Base, SwapDirection::Quote2Base) => {
            let total =
                result.protocol_swap_fee_on_output_amount + result.swap_tax_on_output_amount;
            total
        }
        _ => result.swap_fee,
    };

    Ok((
        maximum_amount_in,
        amount_in_after_transfer_fee.as_u64(),
        total_fees.as_u64(),
    ))
}

pub fn quote_exact_in(
    amount_in: u64,
    swap_direction: SwapDirection,
    protocol_swap_fee_direction: ProtocolSwapFeeDirection,
    taxation_mode: TaxationMode,
    base_token_amount: u64,
    quote_token_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    protocol_swap_fee_numerator: u64,
    protocol_swap_fee_denominator: u64,
    buy_tax: u64,
    sell_tax: u64,
    base_token_transfer_fee: TransferFee,
    quote_token_transfer_fee: TransferFee,
    slippage_numerator: u64,
) -> Result<(u64, u64, u64)> {
    let (input_transfer_fee, output_transfer_fee) = match swap_direction {
        SwapDirection::Base2Quote => (base_token_transfer_fee, quote_token_transfer_fee),
        SwapDirection::Quote2Base => (quote_token_transfer_fee, base_token_transfer_fee),
    };
    let amount_in_transfer_fee = input_transfer_fee.calculate_fee(amount_in).ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to calculate transfer fee for amount_in: {}",
            amount_in
        )
    })?;

    let amount_in_after_deduct_transfer_fee = amount_in.saturating_sub(amount_in_transfer_fee);

    let result = TokenSwapCalculator::<ConstantProduct>::swap_in(
        amount_in_after_deduct_transfer_fee,
        &swap_direction,
        &protocol_swap_fee_direction,
        &taxation_mode,
        base_token_amount,
        quote_token_amount,
        swap_fee_numerator,
        swap_fee_denominator,
        protocol_swap_fee_numerator,
        protocol_swap_fee_denominator,
        buy_tax,
        sell_tax,
    )?;

    let amount_out_transfer_fee = output_transfer_fee
        .calculate_fee(result.swap_amount_out_after_fees.as_u64())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to calculate transfer fee for amount_out: {}",
                result.swap_amount_out_after_fees.as_u64()
            )
        })?;

    let swap_amount_out_after_deduct_transfer_fees = result
        .swap_amount_out_after_fees
        .checked_sub(amount_out_transfer_fee.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to subtract transfer fee from amount_out"))?;

    let slippage_amount = swap_amount_out_after_deduct_transfer_fees
        .checked_mul(slippage_numerator.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate slippage amount"))?
        .checked_ceil_div(10000.into())
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate slippage amount"))?
        .0;

    let minimum_amount_out = swap_amount_out_after_deduct_transfer_fees
        .checked_sub(slippage_amount)
        .ok_or_else(|| anyhow::anyhow!("Failed to calculate minimum amount out"))?
        .as_u64();

    let total_fees = match (taxation_mode, swap_direction) {
        (TaxationMode::Base, SwapDirection::Base2Quote)
        | (TaxationMode::Quote, SwapDirection::Quote2Base) => {
            let total = result.protocol_swap_fee_on_input_amount
                + result.swap_tax_on_input_amount
                + result.swap_fee;
            total
        }
        (TaxationMode::Quote, SwapDirection::Base2Quote)
        | (TaxationMode::Base, SwapDirection::Quote2Base) => {
            let total =
                result.protocol_swap_fee_on_output_amount + result.swap_tax_on_output_amount;
            total
        }
        _ => U128::from(0),
    };

    Ok((
        minimum_amount_out,
        swap_amount_out_after_deduct_transfer_fees.as_u64(),
        total_fees.as_u64(),
    ))
}
