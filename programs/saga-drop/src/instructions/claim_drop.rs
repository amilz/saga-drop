use anchor_lang::prelude::*;
use anchor_spl::metadata::MetadataAccount;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use crate::constants::{CLAIM_SEED, DROP_SEED, PROGRAM_STATE_SEED, SAGA_COLLECTION};
use crate::errors::DropError;
use crate::state::claim::*;
use crate::state::drop::*;
use crate::state::program_state::*;
use std::str::FromStr;

/// # `claim_drop` Function
///
/// ## Process
/// 1. **Verification**
///    - Checks if the claim receipt has already been claimed to avoid duplicates.
///
/// 2. **Transfers**
///    - Calculates the transfer amount based on `drop_pda.drop_size` and `token_mint.decimals`.
///    - Transfers tokens from the escrow account to the claimer's destination.
///
/// 3. **Create Claim Receipt PDA**
///    - Initializes the claim receipt with the corresponding bump seed.
///
/// ## Constraints and Validations
/// - Ensures the claimer owns the NFT token account with a balance.
/// - Verifies the token account's association with the correct mint.
/// - Confirms the metadata account derivation from the same mint account.
/// - Checks the NFT metadata collection pubkey is correct and verified.
///


#[derive(Accounts)]
pub struct ClaimDrop<'info> {
    /// Authority/Signer
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// Program State (check that Program is active)
    #[account(
        seeds = [PROGRAM_STATE_SEED.as_ref()], 
        bump = program_state.bump,
        constraint = program_state.active == true @ DropError::ProgramDisabled
    )]
    pub program_state: Account<'info, ProgramState>,

    /// Drop PDA
    #[account(
        seeds = [
            DROP_SEED.as_ref(),
            Pubkey::from_str(SAGA_COLLECTION).unwrap().as_ref(),
            token_mint.key().as_ref(),
            // TODO - CHECK MY LOGIC HERE. FEELS CIRCULAR
            // WHAT RISKS IF SOMEONE PASSES WRONG PDA
            drop_pda.authority.as_ref()
        ], 
        bump
    )]
    pub drop_pda: Account<'info, Drop>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = drop_pda,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Unchecked: verified in drop_pda seeds Mint account of the token to be dropped
    pub token_mint: Account<'info, Mint>,

    /// Token account for the claim to go to (Create if it doesn't exist)
    #[account(
        init_if_needed,
        payer = claimer,
        associated_token::mint = token_mint,
        associated_token::authority = claimer,
    )]
    pub destination: Account<'info, TokenAccount>,

    /// Claim Receipt - only init w/ canonical bump. if try to init again fail
    #[account(
        init,
        payer = claimer, 
        space = Claim::calculate_account_space(), 
        seeds = [
            CLAIM_SEED.as_ref(),
            drop_pda.key().as_ref(),
            nft_mint.key().as_ref(),
        ], 
        bump
    )]
    pub claim_receipt: Account<'info, Claim>,

    // NFT's mint address--validated it qualifies through metadata constraints
    /// Unchecked - we verify mint in subsequent accounts
    pub nft_mint: Account<'info, Mint>,

    //
    #[account(
        constraint = nft_token_account.owner == claimer.key() @ DropError::DoesNotOwnGenesisToken,
        constraint = nft_token_account.amount == 1 @ DropError::DoesNotOwnGenesisToken,
        token::mint = nft_mint,
        // associated_token::authority = claimer @ DropError::DoesNotOwnGenesisToken,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    #[account(
        address = mpl_token_metadata::accounts::Metadata::find_pda(&nft_mint.key()).0,
        //https://docs.rs/mpl-token-metadata/3.2.3/mpl_token_metadata/accounts/struct.Metadata.html#method.find_pda
        constraint = nft_metadata.collection.as_ref().unwrap().verified == true @ DropError::NotValidGenesisToken,
        constraint = nft_metadata.collection.as_ref().unwrap().key == drop_pda.wl_collection @ DropError::NotValidGenesisToken    
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn claim_drop(ctx: Context<ClaimDrop>) -> Result<()> {
    // Declarations
    let drop_pda = &mut ctx.accounts.drop_pda;
    let token_program = &ctx.accounts.token_program;
    let token_mint = &ctx.accounts.token_mint;
    let escrow_token_account = &mut ctx.accounts.escrow_token_account;
    let destination = &mut ctx.accounts.destination;
    let claim_receipt = &mut ctx.accounts.claim_receipt;

    // 1 - Verifications
    // This should never return true but just to be safe
    require!(!claim_receipt.claimed, DropError::DropTooSmall);

    // TODO EPOCH VERIFICATIONS


    // 2 - Transfers
    let token_mint_key = token_mint.key();
    let saga_buffer = Pubkey::from_str(SAGA_COLLECTION).unwrap();
        
    let bump = &[drop_pda.bump];
    let seeds: &[&[u8]] = &[
        DROP_SEED.as_ref(),
        saga_buffer.as_ref(),
        token_mint_key.as_ref(),
        drop_pda.authority.as_ref(),
        bump,
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: escrow_token_account.to_account_info(),
        to: destination.to_account_info(),
        authority: drop_pda.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    // https://docs.rs/anchor-lang/latest/anchor_lang/context/struct.CpiContext.html#method.with_signer
    let drop_amount = drop_pda
        .drop_size
        .checked_mul(10u64.pow(token_mint.decimals as u32));
    if drop_amount.is_none() {
        return err!(DropError::DropTooLarge);
    }

    transfer(cpi_ctx, drop_amount.unwrap())?;

    // 3 - Create Claim Receipt PDA
    claim_receipt.init(ctx.bumps.claim_receipt);

    Ok(())
}
