//! Run `cargo doc --package pallet-bullposting --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]



// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
    // Import various useful types required by all FRAME pallets.
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // Other imports
    use codec::{EncodeLike, MaxEncodedLen};
    use scale_info::{prelude::fmt::Debug, StaticTypeInfo};
    use frame_support::traits::tokens::{fungible, Preservation, Fortitude, IdAmount};
    use frame_support::traits::fungible::{Inspect, MutateHold, MutateFreeze};
    use frame_support::BoundedVec;
    use frame_support::sp_runtime::traits::{CheckedSub, Zero};


    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
        /// A type representing the post submitted to the chain by a user. Will likely be hashed by the client from a string.
        type Post: MaxEncodedLen + EncodeLike + Decode + StaticTypeInfo + Clone + Debug + PartialEq;
        /// A type representing the token used.
        type NativeBalance: fungible::Inspect<Self::AccountId>
        + fungible::Mutate<Self::AccountId>
        + fungible::hold::Inspect<Self::AccountId>
        + fungible::hold::Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
        + fungible::freeze::Inspect<Self::AccountId>
        + fungible::freeze::Mutate<Self::AccountId, Id = Self::RuntimeFreezeReason>;

        /// A type representing the reason an account's tokens are being held.
        type RuntimeHoldReason: From<HoldReason>;
        /// A type representing the reason an account's tokens are being frozen.
        type RuntimeFreezeReason: From<FreezeReason>;
        /// The ID type for freezes.
		type FreezeIdentifier: Parameter + Member + MaxEncodedLen + Copy;
        /// The maximum number of individual freeze locks that can exist on an account at any time.
		#[pallet::constant]
		type MaxFreezes: Get<u32>;
    }

    type BalanceOf<T> =
        <<T as Config>::NativeBalance as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    /// Used for the direction of votes and results
    #[derive(Debug, PartialEq, Clone, Encode, Decode, TypeInfo)]
    pub enum Direction {
        Bullish,
        Bearish,
    }

    /// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
        /// Submitting a post
        PostBond,
	}

    /// A reason for the pallet freezing funds.
	#[pallet::composite_enum]
	pub enum FreezeReason {
        /// Voting
        Vote,
	}

    // TODO: change i128 to generic, but it needs to be implemented as something bigger than whatever the balance type is
    #[derive(MaxEncodedLen, Debug, PartialEq, Clone, Encode, Decode, TypeInfo)]
    #[scale_info(skip_type_params(T))]

    pub struct Post<T: Config> {
        pub submitter: T::AccountId,
        pub bond: BalanceOf<T>,
        pub bull_votes: BalanceOf<T>,
        pub bear_votes: BalanceOf<T>,
        pub voting_until: BlockNumberFor<T>,
    }

    /// Stores the post ID as the key and a post struct (with the additional info such as the submitter) as the value
    #[pallet::storage]
    pub type Posts<T: Config> =
        StorageMap<_, Blake2_128Concat, <T as pallet::Config>::Post, Post<T>>;

    /// Stores the freeze locks per account.
	#[pallet::storage]
	pub type Freezes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<IdAmount<T::FreezeIdentifier, BalanceOf<T>>, T::MaxFreezes>,
		ValueQuery,
	>;

    /// Events that functions in this pallet can emit.
    ///
    /// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
    /// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
    /// documentation for each event field and its parameters is added to a node's metadata so it
    /// can be used by external interfaces or tools.
    ///
    ///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
    /// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
    /// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Post submitted successfully.
        PostSubmitted {
            /// The post ID.
            post: T::Post,
            /// The account that submitted the post and bonded tokens.
            submitter: T::AccountId,
            /// Amount of bonded tokens.
            bond: BalanceOf<T>,
            /// Duration of voting period.
            voting_until: BlockNumberFor<T>,
        },
        /// Vote submitted successfully.
        VoteSubmitted {
            /// The post ID.
            post: T::Post,
            /// The account voting on the post.
            voter: T::AccountId,
            /// The amount of tokens frozen for the vote.
            vote_amount: BalanceOf<T>,
            /// Bullish or bearish vote.
            direction: Direction,
        },
        /// Vote closed and resolved, unlocking voted tokens and rewarding or slashing the submitter.
        VoteResolved {
            /// The post ID.
            post: T::Post,
            /// The account that submitted the post and bonded tokens.
            submitter: T::AccountId,
            /// Bullish means the submitter was rewarded, Bearish means they were slashed
            result: Direction,
            rewarded: BalanceOf<T>,
            slashed: BalanceOf<T>,
        },
    }

    /// Errors that can be returned by this pallet.
    ///
    /// Errors tell users that something went wrong so it's important that their naming is
    /// informative. Similar to events, error documentation is added to a node's metadata so it's
    /// equally important that they have helpful documentation associated with them.
    ///
    /// This type of runtime error can be up to 4 bytes in size should you want to return additional
    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The voting period was too short.
        PeriodTooShort,
        /// Post already submitted.
        PostAlreadyExists,
        /// TODO: MAKE 1 ERROR FOR HOLD AND 1 FOR FREEZE
        /// Insufficient available balance.
        InsufficientFreeBalance,
        /// Post has not been submitted.
        PostDoesNotExist,
        /// Vote still in progress.
        VoteStillOngoing,
        /// Vote already closed and resolved.
        VoteAlreadyResolved,
        // TODO: SHOULD BE HANDLED BETTER SOMEHOW?
        /// If there is an overflow while doing checked_add().
        Overflow,
    }

    /// The pallet's dispatchable functions ([`Call`]s).
    ///
    /// Dispatchable functions allows users to interact with the pallet and invoke state changes.
    /// These functions materialize as "extrinsics", which are often compared to transactions.
    /// They must always return a `DispatchResult` and be annotated with a weight and call index.
    ///
    /// The [`call_index`] macro is used to explicitly
    /// define an index for calls in the [`Call`] enum. This is useful for pallets that may
    /// introduce new dispatchables over time. If the order of a dispatchable changes, its index
    /// will also change which will break backwards compatibility.
    ///
    /// The [`weight`] macro is used to assign a weight to each call.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO: CHANGE 2X THEIR BOND INTO A GENERIC SO IT'S CONFIGURABLE
        /// Submits a post to the chain for voting.
        /// If the post is ultimately voted as bullish, they will get 2x their bond.
        /// If it is voted as bearish, they lose their bond.
        ///
        /// It checks that the post has not already been submitted in the past,
        /// and that the submitter has enough free tokens to bond.
        ///
        /// The post is then stored with who submitted it, and their tokens are bonded.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If the post has been submitted previously ([`Error::PostAlreadyExists`])
        /// - If the submitter does not have sufficient free tokens to bond ([`Error::InsufficientFreeBalance`])
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::default())]
        pub fn submit_post(
            origin: OriginFor<T>,
            post: T::Post,
            bond: BalanceOf<T>,
            voting_period: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: CHECK IF POST NEEDS TO BE CLONED OR CAN BE REFERENCED
            // Checks if the post exists
            if Posts::<T>::contains_key(post.clone()) {
                return Err(Error::<T>::PostAlreadyExists.into())
            }

            // Checks if they have enough balance available to be bonded
            let reduc_bal = <<T as Config>::NativeBalance>::
            reducible_balance(&who, Preservation::Preserve, Fortitude::Polite);
            reduc_bal.checked_sub(&bond).ok_or(Error::<T>::InsufficientFreeBalance)?;

            // Bonds the balance
            T::NativeBalance::hold(&HoldReason::PostBond.into(), &who, bond)?;

            let voting_until = frame_system::Pallet::<T>::block_number() + voting_period;

            // Stores the submitter and bond info
            Posts::<T>::insert(post.clone(), Post {
                submitter: who.clone(),
                bond,
                bull_votes: Zero::zero(),
                bear_votes: Zero::zero(),
                voting_until,
            });

            // Emit an event.
            Self::deposit_event(Event::PostSubmitted { post, submitter: who, bond, voting_until });

            Ok(())
        }

        /// Submits a vote on whether a particular post is bullish or bearish.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the voting period has already closed ([`Error::VoteAlreadyResolved`])
        /// - If the user tries to vote with more than their balance ([`Error::InsufficientFreeBalance`])
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::default())]
        pub fn submit_vote(
            origin: OriginFor<T>,
            post: T::Post,
            vote_amount: BalanceOf<T>,
            direction: Direction,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Error if the post does not exist.
            if !Posts::<T>::contains_key(post.clone()) {
                return Err(Error::<T>::PostDoesNotExist.into())
            }

            // Check if voting is still open for that post
            let post_struct = Posts::<T>::get(post.clone()).expect("Already checked that it exists");
            // If current block number is higher than the ending period of the post's voting, error.
            if frame_system::Pallet::<T>::block_number() > post_struct.voting_until {
                return Err(Error::<T>::VoteAlreadyResolved.into())
            }

            // Error if they do not have enough balance for the freeze
            if vote_amount > <<T as Config>::NativeBalance>::total_balance(&who) {
                return Err(Error::<T>::InsufficientFreeBalance.into())
            };

            // Extend_freeze
            <<T as Config>::NativeBalance>::extend_freeze(&FreezeReason::Vote.into(), &who, vote_amount)?;

            // Stores vote info/updates post struct according to vote direction
            let updated_post_struct = match direction {
                Direction::Bullish => {
                    Post {
                        bull_votes: post_struct.bull_votes + vote_amount,
                        ..post_struct
                    }
                },
                Direction::Bearish => {
                    Post {
                        bear_votes: post_struct.bear_votes + vote_amount,
                        ..post_struct
                    }
                },
            };

            Posts::<T>::insert(post.clone(), updated_post_struct);

            // Emit an event.
            Self::deposit_event(Event::VoteSubmitted {
                post,
                voter: who,
                vote_amount,
                direction,
            });

            Ok(())
        }


        /// Resolves a vote, rewarding or slashing the submitter and unfreezing voted tokens
        /// Callable by anyone
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the vote is still in progress ([`Error::VoteStillOngoing`])
        /// - If the vote has already been resolved ([`Error::VoteAlreadyResolved`])
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::default())]
        pub fn resolve_vote(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Ok(())
        }
    }
}
