// Modified by 2021 Cycan Technologies for testing GVM-Bridge

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{U256, H160, };

use pallet_contracts::{
	BalanceOf, ContractInfoOf, Schedule,
	chain_extension::{
		Environment, Ext, SysConfig, RetVal,
		UncheckedFrom, InitState, 
	},
};

use codec::{Encode, Decode};
use sp_runtime::{
	traits::{BlakeTwo256, Hash, IdentityLookup, Convert, },
	testing::{Header, H256},
	AccountId32, Perbill, PerThing,
};

use frame_support::{
	assert_ok, parameter_types,  
	traits::{Currency, GenesisBuild},
	weights::{Weight, constants::WEIGHT_PER_SECOND},
	dispatch::{DispatchError}, 
};

use pretty_assertions::assert_eq;
use ink_env::call::{Selector, ExecutionInput};
use sha3::{Keccak256, Digest};

use pallet_evm::{
        FeeCalculator, HashedAddressMapping, EnsureAddressTruncated, Runner, 
		ExitReason, CallInfo, CreateInfo, SubstrateBlockHashMapping, 
};

use frame_system::pallet_prelude::*;

// use crate as gvm_bridge pallet
use crate as pallet_vm_bridge;


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},	
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Randomness: pallet_randomness_collective_flip::{Module, Call, Storage},	
		Contracts: pallet_contracts::{Module, Call, Config<T>, Storage, Event<T>},
		EVM: pallet_evm::{Module, Call, Config, Storage, Event<T>},
		GvmBridge: pallet_vm_bridge::{Module, Call, Config<T>, Storage, Event<T>},
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const Enable2EVM: bool = true;
	pub const Enable2WasmC: bool = true;
}

impl pallet_vm_bridge::Config for Test {
	type Currency = Balances;
	type Call = Call;
	type Event = Event;
	type Enable2EVM = Enable2EVM;
	type Enable2WasmC = Enable2WasmC;
}

/// Fixed gas price of `0`.
pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
        fn min_gas_price() -> U256 {
                0.into()
        }
}

impl pallet_evm_precompile_call_vm::EvmChainExtension<Test> for Test{
	fn call_vm4evm(
		origin: OriginFor<Test>,
		data: Vec<u8>,
		target_gas: Option<u64>
		) -> Result<(Vec<u8>, u64), sp_runtime::DispatchError>
	{
		GvmBridge::call_wasm4evm(origin, data, target_gas)
	}
}

parameter_types! {
        pub const ChainId: u64 = 42;
}

impl pallet_evm::Config for Test {
        type FeeCalculator = FixedGasPrice;
        type GasWeightMapping = ();
        type CallOrigin = EnsureAddressTruncated;
        type WithdrawOrigin = EnsureAddressTruncated;
        type AddressMapping = HashedAddressMapping<BlakeTwo256>;
        type Currency = Balances;
        type Event = Event;
        type Runner = pallet_evm::runner::stack::Runner<Self>;
        type Precompiles = (
                pallet_evm_precompile_simple::ECRecover,
                pallet_evm_precompile_simple::Sha256,
                pallet_evm_precompile_simple::Ripemd160,
                pallet_evm_precompile_simple::Identity,
				pallet_evm_precompile_call_vm::CallVm<Self>,
        );
        type ChainId = ChainId;
        type OnChargeTransaction = ();
		type BlockGasLimit = ();
		type BlockHashMapping = SubstrateBlockHashMapping<Self>;
}

//E



impl pallet_contracts::chain_extension::ChainExtension<Test> for Test{
    fn call<E>(func_id: u32, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = Test>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>
	{
		match func_id {
			0 => GvmBridge::call_evm4wasm::<E>(env),
			_ => Err(DispatchError::from("Passed unknown func_id to chain extension")),			
		}
	}
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(2 * WEIGHT_PER_SECOND);
	pub static ExistentialDeposit: u64 = 0;
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u128;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
parameter_types! {
	pub const SignedClaimHandicap: u64 = 2;
	pub const TombstoneDeposit: u64 = 16;
	pub const DepositPerContract: u64 = 8 * DepositPerStorageByte::get();
	pub const DepositPerStorageByte: u64 = 10_000;
	pub const DepositPerStorageItem: u64 = 10_000;
	pub RentFraction: Perbill = PerThing::from_rational(4u32, 10_000u32);
	pub const SurchargeReward: u64 = 500_000;
	pub const MaxDepth: u32 = 100;
	pub const MaxValueSize: u32 = 16_384;
	pub const DeletionQueueDepth: u32 = 1024;
	pub const DeletionWeightLimit: Weight = 500_000_000_000;
	pub const MaxCodeSize: u32 = 10 * 1024;
}

parameter_types! {
	pub const TransactionByteFee: u64 = 0;
}

impl Convert<Weight, BalanceOf<Self>> for Test {
	fn convert(w: Weight) -> BalanceOf<Self> {
		w.into()
	}
}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = Randomness;
	type Currency = Balances;
	type Event = Event;
	type RentPayment = ();
	type SignedClaimHandicap = SignedClaimHandicap;
	type TombstoneDeposit = TombstoneDeposit;
	type DepositPerContract = DepositPerContract;
	type DepositPerStorageByte = DepositPerStorageByte;
	type DepositPerStorageItem = DepositPerStorageItem;
	type RentFraction = RentFraction;
	type SurchargeReward = SurchargeReward;
	type MaxDepth = MaxDepth;
	type MaxValueSize = MaxValueSize;
	type WeightPrice = Self;
	type WeightInfo = ();
	type ChainExtension = Self;
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type MaxCodeSize = MaxCodeSize;
}

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);
//pub const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);
//pub const DJANGO: AccountId32 = AccountId32::new([4u8; 32]);

const GAS_LIMIT: Weight = 10_000_000_000;

pub struct ExtBuilder {
	existential_deposit: u64,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 1,
		}
	}
}
impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: u64) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn set_associated_consts(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_consts();
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![],
		}.assimilate_storage(&mut t).unwrap();
		pallet_contracts::GenesisConfig {
			current_schedule: Schedule::<Test> {
				enable_println: true,
				..Default::default()
			},
		}.assimilate_storage(&mut t).unwrap();
		pallet_evm::GenesisConfig{
			accounts: std::collections::BTreeMap::new(),
		}.assimilate_storage::<Test>(&mut t).unwrap();		
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Load a given wasm module represented by a .wat file and returns a wasm binary contents along
/// with it's hash.
///
/// The fixture files are located under the `fixtures/` directory.
fn compile_module<T>(
	fixture_name: &str,
) -> wat::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
	T: frame_system::Config,
{
	let fixture_path = ["fixtures/", fixture_name, ".wat"].concat();
	let wasm_binary = wat::parse_file(fixture_path)?;
	let code_hash = T::Hashing::hash(&wasm_binary);
	Ok((wasm_binary, code_hash))
}



use std::fs::File;
use std::io::Read;

fn read_a_file(filename: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    return Ok(data);
}

fn contract_module<T>(
	contract_name: &str,
) -> std::io::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
	T: frame_system::Config,
{
	let contract_path = ["test-contract/", contract_name].concat();
	
	let contract_binary = read_a_file(&contract_path)?;
	let code_hash = T::Hashing::hash(&contract_binary);
	Ok((contract_binary, code_hash))
}

// Perform test for wasm contract  calling  EVM contract
#[test]
fn test_wasm_call_evm(){
	
	// 1.  Get wasm and evm contract bin
	let (wasm, wasm_code_hash) = contract_module::<Test>("wasm_contract_test").unwrap();
	let (evm, _evm_code_hash) = contract_module::<Test>("evm_contract_test").unwrap();
	
	ExtBuilder::default()
	.existential_deposit(100)
	.build()
	.execute_with(|| {
		let _ = Balances::deposit_creating(&ALICE, 1_000_000);
		let subsistence = Contracts::subsistence_threshold();

		// 2. Create wasm contract
		let creation = Contracts::instantiate_with_code(
			Origin::signed(ALICE),
			subsistence * 100,
			GAS_LIMIT,
			wasm,
			vec![],
			vec![],
		);
		let wasm_addr = Contracts::contract_address(&ALICE, &wasm_code_hash, &[]);

		assert_ok!(creation);
		assert!(ContractInfoOf::<Test>::contains_key(&wasm_addr));	
		
		//3. Create EVM contract
		let source = H160::from_slice(&(AsRef::<[u8; 32]>::as_ref(&ALICE)[0..20]));
		
		//EVM::Config::CallOrigin::ensure_address_origin(&source, origin)?;
		
		let creation4evm = <Test as pallet_evm::Config>::Runner::create(   //EVM::create(
			//Origin::signed(ALICE),
			source,
			evm,
			U256::default(),
			1000000,
			Some(U256::default()),
			Some(U256::from(0)),
			<Test as pallet_evm::Config>::config(),
		);
		
		assert_ok!(&creation4evm);
		
		let evm_addr: H160;
		
		match creation4evm.unwrap() {
			CreateInfo {
				exit_reason: ExitReason::Succeed(_),
				value: create_address,
				..
			} => {
				evm_addr = create_address;
			},
			CreateInfo {
				exit_reason: _,
				value: _,
				..
			} => {
				panic!("Create EVM Contract failed!");
			},
		}
		
		
		//4. Get BOB balance of EVM token
		let balance_of_selector = &Keccak256::digest(b"balanceOf(address)")[0..4];
		
		let source_bob = H160::from_slice(&(AsRef::<[u8; 32]>::as_ref(&BOB)[0..20]));
			
		let fun_para: [u8;20] = source_bob.into();
		let balance_of_input = [&balance_of_selector[..], &fun_para, &[0u8,12]].concat();		
		
		let call4evm = <Test as pallet_evm::Config>::Runner::call(
				source_bob,
				evm_addr,
				balance_of_input.clone(),
				U256::default(),
				1000000,
				Some(U256::default()),
				Some(U256::from(0)),
				<Test as pallet_evm::Config>::config(),
			);

		assert_ok!(&call4evm);
		
		let bob_balance_before: u128;
		
		match call4evm.unwrap() {
			CallInfo {
				exit_reason: ExitReason::Succeed(_),
				value: return_value,
				..
			} => {
				let mut a: [u8; 16] = Default::default();
				a.copy_from_slice(&return_value[16..32]);
				bob_balance_before = u128::from_be_bytes(a);
			},
			CallInfo {
				exit_reason: _,
				value: _,
				..			
			} => {
				panic!("Call EVM Contract balanceOf failed!");
			},
		};
		
		//5.  Call wasm contract to call evm transfer evm token to bob.  H160: evm contract address, H160: bob's address  u128: value
		let mut a: [u8; 4] = Default::default();
		a.copy_from_slice(&Keccak256::digest(b"wasmCallEvm")[0..4]);
		let call = ExecutionInput::new(Selector::new(a));
		
		let fun_para: [u8; 20] = source_bob.into();
		let tranfer_value: u128  = 12000000000000000000;
		
		let call = call.push_arg(&evm_addr).push_arg(&fun_para).push_arg(tranfer_value);
				
		assert_ok!(Contracts::call(
				Origin::signed(ALICE),
				wasm_addr,
				0,
				GAS_LIMIT,
				Encode::encode(&call).to_vec(),
			)	
		);		
		
		//6. Get BOB balance of EVM token
		let call4evm = <Test as pallet_evm::Config>::Runner::call(
				source_bob,
				evm_addr,
				balance_of_input,
				U256::default(),
				1000000,
				Some(U256::default()),
				Some(U256::from(0)),
				<Test as pallet_evm::Config>::config(),
			);

		assert_ok!(&call4evm);
		
		let bob_balance_after: u128;
		
		match call4evm.unwrap() {
			CallInfo {
				exit_reason: ExitReason::Succeed(_),
				value: return_value,
				..
			} => {
				let mut a: [u8; 16] = Default::default();
				a.copy_from_slice(&return_value[16..32]);				
				bob_balance_after = u128::from_be_bytes(a);
			},
			CallInfo {
				exit_reason: _,
				value: _,
				..			
			} => {
				panic!("Call EVM Contract balanceOf failed!");
			},
		};		
		
		//7. Test  the balance of BOB being correct
		assert_eq!(bob_balance_after, bob_balance_before + tranfer_value);	
	});
}


// Perform test for EVM contract  calling  wasm contract
#[test]
fn test_evm_call_wasm(){
	
	// 1.  Get wasm and evm contract bin
	let (wasm, wasm_code_hash) = contract_module::<Test>("wasm_contract_test").unwrap();
	let (evm, _evm_code_hash) = contract_module::<Test>("evm_contract_test").unwrap();
	
	ExtBuilder::default()
	.existential_deposit(100)
	.build()
	.execute_with(|| {
		let _ = Balances::deposit_creating(&ALICE, 1_000_000);
		let subsistence = Contracts::subsistence_threshold();

		// 2. Create wasm contract
		let creation = Contracts::instantiate_with_code(
			Origin::signed(ALICE),
			subsistence * 100,
			GAS_LIMIT,
			wasm,
			vec![],
			vec![],
		);
		let wasm_addr = Contracts::contract_address(&ALICE, &wasm_code_hash, &[]);

		assert_ok!(creation);
		assert!(ContractInfoOf::<Test>::contains_key(&wasm_addr));	
		
		//3. Create EVM contract
		let source = H160::from_slice(&(AsRef::<[u8; 32]>::as_ref(&ALICE)[0..20]));
				
		//EVM::Config::CallOrigin::ensure_address_origin(&source, origin)?;
		
		let creation4evm = <Test as pallet_evm::Config>::Runner::create(
			//Origin::signed(ALICE),
			source,
			evm,
			U256::default(),
			1000000,
			Some(U256::default()),
			Some(U256::from(0)),
			<Test as pallet_evm::Config>::config(),
		);
		
		assert_ok!(&creation4evm);
		
		let evm_addr: H160;
		
		match creation4evm.unwrap() {
			CreateInfo {
				exit_reason: ExitReason::Succeed(_),
				value: create_address,
				..
			} => {
				evm_addr = create_address;
			},
			CreateInfo {
				exit_reason: _,
				value: _,
				..
			} => {
				panic!("Create EVM Contract failed!");
			},
		}
		
		
		//4. Get BOB balance of wasm token
		let mut a: [u8; 4] = Default::default();
		a.copy_from_slice(&Keccak256::digest(b"balanceOf")[0..4]);		
		let balance_of_call = ExecutionInput::new( Selector::new(a) );
		
		let source_bob = H160::from_slice(&(AsRef::<[u8; 32]>::as_ref(&BOB)[0..20]));
				
		let balance_of_call = balance_of_call.push_arg(source_bob);
						
		let result = Contracts::bare_call(
					BOB,
					wasm_addr.clone(),
					0,
					GAS_LIMIT,
					Encode::encode(&balance_of_call).to_vec(),
				).exec_result.unwrap();
		assert!(result.is_success());
		
		let bob_balance_before = result.data;
				
		//5.  Call EVM contract to call wasm contract transfer wasm token to bob,  the last bytes32 is the wasm contract accountid
		let evm_call_wasm_selector = &Keccak256::digest(b"evmCallWasm(bytes32,uint256,bytes32)")[0..4];
		let fun_para: [u8; 20] = source_bob.into();
		let tranfer_value: u128  = 12000000000000000000;
		
		let wasm_contract: [u8; 32] = wasm_addr.clone().into();
				
		let evm_call_wasm_input = [&evm_call_wasm_selector[..], &fun_para[..], &[0u8,16], &tranfer_value.to_be_bytes(), &wasm_contract].concat();
		
		let source_alice = H160::from_slice(&(AsRef::<[u8; 32]>::as_ref(&ALICE)[0..20]));
			
		let call4evm = <Test as pallet_evm::Config>::Runner::call(
				source_alice,
				evm_addr,
				evm_call_wasm_input,
				U256::default(),
				1000000,
				Some(U256::default()),
				Some(U256::from(0)),
				<Test as pallet_evm::Config>::config(),
			);

		assert_ok!(call4evm);
		
		//6. Get BOB balance of wasm token
		let result = Contracts::bare_call(
					BOB,
					wasm_addr.clone(),
					0,
					GAS_LIMIT,
					Encode::encode(&balance_of_call).to_vec(),
				).exec_result.unwrap();
		assert!(result.is_success());
		
		let bob_balance_after = result.data;
				
		//7. Test  the balance of BOB being correct
		let after = <u128 as Decode>::decode(&mut &bob_balance_after[..]).unwrap();
		let before = <u128 as Decode>::decode(&mut &bob_balance_before[..]).unwrap();
		assert_eq!(after, before + tranfer_value);	
	});
}