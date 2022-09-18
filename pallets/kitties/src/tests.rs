use crate::{mock::{*, self}, Config, Error, Kitties, Event};
use frame_support::{assert_noop, assert_ok, assert_err};
use super::*;

#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// kitty_id = 0, owner = 1, count =1
		assert!(Kitties::<Test>::contains_key(0));
		// check owner has the kitty_id = 0
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		// confirm if the total size of kitties by owner is one
		assert_eq!(KittiesByOwner::<Test>::get(1).len(), 1usize);
		// check kitties has the kitty_id = 0 too
		let kitty = Kitties::<Test>::get(0).unwrap();
		//assert_eq!(kitty.0, [0; 16]);
		System::assert_has_event(mock::Event::KittyModule(Event::KittyCreated(1, 0, kitty)));		
	});
}

#[test]
fn create_kitty_failed_with_not_enough_balance() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_noop!(KittyModule::create(Origin::signed(3)), Error::<Test>::NotEnoughBalance);
	});
}

#[test]
fn create_kitty_failed_with_too_much_kitties() {
	new_test_ext().execute_with(|| {
	// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));		
		assert_err!(KittyModule::create(Origin::signed(1)), Error::<Test>::TooMuchKitties);
	});
}

#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// transfer to the second owner
		assert_ok!(KittyModule::transfer(Origin::signed(1), 0, 2));

		assert!(Kitties::<Test>::contains_key(0));
		// check if the second owner has the kitty_id = 0
		assert_eq!(KittyOwner::<Test>::get(0), Some(2));
		// check if the first woner hasn't any kitty and the second owner has one kitty after transfered
		assert_eq!(KittiesByOwner::<Test>::get(0).len(), 0usize);
		assert_eq!(KittiesByOwner::<Test>::get(2).len(), 1usize);

		System::assert_has_event(mock::Event::KittyModule(Event::KittyTransferred(1, 2, 0)));		
	});
}

#[test]
fn failed_to_transfer_kitty_with_not_owner() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// failed to transfer kitty with invalid owner
		assert_noop!(KittyModule::transfer(Origin::signed(2), 0, 1), Error::<Test>::NotOwner);
	});
}

#[test]
fn failed_to_transfer_kitty_with_not_enough_balance() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// failed to transfer kitty with invalid owner
		assert_noop!(KittyModule::transfer(Origin::signed(1), 0, 3), Error::<Test>::NotEnoughBalance);
	});
}

#[test]
fn failed_to_transfer_kitty_with_too_much_kitties() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittyModule::create(Origin::signed(2)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// failed to transfer kitty with owner 1 has too many kitties
		assert_err!(KittyModule::transfer(Origin::signed(2), 0, 1), Error::<Test>::TooMuchKitties);
	});
}

#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::breed(Origin::signed(1), 0, 1));

		// check if kitty_id in 0,1,2 and owner is 1
		assert!(Kitties::<Test>::contains_key(2));
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		assert_eq!(KittyOwner::<Test>::get(1), Some(1));
		assert_eq!(KittyOwner::<Test>::get(2), Some(1));
		let kitty = Kitties::<Test>::get(2).unwrap();
		System::assert_has_event(mock::Event::KittyModule(Event::KittyBred(1, 2, kitty)));
	});
}

#[test]
fn failed_breed_kitty_with_invalid_kitty_id() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// failed with invalid kitty id 
		assert_noop!(KittyModule::breed(Origin::signed(1), 0, 2), Error::<Test>::InvalidKittyId);
		assert_noop!(KittyModule::breed(Origin::signed(1), 2, 1), Error::<Test>::InvalidKittyId);
	});	
}

#[test]
fn failed_breed_kitty_with_not_enough_balance() {
	new_test_ext().execute_with(|| {
		// create kitty.
		assert_ok!(KittyModule::create(Origin::signed(2)));
		assert_ok!(KittyModule::create(Origin::signed(2)));
		assert_ok!(KittyModule::create(Origin::signed(2)));
		// failed with not enough balance to breed 
		assert_noop!(KittyModule::breed(Origin::signed(2), 0, 1), Error::<Test>::NotEnoughBalance);
	});	
}