use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Transfer, Mint, Token, TokenAccount},
};
use std::str::FromStr;
use crate::state::drop::*;
use crate::errors::DropError;
use crate::constants::{DROP_SEED, SAGA_COLLECTION};


#[derive(Accounts)]
pub struct ReclaimTokens<'info> {
    /// Authority/Signer
    #[account(
        mut,
        address = drop_pda.authority
    )]
    pub authority: Signer<'info>,

    /// Drop PDA
    #[account(
        mut, 
        seeds = [
            DROP_SEED.as_ref(),
            Pubkey::from_str(SAGA_COLLECTION).unwrap().as_ref(),
            token_mint.key().as_ref(),
            authority.key().as_ref()
        ], 
        bump
    )]
    pub drop_pda: Account<'info, Drop>,

    #[account(
        mut, 
        close = authority,
        associated_token::mint = token_mint,
        associated_token::authority = drop_pda,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// Unchecked: verified in drop_pda seeds Mint account of the token to be dropped
    pub token_mint: Account<'info, Mint>,

    /// Token account for the claim to go to (Create if it doesn't exist)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = authority,
    )]
    pub destination: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn reclaim_tokens(ctx: Context<ReclaimTokens>) -> Result<()> {
    // Declarations
    let drop_pda = &mut ctx.accounts.drop_pda;
    let token_program = &ctx.accounts.token_program;
    let token_mint = &ctx.accounts.token_mint;
    let escrow_token_account = &mut ctx.accounts.escrow_token_account;
    let destination = &mut ctx.accounts.destination;

    // 1 - Verifications
    let current_epoch = Clock::get().unwrap().epoch;
    require!(current_epoch > drop_pda.end_epoch, DropError::DropStillActive);

    // 2 - Transfers
    let token_mint_key = token_mint.key();

    let bump = &[drop_pda.bump];
    let seeds: &[&[u8]] = &[
        DROP_SEED.as_ref(),
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
    let drop_amount = escrow_token_account.amount;

    transfer(cpi_ctx, drop_amount)?;
    
    // 3. Update state of Drop PDA
    drop_pda.reclaim();

    Ok(())
}


