use anchor_lang::prelude::*;
use instructions::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod constants;

declare_id!("DropbPyWB9NzsBTWYxak7QBVHbgs1KWthRk2i6KbKnu");

#[program]
mod saga_drop {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    pub fn create_drop(ctx: Context<CreateDrop>, params: CreateDropParams) -> Result<()>{
        instructions::create_drop::create_drop(ctx, params)
    }

    pub fn claim(ctx: Context<ClaimDrop>) -> Result<()> {
        instructions::claim_drop::claim_drop(ctx)
    }

    pub fn reclaim_tokens(ctx: Context<ReclaimTokens>) -> Result<()> {
        instructions::reclaim_tokens::reclaim_tokens(ctx)
    }

}