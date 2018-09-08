//TODO: <IOLITE> copyright
use std::ops::Deref;
use executive::{Executive, TransactOptions};
use transaction::{SignedTransaction};
use ethereum_types::{U256, Address};
use state::{Backend as StateBackend};
use rlp::{self};

use types::metalogs::MetaLogs;
use types::business_metadata::BusinessMetadata;
use meta::base_meta_executor::{BaseMetaExecutor, MetaExecute, Bytes};

pub struct BusinessMetaExecutor<'a, B: 'a + StateBackend> {
    executor: BaseMetaExecutor,

    transaction: &'a SignedTransaction,
    from: Address,
    read_evm: &'a mut Executive<'a, B>,
}

impl<'a, B: 'a + StateBackend> Deref for BusinessMetaExecutor<'a, B> {
    type Target = BaseMetaExecutor;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}

impl<'a, B: 'a + StateBackend> BusinessMetaExecutor<'a, B> {
    pub fn new(metadata: Bytes, transaction: &'a SignedTransaction, from: Address, read_evm: &'a mut Executive<'a, B>)
            -> Self {
        BusinessMetaExecutor {
            executor: BaseMetaExecutor { metadata: metadata },
            transaction: transaction,
            from: from,
            read_evm: read_evm,
        }
    }
}

impl<'a, B: 'a + StateBackend> MetaExecute for BusinessMetaExecutor<'a, B> {
    fn execute(&mut self) -> Result<MetaLogs, String> {
        if self.metadata.len() == 0 {
            return Err("Error! Metadata is empty.".to_string());
        }

        //TODO: <IOLITE> implement BusinessMetadata
        let business_metadata: BusinessMetadata = rlp::decode(&self.metadata)
            .map_err(|err| err.to_string())?;
        info!("[iolite] Business metadata: {:#?}", business_metadata);

        //TODO: Copy fields from business metadata to tx
        let tx = self.transaction.get_copy_with_metadata_equals_data();
        let transact_options = TransactOptions::with_no_tracing();//with_tracing_and_vm_tracing();
        let result = match self.read_evm.transact_virtual(&tx, transact_options) {
            Ok(executed_result) => executed_result,
            Err(e) => return Err(e.to_string()),
        };

        info!("[iolite] Executed metadata: {:#?}", result.output);
        if result.output.len() != 64 {
            return Err("The business call result does not match the format (address, uint256)".to_string());
        }

        let mut metalogs = MetaLogs::new();
        //TODO: <IOLITE> should we convert address simillar to geth? `common.BytesToAddress(&result.output[:32])`
        metalogs.push(Address::from(&result.output[..32]), U256::from(&result.output[32..]));

        for data in metalogs.logs() {
            info!("[iolite] Decoded Metalogs. To: {}, Value: {}", data.recipient, data.amount);
        }

        Ok(metalogs)
    }
}
