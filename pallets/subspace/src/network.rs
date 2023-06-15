use super::*;
use frame_support::{sp_std::vec};
use sp_std::vec::Vec;
use frame_system::ensure_root;
use frame_support::IterableStorageDoubleMap;
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
use codec::Compact;
use frame_support::pallet_prelude::{DispatchError, DispatchResult};
extern crate alloc;


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfo {
    netuid: Compact<u16>,
    name: Vec<u8>,
    immunity_period: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,
    blocks_since_last_step: Compact<u64>,
    epoch: Compact<u16>,
}


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetParams {
    immunity_period: u16,
    min_allowed_weights: u16,
    max_allowed_uids: u16,
    epoch: u16,
}






impl<T: Config> Pallet<T> { 


    // Returns true if the subnetwork exists.
    //
    pub fn if_subnet_exist( netuid: u16 ) -> bool{
        return SubnetworkN::<T>::contains_key( netuid );
    }

    // get the least staked network
    pub fn least_staked_netuid(stake: u64) -> u16 {
        let mut min_stake: u64 = 0;
        let mut min_stake_netuid: u16 = 0;
        if stake > min_stake {
            for ( netuid, net_stake ) in <TotalSubnetStake<T> as IterableStorageMap<u16, u64> >::iter(){
                if net_stake <= min_stake {
                    min_stake = net_stake;
                    min_stake_netuid = netuid;
                }
            }
        }
        return min_stake_netuid;
    }


    pub fn get_network_stake( netuid: u16 ) -> u64 {
        return TotalSubnetStake::<T>::get( netuid );
    }


    pub fn do_add_network( 
        origin: T::RuntimeOrigin,
        name: Vec<u8>,
        stake: u64,
        immunity_period: u16,
        min_allowed_weights: u16,
        max_allowed_uids: u16,
        epoch: u16,
    ) -> DispatchResult {

        let key = ensure_signed(origin)?;
        // --- 1. Ensure the network name does not already exist.
        ensure!( !Self::if_subnet_name_exists( name.clone() ), Error::<T>::SubnetNameAlreadyExists );

        // --- 16. Ok and done.
        Ok(())
    }


    pub fn default_subnet_params() -> SubnetParams {
        let netuid: u16 = 0;
        return SubnetParams {
            immunity_period: MinAllowedWeights::<T>::get( netuid ),
            min_allowed_weights: ImmunityPeriod::<T>::get( netuid ),
            max_allowed_uids:  MaxAllowedUids::<T>::get( netuid ),
            epoch: Epoch::<T>::get( netuid ),
        }
    }


    pub fn add_network_from_registration( 
        name: Vec<u8>,
        stake: u64,
        key : &T::AccountId,
    ) -> u16 {


        let default_params  = Self::default_subnet_params();

        let netuid = Self::add_network( &key, 
                            name.clone(),
                            stake , 
                            default_params.max_allowed_uids, 
                            default_params.immunity_period,
                            default_params.min_allowed_weights,
                            default_params.epoch);

        // --- 16. Ok and done.
        return netuid;
    }

    pub fn add_network(key: &T::AccountId,  
                       name: Vec<u8>,
                       stake: u64,
                       max_allowed_uids: u16,
                       immunity_period: u16,
                       min_allowed_weights: u16,
                       epoch: u16,
                    ) -> u16 {

        // --- 1. Enfnsure that the network name does not already exist.
        let total_networks = TotalNetworks::<T>::get();
        let max_networks = MaxAllowedSubnets::<T>::get();
        // if networks exceeds max_networks, remove the least staked network
        let netuid : u16 ; 
        if total_networks >= max_networks {
            netuid = Self::least_staked_netuid(stake);
            Self::remove_network_for_netuid( netuid );
        } else {
            netuid = total_networks;

        }

        Epoch::<T>::insert( netuid, epoch);
        MaxAllowedUids::<T>::insert( netuid, max_allowed_uids );
        ImmunityPeriod::<T>::insert( netuid, immunity_period );
        MinAllowedWeights::<T>::insert( netuid, min_allowed_weights );
        RegistrationsThisInterval::<T>::insert( netuid, 0);
        SubnetNamespace::<T>::insert( name.clone(), netuid );

        // set stat once network is created
        TotalNetworks::<T>::mutate( |n| *n += 1 );
        SubnetFounder::<T>::insert( netuid, &key.clone() );
        SubnetworkN::<T>::insert( netuid, 1 );

        // --- 6. Emit the new network event.
        log::info!("NetworkAdded( netuid:{:?}, name:{:?} )", netuid, name.clone());
        Self::deposit_event( Event::NetworkAdded( netuid, name.clone()) );
    

        return netuid;

    }



    // Initializes a new subnetwork under netuid with parameters.
    //
    pub fn if_subnet_name_exists(name: Vec<u8>) -> bool {
       
   
        return  SubnetNamespace::<T>::contains_key(name.clone()).into();
    }


    pub fn get_netuid_for_name( name: Vec<u8> ) -> u16 {
        
        let netuid: u16 = SubnetNamespace::<T>::get(name.clone());
        return netuid;
    }


    pub fn get_name_for_netuid( netuid : u16) -> Vec<u8> {
        for ( name, _netuid ) in <SubnetNamespace<T> as IterableStorageMap<Vec<u8>, u16>>::iter(){
            if _netuid == netuid {
                return name;
            }
        }
        return Vec::new();
    }




    // Removes the network (netuid) and all of its parameters.
    //
    pub fn remove_network_for_netuid( netuid: u16 ) -> u16 {
        let name = Self::get_name_for_netuid( netuid );
        return Self::remove_network( name );
    }

    // Returns true if the account is the founder of the network.
    pub fn is_network_founder( netuid: u16, key: &T::AccountId ) -> bool {
        let founder = SubnetFounder::<T>::get( netuid );
        return founder == key.clone();
    }


    pub fn remove_network( name: Vec<u8>) -> u16 {
        // --- 2. Ensure the network to be removed exists.
        if !Self::if_subnet_name_exists( name.clone() ) {
            return 0;
        }
        let netuid = Self::get_netuid_for_name( name.clone() );
        SubnetNamespace::<T>::remove( name.clone() );
        // --- 4. Erase all memory associated with the network.

        // --- 1. Remove incentive mechanism memory.
        Uids::<T>::clear_prefix( netuid, u32::max_value(), None );
        Keys::<T>::clear_prefix( netuid, u32::max_value(), None );
        Weights::<T>::clear_prefix( netuid, u32::max_value(), None );
        Emission::<T>::remove( netuid );
        Incentive::<T>::remove( netuid );
        Dividends::<T>::remove( netuid );
        LastUpdate::<T>::remove( netuid );
        SubnetFounder::<T>::remove( netuid );

        // --- 2. Erase network parameters.
        Epoch::<T>::remove( netuid );
        MaxAllowedUids::<T>::remove( netuid );
        ImmunityPeriod::<T>::remove( netuid );
        MinAllowedWeights::<T>::remove( netuid );
        RegistrationsThisInterval::<T>::remove( netuid );
        SubnetworkN::<T>::remove( netuid );

        // --- 3. Erase network stake, and remove network from list of networks.
        for ( key, stated_amount ) in <Stake<T> as IterableStorageDoubleMap<u16, T::AccountId, u64> >::iter_prefix(netuid){
            Self::decrease_stake_on_account( netuid, &key, stated_amount );
        }
        // --- 4. Remove all stake.
        Stake::<T>::remove_prefix( netuid, None );
        let total_network_stake: u16 = TotalSubnetStake::<T>::get( netuid ).try_into().unwrap();
        TotalSubnetStake::<T>::mutate(netuid, |n| *n -= total_network_stake as u64 );
        TotalSubnetStake::<T>::remove( netuid );


        TotalNetworks::<T>::mutate(|val| *val -= 1);
        // --- 4. Emit the event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event( Event::NetworkRemoved( netuid ) );

        return netuid;
        

    }

    // Returns true if the passed epoch is allowed.
    //
    pub fn if_epoch_is_valid(epoch: u16) -> bool {
        epoch < u16::MAX
    }


	pub fn get_subnet_info(netuid: u16) -> Option<SubnetInfo> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let immunity_period = Self::get_immunity_period(netuid);
        let name = Self::get_name_for_netuid(netuid);
        let min_allowed_weights = Self::get_min_allowed_weights(netuid);
        let subnetwork_n = Self::get_subnetwork_n(netuid);
        let max_allowed_uids = Self::get_max_allowed_uids(netuid);
        let blocks_since_last_step = Self::get_blocks_since_last_step(netuid);
        let epoch = Self::get_epoch(netuid);



        return Some(SubnetInfo {
            immunity_period: immunity_period.into(),
            name: name,
            netuid: netuid.into(),
            min_allowed_weights: min_allowed_weights.into(),
            subnetwork_n: subnetwork_n.into(),
            max_allowed_uids: max_allowed_uids.into(),
            blocks_since_last_step: blocks_since_last_step.into(),
            epoch: epoch.into(),
        })
	}

    pub fn get_subnets_info() -> Vec<Option<SubnetInfo>> {
        let mut subnets_info = Vec::<Option<SubnetInfo>>::new();
        for ( netuid, net_n ) in < SubnetworkN<T> as IterableStorageMap<u16, u16> >::iter() {
            subnets_info.push(Self::get_subnet_info(netuid));
        }
        return subnets_info;
	}


    // Returns the number of filled slots on a network.
    ///
    pub fn get_subnetwork_n( netuid:u16 ) -> u16 { 
        return SubnetworkN::<T>::get( netuid ) 
    }
    

  

    // Returns true if the uid is set on the network.
    //
    pub fn is_uid_exist_on_network(netuid: u16, uid: u16) -> bool {
        return  Keys::<T>::contains_key(netuid, uid);
    }

    // Returns true if the key holds a slot on the network.
    //
    pub fn is_key_registered_on_network( netuid:u16, key: &T::AccountId ) -> bool { 
        return Uids::<T>::contains_key( netuid, key ) 
    }

    // Returs the key under the network uid as a Result. Ok if the uid is taken.
    //
    pub fn get_key_for_uid( netuid: u16, module_uid: u16) ->  T::AccountId {
        Keys::<T>::try_get(netuid, module_uid).unwrap() 
    }
    

    // Returns the uid of the key in the network as a Result. Ok if the key has a slot.
    //
    pub fn get_uid_for_key( netuid: u16, key: &T::AccountId) -> Result<u16, DispatchError> { 
        return Uids::<T>::try_get(netuid, &key).map_err(|_err| Error::<T>::NotRegistered.into()) 
    }

    pub fn get_uid_for_name ( netuid: u16, name: Vec<u8> ) -> u16  {
        return ModuleNamespace::<T>::get(netuid, name)
    }

    pub fn get_name_for_uid ( netuid: u16, uid: u16 ) -> Vec<u8>  {
        return Names::<T>::get(netuid, uid);
    }


    pub fn if_module_name_exists( netuid: u16, name: Vec<u8> ) -> bool {
        return ModuleNamespace::<T>::contains_key( netuid, name.clone() );
        
    }

    // Returns the stake of the uid on network or 0 if it doesnt exist.
    //
    pub fn get_stake_for_uid( netuid: u16, module_uid: u16) -> u64 { 
        return Self::get_stake_for_key( netuid, &Self::get_key_for_uid( netuid, module_uid) )
    }

    pub fn get_stake_for_key( netuid: u16, key: &T::AccountId) -> u64 { 
        if Self::is_key_registered_on_network( netuid, &key) {
            return Stake::<T>::get( netuid, &key );
        } else {
            return 0;
        }
    }
    
    
    // Return the total number of subnetworks available on the chain.
    //
    pub fn get_number_of_subnets()-> u16 {
        let mut number_of_subnets : u16 = 0;
        for (_, _)  in <SubnetworkN<T> as IterableStorageMap<u16, u16>>::iter(){
            number_of_subnets = number_of_subnets + 1;
        }
        return number_of_subnets;
    }

    // Return a list of all networks a key is registered on.
    //
    pub fn get_registered_networks_for_key( key: &T::AccountId )-> Vec<u16> {
        let mut all_networks: Vec<u16> = vec![];
        for ( network, is_registered)  in <IsNetworkMember<T> as IterableStorageDoubleMap< T::AccountId, u16, bool >>::iter_prefix( key ){
            if is_registered { all_networks.push( network ) }
        }
        all_networks
    }

    // Return true if a key is registered on any network.
    //
    pub fn is_key_registered_on_any_network( key: &T::AccountId )-> bool {
        for ( _, is_registered)  in <IsNetworkMember<T> as IterableStorageDoubleMap< T::AccountId, u16, bool >>::iter_prefix( key ){
            if is_registered { return true }
        }
        false
    }

    // ========================
	// ==== Global Setters ====
	// ========================
    pub fn set_epoch( netuid: u16, epoch: u16 ) { Epoch::<T>::insert( netuid, epoch ); }
    pub fn set_last_adjustment_block( netuid: u16, last_adjustment_block: u64 ) { LastAdjustmentBlock::<T>::insert( netuid, last_adjustment_block ); }
    pub fn set_blocks_since_last_step( netuid: u16, blocks_since_last_step: u64 ) { BlocksSinceLastStep::<T>::insert( netuid, blocks_since_last_step ); }
    pub fn set_registrations_this_block( netuid: u16, registrations_this_block: u16 ) { RegistrationsThisBlock::<T>::insert(netuid, registrations_this_block); }
    pub fn set_last_mechanism_step_block( netuid: u16, last_mechanism_step_block: u64 ) { LastMechansimStepBlock::<T>::insert(netuid, last_mechanism_step_block); }
    pub fn set_registrations_this_interval( netuid: u16, registrations_this_interval: u16 ) { RegistrationsThisInterval::<T>::insert(netuid, registrations_this_interval); }

    // ========================
	// ==== Global Getters ====
	// ========================
    pub fn get_block_emission() -> u64 { BlockEmission::<T>::get() }
    pub fn get_current_block_as_u64( ) -> u64 { TryInto::try_into( <frame_system::Pallet<T>>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.") }

    // ==============================
	// ==== Yomama params ====
	// ==============================
    pub fn get_emission( netuid:u16 ) -> Vec<u64> { Emission::<T>::get( netuid ) }
    pub fn get_incentive( netuid:u16 ) -> Vec<u16> { Incentive::<T>::get( netuid ) }
    pub fn get_dividends( netuid:u16 ) -> Vec<u16> { Dividends::<T>::get( netuid ) }
    pub fn get_last_update( netuid:u16 ) -> Vec<u64> { LastUpdate::<T>::get( netuid ) }
    
    // Emmision is the same as the Yomama params 

    
    pub fn set_last_update_for_uid( netuid:u16, uid: u16, last_update: u64 ) { 
        let mut updated_last_update_vec = Self::get_last_update( netuid ); 
        if (uid as usize) < updated_last_update_vec.len() { 
            updated_last_update_vec[uid as usize] = last_update;
            LastUpdate::<T>::insert( netuid, updated_last_update_vec );
        }  
    }

    pub fn get_emission_for_uid( netuid:u16, uid: u16) -> u64 {let vec =  Emission::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_incentive_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Incentive::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_dividends_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Dividends::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_last_update_for_uid( netuid:u16, uid: u16) -> u64 { let vec = LastUpdate::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_pruning_score_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Emission::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] as u16 } else{ return u16::MAX } }


    // ============================
	// ==== Subnetwork Getters ====
	// ============================
    pub fn get_epoch( netuid:u16 ) -> u16{ Epoch::<T>::get( netuid ) }
    pub fn get_pending_emission( netuid:u16 ) -> u64{ PendingEmission::<T>::get( netuid ) }
    pub fn get_last_adjustment_block( netuid: u16) -> u64 { LastAdjustmentBlock::<T>::get( netuid ) }
    pub fn get_blocks_since_last_step(netuid:u16 ) -> u64 { BlocksSinceLastStep::<T>::get( netuid ) }
    pub fn get_registrations_this_block( netuid:u16 ) -> u16 { RegistrationsThisBlock::<T>::get( netuid ) }
    pub fn get_last_mechanism_step_block( netuid: u16 ) -> u64 { LastMechansimStepBlock::<T>::get( netuid ) }
    pub fn get_registrations_this_interval( netuid: u16 ) -> u16 { RegistrationsThisInterval::<T>::get( netuid ) } 
    pub fn get_module_block_at_registration( netuid: u16, module_uid: u16 ) -> u64 { BlockAtRegistration::<T>::get( netuid, module_uid )}

    // ========================
	// ==== Rate Limiting =====
	// ========================
	pub fn get_last_tx_block( key: &T::AccountId ) -> u64 { LastTxBlock::<T>::get( key ) }
    pub fn set_last_tx_block( key: &T::AccountId, last_tx_block: u64 ) { LastTxBlock::<T>::insert( key, last_tx_block ) }

	// Configure tx rate limiting
	pub fn get_tx_rate_limit() -> u64 { TxRateLimit::<T>::get() }
    pub fn set_tx_rate_limit( tx_rate_limit: u64 ) { TxRateLimit::<T>::put( tx_rate_limit ) }

    pub fn get_weights_set_rate_limit( netuid: u16) -> u64 { WeightsSetRateLimit::<T>::get( netuid ) }
    pub fn set_weights_set_rate_limit( netuid: u16, weights_set_rate_limit: u64 ) { WeightsSetRateLimit::<T>::insert( netuid, weights_set_rate_limit ); }

    pub fn get_adjustment_interval( netuid: u16) -> u16 { AdjustmentInterval::<T>::get( netuid ) }
    pub fn set_adjustment_interval( netuid: u16, adjustment_interval: u16 ) { AdjustmentInterval::<T>::insert( netuid, adjustment_interval ); }

    pub fn get_immunity_period(netuid: u16 ) -> u16 { ImmunityPeriod::<T>::get( netuid ) }
    pub fn set_immunity_period( netuid: u16, immunity_period: u16 ) { ImmunityPeriod::<T>::insert( netuid, immunity_period ); }

    pub fn get_min_allowed_weights( netuid:u16 ) -> u16 {
        let min_allowed_weights = MinAllowedWeights::<T>::get( netuid ) ; 
        let n = Self::get_subnetwork_n(netuid);
        // if n < min_allowed_weights, then return n
        if (n < min_allowed_weights) {
            return n;
        } else {
            return min_allowed_weights;
        }
        }
    pub fn set_min_allowed_weights( netuid: u16, min_allowed_weights: u16 ) { MinAllowedWeights::<T>::insert( netuid, min_allowed_weights ); }

    pub fn get_max_allowed_uids( netuid: u16 ) -> u16  { MaxAllowedUids::<T>::get( netuid ) }
    pub fn set_max_allowed_uids(netuid: u16, max_allowed: u16) { MaxAllowedUids::<T>::insert( netuid, max_allowed ); }
            
    pub fn get_target_registrations_per_interval( netuid: u16 ) -> u16 { TargetRegistrationsPerInterval::<T>::get( netuid ) }
    pub fn set_target_registrations_per_interval( netuid: u16, target_registrations_per_interval: u16 ) { TargetRegistrationsPerInterval::<T>::insert( netuid, target_registrations_per_interval ); }


    pub fn get_max_registrations_per_block( netuid: u16 ) -> u16 { MaxRegistrationsPerBlock::<T>::get( netuid ) }
    pub fn set_max_registrations_per_block( netuid: u16, max_registrations_per_block: u16 ) { MaxRegistrationsPerBlock::<T>::insert( netuid, max_registrations_per_block ); }

}


    
