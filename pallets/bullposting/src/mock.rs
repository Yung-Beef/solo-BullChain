use crate::pallet as pallet_bullposting;
use frame_support::{
    derive_impl,
    parameter_types,
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Bullposting: pallet_bullposting,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
    pub const MaxFreezes: u32 = 10000;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = MaxFreezes;
}

type BlockNumber = u64;

parameter_types! {
    pub const RewardStyle: bool = false;
    pub const FlatReward: u32 = FlatReward::get();
    pub const RewardCoefficient: u32 = 100 ;
    pub const SlashStyle: bool = false;
    pub const FlatSlash: u32 = FlatSlash::get();
    pub const SlashCoefficient: u8 = 100 ;
    pub const VotingPeriod: BlockNumber = 1000;
}

impl pallet_bullposting::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type NativeBalance = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = ();
    type RewardStyle = RewardStyle;
    type FlatReward = FlatReward;
    type RewardCoefficient = RewardCoefficient;
    type SlashStyle = SlashStyle;
    type FlatSlash = FlatSlash;
    type SlashCoefficient = SlashCoefficient;
    type VotingPeriod = VotingPeriod;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
