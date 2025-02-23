use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use libreplex_inscriptions::{
    cpi::accounts::ResizeInscription, instructions::ResizeInscriptionInput,
    program::LibreplexInscriptions, Inscription,
};


use crate::{legacy_inscription::LegacyInscription, LegacyInscriptionErrorCode};

use super::create_legacy_inscription_logic::AuthorityType;


// Adds a metadata to a group
#[derive(Accounts)]
pub struct ResizeLegacyInscriptionAsHolder<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: checked in logic
    pub owner: UncheckedAccount<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub inscription: Account<'info, Inscription>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription_data: UncheckedAccount<'info>,

    #[account(
        token::mint = mint,
        token::authority = owner
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        seeds=[
            "legacy_inscription".as_bytes(),
            mint.key().as_ref()
        ], bump)]
    pub legacy_inscription: Account<'info, LegacyInscription>,

    /// CHECK: The token program
    #[account(
        address = anchor_spl::token::ID
    )]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    pub inscriptions_program: Program<'info, LibreplexInscriptions>,
}

pub fn handler(
    ctx: Context<ResizeLegacyInscriptionAsHolder>,
    input: ResizeLegacyInscriptionInput,
) -> Result<()> {
    let inscriptions_program = &ctx.accounts.inscriptions_program;
    let inscription = &mut ctx.accounts.inscription;
    let inscription_data = &mut ctx.accounts.inscription_data;
    let system_program = &ctx.accounts.system_program;
    let mint = &ctx.accounts.mint;
    let legacy_inscription = &ctx.accounts.legacy_inscription;
    let payer = &ctx.accounts.payer;

    if ctx.accounts.authority.key() != ctx.accounts.owner.key()
            || legacy_inscription.authority_type != AuthorityType::Holder
    {
        // return bad authority - only the owner of the mint / update authority can sign
        return Err(LegacyInscriptionErrorCode::BadAuthority.into());
    }

    let mint_key = mint.key();
    let inscription_auth_seeds: &[&[u8]] = &[
        "legacy_inscription".as_bytes(),
        mint_key.as_ref(),
        &[ctx.bumps.legacy_inscription],
    ];

    libreplex_inscriptions::cpi::resize_inscription(
        CpiContext::new_with_signer(
            inscriptions_program.to_account_info(),
            ResizeInscription {
                payer: payer.to_account_info(),
                authority: legacy_inscription.to_account_info(),
                inscription: inscription.to_account_info(),
                system_program: system_program.to_account_info(),
                inscription_data: inscription_data.to_account_info(),
            },
            &[inscription_auth_seeds],
        ),
        ResizeInscriptionInput {
            change: input.change,
            expected_start_size: input.expected_start_size,
            target_size: input.target_size,
        },
    )?;

    Ok(())
}
