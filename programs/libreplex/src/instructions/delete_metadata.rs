use anchor_lang::prelude::*;

use crate::{state::{CollectionData, Metadata}, CollectionPermissions, assert_valid_user_permissions};
use prog_common::{close_account, TrySub, errors::ErrorCode};

#[derive(Accounts)]
#[instruction(bump_collection_data: u8, bump_metadata: u8)]
pub struct DeleteMetadata<'info> {

    pub authority: Signer<'info>,

    #[account(
        seeds = ["permissions".as_ref(), collection_data.key().as_ref(), authority.key().as_ref()], 
        bump)]
    pub user_permissions: Box<Account<'info, CollectionPermissions>>,

    #[account(mut)]
    pub collection_data: Box<Account<'info, CollectionData>>,

    #[account(mut, seeds = [b"metadata".as_ref(), mint.key().as_ref()],
              bump = bump_metadata, has_one = collection_data, has_one = mint)]
    pub metadata: Box<Account<'info, Metadata>>,

    /// CHECK: Mint address used for seed verification
    pub mint: AccountInfo<'info>,

    /// CHECK: Receiver address for the rent-exempt lamports
    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DeleteMetadata>) -> Result<()> {

    // Set the receiver of the lamports to be reclaimed from the rent of the accounts to be closed
    let receiver = &mut ctx.accounts.receiver;
    let authority = &ctx.accounts.authority;
    let collection_data = &mut ctx.accounts.collection_data;
    let user_permissions = &ctx.accounts.user_permissions;

    assert_valid_user_permissions(user_permissions, &collection_data.key(), authority.key)?;

    if !user_permissions.can_delete_metadatas {
        return Err(ErrorCode::CannotDeleteMetadata.into());
    }

    // Close the collection data state account
    let metadata_account_info = &mut (*ctx.accounts.metadata).to_account_info();
    close_account(metadata_account_info, receiver)?;

    // Decrement collection data counter
    collection_data.collection_count.try_sub_assign(1)?;

    msg!("Metadata with pubkey {} now deleted", ctx.accounts.metadata.key());
    Ok(())
}
