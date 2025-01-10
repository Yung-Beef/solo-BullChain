use crate::{mock::*, Error, Event, Something};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::fungible::Inspect;
use frame_support::traits::tokens::{Preservation, Fortitude};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Dispatch a signed extrinsic.
        assert_ok!(Bullposting::do_something(
            RuntimeOrigin::signed(1),
            42
        ));
        // Read pallet storage and assert an expected result.
        assert_eq!(Something::<Test>::get(), Some(42));
        // Assert that the correct event was deposited
        System::assert_last_event(
            Event::SomethingStored {
                something: 42,
                who: 1,
            }
            .into(),
        );
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Bullposting::cause_error(RuntimeOrigin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}


#[test]
fn test_submit_post() {
    new_test_ext().execute_with(|| {
        let alice = 0;
        let bob = 1;
        let post: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let bond = 1000;
        // Existential deposit is 1
        let balance = bond + 1;
        let voting_period = 5;

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), alice, balance));
        assert_eq!(Balances::free_balance(alice), balance);
        assert_eq!(Balances::reducible_balance(&alice, Preservation::Preserve, Fortitude::Polite), bond);

        // Cannot bond more tokens than you have available
        assert_noop!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post, bond + 1, 5), Error::<Test>::InsufficientFreeBalance);
        
        // Call success with storage and event
        assert_ok!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post, bond, 5));
        let testvote = crate::Post {
            submitter: alice,
            bond,
            votes: 0,
            voting_until: 6,
        };
        assert_eq!(crate::Posts::<Test>::get(post), Some(testvote));
        System::assert_last_event(
            Event::PostSubmitted { 
                post, 
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
        assert_noop!(Bullposting::submit_post(RuntimeOrigin::signed(bob), post, bond, voting_period), Error::<Test>::PostAlreadyExists);
    });
}