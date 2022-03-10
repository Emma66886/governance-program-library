use anchor_lang::{
    account,
    prelude::{Context, Signer},
    Accounts,
};

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use spl_governance::state::realm;

use crate::error::NftLockerError;
use crate::state::{CollectionConfig, Registrar};

#[derive(Accounts)]
pub struct ConfigureCollection<'info> {
    /// Registrar for which we configure this Collection
    #[account(mut,
        constraint = registrar.realm == realm.key() @ NftLockerError::InvalidRegistrarRealm
    )]
    pub registrar: Account<'info, Registrar>,

    /// CHECK: Owned by spl-governance instance specified in registrar.governance_program_id
    pub realm: UncheckedAccount<'info>,

    /// Authority of the Realm
    pub realm_authority: Signer<'info>,

    // Collection which is going to be used for voting
    pub collection: Account<'info, Mint>,
}

pub fn configure_collection(
    ctx: Context<ConfigureCollection>,
    weight: u16,
    size: u32,
) -> Result<()> {
    require!(weight > 0, NftLockerError::InvalidCollectionWeight);
    require!(size > 0, NftLockerError::InvalidCollectionSize);

    let registrar = &mut ctx.accounts.registrar;

    let realm = realm::get_realm_data_for_governing_token_mint(
        &registrar.governance_program_id,
        &ctx.accounts.realm.to_account_info(),
        &registrar.governing_token_mint,
    )?;

    require!(
        realm.authority.unwrap() == ctx.accounts.realm_authority.key(),
        NftLockerError::InvalidRealmAuthority
    );

    let collection = &ctx.accounts.collection;

    let collection_config = CollectionConfig {
        collection: collection.key(),
        weight,
        reserved: [0; 8],
        size,
    };

    let collection_idx = registrar
        .collection_configs
        .iter()
        .position(|cc| cc.collection == collection.key());

    if let Some(collection_idx) = collection_idx {
        registrar.collection_configs[collection_idx] = collection_config;
    } else {
        // Note: In the current runtime version push() would throw an error if we exceed
        // max_collections specified when the Registrar was created
        registrar.collection_configs.push(collection_config);
    }

    // TODO:
    // check max vote weight

    Ok(())
}
