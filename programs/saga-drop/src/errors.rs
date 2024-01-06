use anchor_lang::error_code;

#[error_code]
pub enum DropError {
    #[msg("Not a valid Genesis Token")]
    NotValidGenesisToken,
    #[msg("Signer does not own Genesis Token")]
    DoesNotOwnGenesisToken,
    #[msg("Program is not active")]
    ProgramDisabled,
    #[msg("Drop Too Large (Overflow)")]
    DropTooLarge,
    #[msg("Not a large enough drop for all holders")]
    DropTooSmall,
    #[msg("Cannot start before current epoch")]
    InvalidStartTooEarly,
    #[msg("Must start within 10 epochs")]
    InvalidStartTooLate,
    #[msg("Drop period cannot exceed 10 epochs")]
    InvalidDurationLong,
    #[msg("Drop period must be at least 1 epoch")]
    InvalidDurationShort,
    #[msg("Cannot reclaim until after final epoch")]
    DropStillActive,
}