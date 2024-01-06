use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer as sys_transfer, Transfer as SysTransfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use crate::constants::{
    drop_config, fee_config, DROP_SEED, FEE_SEED, NUM_DROPS, PROGRAM_STATE_SEED, SAGA_COLLECTION,
};
use crate::errors::DropError;
use crate::state::drop::*;
use crate::state::program_state::*;
use std::str::FromStr;

#[derive(Accounts)]
pub struct CreateDrop<'info> {
    /// Authority/Signer
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Program State (check that Program is active)
    #[account(
        seeds = [PROGRAM_STATE_SEED.as_ref()], 
        bump = program_state.bump,
        constraint = program_state.active == true @ DropError::ProgramDisabled
    )]
    pub program_state: Account<'info, ProgramState>,

    /// Drop PDA
    #[account(
        init,
        payer = authority, 
        space = Drop::calculate_account_space(), 
        seeds = [
            DROP_SEED.as_ref(),
            Pubkey::from_str(SAGA_COLLECTION).unwrap().as_ref(),
            mint.key().as_ref(),
            authority.key().as_ref()
        ], 
        bump
    )]
    pub drop_pda: Account<'info, Drop>,

    /// Mint account of the token to be dropped
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// Token account for the drop escrow
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = drop_pda,
    )]
    pub drop_escrow_account: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    // For some reason this wasn't working if it was above ata program...very weird.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [FEE_SEED.as_ref()],
        bump
    )]
    pub fee_vault: SystemAccount<'info>,
}

pub fn create_drop(ctx: Context<CreateDrop>, params: CreateDropParams) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let program_state = &ctx.accounts.program_state;
    let drop_pda = &mut ctx.accounts.drop_pda;
    let drop_escrow_account = &mut ctx.accounts.drop_escrow_account;
    let token_account_key = drop_escrow_account.key();
    let drop_pda_bump = ctx.bumps.drop_pda;
    let mint = &ctx.accounts.mint;
    let token_mint_key = ctx.accounts.mint.key();
    let authority_key = authority.key();
    let source_token_account = &mut ctx.accounts.source_token_account;
    let token_program = &ctx.accounts.token_program;
    let current_epoch = Clock::get().unwrap().epoch;

    // Perform any necessary checks before initialization
    require!(program_state.active, DropError::ProgramDisabled);
    require!(params.drop_size > 0, DropError::DropTooSmall);
    require!(
        params.start_epoch >= current_epoch,
        DropError::InvalidStartTooEarly
    );
    require!(
        params.start_epoch <= (current_epoch + drop_config::BUFFER_LIMIT),
        DropError::InvalidStartTooLate
    );
    require!(
        params.end_epoch > params.start_epoch,
        DropError::InvalidDurationShort
    );
    require!(
        (params.end_epoch - params.start_epoch) <= drop_config::MAX_DURATION,
        DropError::InvalidDurationLong
    );

    // Get decimal value for drop amount
    let amount_per_drop = params
        .drop_size
        .checked_mul(10u64.pow(mint.decimals as u32));
    if amount_per_drop.is_none() {
        return err!(DropError::DropTooLarge);
    }
    let amount = NUM_DROPS.checked_mul(amount_per_drop.unwrap());
    if amount.is_none() {
        return err!(DropError::DropTooLarge);
    }

    // Transfer tokens to escrow
    let cpi_accounts = Transfer {
        from: source_token_account.to_account_info(),
        to: drop_escrow_account.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    transfer(cpi_ctx, amount.unwrap())?;

    // Initiate Drop PDA
    let drop_init_config = DropInitConfig {
        authority: authority_key,
        token_mint: token_mint_key,
        token_account: token_account_key,
        drop_size: params.drop_size,
        start_epoch: params.start_epoch,
        end_epoch: params.end_epoch,
        bump: drop_pda_bump,
    };
    drop_pda.init(drop_init_config);

    // Send Payment Fee if Applicable
    if program_state.creator_fee {
        sys_transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                SysTransfer {
                    from: authority.to_account_info(),
                    to: ctx.accounts.fee_vault.to_account_info(),
                },
            ),
            fee_config::INITIATE_FEE,
        )?;
    }

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct CreateDropParams {
    pub drop_size: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
}
