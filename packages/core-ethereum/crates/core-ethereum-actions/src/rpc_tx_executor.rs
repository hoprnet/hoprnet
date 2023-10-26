// TODO: move this to core-ethereum-api once #5623 is merged

use std::sync::Arc;
use async_lock::Mutex;
use async_trait::async_trait;
use ethers::prelude::Eip1559TransactionRequest;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use primitive_types::H160;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_ethereum_misc::ContractAddresses;
use core_ethereum_rpc::HoprRpcOperations;
use core_crypto::types::Hash;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::announcement::AnnouncementData;
use utils_types::primitives::{Address, Balance, BalanceType};

use crate::payload::{BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator};
use crate::transaction_queue::{TransactionExecutor, TransactionResult};
use crate::errors::Result;

pub struct RpcTransactionExecutor<Rpc: HoprRpcOperations> {
    rpc: Arc<Mutex<Rpc>>,
    safe_generator: SafePayloadGenerator,
    basic_generator: BasicPayloadGenerator,
    contract_addrs: ContractAddresses,
    node_module: Option<Address>,
}

impl<Rpc: HoprRpcOperations> RpcTransactionExecutor<Rpc> {
    pub fn new(rpc: Arc<Mutex<Rpc>>, chain_keypair: &ChainKeypair, contract_addrs: ContractAddresses, node_module: Option<Address>) -> Self {
        Self {
            rpc,
            contract_addrs,
            node_module,
            safe_generator: SafePayloadGenerator::new(chain_keypair, contract_addrs),
            basic_generator: BasicPayloadGenerator::new(chain_keypair.public().to_address())
        }
    }

    /// Submits transaction and awaits until it is confirmed mined by the indexer
    async fn process_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        let hash = self.rpc.lock().await.send_transaction(tx).await?;
        // TODO: send request to indexer to await this TX
        Ok(hash)
    }
}

#[async_trait(? Send)]
impl<Rpc: HoprRpcOperations> TransactionExecutor for RpcTransactionExecutor<Rpc> {
    async fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = self.node_module {
            tx.set_data(match self.safe_generator.redeem_ticket(&acked_ticket) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.redeem_ticket(&acked_ticket) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(self.contract_addrs.channels));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::TicketRedeemed { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = self.node_module {
            tx.set_data(match self.safe_generator.fund_channel(&destination, &balance) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.fund_channel(&destination, &balance) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(self.contract_addrs.channels));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::ChannelFunded { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = self.node_module {
            tx.set_data(match self.safe_generator.initiate_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.initiate_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(self.contract_addrs.channels));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::ChannelClosureInitiated { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = self.node_module {
            tx.set_data(match self.safe_generator.finalize_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.finalize_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(self.contract_addrs.channels));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn close_incoming_channel(&self, src: Address) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = self.node_module {
            tx.set_data(match self.safe_generator.close_incoming_channel(&src) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.close_incoming_channel(&src) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.contract_addrs.channels));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        match amount.balance_type() {
            BalanceType::HOPR => {
                tx.set_data(match self.basic_generator.transfer(&recipient, &amount) {
                    Ok(payload) => payload.into(),
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                });
                tx.set_to(H160::from(self.contract_addrs.token));
            }
            BalanceType::Native => {
                tx.set_to(H160::from(recipient));
                tx.set_value(ethers::types::U256(primitive_types::U256::from(amount.value()).0));
            }
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::Withdrawn { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn announce(&self, data: AnnouncementData, use_node_module: Option<Address>) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        if let Some(node_module) = use_node_module {
            assert!(self.node_module.is_none() || self.node_module.unwrap().eq(&node_module), "inconsistent node module address");

            tx.set_data(match self.safe_generator.announce(&data) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(node_module));
        } else {
            tx.set_data(match self.basic_generator.announce(&data) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });

            tx.set_to(H160::from(self.contract_addrs.announcements));
        }

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::Announced { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }

    async fn register_safe(&self, safe_address: Address) -> TransactionResult {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        tx.set_data(match self.basic_generator.register_safe_by_node(&safe_address) {
            Ok(payload) => payload.into(),
            Err(e) => return TransactionResult::Failure(e.to_string()),
        });
        tx.set_to(H160::from(self.contract_addrs.safe_registry));

        match self.process_transaction(tx).await {
            Ok(tx_hash) => TransactionResult::SafeRegistered { tx_hash },
            Err(e) => TransactionResult::Failure(e.to_string())
        }
    }
}