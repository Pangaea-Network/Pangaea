//! Module for managing deeds
// No std library inorder to compile to wasm
#![cfg_attr(not(feature = "std"), no_std)]


use frame_support::{ codec::{Encode, Decode },
    decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
	// Define Events  
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

//Structures for property deeds
#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct StreetAddress<AccountId> {
	address: Vec<u8>,
    issuer: AccountId,
    owner: AccountId,
}

type InnerStreet<T> = StreetAddress<<T as frame_system::Config>::AccountId>;

decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	trait Store for Module<T: Config> as DeedModule {
		Deed: map hasher(blake2_128_concat) Vec<u8> => InnerStreet<T>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
        //Event emitted when property deed is created on chain
        DeedCreated(AccountId, Vec<u8>),
        //Event emitted when property is destroyed
        PropertyBurn(AccountId, Vec<u8>),
        //Event emitted when property deed is transfered
        DeedTransfer(AccountId, AccountId, Vec<u8>),
    }
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
        /// The land Address has already been claimed.
        AddressAlreadyClaimed,
        NotAddressIssuer,
        NoSuchProperty,
        NotOwner
    }
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;
        #[weight = 10_000]
        fn create_deed(origin, address: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let issuer = ensure_signed(origin)?;
            let deed_token = StreetAddress {
                address: address.clone(),
                issuer: issuer.clone(),
                owner: issuer.clone(),
            };
            // Verify that the specified proof has not already been claimed.
            ensure!(!Deed::<T>::contains_key(&address), Error::<T>::AddressAlreadyClaimed);
            
            // Store the proof with the sender and block number.
            <Deed<T>>::insert(&address, deed_token);

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::DeedCreated(issuer, address));
        }
    

        
        #[weight = 10_000]
        fn revoke_deed(origin, address: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Deed::<T>::contains_key(&address), Error::<T>::NoSuchProperty);

            // Get owner of the claim.
            let deed_token = <Deed::<T>>::get(&address);
            let issuer = deed_token.issuer;
            // Verify that sender of the current call is the claim owner.
            ensure!(issuer == who, Error::<T>::NotAddressIssuer);

            // Remove claim from storage.
            <Deed::<T>>::remove(&address);

            // Emit an event that the claim was erased
            Self::deposit_event(RawEvent::PropertyBurn(who, address));
        }
        //transfer deed
        #[weight = 10_000]
        fn transfer_owner(origin, address: Vec<u8>, reciever: T::AccountId) {
            
            let who = ensure_signed(origin)?;
            ensure!(Deed::<T>::contains_key(&address), Error::<T>::NoSuchProperty);
            let deed_token = <Deed::<T>>::get(&address);
            let property_owner = deed_token.owner;
            ensure!(property_owner == who, Error::<T>::NotOwner);
            let new_owner = StreetAddress {
                address: address.clone(),
                issuer: deed_token.issuer.clone(),
                owner: reciever.clone(),
            };
            <Deed::<T>>::insert(&address, new_owner);

            Self::deposit_event(RawEvent::DeedTransfer(who, reciever, address));



        }
    }
}