use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};

use libreplex_inscriptions::{
    cpi::accounts::CreateInscriptionV2,
    cpi::accounts::MakeInscriptionImmutable,
    cpi::accounts::ResizeInscription,
    cpi::accounts::WriteToInscription,
    instructions::{SignerType, WriteToInscriptionInput},
    InscriptionSummary,
};
use libreplex_shared::{
    create_mint_metadata_and_masteredition::create_mint_with_metadata_and_masteredition,
    MintAccounts,
};
use mpl_token_metadata::types::TokenStandard;

use crate::{Deployment, Hashlist};

pub mod sysvar_instructions_program {
    use anchor_lang::declare_id;
    declare_id!("Sysvar1nstructions1111111111111111111111111");
}

/*

    Deploy takes no input parameters as all of the
    string parameter + decimals have already been set by
    initialise.

    Deploy creates all on-chain objects (inscriptions,
    mints + any metadata) that are required to keep track of the
    launch lifecycle.
*/
#[derive(Accounts)]
pub struct DeployLegacyCtx<'info> {
    #[account(
        mut,
        // deployment must be executed by the payer 
        constraint = deployment.creator == payer.key(),
        seeds=["deployment".as_bytes(), deployment.ticker.as_bytes()],
        bump
    )]
    pub deployment: Account<'info, Deployment>,

    #[account(init, seeds = ["hashlist".as_bytes(), 
    deployment.key().as_ref()],
    bump, payer = payer, space = 8 + 32 + 4)]
    pub hashlist: Account<'info, Hashlist>,

    #[account(mut,
        // constraint = payer.key().to_string() == "4aAifU9ck88koMhSK6fnUSQHMzpyuLzGa6q7nfvqA6vx".to_owned()
    )]
    pub payer: Signer<'info>,

    /* INITIALISE FUNGIBLE ACCOUNTS */
    #[account(mut)]
    pub fungible_mint: Signer<'info>,

    /// CHECK: checked in code
    #[account(mut)]
    pub fungible_escrow_token_account: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub fungible_metadata: UncheckedAccount<'info>,

    /* INITIALISE NON_FUNGIBLE ACCOUNTS. NB: no token account neede until mint */
    #[account(mut)]
    pub non_fungible_mint: Signer<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_metadata: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_master_edition: UncheckedAccount<'info>,

    /// CHECK: gets created, passed into libreplex_fair_launch via  CPI
    #[account(mut)]
    pub non_fungible_token_account: UncheckedAccount<'info>,

    /* INTERACT WITH INSCRIPTIONS PROGRAM  */
    #[account(mut)]
    pub inscription_summary: Account<'info, InscriptionSummary>,

    /// CHECK: passed in via CPI to libreplex_inscriptions program
    #[account(mut)]
    pub inscription: UncheckedAccount<'info>,

    /// CHECK: passed in via CPI to libreplex_inscriptions program
    #[account(mut)]
    pub inscription_v3: UncheckedAccount<'info>,

    /// CHECK: passed in via CPI to libreplex_inscriptions program
    #[account(mut)]
    pub inscription_data: UncheckedAccount<'info>,

    /* BOILERPLATE PROGRAM ACCOUNTS */
    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// CHECK: ID checked via constraint
    #[account(
        constraint = inscriptions_program.key() == libreplex_inscriptions::ID
    )]
    pub inscriptions_program: UncheckedAccount<'info>,

    #[account()]
    pub system_program: Program<'info, System>,

    /// CHECK: Id checked in constraint
    #[account(
        constraint = metadata_program.key() == mpl_token_metadata::ID
    )]
    #[account()]
    pub metadata_program: UncheckedAccount<'info>,

    /// CHECK: Id checked in constraint
    #[account(
        constraint = sysvar_instructions.key() == sysvar_instructions_program::ID
    )]
    #[account()]
    pub sysvar_instructions: UncheckedAccount<'info>,
}

pub fn deploy(ctx: Context<DeployLegacyCtx>) -> Result<()> {
    let deployment = &mut ctx.accounts.deployment;
    let hashlist = &mut ctx.accounts.hashlist;

    hashlist.deployment = deployment.key();
    deployment.deployed = true;

    let deployment = &mut ctx.accounts.deployment;
    let system_program = &ctx.accounts.system_program;
    let payer = &ctx.accounts.payer;
    let inscriptions_program = &ctx.accounts.inscriptions_program;
    let inscription_summary = &ctx.accounts.inscription_summary;
    let inscription = &ctx.accounts.inscription;
    let inscription_v3 = &ctx.accounts.inscription_v3;
    let inscription_data = &ctx.accounts.inscription_data;
    let fungible_mint = &ctx.accounts.fungible_mint;
    let fungible_metadata = &ctx.accounts.fungible_metadata;
    let non_fungible_mint = &ctx.accounts.non_fungible_mint;
    let non_fungible_metadata = &ctx.accounts.non_fungible_metadata;
    let non_fungible_master_edition = &ctx.accounts.non_fungible_master_edition;
    let non_fungible_token_account = &ctx.accounts.non_fungible_token_account;
    let sysvar_instructions = &ctx.accounts.sysvar_instructions;
    let token_program = &ctx.accounts.token_program;
    let metadata_program = &ctx.accounts.metadata_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let fungible_escrow_token_account = &ctx.accounts.fungible_escrow_token_account;

    deployment.deployed = true;
    deployment.fungible_mint = fungible_mint.key();

    let deployment_seeds: &[&[u8]] = &[
        "deployment".as_bytes(),
        deployment.ticker.as_ref(),
        &[ctx.bumps.deployment],
    ];

    create_mint_with_metadata_and_masteredition(
        MintAccounts {
            authority_pda: deployment.to_account_info(),
            payer: payer.to_account_info(),
            nft_owner: deployment.to_account_info(),
            nft_mint: fungible_mint.to_account_info(),
            nft_mint_authority: deployment.to_account_info(),
            nft_metadata: fungible_metadata.to_account_info(),
            nft_master_edition: None,
            token: Some(fungible_escrow_token_account.to_account_info()), // do not mint anything
            token_metadata_program: metadata_program.to_account_info(),
            spl_token_program: token_program.to_account_info(),
            spl_ata_program: associated_token_program.to_account_info(),
            system_program: system_program.to_account_info(),
            sysvar_instructions: sysvar_instructions.to_account_info(),
        },
        deployment_seeds,
        deployment.ticker.clone(),
        "".to_owned(),
        0,
        deployment.offchain_url.clone(),
        None,
        0,  // number of print editions. always 0.
        false,
        0,
        deployment.decimals,
        TokenStandard::Fungible,
    )?;

    create_mint_with_metadata_and_masteredition(
        MintAccounts {
            authority_pda: deployment.to_account_info(),
            payer: payer.to_account_info(),
            nft_owner: deployment.to_account_info(),
            nft_mint: non_fungible_mint.to_account_info(),
            nft_mint_authority: deployment.to_account_info(),
            nft_metadata: non_fungible_metadata.to_account_info(),
            nft_master_edition: Some(non_fungible_master_edition.to_account_info()),
            token: Some(non_fungible_token_account.to_account_info()), // do not mint anything
            token_metadata_program: metadata_program.to_account_info(),
            spl_token_program: token_program.to_account_info(),
            spl_ata_program: associated_token_program.to_account_info(),
            system_program: system_program.to_account_info(),
            sysvar_instructions: sysvar_instructions.to_account_info(),
        },
        deployment_seeds,
        deployment.ticker.clone(),
        "".to_owned(),
        0,
        deployment.offchain_url.clone(),
        None,
        0, // number of print editions. always 0.
        false,
        1, // this is the deployment mint. once mint + inscription made when
                        // a deployment is deployed.
        0,
        TokenStandard::NonFungible,
    )?;

    // Create the deploy inscription
    libreplex_inscriptions::cpi::create_inscription_v2(
        CpiContext::new(
            inscriptions_program.to_account_info(),
            CreateInscriptionV2 {
                /* the inscription root is set to metaplex
                    inscription object.
                */
                inscription_summary: inscription_summary.to_account_info(),

                root: non_fungible_mint.to_account_info(),
                /// since root in this case can sign (we are creating a brand new mint),
                /// it will sign
                signer: non_fungible_mint.to_account_info(),
                inscription: inscription.to_account_info(),
                inscription2: inscription_v3.to_account_info(),

                system_program: system_program.to_account_info(),
                payer: payer.to_account_info(),
                inscription_data: inscription_data.to_account_info(),
            },
        ),
        libreplex_inscriptions::instructions::CreateInscriptionInput {
            // the authority here doesn't matter because we will make this immutable at the
            // end of the transaction
            authority: Some(payer.key()),
            current_rank_page: 0,
            signer_type: SignerType::Root,
            validation_hash: None,
        },
    )?;

    let data_bytes = deployment.deployment_template.clone().into_bytes();

    libreplex_inscriptions::cpi::resize_inscription(
        CpiContext::new(
            inscriptions_program.to_account_info(),
            ResizeInscription {
                /* the inscription root is set to metaplex
                 inscription object.
                */
                authority: payer.to_account_info(),

                inscription: inscription.to_account_info(),
                system_program: system_program.to_account_info(),
                payer: payer.to_account_info(),
                inscription_data: inscription_data.to_account_info(),
                inscription2: Some(inscription_v3.to_account_info()),
            },
        ),
        libreplex_inscriptions::instructions::ResizeInscriptionInput {
            change: data_bytes.len() as i32 - 8,
            expected_start_size: 8,
            target_size: data_bytes.len() as u32,
        },
    )?;

    libreplex_inscriptions::cpi::write_to_inscription(
        CpiContext::new(
            inscriptions_program.to_account_info(),
            WriteToInscription {
                authority: payer.to_account_info(),
                payer: payer.to_account_info(),
                inscription: inscription.to_account_info(),
                inscription2: Some(inscription_v3.to_account_info()),
                system_program: system_program.to_account_info(),
                inscription_data: inscription_data.to_account_info(),
            },
        ),
        WriteToInscriptionInput {
            data: data_bytes,
            start_pos: 0,
            media_type: Some("text/plain".to_owned()),
            encoding_type: Some("ascii".to_owned()),
        },
    )?;

    // set update auth to 1111111111111111
    libreplex_inscriptions::cpi::make_inscription_immutable(CpiContext::new(
        inscriptions_program.to_account_info(),
        MakeInscriptionImmutable {
            payer: payer.to_account_info(),
            authority: payer.to_account_info(),
            inscription_summary: inscription_summary.to_account_info(),
            inscription: inscription.to_account_info(),
            inscription2: Some(inscription_v3.to_account_info()),
            system_program: system_program.to_account_info(),
        },
    ))?;


    Ok(())
}
