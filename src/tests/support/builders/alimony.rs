use crate::tests::*;

pub(in crate::tests) fn paid_alimony(payment: AlimonyPayment) -> AlimonyPaid {
    AlimonyPaid { owner: Owner::Taxpayer, payment }
}

pub(in crate::tests) fn alimony_record(agreement: u32) -> AlimonyPayment {
    AlimonyPayment {
        decree: true,
        recapture: AlimonyRecapture::None,
        agreement,
        new_rules: AlimonyNewRules::No,
        paid: 1800000,
        alimony_due: 1200000,
        child_support_due: 800000,
        eligible: true,
    }
}

pub(in crate::tests) fn recapture_alimony() -> AlimonyPayment {
    AlimonyPayment {
        paid: 2800000,
        alimony_due: 2800000,
        child_support_due: 0,
        recapture: AlimonyRecapture::ThirdYear {
            first: 5000000,
            second: 3900000,
            first_excluded: 0,
            second_excluded: 0,
            current_excluded: 0,
            death_or_remarriage: false,
        },
        ..alimony_record(20181231)
    }
}
