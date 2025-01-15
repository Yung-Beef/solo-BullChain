use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::fungible::Inspect;
use frame_support::traits::tokens::{Preservation, Fortitude};


#[test]
fn test_submit_post() {
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

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), alice, balance));
        assert_eq!(Balances::free_balance(alice), balance);
        assert_eq!(Balances::reducible_balance(&alice, Preservation::Preserve, Fortitude::Polite), bond);

        // Cannot submit an empty post
        assert_noop!(Bullposting::submit_post(RuntimeOrigin::signed(alice), empty_post, bond), Error::<Test>::Empty);


        // Cannot bond more tokens than you have available
        assert_noop!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond + 1), Error::<Test>::InsufficientFreeBalance);
        
        // Call success with storage and event
        assert_ok!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));
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
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), bob, balance));
        assert_eq!(Balances::free_balance(bob), balance);
        assert_eq!(Balances::reducible_balance(&bob, Preservation::Preserve, Fortitude::Polite), bond);
        assert_noop!(Bullposting::submit_post(RuntimeOrigin::signed(bob), post_url, bond), Error::<Test>::PostAlreadyExists);
    });
}

#[test]
fn test_submit_vote() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let charlie = 2;
        let david = 3;
        let bond = 1000;
        // Existential deposit is 1
        let balance = bond + 1;
        let vote_amount = 500;
        let voting_period = 1000;
        let post_url: Vec<u8> = "https://paritytech.github.io/polkadot-sdk/master/sp_test_primitives/type.BlockNumber.html".into();
        let post_id = sp_io::hashing::blake2_256(&post_url);
        let empty_vote: Vec<u8> = "".into();
        let fake_post_url = "get rekt kid".into();

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), alice, balance));
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), bob, balance));
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), charlie, balance));
        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), david, balance));


        // Call success with storage and event
        assert_ok!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post_url.clone(), bond));

        // Can't submit an empty post info with your vote
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), empty_vote, vote_amount, crate::Direction::Bullish), Error::<Test>::Empty);
        
        // Can't vote on a non-existant post
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), fake_post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::PostDoesNotExist);

        // Can't vote with more than your balance
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), 1500, crate::Direction::Bullish), Error::<Test>::InsufficientFreeBalance);

        // Vote Bullish
        assert_ok!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bullish));
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
        assert_ok!(Bullposting::submit_vote(RuntimeOrigin::signed(charlie), post_url.clone(), vote_amount, crate::Direction::Bearish));
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
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount + 50, crate::Direction::Bullish), Error::<Test>::AlreadyVoted);
        // Tries to change direction
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(bob), post_url.clone(), vote_amount, crate::Direction::Bearish), Error::<Test>::AlreadyVoted);

        // Can't vote is the voting period has ended
        System::set_block_number(voting_period + 1);
        assert_noop!(Bullposting::submit_vote(RuntimeOrigin::signed(david), post_url, vote_amount, crate::Direction::Bullish), Error::<Test>::VotingEnded);
    });
}