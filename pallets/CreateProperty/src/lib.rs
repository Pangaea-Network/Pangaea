//! Module for managing deeds
// No std library inorder to compile to wasm
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{EnsureOrigin};
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
    type ApproveIssuer: EnsureOrigin<Self::Origin>;
    type FreezeOrigin: EnsureOrigin<Self::Origin>;
}

//Structures for property deeds
#[derive(Encode, Decode, Debug, Clone, Default)]
pub struct StreetAddress<AccountId> {
	address: Vec<u8>,
    issuer: AccountId,
    owner: AccountId,
    freeze: bool,
}

type InnerDeed<T> = StreetAddress<<T as frame_system::Config>::AccountId>;

decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	trait Store for Module<T: Config> as DeedModule {
        Deed get(fn get_deed): map hasher(blake2_128_concat) Vec<u8> => InnerDeed<T>;
        IssuerCheck get(fn issuer_check): map hasher(blake2_128_concat) T::AccountId => bool;
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
        //Allow to issue land
        IssuerCreated(AccountId),
        //Disallow to issue land
        IssuerDestroy(AccountId),
        //freeze property
        FreezeDeed(Vec<u8>),
    }
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
        /// The land Address has already been claimed.
        AddressAlreadyClaimed,
        NotAddressIssuer,
        NoSuchProperty,
        NotOwner,
        NotOwnerOrIssuer,
        UnpermittedIssuer,
        PropertyFrozen,
        NotCouncilMember,        
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
            let issuer = ensure_signed(origin)?;
            //ensure issuer is allowed to mint new land
            let check = <IssuerCheck::<T>>::get(&issuer);
            ensure!(check == true, Error::<T>::UnpermittedIssuer);
            let deed_token = StreetAddress {
                address: address.clone(),
                issuer: issuer.clone(),
                owner: issuer.clone(),
                freeze: false,
            };
            // Verify that the specified proof has not already been claimed.
            ensure!(!Deed::<T>::contains_key(&address), Error::<T>::AddressAlreadyClaimed);
                       
            // Store the proof with the sender and block number.
            <Deed<T>>::insert(&address, deed_token);

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::DeedCreated(issuer, address));
        }

        //Allows for an adress to issue deeds, must be approved by referendum
        #[weight = 10000]
        fn issuer_create(origin, minted_issuer: T::AccountId) {
            T::ApproveIssuer::ensure_origin(origin)?;
            <IssuerCheck::<T>>::insert(&minted_issuer, true);
            Self::deposit_event(RawEvent::IssuerCreated(minted_issuer));
        }

        //
        #[weight = 10000]
        fn issuer_freeze(origin, freeze_issuer: T::AccountId) {
            T::ApproveIssuer::ensure_origin(origin)?;
            <IssuerCheck::<T>>::insert(&freeze_issuer, false);
            Self::deposit_event(RawEvent::IssuerDestroy(freeze_issuer));
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
            ensure!( deed_token.freeze == false, Error::<T>::PropertyFrozen);
            let property_owner = deed_token.owner;
            ensure!(property_owner == who, Error::<T>::NotOwner);
            let new_owner = StreetAddress {
                address: address.clone(),
                issuer: deed_token.issuer,
                owner: reciever.clone(),
                freeze: deed_token.freeze,
            };
            <Deed::<T>>::insert(&address, new_owner);

            Self::deposit_event(RawEvent::DeedTransfer(who, reciever, address));



        }

        //freeze deed
        #[weight = 10_000]
        fn freeze_member(origin, address: Vec<u8>) {
            T::FreezeOrigin::ensure_origin(origin)?;           
            ensure!(Deed::<T>::contains_key(&address), Error::<T>::NoSuchProperty);
            let deed_token = <Deed::<T>>::get(&address);
            let frozen = StreetAddress {
                address: deed_token.address,
                issuer: deed_token.issuer,
                owner: deed_token.owner,
                freeze: true,
            };
            <Deed::<T>>::insert( &address, frozen);

            Self::deposit_event(RawEvent::FreezeDeed(address));
        }

        #[weight = 10_000]
        fn freeze_owner(origin, address: Vec<u8>) {
            let who = ensure_signed(origin)?;
            ensure!(Deed::<T>::contains_key(&address), Error::<T>::NoSuchProperty);
            let deed_token = <Deed::<T>>::get(&address);
            ensure!( who == deed_token.owner || who == deed_token.issuer, Error::<T>::NotOwnerOrIssuer);
            let frozen = StreetAddress {
                address: deed_token.address,
                issuer: deed_token.issuer,
                owner: deed_token.owner,
                freeze: true,
            };
            <Deed::<T>>::insert(&address, frozen);
            Self::deposit_event(RawEvent::FreezeDeed(address));

        }
        
    }
}