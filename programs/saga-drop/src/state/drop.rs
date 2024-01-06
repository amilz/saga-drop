use anchor_lang::prelude::*;
use std::str::FromStr;
use crate::constants::{SAGA_COLLECTION};

#[account]
pub struct Drop {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub wl_collection: Pubkey,
    // Number of Tokens per Holder
    pub drop_size: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub reclaimed: bool,
    pub bump: u8
}

impl Drop {
    pub fn init(&mut self, config: DropInitConfig) {
        self.authority = config.authority;
        self.token_mint = config.token_mint;
        self.token_account = config.token_account;
        // For now, only allowed for Saga Mints
        self.wl_collection = Pubkey::from_str(SAGA_COLLECTION).unwrap();
        self.drop_size = config.drop_size;
        self.start_epoch = config.start_epoch;
        self.end_epoch = config.end_epoch;
        self.reclaimed = false;
        self.bump = config.bump;
    }
    pub fn calculate_account_space() -> usize {
        8  + // discriminator
        32 + // authority
        32 + // token_account
        32 + // wl_collection
        32 + // token_mint
        8  + // drop_size
        8  + // start_epoch
        8  + // end_epoch
        1  + // reclaimed
        1    // bump
    }
    pub fn reclaim(&mut self) {
        self.reclaimed = true;
    }
}

pub struct DropInitConfig {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub drop_size: u64,
    pub start_epoch: u64,
    pub end_epoch: u64,
    pub bump: u8,
}
