//TODO: <IOLITE> copyright
use std::{fmt};
use types::metalogs::MetaLogs;
use transaction::{SignedTransaction};
use ethereum_types::{U256, Address};
use executive::Executive;
use state::{Backend as StateBackend};

use meta::base_meta_payer::{PaymentOptions, MetaPayable, MetaPay};
use meta::simple_meta_payer::SimpleMetaPayer;
use meta::business_meta_payer::BusinessMetaPayer;
use meta::base_meta_executor::MetaExecute;
use meta::simple_meta_executor::SimpleMetaExecutor;
use meta::business_meta_executor::BusinessMetaExecutor;

type Bytes = Vec<u8>;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum MetaUtilError {
    // Insufficient funds: (provided: u64, expected: u64)
    InsufficientFunds,
    //TODO: <IOLITE> since we don't have `IntrinsicGas()` method in parity
    // this error could be not relevant or redundant
    // Intrinsic gas (provided: u64, expected: u64)
    IntrinsicGasFailed,
}

impl fmt::Display for MetaUtilError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            MetaUtilError::InsufficientFunds => {
                format!("insufficient funds for metadata payment or payment are not allowed.")
            },
            MetaUtilError::IntrinsicGasFailed => {
                format!("Metadata intrinsic gas error.")
            },
        };

        f.write_fmt(format_args!("MetaUtilError({})", msg))
    }
}

pub fn unpack_simple_metadata<'a, T: 'a + StateBackend>(from: Address, metadata: Bytes, meta_limit: U256, read_evm: &'a mut Executive<'a, T>)//read_evm: Executive)
        // return (payer, meta_logs, payment, intrinsic_gas)
        -> Result<(SimpleMetaPayer<'a, T>, MetaLogs, U256, u64), String/*MetaUtilError*/> {
    println!("[iolite] UnpackSimpleMetadata. Metalimit={}", meta_limit);

    let mut executor = SimpleMetaExecutor::new(metadata);

    //TODO: <IOLITE> do we really need this?
    let executor_gas = executor.intrinsic_gas()?;

    let meta_logs = executor.execute()?;

    //TODO: <Kirill A> get rid of clonning metalogs. Use reference of metalogs in payers instead.
    let mut payer = SimpleMetaPayer::new(from, meta_logs.clone(), meta_limit, read_evm);
    //TODO: <IOLITE> do we really need this?
    let payer_gas = payer.intrinsic_gas()?;

    let payment = match payer.can_pay() {
        PaymentOptions::CanPay(payment, ) => payment,
        PaymentOptions::CantPay => return Err(MetaUtilError::InsufficientFunds.to_string()),
    };

    let payment = match payer.pay(payer_gas) {
        Ok((paid_amount, _)) => paid_amount,
        _ => return Err(MetaUtilError::InsufficientFunds.to_string()),
    };

    //TODO: <IOLITE> do we really need this?
    let intrinsic_gas = executor_gas + payer_gas;
    if intrinsic_gas < executor_gas {
        return Err(MetaUtilError::IntrinsicGasFailed.to_string());
    }

    Ok((payer, meta_logs, payment, intrinsic_gas))
}


pub fn unpack_business_metadata<'a, T: 'a + StateBackend>(from: Address, metadata: Bytes, meta_limit: U256,
                                                          transaction: &'a SignedTransaction,
                                                          read_evm: &'a mut Executive<'a, T>,
                                                          write_evm: &'a mut Executive<'a, T>)
                            //read_state: State, write_state: State)
        // return (payer, meta_logs, payment, intrinsic_gas)
        -> Result<(BusinessMetaPayer<'a, T>, MetaLogs, U256, u64), String/*MetaUtilError*/> {
    println!("[iolite] UnpackBusinessMetadata. Metalimit={}", meta_limit);

    let mut executor = BusinessMetaExecutor::new(metadata, transaction, from, read_evm);

    //TODO: <IOLITE> do we really need this?
    let executor_gas = executor.intrinsic_gas()?;

    let meta_logs = executor.execute()?;

    //TODO: <Kirill A> get rid of clonning metalogs. Use reference of metalogs in payers instead.
    let payer = BusinessMetaPayer::new(from, meta_logs.clone(), meta_limit, transaction, write_evm);
    //TODO: <IOLITE> do we really need this?
    let payer_gas = payer.intrinsic_gas()?;

    let payment = match payer.can_pay() {
        PaymentOptions::CanPay(payment) => payment,
        PaymentOptions::CantPay => return Err(MetaUtilError::InsufficientFunds.to_string()),
    };

    //TODO: <IOLITE> do we really need this?
    let intrinsic_gas = executor_gas + payer_gas;
    if intrinsic_gas < executor_gas {
        return Err(MetaUtilError::IntrinsicGasFailed.to_string());
    }

    Ok((payer, meta_logs, payment, intrinsic_gas))
}
