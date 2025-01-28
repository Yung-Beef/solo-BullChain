//! Benchmarking setup for pallet-bullposting
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as BullPosting;
use frame_benchmarking::v2::*;
use frame_system::{RawOrigin};
use frame_support::traits::{Get, fungible::{Inspect, Mutate}};
use frame_support::sp_runtime::*;

const SEED: u32 = 0;
const MAX_POSTS: u32 = u32::MAX;
const MAX_URL: usize = 4294967295;
const MAX_URL32: u32 = u32::MAX;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
    use super::*;

	fn setup_storage(count: u32) {
		// submit a post `count` times to fill the storage up
		
	}

    #[benchmark]
    fn submit_post<T: Config>(
		p: Linear<0, MAX_URL32>,
		b: Linear<0, u32::MAX>,
		s: Linear<0, MAX_POSTS>,
	) -> Result<(), BenchmarkError>{
		let horrible_post: String = String::from_iter(["倨"; MAX_URL]);
		let post_vec: Vec<u8> = String::into_bytes(horrible_post.clone());
		let post_id: [u8; 32] = sp_io::hashing::blake2_256(&post_vec);
		let alice: T::AccountId = account("Alice", 0, SEED);
		let bob: T::AccountId = account("Bob", 0, SEED);
		let balance = <T as pallet::Config>::NativeBalance::minimum_balance().saturating_add(4294967295u32.into());

        <T as pallet::Config>::NativeBalance::set_balance(&alice, balance);
		<T as pallet::Config>::NativeBalance::set_balance(&bob, balance);

		// let storage_filler: [Vec<u8>; MAX_POSTS.into()];
		// for i in 0..s {
		// 	storage_filler.insert(i);
		// 	try_submit_post::<<T as pallet::Config>::RawOrigin>(alice, storage_filler[i], b);
		// }

        #[extrinsic_call]
		try_submit_post(RawOrigin::Signed(bob.clone()), horrible_post, balance.saturating_sub(1u32.into()));

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
