//! Benchmarking setup for pallet-bullposting
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as BullPosting;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

// #[benchmarks]
// mod benchmarks {
//     use super::*;

//     #[benchmark]
//     fn do_something() {
//         let value = 100u32;
//         let caller: T::AccountId = whitelisted_caller();
//         #[extrinsic_call]
//         do_something(RawOrigin::Signed(caller), value);

//         assert_eq!(Something::<T>::get(), Some(value));
//     }

//     #[benchmark]
//     fn cause_error() {
//         Something::<T>::put(100u32);
//         let caller: T::AccountId = whitelisted_caller();
//         #[extrinsic_call]
//         cause_error(RawOrigin::Signed(caller));

//         assert_eq!(Something::<T>::get(), Some(101u32));
//     }

let max_posts = BalanceOf::<T>::total_issuance() / T::BondMinimum::get();

    #[benchmark]
    fn submit_post(
		p: Linear<0, T::MaxUrlLength::get()>,
		b: Linear<0, BalanceOf::<T>::max_value()>,
		s: Linear<0, max_posts>,
	) -> Result<(), BenchmarkError>{
		let horrible_post: Vec<u8> = [倨; T::MaxUrlLength::get()].into();
		let alice: T::AccountId = account("Alice", 0, SEED);
		let bob: T::AccountId = account("Bob", 0, SEED);

        Balances::<T>::force_set_balance(RuntimeOrigin::root(), alice, BalanceOf::<T>::max_value());
		Balances::<T>::force_set_balance(RuntimeOrigin::root(), bob, BalanceOf::<T>::max_value());

		for i in 0..s {
			let vec: Vec<u8> = [g; p];
			try_submit_post(alice, vec, b);
		}

        #[extrinsic_call]
		try_submit_post<T::RuntimeOrigin>(bob, horrible_post, (BalanceOf::<T>::max_value - 1))

		ensure!(T::Posts::contains_key(horrible_post), "Post not submitted");
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
// }
