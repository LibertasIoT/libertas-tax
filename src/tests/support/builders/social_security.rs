use crate::tests::*;

pub(in crate::tests) fn social_security_net(amount: i64, withholding: i64) -> FederalIncome {
    FederalIncome::SocialSecurity {
        data: SocialBenefitStatement::Ssa1099(ssa_document(ssa_1099_net(amount, withholding))),
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

// Synthetic net-only fixtures use the simplest coherent printed gross/repayment split.
pub(in crate::tests) fn ssa_1099_net(amount: i64, withholding: i64) -> Ssa1099 {
    Ssa1099 {
        benefits_paid: amount.max(0),
        benefits_repaid: amount.saturating_neg().max(0),
        benefits: amount,
        withholding,
        ..Default::default()
    }
}
