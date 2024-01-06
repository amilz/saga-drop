use anchor_lang::prelude::*;

#[account]
pub struct ProgramState {
    pub active: bool,
    pub current_version: u64,
    pub creator_fee: bool,
    pub claimer_fee: bool,
    pub bump: u8,
}

impl ProgramState {
    pub fn init(&mut self, bump: u8) {
        self.active = true;
        self.current_version = 1;
        self.creator_fee = true;
        self.claimer_fee = false;
        self.bump = bump;
    }
    pub fn increment_version(&mut self) {
        self.current_version += 1;
    }
}