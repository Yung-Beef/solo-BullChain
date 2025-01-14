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
    use codec::MaxEncodedLen;
    use scale_info::prelude::fmt::Debug;
    use frame_support::traits::tokens::{fungible, Preservation, Fortitude, Precision};
    use frame_support::traits::fungible::{Inspect, Mutate, MutateHold, MutateFreeze};
    use frame_support::sp_runtime::traits::{CheckedSub, CheckedMul, CheckedDiv, Zero};


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

        /// Determines whether Bullish post rewards are a flat number or coefficient.
        /// A value of 0 (false) equals a flat number.
        /// A value of 1 (true) equals a coefficient.
        #[pallet::constant]
        type RewardStyle: Get<bool>;

        /// The reward given to submitters of Bullish posts.
        #[pallet::constant]
        type FlatReward: Get<u32>;

        /// The coefficient used to determine a submitter's reward if their post is voted Bullish.
        /// A value of 1 (bond 10 tokens, end up with 20 total)
        /// A value of 2 will reward them with 2x their bond (bond 10 tokens, end up with 30 total)
        // TODO: ADD A BOOL TYPE FOR WHETHER TO USE A COEFFICIENT OR A FLAT NUMBER, UPDATE RESOLVE() ACCORDINGLY
        #[pallet::constant]
        type RewardCoefficient: Get<u32>;

        /// Determines whether the bond for Bearish posts are slashed by a flat number or a coefficient.
        /// A value of 0 (false) equals a flat number.
        /// A value of 1 (true) equals a coefficient.
        #[pallet::constant]
        type SlashStyle: Get<bool>;

        /// The amount of tokens slashed from the submitter of a Bearish post.
        #[pallet::constant]
        type FlatSlash: Get<u32>;

        /// The coefficient used to determine how much of a a submitter's bond is slashed if their post is voted Bearish.
        /// A value of 1 will slash their entire bond, a value of 3 will slash a third of their bond.
        // TODO: ADD A BOOL TYPE FOR WHETHER TO USE A COEFFICIENT OR A FLAT NUMBER, UPDATE RESOLVE() ACCORDINGLY
        #[pallet::constant]
        type SlashDenominator: Get<u8>;

        // TODO: FIGURE OUT HOW TO SET A CONSTANT VOTING PERIOD


    }

    type BalanceOf<T> =
        <<T as Config>::NativeBalance as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    /// Used for the direction of votes and results
    #[derive(Debug, PartialEq, Clone, Encode, Decode, TypeInfo, Default, MaxEncodedLen)]
    pub enum Direction {
        #[default]
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

    #[derive(MaxEncodedLen, Debug, PartialEq, Clone, Encode, Decode, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Post<T: Config> {
        pub submitter: T::AccountId,
        pub bond: BalanceOf<T>,
        pub bull_votes: BalanceOf<T>,
        pub bear_votes: BalanceOf<T>,
        pub voting_until: BlockNumberFor<T>,
        pub resolved: bool,
    }

    /// Stores the post ID as the key and a post struct (with the additional info such as the submitter) as the value
    #[pallet::storage]
    pub type Posts<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], Post<T>>;

    
    /// Stores the vote size per account and post
    #[pallet::storage]
    pub type Votes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    Post<T>,
    (BalanceOf<T>, Direction),
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
            id: [u8; 32],
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
            id: [u8; 32],
            /// The account voting on the post.
            voter: T::AccountId,
            /// The amount of tokens frozen for the vote.
            vote_amount: BalanceOf<T>,
            /// Bullish or bearish vote.
            direction: Direction,
        },
        /// Vote updated successfully.
        VoteUpdated {
            /// The post ID.
            id: [u8; 32],
            /// The account voting on the post.
            voter: T::AccountId,
            /// The amount of tokens frozen for the vote.
            vote_amount: BalanceOf<T>,
            /// Bullish or bearish vote.
            direction: Direction,
        },
        /// Vote closed and resolved, unlocking voted tokens and rewarding or slashing the submitter.
        PostResolved {
            /// The post ID.
            id: [u8; 32],
            /// The account that submitted the post and bonded tokens.
            submitter: T::AccountId,
            /// Bullish means the submitter was rewarded, Bearish means they were slashed
            result: Direction,
            rewarded: BalanceOf<T>,
            slashed: BalanceOf<T>,
        },
        VoteUnfrozen {
            id: [u8; 32],
            account: T::AccountId,
            amount: BalanceOf<T>,
        }
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
        /// Submitted URL was empty.
        Empty,
        /// The voting period was too short.
        PeriodTooShort,
        /// Post already submitted.
        PostAlreadyExists,
        /// Insufficient available balance.
        InsufficientFreeBalance,
        /// Post has not been submitted.
        PostDoesNotExist,
        /// Account already voted on a particular post
        AlreadyVoted,
        /// If you try to unfreeze a vote that was already unfrozen or never happened in the first place.
        VoteDoesNotExist,
        /// Vote still in progress.
        VotingStillOngoing,
        /// Voting has ended but nobody has called resolve() yet.
        PostUnresolved,
        /// Vote already closed and resolved.
        PostAlreadyResolved,
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
        /// If there is a tie, their tokens are simply unlocked.
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
        /// - If they submit nothing for the post_url ([`Error::Empty`])
        /// - If the post has been submitted previously ([`Error::PostAlreadyExists`])
        /// - If the submitter does not have sufficient free tokens to bond ([`Error::InsufficientFreeBalance`])
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::default())]
        pub fn submit_post(
            origin: OriginFor<T>,
            post_url: Vec<u8>,
            bond: BalanceOf<T>,
            voting_period: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!post_url.is_empty(), Error::<T>::Empty);
            let id = sp_io::hashing::blake2_256(&post_url);

            // Checks if the post exists
            ensure!(Posts::<T>::contains_key(&id), Error::<T>::PostAlreadyExists);

            // Checks if they have enough balance available to be bonded
            let reduc_bal = <<T as Config>::NativeBalance>::
            reducible_balance(&who, Preservation::Preserve, Fortitude::Polite);
            reduc_bal.checked_sub(&bond).ok_or(Error::<T>::InsufficientFreeBalance)?;

            // Bonds the submitter's balance
            T::NativeBalance::hold(&HoldReason::PostBond.into(), &who, bond)?;

            let voting_until = frame_system::Pallet::<T>::block_number() + voting_period;

            // Stores the submitter and bond info
            Posts::<T>::insert(&id, Post {
                submitter: who.clone(),
                bond,
                bull_votes: Zero::zero(),
                bear_votes: Zero::zero(),
                voting_until,
                resolved: false,
            });

            // Emit an event.
            Self::deposit_event(Event::PostSubmitted { id, submitter: who, bond, voting_until });

            Ok(())
        }

        /// Submits a vote on whether a particular post is bullish or bearish.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If they submit nothing for the post_url ([`Error::Empty`])
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the voting period has already closed ([`Error::PostAlreadyResolved`])
        /// - If they have already voted once ([`Error::AlreadyVoted`])
        /// - If the user tries to vote with more than their balance ([`Error::InsufficientFreeBalance`])
        /// - If extend_freeze() underflows
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::default())]
        pub fn submit_vote(
            origin: OriginFor<T>,
            post_url: Vec<u8>,
            vote_amount: BalanceOf<T>,
            direction: Direction,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!post_url.is_empty(), Error::<T>::Empty);
            let id = sp_io::hashing::blake2_256(&post_url);

            // Error if the post does not exist.
            ensure!(!Posts::<T>::contains_key(&id), Error::<T>::PostDoesNotExist);
            let post_struct = Posts::<T>::get(&id).expect("Already checked that it exists");
            
            // Check if voting is still open for that post
            // If current block number is higher than the ending period of the post's voting, error.
            ensure!(frame_system::Pallet::<T>::block_number() < post_struct.voting_until, Error::<T>::PostAlreadyResolved);

            // Error if the post is already resolved
            ensure!(!post_struct.resolved, Error::<T>::PostAlreadyResolved);

            // Check if they have already voted
            ensure!(!Votes::contains_key(&who, &post_struct), Error::<T>::AlreadyVoted);

            // Check if they have enough balance for the freeze
            ensure!(vote_amount < <<T as Config>::NativeBalance>::total_balance(&who), Error::<T>::InsufficientFreeBalance);

            // Extend_freeze
            <<T as Config>::NativeBalance>::extend_freeze(&FreezeReason::Vote.into(), &who, vote_amount)?;

            // Store vote
            Votes::insert(&who, &post_struct, (vote_amount, &direction));

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

            Posts::<T>::insert(&id, updated_post_struct);

            // Emit an event.
            Self::deposit_event(Event::VoteSubmitted {
                id,
                voter: who,
                vote_amount,
                direction,
            });

            Ok(())
        }


        /// Updates an account's vote and freeze accordingly. Only possible before a vote is resolved.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If they submit nothing for the post_url ([`Error::Empty`])
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the vote has already been resolved ([`Error::PostAlreadyResolved`])
        /// - If this particular vote doesn't exist (['Error::VoteDoesNotExist'])
        /// - If the user does not have enough balance for their new vote ([`Error::InsufficientBalance`])
        /// - If extend_freeze() underflows
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::default())]
        pub fn update_vote(
            origin: OriginFor<T>,
            post_url: Vec<u8>,
            new_vote: BalanceOf<T>,
            direction: Direction
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!post_url.is_empty(), Error::<T>::Empty);
            let id = sp_io::hashing::blake2_256(&post_url);

            // Error if the post does not exist.
            ensure!(!Posts::<T>::contains_key(&id), Error::<T>::PostDoesNotExist);
            let post_struct = Posts::<T>::get(&id).expect("Already checked that it exists");

            // Check if voting is still open for that post
            // If current block number is higher than the ending period of the post's voting, error.
            ensure!(frame_system::Pallet::<T>::block_number() < post_struct.voting_until, Error::<T>::PostAlreadyResolved);

            // Error if the post is already resolved
            ensure!(!post_struct.resolved, Error::<T>::PostAlreadyResolved);

            // Error if this particular vote no longer exists or never existed.
            ensure!(Votes::<T>::contains_key(&who, &post_struct), Error::<T>::VoteDoesNotExist);

            // Error if they do not have enough balance for the freeze
            ensure!(new_vote < <<T as Config>::NativeBalance>::total_balance(&who), Error::<T>::InsufficientFreeBalance);

            let (previous_amount, previous_direction) = Votes::take(&who, &post_struct);

            // Extend_freeze
            <<T as Config>::NativeBalance>::extend_freeze(&FreezeReason::Vote.into(), &who, new_vote)?;

            // Store vote
            Votes::insert(&who, &post_struct, (new_vote, &direction));

            // Updates post struct's vote totals according to vote amount and direction
            // Removes previous directional vote and adds new vote
            let updated_post_struct = match direction {
                Direction::Bullish => {
                    if previous_direction == Direction::Bullish {
                        Post {
                            bull_votes: post_struct.bull_votes - previous_amount + new_vote,
                            ..post_struct
                        }
                    } else {
                        Post {
                            bull_votes: post_struct.bull_votes + new_vote,
                            bear_votes: post_struct.bear_votes - previous_amount,
                            ..post_struct
                        }
                    }
                },
                Direction::Bearish => {
                    if previous_direction == Direction::Bearish {
                        Post {
                            bear_votes: post_struct.bear_votes - previous_amount + new_vote,
                            ..post_struct
                        }
                    } else {
                        Post {
                            bull_votes: post_struct.bull_votes - previous_amount,
                            bear_votes: post_struct.bear_votes + new_vote,
                            ..post_struct
                        }
                    }
                },
            };

            Posts::<T>::insert(&id, updated_post_struct);

            // Emit an event.
            Self::deposit_event(Event::VoteSubmitted {
                id,
                voter: who,
                vote_amount: new_vote,
                direction,
            });

            
            Ok(())
        }


        /// Resolves a post, rewarding or slashing the submitter and enabling unfreeze_vote.
        /// Callable by anyone.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If they submit nothing for the post_url ([`Error::Empty`])
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the vote is still in progress ([`Error::VotingStillOngoing`])
        /// - If the vote has already been resolved ([`Error::PostAlreadyResolved`])
        /// - If release() is unsuccessful
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::default())]
        pub fn resolve_post(origin: OriginFor<T>, post_url: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!post_url.is_empty(), Error::<T>::Empty);
            let id = sp_io::hashing::blake2_256(&post_url);

            // Error if the post does not exist.
            ensure!(!Posts::<T>::contains_key(&id), Error::<T>::PostDoesNotExist);
            let post_struct = Posts::<T>::get(&id).expect("Already checked that it exists");
            let submitter = post_struct.submitter;

            // Check if the voting period is over for that post
            // If current block number is lower than the ending period of the post's voting, voting has not ended; error.
            ensure!(frame_system::Pallet::<T>::block_number() < post_struct.voting_until, Error::<T>::PostAlreadyResolved);

            // Error if already resolved.
            ensure!(!post_struct.resolved, Error::<T>::PostAlreadyResolved);

            // Resolve the vote and update storage
            let updated_post_struct = Post {
                resolved: true,
                ..post_struct
            };
            Posts::<T>::insert(&id, &updated_post_struct);

            // Reward/slash amount
            let bond = post_struct.bond;

            // Unlock submitter's bond
            T::NativeBalance::release(&HoldReason::PostBond.into(), &submitter, bond, Precision::BestEffort)?;

            let result: Direction = match updated_post_struct.bull_votes > updated_post_struct.bear_votes {
                true => Direction::Bullish,
                false => Direction::Bearish,
            };

            // Reward/slash submitter or do nothing if there is a tie
            if result == Direction::Bullish {
                // Reward the submitter
                let reward: BalanceOf<T>;
                // TODO: SAFE MATH
                if T::RewardStyle::get() == false {
                    let reward = Self::reward_flat(&submitter)?;
                } else {
                    let reward= Self::reward_coefficient(&submitter, &bond)?;
                }

                Self::deposit_event(Event::PostResolved { 
                    id,
                    submitter,
                    result,
                    rewarded: reward,
                    slashed: Zero::zero(),
                });
            } else {
                // Slashes the submitter
                let slash: BalanceOf<T>;
                // TODO: SAFE MATH
                if T::SlashStyle::get() == false {
                    let slash = Self::slash_flat(&submitter)?;
                } else {
                    let slash= Self::slash_coefficient(&submitter, &bond)?;
                }

                Self::deposit_event(Event::PostResolved { 
                    id,
                    submitter,
                    result,
                    rewarded: Zero::zero(),
                    slashed: slash
                });
            }

            Ok(())
        }


        /// Unfreezes the tokens used in a user's vote.
        /// Callable by anyone.
        ///
        /// ## Errors
        ///
        /// The function will return an error under the following conditions:
        ///
        /// - If they submit nothing for the post_url ([`Error::Empty`])
        /// - If the post does not exist ([`Error::PostDoesNotExist`])
        /// - If the vote is unresolved ([`Error::PostUnresolved`])
        /// - If this particular vote no longer exists or never existed (['Error::VoteDoesNotExist'])
        /// - If decrease_frozen() underflows
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::default())]
        pub fn unfreeze_vote(origin: OriginFor<T>, account: T::AccountId, post_url: Vec<u8>,) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            ensure!(!post_url.is_empty(), Error::<T>::Empty);
            let id = sp_io::hashing::blake2_256(&post_url);

            // Error if the post does not exist.
            ensure!(!Posts::<T>::contains_key(&id), Error::<T>::PostDoesNotExist);
            let post_struct = Posts::<T>::get(&id).expect("Already checked that it exists");

            // Error if the post is not resolved yet
            ensure!(post_struct.resolved, Error::<T>::PostUnresolved);

            // Error if this particular vote no longer exists or never existed.
            ensure!(Votes::<T>::contains_key(&account, &post_struct), Error::<T>::VoteDoesNotExist);
            let (amount, _direction) = Votes::take(&account, post_struct);
            
            // Remove freeze
            <<T as Config>::NativeBalance>::decrease_frozen(&FreezeReason::Vote.into(), &account, amount.clone())?;

            // Emit an event
            Self::deposit_event(Event::VoteUnfrozen {
                id,
                account,
                amount,
            });
            

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Reward a flat amount
        pub(crate) fn reward_flat(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
            // Reward the submitter
            let reward = T::FlatReward::get().into();
            T::NativeBalance::mint_into(&who, reward)?;

            Ok(reward)
        }
        
        // Reward based on a coefficient and how much they bonded
        pub(crate) fn reward_coefficient(who: &T::AccountId, bond: &BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            // Reward the submitter
            // TODO: SAFE MATH
            let reward = bond.checked_mul(T::RewardCoefficient::get().into()).expect("TODO");
            T::NativeBalance::mint_into(&who, reward)?;

            Ok(reward)
        }

        // Slash a flat amount
        pub(crate) fn slash_flat(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
            let slash = T::FlatSlash::get().into();
            T::NativeBalance::burn_from(&who, slash, Preservation::Protect, Precision::BestEffort, Fortitude::Force)?;

            Ok(slash)
        }

        // Slash based on a coefficient and how much they bonded
        pub(crate) fn slash_coefficient(who: &T::AccountId, bond: &BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
            // Slashes the submitter
            // TODO: SAFE MATH
            let slash = bond.checked_div(T::SlashDenominator::get().into()).expect("TODO");
            T::NativeBalance::burn_from(&who, slash, Preservation::Protect, Precision::BestEffort, Fortitude::Force)?;
            
            Ok(slash)
        }
    }
}
