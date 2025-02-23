use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use libreplex_inscriptions::{
    cpi::accounts::WriteToInscription,
    instructions::WriteToInscriptionInput,
    program::LibreplexInscriptions,
};

use crate::legacy_inscription::LegacyInscription;

// // having to redefine this here as otherwise anchor IDL will be missing a type
// // hope this gets sorted at some point!
// #[derive(Clone, AnchorDeserialize, AnchorSerialize)]
// pub struct WriteToLegacyInscriptionInput {
//     pub data: Vec<u8>,
//     pub start_pos: u32,
//     pub media_type: Option<MediaType>
// }


// // and another replicated struct
// #[derive(Clone, AnchorDeserialize, AnchorSerialize)]
// pub enum MediaType {
//     // every inscription starts its lifecycle with MediaType == None
//     // when media is inscribed, the media type is updated accordingly
//     None,
//     // mime types
//     Audio {
//         subtype: String
//     },
//     Application {
//         subtype: String
//     },
//     Image {
//         subtype: String
//     },
//     Video {
//         subtype: String
//     },
//     Text {
//         subtype: String
//     },
//     Custom {
//         // allows you to specify full media type such as
//         // animalia/hedgehog
//         media_type: String
//     },
//     /* OTHER CUSTOM TYPES */

//     // ERC gets its own MediaType because of its prevalence, even
//     // though technically it is a subset of application/json
//     Erc721
// }

// Adds a metadata to a group
#[derive(Accounts)]
pub struct WriteToLegacyInscriptionAsHolder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: Checked via a CPI call
    #[account(mut)]
    pub inscription: UncheckedAccount<'info>,

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

    pub system_program: Program<'info, System>,

    pub inscriptions_program: Program<'info, LibreplexInscriptions>,
}

pub fn handler(
    ctx: Context<WriteToLegacyInscriptionAsHolder>,
    input: WriteToInscriptionInput,
) -> Result<()> {
    let inscriptions_program = &ctx.accounts.inscriptions_program;
    let inscription = &mut ctx.accounts.inscription;

    let inscription_data = &mut ctx.accounts.inscription_data;
    let owner = &mut ctx.accounts.owner;
    let system_program = &ctx.accounts.system_program;
    let mint = &ctx.accounts.mint;
    let legacy_inscription = &ctx.accounts.legacy_inscription;
    // make sure we are dealing with the correct metadata object.
    // this is to ensure that the mint in question is in fact a legacy
    // metadata object
    let mint_key = mint.key();
    let inscription_auth_seeds: &[&[u8]] = &[
        "legacy_inscription".as_bytes(),
        mint_key.as_ref(),
        &[ctx.bumps.legacy_inscription],
    ];

    libreplex_inscriptions::cpi::write_to_inscription(
        CpiContext::new_with_signer(
            inscriptions_program.to_account_info(),
            WriteToInscription {
                authority: legacy_inscription.to_account_info(),
                payer: owner.to_account_info(),
                inscription: inscription.to_account_info(),
                system_program: system_program.to_account_info(),
                inscription_data: inscription_data.to_account_info(),
            },
            &[inscription_auth_seeds],
        ),
        input
    )?;

    Ok(())
}
