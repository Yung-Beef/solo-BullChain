//! Benchmarking setup for pallet-bullposting
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as BullPosting;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn do_something() {
        let value = 100u32;
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        do_something(RawOrigin::Signed(caller), value);

        assert_eq!(Something::<T>::get(), Some(value));
    }

    #[benchmark]
    fn cause_error() {
        Something::<T>::put(100u32);
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        cause_error(RawOrigin::Signed(caller));

        assert_eq!(Something::<T>::get(), Some(101u32));
    }

    #[benchmark]
    fn submit_post() {
        let alice = 0;
        let bond = u64::Max - 1;
        let balance = u64::Max;
        let voting_period = u64::Max;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_post: Vec<u8> = "".into();
        let strange_post: Vec<u8> = "1234234asd!#%2lvliasdè÷ĳˇԦץڷॷ✗㈧倨".into();

        Balances::<T>::force_set_balance(RuntimeOrigin::root(), alice, balance);

        #[extrinsic_call]

    }

    #[benchmark]
    fn submit_vote() {

        #[extrinsic_call]

    }

    #[benchmark]
    fn update_vote() {

        #[extrinsic_call]

    }

    #[benchmark]
    fn resolve_post() {

        #[extrinsic_call]

    }

    #[benchmark]
    fn unfreeze_vote() {

        #[extrinsic_call]

    }

    impl_benchmark_test_suite!(BullPosting, crate::mock::new_test_ext(), crate::mock::Test);
}
