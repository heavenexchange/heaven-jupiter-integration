use anchor_lang::prelude::*;

#[error_code]
pub enum AmmErrorCode {
    #[msg("Unsupported token mint")]
    UnsupportedTokenMint,
    #[msg("Invalid token vault balance")]
    InvalidTokenVaultBalance,
    #[msg("Invalid user token")]
    InvalidUserToken,
    #[msg("Invalid taxation mode")]
    InvalidTaxationMode,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Invalid lock liquidity provider token percentage")]
    InvalidLockLiquidityProviderTokenPercentage,
    #[msg("Cannot create pool with the a disabled protocol config version")]
    CannotCreatePoolWithDisabledProtocolConfigVersion,
    #[msg("Invalid token input amount")]
    InvalidTokenInputAmount,
    #[msg("Invalid swap tax")]
    InvalidSwapTax,
    #[msg("Invalid fee mode")]
    InvalidFeeMode,
    #[msg("Invalid liquidity provider token lock vault")]
    InvalidLiquidityProviderTokenLockVault,
    #[msg("Invalid liquidity provider token vault")]
    InvalidUserLiquidityProviderTokenVault,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Exceeded slippage")]
    ExceededSlippage,
    #[msg("Invalid add liquidity input")]
    InvalidAddLiquidityInput,
    #[msg("Invalid remove liquidity input")]
    InvalidRemoveLiquidityInput,
    #[msg("Add liquidity is disabled")]
    AddLiquidityDisabled,
    #[msg("Remove liquidity is disabled")]
    RemoveLiquidityDisabled,
    #[msg("Swap is disabled")]
    SwapDisabled,
    #[msg("Liquidity pool is not open yet")]
    LiquidityPoolIsNotOpenYet,
    #[msg("Invalid swap in inputs")]
    InvalidSwapInInputs,
    #[msg("Invalid protocol swap fee wallet")]
    InvalidProtocolSwapFeeWallet,
    #[msg("Invalid swap out inputs")]
    InvalidSwapOutInputs,
    #[msg("Invalid post fee amount")]
    InvalidPostFeeAmount,
    #[msg("Exceeded quote token slippage")]
    ExceededQuoteTokenSlippage,
    #[msg("Exceeded base token slippage")]
    ExceededBaseTokenSlippage,
    #[msg("Lp tokens locked")]
    LpTokensLocked,
    #[msg("Invalid protocol base token swap fee vault")]
    InvalidProtocolBaseTokenSwapFeeVault,
    #[msg("Invalid protocol quote token swap fee vault")]
    InvalidProtocolQuoteTokenSwapFeeVault,
    #[msg("Invalid user pool stats account")]
    InvalidUserPoolStatsAccount,
    #[msg("Invalid user global stats account")]
    InvalidUserGlobalStatsAccount,
    #[msg("Cannot update lp lock")]
    CannotUpdateLpLock,
    #[msg("Zero amount")]
    ZeroAmount,
    #[msg("Cannot update lp open time")]
    CannotUpdateLpOpenTime,
    #[msg("Cannot set lock burn lp tokens")]
    CannotSetLockBurnLpTokens,
    #[msg("Invalid tax")]
    InvalidTax,
    #[msg("Invalid chainlink feed account")]
    InvalidChainlinkFeedAccount,
    #[msg("Invalid chainlink program")]
    InvalidChainlinkProgram,
    #[msg("Invalid config version")]
    InvalidConfigVersion,
}
