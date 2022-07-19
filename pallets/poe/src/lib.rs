#![cfg_attr(not(feature = "std"), no_std)]

/// A module for proof of existence
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    // Runtime configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        //The event type definition, the pallet will emits events.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    // Declare the Pallet type
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Declare Runtime Storage for proofs, key is Vec<u8> and value is a tuple (account, blocknumber)
    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub(super) type Proofs<T: Config> =
    StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber)>;

    //Declare runtime events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event will be emitted when claim has been created and its parameters. [AccountId, Vec<u8>]
        ClaimCreated(T::AccountId, Vec<u8>),
        /// Event will be emitted when claim has been revoke and its parameters. [AccountId, Vec<u8>]
        ClaimRevoked(T::AccountId, Vec<u8>),
        /// Event will be emitted when the claim has been transferred to another account. [AccountId, AccountId, Vec<u8>]
        ClaimTrasnferred(T::AccountId, T::AccountId, Vec<u8>),
    }

    //Declare some errors the extrinsic used
    #[pallet::error]
    pub enum Error<T> {
        /// Error while the proofs already exists
        ProofAlreadyExists,
        /// Error while the proofs is not exists when create a new proofs
        ClaimNotExists,
        /// Error while the owner of proofs is not origin sender when revoke a proofs
        NotProofOwner
    }

    // Hooks , for example: on_initialized
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // the implementation of the pallet
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// create a claim
        #[pallet::weight(0)]
        pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
            // verify signed
            let sender = ensure_signed(origin)?;
            // check if proof is already in storage
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExists);
            //create a new proof
            Proofs::<T>::insert(&claim, (sender.clone(), frame_system::Pallet::<T>::block_number()));
            // emits event
            Self::deposit_event(Event::ClaimCreated(sender, claim));

            Ok(().into())
        }

        /// revoke an already exists claim
        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            // verify signed
            let sender = ensure_signed(origin)?;
            // get value from storage
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExists)?;

            // Verify that trigger of the current call is the claim owner.
            ensure!(owner == sender, Error::<T>::NotProofOwner);

            // Remove claim from storage map.
            Proofs::<T>::remove(&claim);
            // emits event
            Self::deposit_event(Event::ClaimRevoked(sender, claim));

            Ok(().into())
        }

        // Transfer claim to other account.
        #[pallet::weight(0)]
        pub fn transfer_claim(
            origin: OriginFor<T>,
            to: T::AccountId,
            claim: Vec<u8>
        ) -> DispatchResultWithPostInfo {
            // verify signed
            let sender = ensure_signed(origin)?;
            // get value from storage
            let (owner, block_number) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExists)?;

            // Verify that sender of the current call is the claim owner.
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Update value with key for storage map
            Proofs::<T>::mutate(&claim, |v| {
                *v = Some((to.clone(), block_number))
            });

            // emits event
            Self::deposit_event(Event::ClaimTrasnferred(sender, to, claim));

            Ok(().into())
        }
    }
}

