use crate::{mock::*, Error, Event, Something};
use frame_support::{assert_noop, assert_ok};

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
        let post: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

        // Go past genesis block so events get deposited
        System::set_block_number(1);

        assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), alice, 100));
        assert_eq!(Balances::free_balance(alice), 100);
        assert_ok!(Bullposting::submit_post(RuntimeOrigin::signed(alice), post, 10));
        assert_eq!(crate::PostSubmitter::<Test>::get(post), Some(alice));
        System::assert_last_event(
            Event::PostSubmitted { 
                post: post, 
                submitter: alice, 
                bond: 10
            }.into()
        );

        // TODO: Add more stuff and then probably split into multiple tests
    });
}