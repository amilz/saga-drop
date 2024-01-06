use anchor_lang::prelude::*;

#[account]
pub struct Claim {
    pub claimed: bool,
    pub bump: u8,
}

impl Claim {
    pub fn init(&mut self, bump: u8) {
        self.claimed = true;
        self.bump = bump;
    }
    pub fn calculate_account_space() -> usize {
        8  + // discriminator
        1  + // claimed
        1    // bump
    }
}