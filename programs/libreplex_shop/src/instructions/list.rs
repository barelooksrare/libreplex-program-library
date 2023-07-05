use crate::state::{Listing, Price};
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{TokenAccount},
};
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use libreplex_shared::transfer_tokens;

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct ListInput {
    pub price: Price,
    pub amount: u64,
    pub listing_bump: u8
}

impl ListInput {
    pub fn get_size(&self) -> u32 {
        2 + match &self.price {
            Price::Native { lamports: _ } => 8,
            Price::Spl { mint: _, amount: _ } => 32 + 8,
        }
    }
}

#[derive(Accounts)]
#[instruction(list_input: ListInput)]
pub struct List<'info> {
    #[account(mut)]
    pub lister: Signer<'info>,

    /// CHECK: Checked against ID constraint
    #[account(
        constraint = mint.owner.eq(&TOKEN_2022_PROGRAM_ID)
    )]
    pub mint: UncheckedAccount<'info>,

    #[account(init,
    payer=lister,
        space = Listing::BASE_SIZE + list_input.get_size() as usize,
        seeds=[b"listing", mint.key().as_ref()], 
        bump)]
    pub listing: Account<'info, Listing>,

    /// CHECK: Will need to be created, hence unchecked
    #[account(mut)]
    pub escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: Is allowed to be empty in which case we create it
    #[account(
        mut,
        constraint = token_program.key.eq(&TOKEN_2022_PROGRAM_ID)
    )]
    pub lister_token_account:  UncheckedAccount<'info>,
    // Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: Checked against ID constraint
    #[account(
        constraint = token_program.key.eq(&TOKEN_2022_PROGRAM_ID)
    )]
    pub token_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<List>, list_input: ListInput) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let lister = &mut ctx.accounts.lister;
    let lister_token_account = &mut ctx.accounts.lister_token_account;
    let mint = &mut ctx.accounts.mint;
    let escrow_token_account = &mut ctx.accounts.escrow_token_account;
    let associated_token_program = &mut ctx.accounts.associated_token_program;
    let system_program = &mut ctx.accounts.system_program;
    let token_program = &mut ctx.accounts.token_program;
    

    listing.lister = lister.key();
    listing.mint = mint.key();
    listing.price = list_input.price;
    listing.listing_bump = list_input.listing_bump;

    

    transfer_tokens(
        &token_program.to_account_info(),
        &lister_token_account.to_account_info(),
        &escrow_token_account.to_account_info(),
        &lister.to_account_info(),
        &mint.to_account_info(),
        &listing.to_account_info(),
        &&associated_token_program.to_account_info(),
        &&system_program.to_account_info(),
        None,
        &lister.to_account_info(),
        listing.amount,
    )?;

    Ok(())
}
