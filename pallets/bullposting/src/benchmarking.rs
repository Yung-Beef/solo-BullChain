//! Benchmarking setup for pallet-bullposting
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as BullPosting;
use frame_benchmarking::v2::*;
use frame_system::{RawOrigin};
use frame_support::traits::{Get, fungible::{Inspect, Mutate}};
use frame_support::sp_runtime::*;
use scale_info::prelude::vec;

const SEED: u32 = 0;
const MAX_POSTS: u32 = 2000;
const MAX_URL: usize = 2000;
const MAX_URL32: u32 = 2000;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
    use super::*;

	fn setup_storage<T: Config>(count: u32) {
		// submit a post `count` times to fill the storage up
		let alice: T::AccountId = account("Alice", 0, SEED);
		let balance = <T as pallet::Config>::NativeBalance::minimum_balance().saturating_add(u32::MAX.into());
		let one = <T as pallet::Config>::NativeBalance::minimum_balance().saturating_add(1u32.into());

		<T as pallet::Config>::NativeBalance::set_balance(&alice, balance);

		for i in 0..=count {
			let v: Vec<u8> = vec![i.try_into().unwrap()];
			let _ = BullPosting::<T>::try_submit_post(RawOrigin::Signed(alice.clone()).into(), v, one);
		}
	}

    #[benchmark]
    fn try_submit_post<T: Config>(
		p: Linear<0, MAX_URL32>,
		b: Linear<0, u32::MAX>,
		s: Linear<0, MAX_POSTS>,
	) -> Result<(), BenchmarkError>{
		let post: Vec<u8> = [255u8; MAX_URL].to_vec();
		let post_id: [u8; 32] = sp_io::hashing::blake2_256(&post);
		let alice: T::AccountId = account("Alice", 0, SEED);
		let bob: T::AccountId = account("Bob", 0, SEED);
		let balance = <T as pallet::Config>::NativeBalance::minimum_balance().saturating_add(4294967295u32.into());

        <T as pallet::Config>::NativeBalance::set_balance(&alice, balance);
		<T as pallet::Config>::NativeBalance::set_balance(&bob, balance);

		setup_storage::<T>(s);

        #[extrinsic_call]
		try_submit_post(RawOrigin::Signed(bob.clone()), post, balance.saturating_sub(1u32.into()));

		let voting_until = frame_system::Pallet::<T>::block_number() +
            T::VotingPeriod::get();

		assert_last_event::<T>(Event::PostSubmitted {
			id: post_id,
			submitter: bob,
			bond: balance.saturating_sub(1u32.into()),
			voting_until,
		}.into());
		Ok(())
    }

//     #[benchmark]
//     fn submit_vote() {

//         #[extrinsic_call]

//     }

//     #[benchmark]
//     fn update_vote() {

//         #[extrinsic_call]

//     }

//     #[benchmark]
//     fn resolve_post() {

//         #[extrinsic_call]

//     }

//     #[benchmark]
//     fn unfreeze_vote() {

//         #[extrinsic_call]

//     }

	impl_benchmark_test_suite!(BullPosting, crate::mock::new_test_ext(), crate::mock::Test);
}
