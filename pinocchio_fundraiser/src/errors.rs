use pinocchio::error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FundraiserError {
    InvalidInstructionData = 0,
    PdaMismatch = 1,
    InvalidOwner = 2,
    MissingRequiredSignature = 3,
    MintMismatch = 4,
    ContributionExceedsMax = 5,
    FundraiserExpired = 6,
    ContributePdaMismatch = 7,
    InvalidContributionAmount = 8,
    VaultOwnerMismatch = 9,
    VaultAmountMismatch = 10
}

impl From<FundraiserError> for ProgramError {
    fn from(e: FundraiserError) -> Self {
        Self::Custom(e as u32)
    }
}
