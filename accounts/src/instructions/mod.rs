pub mod create_liquidity_pool;

use anchor_lang::event;
pub use create_liquidity_pool::*;

pub mod swap_in;
pub use swap_in::*;

pub mod swap_out;
pub use swap_out::*;

use anchor_lang::prelude::*;

pub mod seeds {
    pub const LIQUIDITY_POOL_STATE: &str = "liquidity_pool_state";
    pub const USER_AMM_STATS: &str = "user_amm_stats";
    pub const USER_GLOBAL_STATS: &str = "user_global_stats";
    pub const PROTOCOL_CONFIG_STATE: &str = "protocol_config_state";
    pub const AUTHORITY: &str = "authority";
    pub const LOCK_LP_AUTHORITY: &str = "lock_lp_authority";
    pub const LIQUIDITY_PROVIDER_TOKEN_MINT: &str = "liquidity_provider_token_mint";
    pub const LIQUIDITY_POOL_TOKEN_VAULT: &str = "liquidity_pool_token_vault";
    pub const LIQUIDITY_POOL_SWAP_TAX_TOKEN_VAULT: &str = "lp_swap_tax_token_vault";
    pub const PROTOCOL_SWAP_FEE_VAULT: &str = "protocol_swap_fee_vault";
    pub const PROTOCOL_OWNER_STATE: &str = "protocol_owner_state";
    pub const EXTRAS_ACCOUNT: &str = "extras_account";
}

pub struct AmmInstructions<T> {
    _marker: std::marker::PhantomData<T>,
}

pub struct Heaven;
pub struct Raydium;

#[cfg(feature = "localnet")]
pub mod protocol_account_config {
    pub mod owner_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("6GM1A5oYPLYn1c1sX1YmiUhWRP9yV5BgGaFTBf58XvdX");
    }
    pub mod pool_creation_fee_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("CYTDCTNLEaBFD5GLs6MyaoVh4nqozH2B4vNPWEBgNBsX");
    }
}
#[cfg(feature = "devnet")]
pub mod protocol_account_config {
    use anchor_lang::prelude::declare_id;
    pub mod owner_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("6GM1A5oYPLYn1c1sX1YmiUhWRP9yV5BgGaFTBf58XvdX");
    }
    pub mod pool_creation_fee_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("CYTDCTNLEaBFD5GLs6MyaoVh4nqozH2B4vNPWEBgNBsX");
    }
}
#[cfg(not(any(feature = "localnet", feature = "devnet")))]
pub mod protocol_account_config {
    pub mod owner_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("4pzsDpf674wzgSdn5oRzGLBTd9gWgWoHqycMymg9hmPq");
    }
    pub mod pool_creation_fee_wallet {
        use anchor_lang::prelude::declare_id;
        declare_id!("EwyiKUDFcQp8wCFMYjYYb933SzEPea4HwSDLsV6Mvyv6");
    }
}

pub mod owner_wallet_devnet {
    use anchor_lang::prelude::declare_id;
    declare_id!("6GM1A5oYPLYn1c1sX1YmiUhWRP9yV5BgGaFTBf58XvdX");
}

pub mod owner_wallet_mainnet {
    use anchor_lang::prelude::declare_id;
    declare_id!("4pzsDpf674wzgSdn5oRzGLBTd9gWgWoHqycMymg9hmPq");
}

pub mod pool_creation_fee_wallet_devnet {
    use anchor_lang::prelude::declare_id;
    declare_id!("CYTDCTNLEaBFD5GLs6MyaoVh4nqozH2B4vNPWEBgNBsX");
}

pub mod pool_creation_fee_wallet_mainnet {
    use anchor_lang::prelude::declare_id;
    declare_id!("EwyiKUDFcQp8wCFMYjYYb933SzEPea4HwSDLsV6Mvyv6");
}

pub fn get_pool_creation_fee_wallet(is_devnet: bool) -> Pubkey {
    if is_devnet {
        pool_creation_fee_wallet_devnet::ID
    } else {
        pool_creation_fee_wallet_mainnet::ID
    }
}

pub fn get_owner_wallet(is_devnet: bool) -> Pubkey {
    if is_devnet {
        owner_wallet_devnet::ID
    } else {
        owner_wallet_mainnet::ID
    }
}

pub mod stable_coin {
    pub mod wsol {
        use anchor_lang::prelude::declare_id;
        declare_id!("So11111111111111111111111111111111111111112");
    }

    pub mod usdc {
        use anchor_lang::prelude::declare_id;
        declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    }

    pub mod usdt {
        use anchor_lang::prelude::declare_id;
        declare_id!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
    }
}

pub const TEN_THOUSAND: u64 = 10_000;

#[event]
pub struct UserDefinedEvent {
    #[index]
    pub liquidity_pool_id: Pubkey,
    pub instruction_name: String,
    pub base64_data: String,
}
