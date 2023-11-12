use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use ethers_providers::{JsonRpcClient, Middleware};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::future::poll_fn;
use utils_log::{debug, error, info, warn};
use utils_types::sma::{NoSumSMA, SMA};

use crate::{BlockWithLogs, EventsQuery, HoprIndexerRpcOperations, Log};
use crate::rpc::{HoprMiddleware, RpcOperations};
use crate::errors::RpcError::{GeneralError, NoSuchBlock};
use crate::errors::Result;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::{sleep, spawn_local};

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;


async fn send_block_with_logs(tx: &mut UnboundedSender<BlockWithLogs>, block: BlockWithLogs) -> Result<()> {
    match poll_fn(|cx| Pin::new(&tx).poll_ready(cx)).await {
        Ok(_) => {
            match tx.start_send(block) {
                Ok(_) => Ok(()),
                Err(_) => Err(GeneralError("failed to pass block with logs to the receiver".into()))
            }
        }
        Err(_) => Err(GeneralError("receiver has been closed".into()))
    }
}

async fn get_block_with_logs_from_provider<P: JsonRpcClient + 'static>(block_number: u64, filters: &Vec<EventsQuery>, provider: Arc<HoprMiddleware<P>>) -> Result<BlockWithLogs> {
    debug!("getting block #{block_number} with logs");
    match provider.get_block(block_number).await? {
        Some(block) => {
            let mut logs = Vec::new();
            for filter in filters {
                debug!("getting logs from #{block_number} for {filter}");

                let mut contract_logs = Vec::new();
                for mut query in Vec::<ethers::types::Filter>::from(filter.clone()) {
                    provider.get_logs(&query.from_block(block_number).to_block(block_number))
                        .await?
                        .into_iter()
                        .map(crate::Log::from)
                        .for_each(|log| contract_logs.push(log));
                }

                debug!("retrieved {} logs of {filter}", contract_logs.len());
                logs.push((filter.address, contract_logs));
            }

            Ok(BlockWithLogs {
                block: block.into(),
                logs
            })
        },
        None => Err(NoSuchBlock)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprIndexerRpcOperations for RpcOperations<P> {
    async fn block_number(&self) -> Result<u64> {
        let r = self.provider.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn poll_blocks_with_logs(&self, start_block_number: Option<u64>, filters: Vec<EventsQuery>) -> Result<UnboundedReceiver<BlockWithLogs>> {
        let (mut tx, rx) = futures::channel::mpsc::unbounded::<BlockWithLogs>();

        // The provider internally performs retries on timeouts and errors.
        let provider = self.provider.clone();
        let latest_block = self.block_number().await?;
        let start_block = start_block_number.unwrap_or(latest_block);
        let initial_poll_backoff = self.cfg.expected_block_time;

        spawn_local(async move {
            let mut current = start_block;
            let mut latest  = latest_block;

            let mut block_time_sma = NoSumSMA::<Duration, u32>::new(10);
            let mut prev_block = None;

            while current < latest_block {
                match get_block_with_logs_from_provider(current, &filters, provider.clone()).await {
                    Ok(block_with_logs) => {
                        let new_current = block_with_logs.block.number.expect("past block must not be pending");
                        let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                        info!("got past {block_with_logs}");

                        if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                            error!("failed to dispatch past block: {e}");
                            break;  // Receiver is closed, terminate the stream
                        }

                        if let Some(prev_block_ts) = prev_block {
                            block_time_sma.add_sample( block_ts - prev_block_ts);
                        }

                        prev_block = Some(block_ts);
                        current = new_current;

                        match provider.get_block_number().await.map(|u| u.as_u64()) {
                            Ok(block_number) => {
                                latest = block_number;
                            },
                            Err(e) => {
                                error!("failed to get latest block number: {e}");
                            }
                        }
                    },
                    Err(e) => {
                        error!("failed to obtain block #{current} with logs: {e}");
                    }
                }
            }
            if current >= latest {
                let mut current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                debug!("done receiving past blocks {start_block}-{latest}, polling for new blocks > {current} with initial backoff {} ms", current_backoff.as_millis());
                loop {
                    match get_block_with_logs_from_provider(current + 1, &filters, provider.clone()).await {
                        Ok(block_with_logs) => {
                            let new_current = block_with_logs.block.number.expect("new block must not be pending");
                            let block_ts = Duration::from_secs(block_with_logs.block.timestamp.as_u64());
                            info!("got new {block_with_logs}");

                            if let Err(e) = send_block_with_logs(&mut tx, block_with_logs).await {
                                error!("failed to dispatch new block: {e}");
                                break; // Receiver is closed, terminate the stream
                            }

                            if let Some(prev_block_ts) = prev_block {
                                block_time_sma.add_sample( block_ts - prev_block_ts);
                            }

                            prev_block = Some(block_ts);
                            current = new_current;
                            current_backoff = block_time_sma.get_average().unwrap_or(initial_poll_backoff);
                        }
                        Err(NoSuchBlock) => {
                            let next_wait = current_backoff.min(Duration::from_millis(100));
                            debug!("no block #{current}, waiting {} ms", next_wait.as_millis());

                            sleep(next_wait).await;
                            current_backoff /= 2;
                        }
                        Err(e) => {
                            error!("failed to obtain block {} with logs: {e}", current + 1);
                        }
                    }
                }
            } else {
                error!("processing past blocks did not get up to the latest block {current} < {latest}");
            }

            warn!("block processing done");
            tx.close_channel();
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;
    use ethers::abi::Token;
    use ethers::contract::EthEvent;
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::middleware::SignerMiddleware;
    use ethers::signers::{LocalWallet, Signer, Wallet};
    use ethers::types::{Bytes, Eip1559TransactionRequest};
    use ethers::utils::{Anvil, AnvilInstance};
    use ethers_providers::{Http, Middleware, Provider};
    use futures::StreamExt;
    use hex_literal::hex;
    use primitive_types::H160;
    use bindings::hopr_announcements::HoprAnnouncements;
    use bindings::hopr_channels::{ChannelOpenedFilter, HoprChannels};
    use bindings::hopr_node_safe_registry::HoprNodeSafeRegistry;
    use bindings::hopr_token::HoprToken;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_ethereum_misc::ContractAddresses;
    use utils_types::primitives::Address;
    use utils_types::traits::BinarySerializable;
    use crate::{EventsQuery, HoprIndexerRpcOperations};
    use crate::rpc::{RpcOperations};
    use crate::rpc::tests::{mock_config};

    fn get_provider() -> (AnvilInstance, Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>) {
        let anvil: AnvilInstance = Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .spawn();
        let wallet: LocalWallet = anvil.keys()[0].clone().into();

        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(std::time::Duration::from_millis(10u64));

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));
        let client = Arc::new(client);

        (anvil, client)
    }

    pub async fn deploy_contracts(client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, self_addr: Address) -> ContractAddresses {
        // Node safe registry
        let node_safe_registry = HoprNodeSafeRegistry::deploy(client.clone(), ()).unwrap().send().await.unwrap();

        let mut tx = Eip1559TransactionRequest::new();
        tx = tx.to(H160::from_str("a990077c3205cbDf861e17Fa532eeB069cE9fF96").unwrap());
        tx = tx.value(80000000000000000u128);

        client.send_transaction(tx, None).await.unwrap();

        client.send_raw_transaction(
            hex!("f90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820")
                .into()).await.unwrap();

        let self_address = ethers::types::Address::from_slice(&self_addr.to_bytes());

        // Hopr Token contract
        let hopr_token = HoprToken::deploy(client.clone(), ()).unwrap().send().await.unwrap();
        hopr_token
            .grant_role(hopr_token.minter_role().await.unwrap(), self_address)
            .send()
            .await
            .unwrap();

        // Mint us 1000 HOPR
        hopr_token
            .mint(self_address, 1000_u32.into(), Bytes::new(), Bytes::new())
            .send()
            .await
            .unwrap();

        // Deploy channels contract
        let hopr_channels = HoprChannels::deploy(client.clone(), Token::Tuple(vec![
                Token::Address(hopr_token.address()),
                Token::Uint(1_u32.into()),
                Token::Address(node_safe_registry.address()),
            ]),
        )
        .unwrap()
        .send()
        .await
        .unwrap();

        let hopr_announcements = HoprAnnouncements::deploy(client.clone(), Token::Address(node_safe_registry.address()))
            .unwrap()
            .send()
            .await
            .unwrap();

        ContractAddresses {
            token: hopr_token.address().0.into(),
            channels: hopr_channels.address().0.into(),
            announcements: hopr_announcements.address().0.into(),
            network_registry: Default::default(),
            safe_registry: node_safe_registry.address().0.into(),
            price_oracle: Default::default(),
        }
    }

    async fn fund_channel(
        counterparty: Address,
        hopr_token: HoprToken<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        hopr_channels: HoprChannels<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) {
        hopr_token
            .approve(hopr_channels.address(), 1u128.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_channels
            .fund_channel(ethers::types::Address::from_slice(&counterparty.to_bytes()), 1u128)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    async fn deploy_anvil_and_contracts() -> (AnvilInstance, ContractAddresses) {
        let (anvil, client) = get_provider();
        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let addrs = deploy_contracts(client.clone(), chain_key_1.public().to_address()).await;
        (anvil, addrs)
    }

    #[tokio::test]
    async fn test_poll_with_logs() {
        let _ = env_logger::builder().is_test(true).try_init();

        let (anvil, addrs) = deploy_anvil_and_contracts().await;

        let chain_key_1 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_2 = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();

        let mut cfg = mock_config();
        cfg.contract_addrs = addrs;

        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_1, cfg).expect("failed to construct rpc");

        /*rpc.token
            .approve(rpc.channels.address(), 1u128.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        rpc.channels
            .fund_channel(chain_key_2.public().to_address().into(), 1u128)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
*/
        let blocks = rpc.poll_blocks_with_logs(Some(1), vec![EventsQuery {
            address: addrs.channels,
            topics: vec![ChannelOpenedFilter::signature()],
        }])
        .await
        .unwrap()
        .take(3)
        .collect::<Vec<_>>()
        .await;
    }

}