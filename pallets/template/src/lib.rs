#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use codec::FullCodec;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure,
    traits::{EnsureOrigin, Get},
    Hashable,};

use frame_system::ensure_signed;
use sp_runtime::traits::{Hash, Member};
use sp_std::{cmp::Eq, fmt::Debug, vec::Vec};

pub mod nft;
pub use crate::nft::UniqueAssets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


pub trait Trait<I = DefaultInstance>: frame_system::Trait {
	type CommodityAdmin: EnsureOrigin<Self::Origin>;
    /// The data type that is used to describe this type of commodity.
    type CommodityInfo: Hashable + Member + Debug + Default + FullCodec + Ord;
    /// The maximum number of this type of commodity that may exist (minted - burned).
    type CommodityLimit: Get<u128>;
    /// The maximum number of this type of commodity that any single account may own.
    type UserCommodityLimit: Get<u64>;
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;
}
/// The runtime system's hashing algorithm is used to uniquely identify commodities.
pub type CommodityId<T> = <T as frame_system::Trait>::Hash;

/// Associates a commodity with its ID.
pub type Commodity<T, I> = (CommodityId<T>, <T as Trait<I>>::CommodityInfo);



// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait<I>, I: Instance = DefaultInstance> as Commodity {
        /// The total number of this type of commodity that exists (minted - burned).
        Total get(fn total): u128 = 0;
        /// The total number of this type of commodity that has been burned (may overflow).
        Burned get(fn burned): u128 = 0;
        /// The total number of this type of commodity owned by an account.
        TotalForAccount get(fn total_for_account): map hasher(blake2_128_concat) T::AccountId => u64 = 0;
        /// A mapping from an account to a list of all of the commodities of this type that are owned by it.
        CommoditiesForAccount get(fn commodities_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<Commodity<T, I>>;
        /// A mapping from a commodity ID to the account that owns it.
        AccountForCommodity get(fn account_for_commodity): map hasher(identity) CommodityId<T> => T::AccountId;
    }

    add_extra_genesis {
        config(balances): Vec<(T::AccountId, Vec<T::CommodityInfo>)>;
        build(|config: &GenesisConfig<T, I>| {
            for (who, assets) in config.balances.iter() {
                for asset in assets {
                    match <Module::<T, I> as UniqueAssets::<T::AccountId>>::mint(who, asset.clone()) {
                        Ok(_) => {}
                        Err(err) => { panic!(err) },
                    }
                }
            }
        });
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	// pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
	// 	/// Event documentation should end with an array that provides descriptive names for event
	// 	/// parameters. [something, who]
	// 	SomethingStored(u32, AccountId),
	// }
	pub enum Event<T, I = DefaultInstance>
    where
        CommodityId = <T as frame_system::Trait>::Hash,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// The commodity has been burned.
        Burned(CommodityId),
        /// The commodity has been minted and distributed to the account.
        Minted(CommodityId, AccountId),
        /// Ownership of the commodity has been transferred to the account.
		Transferred(CommodityId, AccountId),
		SomethingStored(u32, AccountId),
    }
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait<I>, I: Instance> {
        // Thrown when there is an attempt to mint a duplicate commodity.
        CommodityExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent commodity.
        NonexistentCommodity,
        // Thrown when someone who is not the owner of a commodity attempts to transfer or burn it.
        NotCommodityOwner,
        // Thrown when the commodity admin attempts to mint a commodity and the maximum number of this
        // type of commodity already exists.
        TooManyCommodities,
        // Thrown when an attempt is made to mint or transfer a commodity to an account that already
        // owns the maximum number of this type of commodity.
        TooManyCommoditiesForAccount,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait<I>, I: Instance = DefaultInstance> for enum Call where origin: T::Origin {
    
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T, I>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		// / An example dispatchable that takes a singles value as a parameter, writes the value to
		// / storage and emits an event. This function must be dispatched by a signed extrinsic.
		// #[weight = 10_000 + T::DbWeight::get().writes(1)]
		// pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
		// 	// Check that the extrinsic was signed and get the signer.
		// 	// This function will return an error if the extrinsic is not signed.
		// 	// https://substrate.dev/docs/en/knowledgebase/runtime/origin
		// 	let who = ensure_signed(origin)?;

		// 	// Update storage.
		// 	Something::put(something);

		// 	// Emit an event.
		// 	Self::deposit_event(RawEvent::SomethingStored(something, who));
		// 	// Return a successful DispatchResult
		// 	Ok(())
		// }

		#[weight = 10_000]
        pub fn mint(origin, owner_account: T::AccountId, commodity_info: T::CommodityInfo) -> dispatch::DispatchResult {
            T::CommodityAdmin::ensure_origin(origin)?;

            let commodity_id = <Self as UniqueAssets<_>>::mint(&owner_account, commodity_info)?;
            Self::deposit_event(RawEvent::Minted(commodity_id, owner_account.clone()));
            Ok(())
		}
		
		#[weight = 10_000]
        pub fn burn(origin, commodity_id: CommodityId<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::account_for_commodity(&commodity_id), Error::<T, I>::NotCommodityOwner);

            <Self as UniqueAssets<_>>::burn(&commodity_id)?;
            Self::deposit_event(RawEvent::Burned(commodity_id.clone()));
            Ok(())
		}
		#[weight = 10_000]
        pub fn transfer(origin, dest_account: T::AccountId, commodity_id: CommodityId<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::account_for_commodity(&commodity_id), Error::<T, I>::NotCommodityOwner);

            <Self as UniqueAssets<_>>::transfer(&dest_account, &commodity_id)?;
            Self::deposit_event(RawEvent::Transferred(commodity_id.clone(), dest_account.clone()));
            Ok(())
        }

		
	}
}



impl<T: Trait<I>, I: Instance> UniqueAssets<T::AccountId> for Module<T, I> {
    type AssetId = CommodityId<T>;
    type AssetInfo = T::CommodityInfo;
    type AssetLimit = T::CommodityLimit;
    type UserAssetLimit = T::UserCommodityLimit;

    fn total() -> u128 {
        Self::total()
    }

    fn burned() -> u128 {
        Self::burned()
    }

    fn total_for_account(account: &T::AccountId) -> u64 {
        Self::total_for_account(account)
    }

    fn assets_for_account(account: &T::AccountId) -> Vec<Commodity<T, I>> {
        Self::commodities_for_account(account)
    }

    fn owner_of(commodity_id: &CommodityId<T>) -> T::AccountId {
        Self::account_for_commodity(commodity_id)
    }

    fn mint(
        owner_account: &T::AccountId,
        commodity_info: <T as Trait<I>>::CommodityInfo,
    ) -> dispatch::result::Result<CommodityId<T>, dispatch::DispatchError> {
        let commodity_id = T::Hashing::hash_of(&commodity_info);

        ensure!(
            !AccountForCommodity::<T, I>::contains_key(&commodity_id),
            Error::<T, I>::CommodityExists
        );

        ensure!(
            Self::total_for_account(owner_account) < T::UserCommodityLimit::get(),
            Error::<T, I>::TooManyCommoditiesForAccount
        );

        ensure!(
            Self::total() < T::CommodityLimit::get(),
            Error::<T, I>::TooManyCommodities
        );

        let new_commodity = (commodity_id, commodity_info);

        Total::<I>::mutate(|total| *total += 1);
        TotalForAccount::<T, I>::mutate(owner_account, |total| *total += 1);
        CommoditiesForAccount::<T, I>::mutate(owner_account, |commodities| {
            match commodities.binary_search(&new_commodity) {
                Ok(_pos) => {} // should never happen
                Err(pos) => commodities.insert(pos, new_commodity),
            }
        });
        AccountForCommodity::<T, I>::insert(commodity_id, &owner_account);

        Ok(commodity_id)
    }

    fn burn(commodity_id: &CommodityId<T>) -> dispatch::DispatchResult {
        let owner = Self::owner_of(commodity_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T, I>::NonexistentCommodity
        );

        let burn_commodity = (*commodity_id, <T as Trait<I>>::CommodityInfo::default());

        Total::<I>::mutate(|total| *total -= 1);
        Burned::<I>::mutate(|total| *total += 1);
        TotalForAccount::<T, I>::mutate(&owner, |total| *total -= 1);
        CommoditiesForAccount::<T, I>::mutate(owner, |commodities| {
            let pos = commodities
                .binary_search(&burn_commodity)
                .expect("We already checked that we have the correct owner; qed");
            commodities.remove(pos);
        });
        AccountForCommodity::<T, I>::remove(&commodity_id);

        Ok(())
    }

    fn transfer(
        dest_account: &T::AccountId,
        commodity_id: &CommodityId<T>,
    ) -> dispatch::DispatchResult {
        let owner = Self::owner_of(&commodity_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T, I>::NonexistentCommodity
        );

        ensure!(
            Self::total_for_account(dest_account) < T::UserCommodityLimit::get(),
            Error::<T, I>::TooManyCommoditiesForAccount
        );

        let xfer_commodity = (*commodity_id, <T as Trait<I>>::CommodityInfo::default());

        TotalForAccount::<T, I>::mutate(&owner, |total| *total -= 1);
        TotalForAccount::<T, I>::mutate(dest_account, |total| *total += 1);
        let commodity = CommoditiesForAccount::<T, I>::mutate(owner, |commodities| {
            let pos = commodities
                .binary_search(&xfer_commodity)
                .expect("We already checked that we have the correct owner; qed");
            commodities.remove(pos)
        });
        CommoditiesForAccount::<T, I>::mutate(dest_account, |commodities| {
            match commodities.binary_search(&commodity) {
                Ok(_pos) => {} // should never happen
                Err(pos) => commodities.insert(pos, commodity),
            }
        });
        AccountForCommodity::<T, I>::insert(&commodity_id, &dest_account);

        Ok(())
    }
}