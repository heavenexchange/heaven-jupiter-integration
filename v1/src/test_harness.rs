use anchor_lang::Discriminator;
use anyhow::Result;
use heaven_exchange::instructions::LiquidityPoolState;
use jupiter_amm_interface::{Amm, AmmContext, ClockRef, KeyedAccount, QuoteParams, SwapMode};
use solana_client::rpc_client::RpcClient;
use std::{collections::HashMap, sync::Arc};

use crate::amm::{HeavenAmm, PROGRAM_ID};

pub struct AmmTestHarness {
    pub client: RpcClient,
}

impl AmmTestHarness {
    pub fn new() -> Self {
        let rpc_string = "https://api.devnet.solana.com".to_string(); // env::var("RPC_URL").unwrap();
        let rpc_url = rpc_string.as_str();
        Self {
            client: RpcClient::new(rpc_url),
        }
    }
    pub fn get_all_keyed_account(&self) -> Result<Vec<KeyedAccount>> {
        let accounts = self.client.get_program_accounts(&PROGRAM_ID).unwrap();
        let keyed_accounts = &mut vec![];
        for (key, account) in accounts {
            let discriminator = LiquidityPoolState::discriminator();
            let data: &[u8] = account.data.as_ref();
            let acc_discriminator: &[u8] = &data[..8];
            if discriminator == acc_discriminator {
                println!("size: {}", data.len());
                keyed_accounts.push(KeyedAccount {
                    key,
                    account,
                    params: None,
                })
            }
        }
        Ok(keyed_accounts.clone())
    }

    pub fn update_amm(&self, amm: &mut dyn Amm) {
        let accounts_to_update = amm.get_accounts_to_update();

        let accounts_map = self
            .client
            .get_multiple_accounts(&accounts_to_update)
            .unwrap()
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut m, (index, account)| {
                if let Some(account) = account {
                    m.insert(accounts_to_update[index], account.clone());
                }
                m
            });
        amm.update(&accounts_map).unwrap();
    }
}

#[test]
fn test_quote() {
    use crate::test_harness::AmmTestHarness;
    use num::pow;

    let test_harness = AmmTestHarness::new();
    let all_keyed_account = test_harness.get_all_keyed_account().unwrap();

    let epoch_info = test_harness.client.get_epoch_info().unwrap();

    let clock_ref = ClockRef {
        epoch: Arc::new(epoch_info.epoch.into()),
        ..Default::default()
    };

    let context = AmmContext { clock_ref };

    for keyed_account in all_keyed_account {
        let amm = &mut HeavenAmm::from_keyed_account(&keyed_account, &context).unwrap();
        test_harness.update_amm(amm);

        println!(
            "Pool: {}, {}",
            amm.state.base_token_mint.to_string(),
            amm.state.quote_token_mint.to_string()
        );

        let in_amount = pow(10, usize::from(amm.state.quote_token_mint_decimals));
        let quote = amm
            .quote(&QuoteParams {
                input_mint: amm.state.quote_token_mint,
                amount: in_amount,
                output_mint: amm.state.base_token_mint,
                swap_mode: SwapMode::ExactIn,
            })
            .unwrap();

        println!(
            "  Token mints: from {}, to {}",
            amm.state.quote_token_mint.to_string(),
            amm.state.base_token_mint.to_string()
        );
        println!("  In amount: {}", in_amount);
        println!(
            "  Out amount: {:?}, Fee amount: {:?}",
            quote.out_amount, quote.fee_amount
        );
    }
}
