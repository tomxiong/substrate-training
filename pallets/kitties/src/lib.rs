#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Randomness, Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::DispatchError;
use sp_std::prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32Bit, Bounded};
    use scale_info::TypeInfo;
	use codec::Encode;
	
	//#[pallet::type_value]
	//pub fn GetDefaultValue() -> KittyIndex {0_u32}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;		
		type KittyIndex: Parameter				
			+ Default			
			+ AtLeast32Bit
			+ Copy
			+ MaxEncodedLen;
		#[pallet::constant]
		type MaxKittyLength: Get<u32>;
		type Currency: ReservableCurrency<Self::AccountId>;
		#[pallet::constant]
		type Price: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_by_owner)]
	pub type KittiesByOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyLength>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreated(T::AccountId, T::KittyIndex, Kitty),
		KittyBred(T::AccountId, T::KittyIndex, Kitty),
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidKittyId,
		NotOwner,
		SameKittyId,
		TooMuchKitties,
		NotEnoughBalance,
	}

	#[pallet::call]
	impl<T :Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id();
			ensure!(kitty_id != T::KittyIndex::max_value(), Error::<T>::InvalidKittyId);
			//.map_err(|_| Error::<T>::InvalidKittyId)?;

			let price = T::Price::get();
			T::Currency::reserve(&who, price).map_err(|_| Error::<T>::NotEnoughBalance)?;
			
			let dna = Self::random_value(&who);
			let kitty = Kitty(dna);

			Kitties::<T>::insert(kitty_id, &kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittiesByOwner::<T>::try_mutate(&who,|ref mut kitties|{
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::TooMuchKitties)?;
				Ok::<(), DispatchError>(())
			})?;
		
			NextKittyId::<T>::set(Some(kitty_id + 1u32.into()));

			// Emit a event to notify the creator
			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2 : T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Validate kitty id
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
			let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;

			let price = T::Price::get();
			T::Currency::reserve(&who, price).map_err(|_| Error::<T>::NotEnoughBalance)?;
			// Get next id
			let kitty_id = Self::get_next_id();
			ensure!(kitty_id != T::KittyIndex::max_value(), Error::<T>::InvalidKittyId);

			// selector for breeding
			let selector = Self::random_value(&who);

			let mut data = [0u8; 16];
			for i in 0..kitty_1.0.len() {
				data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
			}
			let new_kitty = Kitty(data);
			<Kitties<T>>::insert(kitty_id, &new_kitty);
			KittyOwner::<T>::insert(kitty_id, &who);
			KittiesByOwner::<T>::try_mutate(&who,|ref mut kitties|{
				kitties.try_push(kitty_id).map_err(|_| Error::<T>::TooMuchKitties)?;
				Ok::<(), DispatchError>(())
			})?;
			NextKittyId::<T>::set(Some(kitty_id+1u32.into()));

			Self::deposit_event(Event::KittyBred(who, kitty_id, new_kitty));

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, kitty_id: T::KittyIndex, new_owner: T::AccountId) -> DispatchResult {
			// verify account signed
			let who = ensure_signed(origin)?;
			// check if the kitty_id is valid or not
			Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
			// confirm the account is the owner of the kitty
			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);			
			let price = T::Price::get();
			// reserve balance with price for new owner and then unreserve for old owner
			T::Currency::reserve(&who, price).map_err(|_| Error::<T>::NotEnoughBalance)?;
			T::Currency::unreserve(&new_owner, price);			
			// take action to save data
			<KittyOwner<T>>::insert(kitty_id, new_owner.clone());			
			KittiesByOwner::<T>::try_mutate(&who,|ref mut kitties|{
				let index = kitties.iter().position(|&r| r == kitty_id).unwrap();
				kitties.remove(index);
				Ok::<(), DispatchError>(())
			})?;
			let kitty = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
			KittiesByOwner::<T>::try_mutate(&new_owner,|ref mut kitties|{				
				kitties.try_push(kitty_id).map_err(|_|Error::<T>::TooMuchKitties)?;
				Ok::<(), DispatchError>(())
			})?;
			Self::deposit_event(Event::KittyTransferred(who, new_owner, kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Get a random 256 of u8
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet::<T>>::extrinsic_index(),
				);
			payload.using_encoded(blake2_128)
		}

		fn get_next_id() -> T::KittyIndex {
			match Self::next_kitty_id() {
				Some(id)  => id,
				None => 0u32.into(),
			}
		}

		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}		
	}
}
