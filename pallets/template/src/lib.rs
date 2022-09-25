#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use sp_core::crypto::KeyTypeId;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_TYPE);

    pub struct OcwAuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OcwAuthId
        {
            type RuntimeAppPublic = Public;
            type GenericSignature = sp_core::sr25519::Signature;
            type GenericPublic = sp_core::sr25519::Public;
        }
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
	use sp_io::offchain_index;
	use sp_runtime::offchain::storage::StorageValueRef;
	use scale_info::prelude::string::String;   
	
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendSignedTransaction,
			Signer,
		},
	};

	const ONCHAIN_TX_KEY: &[u8] = b"template_pallet::indexing1";

	#[derive(Debug, Encode, Decode, Default)]
	struct IndexingData(Vec<u8>, u32);
	
	
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(0)]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			log::info!("---origin : {:?} and something ： {:?}", who, something);

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(100 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::weight(0)]
        pub fn submit_data(origin: OriginFor<T>, payload: Vec<u8>) -> DispatchResultWithPostInfo {

            let _who = ensure_signed(origin)?;

            log::info!("in submit_data call: {:?}", payload);

            Ok(().into())
        }

		#[pallet::weight(100)]
		pub fn store_to_offchain_storage(origin: OriginFor<T>, number: u32) -> DispatchResult {

			let _who = ensure_signed(origin)?;

			let key = Self::derived_key(frame_system::Module::<T>::block_number());
			
			let data = IndexingData(ONCHAIN_TX_KEY.to_vec(), number);
			

			offchain_index::set(&key, &data.encode());

			log::info!("--------current block: {:?} and store number: {:?} with key {:?}", frame_system::Module::<T>::block_number(), number, key);

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hello world from offchain workers!");
			//let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::info!("Offchain work current block: {:?} ", block_number);
			let key = Self::derived_key(block_number);
			let storage_ref = StorageValueRef::persistent(&key);
			/*
			let timeout = sp_io::offchain::timestamp()
					.add(sp_runtime::offchain::Duration::from_millis(8000));
				sp_io::offchain::sleep_until(timeout);
			*/
			log::info!("-----try to read local storage data by key: {:?}, {:?}", key, block_number);
			if let Ok(Some(data)) = storage_ref.get::<IndexingData>() {
				log::info!("----local storage data: {:?}, {:?}", String::from_utf8(data.0).unwrap(), data.1);
				_ = Self::trigger_signed_transaction(data.1);
			} else {
				log::info!("Error reading from local storage.");
			}
		}
	}

	impl<T: Config> Pallet<T> {

		fn trigger_signed_transaction(payload:u32) -> Result<(), &'static str> {
			log::info!("---try to send tx for payload {:?}", payload);
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            if !signer.can_sign() {
				log::info!("---Failed try to send tx for payload {:?} with signer {:?}", payload, signer.can_sign());
                return Err(
                    "No local accounts available. Consider adding one via `author_insertKey` RPC.",
                    )
            }
			log::info!("---try to send tx for payload {:?} with signer {:?}", payload, signer.can_sign());
            let results = signer.send_signed_transaction(|_account| {				
                Call::do_something { something: payload.clone() }
            });

            for (acc, res) in &results {
                match res {
                    Ok(()) => log::info!("[{:?}] Submitted data:{:?}", acc.id, payload),
                    Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
                }
            }

            Ok(())
        }

		fn derived_key(block_number: T::BlockNumber) -> Vec<u8> {
			block_number.using_encoded(|encoded_bn| {
				ONCHAIN_TX_KEY.clone().into_iter()
					.chain(b"/".into_iter())
					.chain(encoded_bn)
					.copied()
					.collect::<Vec<u8>>()
			})
		}

	}
}
