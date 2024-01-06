/// The public key of the native mint account on the Solana blockchain.
pub const SAGA_COLLECTION: &str = "Saga5xJVLEAvm23n5NB3bCTsyFvEWWc3Rdgjz3zHUXt";
// Example: https://explorer.solana.com/address/Fy79Aii43rLGS4k5U2oW7fn6CgN9tsjBr2FnssY5ZqRR/metadata
// Example2: https://explorer.solana.com/address/46pcSL5gmjBrPqGKFaLbbCmR6iVuLJbnQy13hAe7s6CC/metadata
// Devnet: "Saga5xJVLEAvm23n5NB3bCTsyFvEWWc3Rdgjz3zHUXt" 
// Mainnet: "46pcSL5gmjBrPqGKFaLbbCmR6iVuLJbnQy13hAe7s6CC"

/// Number of Saga Genesis Mints
pub const NUM_DROPS: u64 = 20_000;


pub mod drop_config {
    /// Max Duration (in epochs)
    pub const MAX_DURATION: u64 = 10;
    
    /// Max Time in Future Drop can Start
    pub const BUFFER_LIMIT: u64 = 10;
}


pub mod fee_config {
    /// The current fee collected per new drop from the authority.
    pub const INITIATE_FEE: u64 = 2_000_000_000; // 2 SOL in lamports

    /// The current fee collected per claim from the claimer.
    pub const CLAIM_FEE: u64 = 13_100_000; // 0.0131 SOL in lamports
}