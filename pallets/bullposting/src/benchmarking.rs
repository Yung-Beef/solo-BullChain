//! Benchmarking setup for pallet-bullposting
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as BullPosting;
use frame_benchmarking::v2::*;
use frame_system::{RawOrigin};
use frame_support::{ensure, traits::{Get, fungible, fungible::{Inspect, Mutate}, tokens::Balance}, sp_runtime::traits::Bounded};

const SEED: u32 = 0;
const MAX_POSTS: u32 = u32::MAX;
const MAX_URL: u32 = u32::MAX;

fn max_url_length<T: Config>() -> u32 {
	T::MaxUrlLength::get()
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_post<T: Config>(
		p: Linear<0, u32::MAX>,
		b: Linear<0, u32::MAX>,
		s: Linear<0, MAX_POSTS>,
	) -> Result<(), BenchmarkError>{
		let horrible_post: Vec<u8> = ["倨"; T::MaxUrlLength::get().into()];
		let alice: T::AccountId = account("Alice", 0, SEED);
		let bob: T::AccountId = account("Bob", 0, SEED);
		let length = T::MaxUrlLength::get().into();
		let balance: BalanceOf<T> = u64::MAX.into();

        <T as pallet::Config>::NativeBalance::set_balance(&alice, balance);
		<T as pallet::Config>::NativeBalance::set_balance(&bob, balance);

		let storage_filler: [Vec<u8>; MAX_POSTS.into()];

		for i in 0..s {
			storage_filler.insert(i);
			try_submit_post(alice, storage_filler[i], b);
		}

        #[extrinsic_call]
		try_submit_post::<<T as pallet::Config>::RawOrigin>(bob, horrible_post, balance - 1);

		ensure!(<T as pallet::Config>::Posts::contains_key(horrible_post), "Post not submitted");
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

//     impl_benchmark_test_suite!(BullPosting, crate::mock::new_test_ext(), crate::mock::Test);
}
