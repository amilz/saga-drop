use anchor_lang::prelude::*;
use crate::constants::PROGRAM_STATE_SEED;
use crate::state::program_state::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer, 
        space = 8 + 1 + 1 + 8 + 1 + 1,
        seeds = [PROGRAM_STATE_SEED.as_ref()], 
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    program_state.init(ctx.bumps.program_state);
    Ok(())
}


