use crate::accepted_income::FederalIncome;
use crate::{DistributionCode, FederalRetirement, R1099Source, RetirementSource};

impl FederalIncome {
    /// The pension or IRA distribution as the internal record the calculations read.
    pub(crate) fn retirement(&self) -> Option<FederalRetirement> {
        match self {
            Self::Retirement { data } => Some(data.into()),
            _ => None,
        }
    }
    /// The Roth IRA withdrawal as the internal record the calculations read.
    pub(crate) fn roth(&self) -> Option<crate::RothWithdrawal> {
        match self {
            Self::Roth { data } => Some(data.into()),
            _ => None,
        }
    }
    /// The Form 1099-R and its answers, whichever income kind they file under.
    pub(crate) fn r1099_source(&self) -> Option<&R1099Source> {
        match self {
            Self::Retirement { data: RetirementSource::R1099(data) }
            | Self::Roth { data }
            | Self::TrumpDeath { data }
            | Self::TrumpAbleRollover { data } => Some(data),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn r1099_source_mut(&mut self) -> Option<&mut R1099Source> {
        match self {
            Self::Retirement { data: RetirementSource::R1099(data) }
            | Self::Roth { data }
            | Self::TrumpDeath { data }
            | Self::TrumpAbleRollover { data } => Some(data),
            _ => None,
        }
    }
}

impl R1099Source {
    /// The income kind this form's treatment answer files it under.
    pub(crate) fn income(self) -> FederalIncome {
        match self.code {
            Some(DistributionCode::RothIra { .. }) => FederalIncome::Roth { data: self },
            Some(DistributionCode::TrumpDeath { .. }) => FederalIncome::TrumpDeath { data: self },
            Some(DistributionCode::TrumpToAble { .. }) => {
                FederalIncome::TrumpAbleRollover { data: self }
            }
            _ => FederalIncome::Retirement { data: RetirementSource::R1099(self) },
        }
    }
}
