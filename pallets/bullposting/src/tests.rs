use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::fungible::Inspect;
use frame_support::traits::tokens::{Preservation, Fortitude};


#[test]
fn test_try_submit_post() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let bond = 1000;
        // Existential deposit is 1
        let balance = bond + 1;
        let voting_period = 1000;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_post: Vec<u8> = "".into();
        let strange_post: Vec<u8> = "1234234asd!#%2lvliasdè÷ĳˇԦץڷॷ✗㈧倨".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        assert_eq!(Balances::free_balance(alice), balance);
        assert_eq!(Balances::reducible_balance(&alice, Preservation::Preserve, Fortitude::Polite), bond);

        // Cannot submit an empty post
        assert_noop!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), empty_post, bond), Error::<Test>::Empty);

        assert_noop!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), 25), Error::<Test>::BondTooLow);

        // Cannot bond more tokens than you have available
        assert_noop!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond + 1), Error::<Test>::InsufficientFreeBalance);
        
        // Call success with storage and event
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));
        let testpost = crate::Post {
            submitter: alice,
            bond,
            bull_votes: 0,
            bear_votes: 0,
            voting_until: System::block_number() + voting_period,
            resolved: false,
        };
        assert_eq!(crate::Posts::<Test>::get(post_id), Some(testpost));
        System::assert_last_event(
            Event::PostSubmitted { 
                id: post_id, 
                submitter: alice, 
                bond,
                voting_until: System::block_number() + voting_period,
            }.into()
        );
        
        // Tokens bonded
        assert_eq!(Balances::free_balance(alice), 1);
        assert_eq!(Balances::reducible_balance(&alice, Preservation::Preserve, Fortitude::Polite), 0);

        // Cannot resubmit an existing post
        assert_eq!(Balances::free_balance(bob), balance);
        assert_eq!(Balances::reducible_balance(&bob, Preservation::Preserve, Fortitude::Polite), bond);
        assert_noop!(Bullposting::try_submit_post(RuntimeOrigin::signed(bob), post_url, bond), Error::<Test>::PostAlreadyExists);

        // Can submit post with a weird input
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(bob), strange_post, bond));
    });
}

#[test]
fn test_try_submit_vote() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let charlie = 2;
        let david = 3;
        let bond = 1000;
        let vote_amount = 500;
        let voting_period = 1000;
        let post_url: Vec<u8>  = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_vote: Vec<u8> = "".into();
        let fake_post_url: Vec<u8> = "get rekt kid".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Call success with storage and event
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));

        // Can't submit an empty post info with your vote
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), empty_vote, vote_amount, crate::Direction::Bullish), Error::<Test>::Empty);
        
        // Can't vote on a non-existant post
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), fake_post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::PostDoesNotExist);

        // Can't vote with more than your balance
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), 1500, crate::Direction::Bullish), Error::<Test>::InsufficientFreeBalance);

        // Vote Bullish
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish));
        // Event
        System::assert_last_event(
            Event::VoteSubmitted { 
                id: post_id, 
                voter: bob, 
                vote_amount,
                direction: crate::Direction::Bullish,
            }.into()
        );
        // Check that storage was updated
        assert_eq!(crate::Votes::<Test>::contains_key(bob, post_id), true);

        // Vote Bearish
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(charlie), post_url.clone(), vote_amount, crate::Direction::Bearish));
        // Event
        System::assert_last_event(
            Event::VoteSubmitted { 
                id: post_id, 
                voter: charlie, 
                vote_amount,
                direction: crate::Direction::Bearish,
            }.into()
        );

        // Can't cast an initial vote if you've already voted
        // Tries to change amount
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount + 50, crate::Direction::Bullish), Error::<Test>::AlreadyVoted);
        // Tries to change direction
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bearish), Error::<Test>::AlreadyVoted);

        // Can't vote is the voting period has ended
        System::set_block_number(voting_period + 1);
        assert_noop!(Bullposting::try_submit_vote(RuntimeOrigin::signed(david), post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::VotingEnded);
    });
}

#[test]
fn test_try_update_vote() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let charlie = 2;
        let bond = 1000;
        let vote_amount = 500;
        let new_vote_amount = 505;
        let voting_period = 1000;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_vote: Vec<u8> = "".into();
        let fake_post_url: Vec<u8> = "get rekt kid".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Submit post
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));

        // Cannot update a vote without an initial vote
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish), Error::<Test>::VoteDoesNotExist);

        // Vote Bullish
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish));
        // Event
        System::assert_last_event(
            Event::VoteSubmitted { 
                id: post_id, 
                voter: bob, 
                vote_amount,
                direction: crate::Direction::Bullish,
            }.into()
        );
        // Check that storage was updated
        assert_eq!(crate::Votes::<Test>::contains_key(bob, post_id), true);
        let initial = crate::Votes::<Test>::get(bob, post_id);

        // Can't submit an empty post info with your vote
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), empty_vote, vote_amount, crate::Direction::Bullish), Error::<Test>::Empty);
        
        // Can't vote on a non-existant post
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), fake_post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::PostDoesNotExist);

        // Can't vote with more than your balance
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), post_url.clone(), 1500, crate::Direction::Bullish), Error::<Test>::InsufficientFreeBalance);

        // Someone else cannot update your vote
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(charlie), post_url.clone(), vote_amount, crate::Direction::Bullish), Error::<Test>::VoteDoesNotExist);
        
        // Successful vote update to Bearish with higher vote
        assert_ok!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), post_url.clone(), new_vote_amount, crate::Direction::Bearish));
        // Event
        System::assert_last_event(
            Event::VoteUpdated { 
                id: post_id, 
                voter: bob, 
                vote_amount: new_vote_amount,
                direction: crate::Direction::Bearish,
            }.into()
        );
        // Check that storage was updated
        let new = crate::Votes::<Test>::get(bob, post_id);
        assert_ne!(initial, new);

        // Successful vote update to Bearish with higher vote
        assert_ok!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), post_url.clone(), new_vote_amount - 100, crate::Direction::Bullish));
        // Event
        System::assert_last_event(
            Event::VoteUpdated { 
                id: post_id, 
                voter: bob, 
                vote_amount: new_vote_amount - 100,
                direction: crate::Direction::Bullish,
            }.into()
        );
        // Check that storage was updated
        let new2 = crate::Votes::<Test>::get(bob, post_id);
        assert_ne!(new, new2);

        // Can't vote is the voting period has ended
        System::set_block_number(voting_period + 1);
        assert_noop!(Bullposting::try_update_vote(RuntimeOrigin::signed(bob), post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::VotingEnded);

    });
}

#[test]
fn test_try_resolve_post() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let charlie = 2;
        let bond = 500;
        let vote_amount = 500;
        let voting_period = 1000;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let post_2_url: Vec<u8> = "testingtestingblahblah".into();
        let post_2_id = sp_io::hashing::blake2_256(&post_2_url);
        let empty_post: Vec<u8> = "".into();
        let fake_post_url: Vec<u8> = "get rekt kid".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Submit post
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_2_url.clone(), bond));

        // Vote
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish));
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(charlie), post_2_url.clone(), vote_amount, crate::Direction::Bearish));


        // Cannot resolve during the voting period
        assert_noop!(Bullposting::try_resolve_post(RuntimeOrigin::signed(alice), post_url.clone()), Error::<Test>::VotingStillOngoing);

        // End voting period
        System::set_block_number(voting_period + 1);

        // Error if submit an empty input for the post
        assert_noop!(Bullposting::try_resolve_post(RuntimeOrigin::signed(alice), empty_post), Error::<Test>::Empty);

        // Error if submit an empty input for the post
        assert_noop!(Bullposting::try_resolve_post(RuntimeOrigin::signed(alice), fake_post_url), Error::<Test>::PostDoesNotExist);

        // Resolve the post
        assert_ok!(Bullposting::try_resolve_post(RuntimeOrigin::signed(alice), post_url.clone()));
        // Switch which of the below events is commented out and change `pub const RewardStyle: bool` in mock.rs
        // Rewarded event with RewardStyle = false (FlatReward)
        // System::assert_last_event(
        //     Event::PostResolved { 
        //         id: post_id, 
        //         submitter: alice, 
        //         result: crate::Direction::Bullish,
        //         rewarded: 500,
        //         slashed: 0,
        //     }.into()
        // );
        // Rewarded event with RewardStyle = true (RewardCoefficient)
        System::assert_last_event(
            Event::PostResolved { 
                id: post_id, 
                submitter: alice, 
                result: crate::Direction::Bullish,
                rewarded: bond,
                slashed: 0,
            }.into()
        );

        // Error if the post has already been resolved
        assert_noop!(Bullposting::try_resolve_post(RuntimeOrigin::signed(bob), post_url), Error::<Test>::PostAlreadyResolved);

        // Post can be resolved by someone who is not the submitter
        assert_ok!(Bullposting::try_resolve_post(RuntimeOrigin::signed(bob), post_2_url.clone()));
        // Switch which of the below events is commented out and change `pub const SlashStyle: bool` in mock.rs
        // Slashed event with SlashStyle = false (FlatSlash)
        // System::assert_last_event(
        //     Event::PostResolved { 
        //         id: post_2_id, 
        //         submitter: alice, 
        //         result: crate::Direction::Bearish,
        //         rewarded: 0,
        //         slashed: 500,
        //     }.into()
        // );
        // Slashed event with SlashStyle = true (SlashCoefficient)
        System::assert_last_event(
            Event::PostResolved { 
                id: post_2_id, 
                submitter: alice, 
                result: crate::Direction::Bearish,
                rewarded: 0,
                slashed: bond,
            }.into()
        );
    });
}

#[test]
fn test_try_unfreeze_vote() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let charlie = 2;
        let david = 3;
        let bond = 500;
        let vote_amount = 500;
        let voting_period = 1000;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_post: Vec<u8> = "".into();
        let fake_post_url: Vec<u8> = "get rekt kid".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Submit post
        assert_ok!(Bullposting::try_submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));
        // Vote on post
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish));
        assert_ok!(Bullposting::try_submit_vote(RuntimeOrigin::signed(charlie), post_url.clone(), vote_amount, crate::Direction::Bearish));

        // End voting period
        System::set_block_number(voting_period + 1);

        // Error if the post is not yet resolved
        assert_noop!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), bob, post_url.clone()), Error::<Test>::PostUnresolved);

        // Resolve post
        assert_ok!(Bullposting::try_resolve_post(RuntimeOrigin::signed(alice), post_url.clone()));

        // Error on empty post input
        assert_noop!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), bob, empty_post.clone()), Error::<Test>::Empty);

        // Error if the post does not exist
        assert_noop!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), bob, fake_post_url), Error::<Test>::PostDoesNotExist);

        // Error if that vote never existed
        assert_noop!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), david, post_url.clone()), Error::<Test>::VoteDoesNotExist);


        // Successfully unfreeze a vote
        assert_ok!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), bob, post_url.clone()));
        // Event
        System::assert_last_event(
            Event::VoteUnfrozen { 
                id: post_id, 
                account: bob,
                amount: vote_amount,
            }.into()
        );
        // Check vote was removed from storage
        assert_eq!(crate::Votes::<Test>::contains_key(bob, post_id), false);

        assert_ok!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), charlie, post_url.clone()));
        // Event
        System::assert_last_event(
            Event::VoteUnfrozen { 
                id: post_id, 
                account: charlie,
                amount: vote_amount,
            }.into()
        );
        // Check vote was removed from storage
        assert_eq!(crate::Votes::<Test>::contains_key(charlie, post_id), false);

        // Error if that vote no longer exists (already unfrozen)
        assert_noop!(Bullposting::try_unfreeze_vote(RuntimeOrigin::signed(bob), bob, post_url), Error::<Test>::VoteDoesNotExist);
    });
}