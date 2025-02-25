use std::{
    fs::File,
    path::{Path, PathBuf},
};

use aleph_client::{AccountId, Balance, TxStatus};
use clap::{clap_derive::ValueEnum, Args, Subcommand};
use primitives::{BlockNumber, CommitteeSeats, SessionIndex};
use serde::{Deserialize, Serialize};
use sp_core::H256;
#[cfg(feature = "liminal")]
use {
    crate::snark_relations::{
        parsing::parse_some_system, NonUniversalProvingSystem, RelationArgs, SomeProvingSystem,
        UniversalProvingSystem,
    },
    aleph_client::{
        pallet_baby_liminal::systems::ProvingSystem,
        pallets::baby_liminal::VerificationKeyIdentifier,
    },
};

#[derive(Debug, Clone, Args)]
pub struct ContractOptions {
    /// balance to transfer from the call origin to the contract
    #[clap(long, default_value = "0")]
    pub balance: Balance,
    /// The gas limit enforced when executing the constructor
    #[clap(long, default_value = "1000000000")]
    pub gas_limit: u64,
    /// The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed
    #[clap(long)]
    pub storage_deposit_limit: Option<Balance>,
}

#[derive(Debug, Clone, Args)]
pub struct ContractUploadCode {
    /// Path to the .wasm artifact
    #[clap(long, parse(from_os_str))]
    pub wasm_path: PathBuf,
    /// The maximum amount of balance that can be charged/reserved from the caller to pay for the storage consumed
    #[clap(long)]
    pub storage_deposit_limit: Option<Balance>,
}

#[derive(Debug, Clone, Args)]
pub struct ContractInstantiateWithCode {
    /// Path to the .wasm artifact
    #[clap(long, parse(from_os_str))]
    pub wasm_path: PathBuf,
    /// Path to the .json file with contract metadata (abi)
    #[clap(long, parse(from_os_str))]
    pub metadata_path: PathBuf,
    /// The name of the contract constructor to call
    #[clap(name = "constructor", long, default_value = "new")]
    pub constructor: String,
    /// The constructor arguments, encoded as strings, space separated
    #[clap(long, multiple_values = true)]
    pub args: Option<Vec<String>>,
    /// additional options
    #[clap(flatten)]
    pub options: ContractOptions,
}

#[derive(Debug, Clone, Args)]
pub struct ContractInstantiate {
    /// Code hash of the deployed contract
    #[clap(long, parse(try_from_str))]
    pub code_hash: H256,
    /// Path to the .wasm artifact
    #[clap(long, parse(from_os_str))]
    pub metadata_path: PathBuf,
    /// The name of the contract constructor to call
    #[clap(long, default_value = "new")]
    pub constructor: String,
    /// The constructor arguments, encoded as strings
    #[clap(long, multiple_values = true)]
    pub args: Option<Vec<String>>,
    /// additional options
    #[clap(flatten)]
    pub options: ContractOptions,
}

#[derive(Debug, Clone, Args)]
pub struct ContractCall {
    /// Address of the contract to call
    #[clap(long, parse(try_from_str))]
    pub destination: AccountId,
    /// Path to the .json fiel with contract metadata (abi)
    #[clap(long, parse(from_os_str))]
    pub metadata_path: PathBuf,
    /// The name of the contract message to call
    #[clap(long)]
    pub message: String,
    /// The message arguments, encoded as strings
    #[clap(long, multiple_values = true)]
    pub args: Option<Vec<String>>,
    /// additional options
    #[clap(flatten)]
    pub options: ContractOptions,
}

#[derive(Debug, Clone, Args)]
pub struct ContractOwnerInfo {
    /// Code hash of the contract code
    #[clap(long, parse(try_from_str))]
    pub code_hash: H256,
}

#[derive(Debug, Clone, Args)]
pub struct ContractRemoveCode {
    /// Code hash of the contract code
    #[clap(long, parse(try_from_str))]
    pub code_hash: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeValidatorArgs {
    pub reserved_validators: Option<Vec<AccountId>>,
    pub non_reserved_validators: Option<Vec<AccountId>>,
    pub committee_size: Option<CommitteeSeats>,
}

pub type Version = u32;

impl std::str::FromStr for ChangeValidatorArgs {
    type Err = serde_json::Error;

    fn from_str(change_validator_args: &str) -> Result<Self, Self::Err> {
        let path = Path::new(change_validator_args);
        if path.exists() {
            let file = File::open(path).expect("Failed to open metadata file");
            return serde_json::from_reader(file);
        }
        serde_json::from_str(change_validator_args)
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExtrinsicState {
    InBlock,
    Finalized,
}

impl From<ExtrinsicState> for TxStatus {
    fn from(state: ExtrinsicState) -> Self {
        match state {
            ExtrinsicState::InBlock => TxStatus::InBlock,
            ExtrinsicState::Finalized => TxStatus::Finalized,
        }
    }
}

#[cfg(feature = "liminal")]
#[derive(Debug, Clone, Subcommand)]
pub enum BabyLiminal {
    /// Store a verification key under an identifier in the pallet's storage.
    StoreKey {
        /// The key identifier.
        #[clap(long, value_parser(parsing::parse_identifier))]
        identifier: VerificationKeyIdentifier,

        /// Path to a file containing the verification key.
        #[clap(long)]
        vk_file: PathBuf,
    },

    /// Delete the verification key under an identifier in the pallet's storage.
    DeleteKey {
        /// The key identifier.
        #[clap(long, value_parser(parsing::parse_identifier))]
        identifier: VerificationKeyIdentifier,
    },

    /// Overwrite the verification key under an identifier in the pallet's storage.
    OverwriteKey {
        /// The key identifier.
        #[clap(long, value_parser(parsing::parse_identifier))]
        identifier: VerificationKeyIdentifier,

        /// Path to a file containing the verification key.
        #[clap(long)]
        vk_file: PathBuf,
    },

    /// Verify a proof against public input with a stored verification key.
    Verify {
        /// The key identifier.
        #[clap(long, value_parser(parsing::parse_identifier))]
        identifier: VerificationKeyIdentifier,

        /// Path to a file containing the proof.
        #[clap(long)]
        proof_file: PathBuf,

        /// Path to a file containing the public input.
        #[clap(long)]
        input_file: PathBuf,

        /// The proving system to be used.
        #[clap(long, value_parser(parsing::parse_system))]
        system: ProvingSystem,
    },
}

#[cfg(feature = "liminal")]
#[derive(Debug, Clone, Subcommand)]
pub enum SnarkRelation {
    GenerateSrs {
        /// Proving system to use.
        #[clap(long, short, value_enum, default_value = "marlin")]
        system: UniversalProvingSystem,

        /// Maximum supported number of constraints.
        #[clap(long, default_value = "10000")]
        num_constraints: usize,

        /// Maximum supported number of variables.
        #[clap(long, default_value = "10000")]
        num_variables: usize,

        /// Maximum supported polynomial degree.
        #[clap(long, default_value = "10000")]
        degree: usize,
    },

    /// Generate verifying and proving key from SRS and save them to separate binary files.
    GenerateKeysFromSrs {
        ///Relation to work with.
        #[clap(subcommand)]
        relation: RelationArgs,

        /// Proving system to use.
        #[clap(long, short, value_enum, default_value = "marlin")]
        system: UniversalProvingSystem,

        /// Path to a file containing SRS.
        #[clap(long)]
        srs_file: PathBuf,
    },

    /// Generate verifying and proving key and save them to separate binary files.
    GenerateKeys {
        /// Relation to work with.
        #[clap(subcommand)]
        relation: RelationArgs,

        /// Proving system to use.
        #[clap(long, short, value_enum, default_value = "groth16")]
        system: NonUniversalProvingSystem,
    },

    /// Generate proof and public input and save them to separate binary files.
    GenerateProof {
        /// Relation to work with.
        #[clap(subcommand)]
        relation: RelationArgs,

        /// Proving system to use.
        ///
        /// Accepts either `NonUniversalProvingSystem` or `UniversalProvingSystem`.
        #[clap(long, short, value_enum, default_value = "groth16", value_parser = parse_some_system)]
        system: SomeProvingSystem,

        /// Path to a file containing proving key.
        #[clap(long, short)]
        proving_key_file: PathBuf,
    },

    /// Verify proof.
    Verify {
        /// Path to a file containing verifying key.
        #[clap(long, short)]
        verifying_key_file: PathBuf,

        /// Path to a file containing proof.
        #[clap(long, short)]
        proof_file: PathBuf,

        /// Path to a file containing public input.
        #[clap(long, short)]
        public_input_file: PathBuf,

        /// Proving system to use.
        ///
        /// Accepts either `NonUniversalProvingSystem` or `UniversalProvingSystem`.
        #[clap(long, short, value_enum, default_value = "groth16", value_parser = parse_some_system)]
        system: SomeProvingSystem,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Staking call to bond stash with controller
    Bond {
        /// SS58 id of the controller account
        #[clap(long)]
        controller_account: String,

        /// a Stake to bond (in tokens)
        #[clap(long)]
        initial_stake_tokens: u32,
    },

    /// Change the validator set for the session after the next
    ChangeValidators {
        /// The new reserved validators list
        #[clap(long)]
        change_validators_args: ChangeValidatorArgs,
    },

    /// Force new era in staking world. Requires sudo.
    ForceNewEra,

    /// Finalize the specified block using seed as emergency finalizer.
    Finalize {
        /// Block number to finalize.
        #[clap(long)]
        block: BlockNumber,

        /// Block hash to finalize either with or without leading '0x'.
        #[clap(long)]
        hash: String,

        /// The seed of the key to use as emergency finalizer key.
        /// If not given, a user is prompted to provide finalizer seed
        #[clap(long)]
        finalizer_seed: Option<String>,
    },

    /// Sets seed as the emergency finalizer. Requires sudo.
    SetEmergencyFinalizer {
        /// The seed of the key to use as emergency finalizer key.
        /// If not given, a user is prompted to provide finalizer seed
        #[clap(long)]
        finalizer_seed: Option<String>,
    },

    /// Gets next session keys for a validator with specified AccountId
    NextSessionKeys {
        /// SS58 id of the validator for which we want to retrieve the keys
        #[clap(long)]
        account_id: String,
    },

    /// Declare the desire to nominate target account
    Nominate {
        #[clap(long)]
        nominee: String,
    },

    /// Associate the node with a specific staking account.
    PrepareKeys,

    /// Call rotate_keys() RPC call and prints them to stdout
    RotateKeys,

    /// Sets given keys for origin controller
    SetKeys {
        /// 64 byte hex encoded string in form 0xaabbcc..
        /// where aabbcc...  must be exactly 128 characters long
        #[clap(long)]
        new_keys: String,
    },

    /// Command to convert given seed to SS58 Account id
    SeedToSS58 {
        /// Seed which will be converted.
        /// If not given, a user is prompted to provide finalizer seed
        #[clap(long)]
        input: Option<String>,
    },

    /// Sets lower bound for nominator and validator. Requires root account.
    SetStakingLimits {
        /// Nominator lower bound
        #[clap(long)]
        minimal_nominator_stake: u64,

        /// Validator lower bound
        #[clap(long)]
        minimal_validator_stake: u64,

        /// Maximum number of nominators
        #[clap(long)]
        max_nominators_count: Option<u32>,

        /// Maximum number of validators
        #[clap(long)]
        max_validators_count: Option<u32>,
    },

    /// Transfer funds via balances pallet
    Transfer {
        /// Number of tokens to send,
        #[clap(long)]
        amount_in_tokens: u64,

        /// SS58 id of target account
        #[clap(long)]
        to_account: String,
    },

    /// Make a proposal to the treasury.
    TreasuryPropose {
        /// How many tokens we intend to give to the beneficiary.
        #[clap(long)]
        amount_in_tokens: u64,

        /// SS58 id of the beneficiary account.
        #[clap(long)]
        beneficiary: String,
    },

    /// Approve proposal to the treasury.
    TreasuryApprove {
        /// Id of the proposal to be approved.
        #[clap(long)]
        proposal_id: u32,
    },

    /// Reject proposal to the treasury.
    TreasuryReject {
        /// Id of the proposal to be rejected.
        #[clap(long)]
        proposal_id: u32,
    },

    /// Send new runtime (requires sudo account)
    UpdateRuntime {
        #[clap(long)]
        /// Path to WASM file with runtime
        runtime: String,
    },

    /// Call staking validate call for a given controller
    Validate {
        /// Validator commission percentage
        #[clap(long)]
        commission_percentage: u8,
    },

    /// Update vesting for the calling account.
    Vest,

    /// Update vesting on behalf of the given account.
    VestOther {
        /// Account seed for which vesting should be performed.
        #[clap(long)]
        vesting_account: String,
    },

    /// Transfer funds via balances pallet
    VestedTransfer {
        /// Number of tokens to send.
        #[clap(long)]
        amount_in_tokens: u64,

        /// Seed of the target account.
        #[clap(long)]
        to_account: String,

        /// How much balance (in rappens, not in tokens) should be unlocked per block.
        #[clap(long)]
        per_block: Balance,

        /// Block number when unlocking should start.
        #[clap(long)]
        starting_block: BlockNumber,
    },

    /// Deploys a new contract, returns its code hash and the AccountId of the instance.
    ///
    /// Contract cannot already exist on-chain
    /// API signature: https://polkadot.js.org/docs/substrate/extrinsics/#instantiatewithcodevalue-compactu128-gas_limit-compactu64-storage_deposit_limit-optioncompactu128-code-bytes-data-bytes-salt-bytes
    ContractInstantiateWithCode(ContractInstantiateWithCode),

    /// Uploads new code without instantiating a contract from it, returns its code hash.
    ///
    /// API signature: https://polkadot.js.org/docs/substrate/extrinsics/#uploadcodecode-bytes-storage_deposit_limit-optioncompactu128
    ContractUploadCode(ContractUploadCode),

    ///  Instantiates a contract from a previously deployed wasm binary, returns the AccountId of the instance.
    ///
    /// API signature: https://polkadot.js.org/docs/substrate/extrinsics/#instantiatevalue-compactu128-gas_limit-compactu64-storage_deposit_limit-optioncompactu128-code_hash-h256-data-bytes-salt-bytes
    ContractInstantiate(ContractInstantiate),

    /// Calls a contract.
    ///
    /// API signature: https://polkadot.js.org/docs/substrate/extrinsics/#calldest-multiaddress-value-compactu128-gas_limit-compactu64-storage_deposit_limit-optioncompactu128-data-bytes
    ContractCall(ContractCall),

    /// Returns OwnerInfo if code hash is stored on chain
    ContractOwnerInfo(ContractOwnerInfo),

    /// Removes the code stored under code_hash and refund the deposit to its owner.
    ///
    /// Code can only be removed by its original uploader (its owner) and only if it is not used by any contract.
    /// API signature: https://polkadot.js.org/docs/substrate/extrinsics/#removecodecode_hash-h256
    ContractRemoveCode(ContractRemoveCode),

    /// Schedules a version upgrade of the network.
    VersionUpgradeSchedule {
        #[clap(long)]
        version: Version,

        #[clap(long)]
        session: SessionIndex,

        #[clap(long, value_enum, default_value_t=ExtrinsicState::Finalized)]
        expected_state: ExtrinsicState,
    },

    /// Interact with `pallet_baby_liminal`.
    #[cfg(feature = "liminal")]
    #[clap(subcommand)]
    BabyLiminal(BabyLiminal),

    /// Interact with `relations` crate.
    ///
    /// Inner object is boxed, because it is significantly bigger than any other variant (clippy).
    #[cfg(feature = "liminal")]
    #[clap(subcommand)]
    SnarkRelation(Box<SnarkRelation>),
}

#[cfg(feature = "liminal")]
mod parsing {
    use aleph_client::{
        pallet_baby_liminal::systems::ProvingSystem,
        pallets::baby_liminal::VerificationKeyIdentifier,
    };
    use anyhow::anyhow;

    /// Try to convert `&str` to `VerificationKeyIdentifier`.
    ///
    /// We handle one, most probable error type ourselves (i.e. incorrect length) to give a better
    /// message than the default `"could not convert slice to array"`.
    pub fn parse_identifier(ident: &str) -> anyhow::Result<VerificationKeyIdentifier> {
        match ident.len() {
            4 => Ok(ident.as_bytes().try_into()?),
            _ => Err(anyhow!(
                "Identifier has an incorrect length (should be 4 characters)"
            )),
        }
    }

    /// Try to convert `&str` to `ProvingSystem`.
    pub fn parse_system(system: &str) -> anyhow::Result<ProvingSystem> {
        match system.to_lowercase().as_str() {
            "groth16" => Ok(ProvingSystem::Groth16),
            "gm17" => Ok(ProvingSystem::Gm17),
            "marlin" => Ok(ProvingSystem::Marlin),
            _ => Err(anyhow!("Unknown proving system")),
        }
    }
}
