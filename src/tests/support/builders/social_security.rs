use crate::tests::*;

pub(in crate::tests) fn social_security_net(amount: i64, withholding: i64) -> FederalIncome {
    FederalIncome::SocialSecurity {
        data: FederalSocial { benefits: amount, withholding, special_treatment: false },
    }
}

pub(in crate::tests) fn social_prior_year() -> SocialBenefitYear {
    SocialBenefitYear {
        year: 2025,
        status: FederalStatus::Single,
        apart: true,
        received: 200000,
        benefits: 0,
        earlier_lumps: 0,
        agi: 2300000,
        taxable: 0,
        refigured: 0,
        exempt: 0,
        addbacks: 0,
    }
}
