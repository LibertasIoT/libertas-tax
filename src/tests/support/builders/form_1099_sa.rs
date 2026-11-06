use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn sa_source(form: Sa1099) -> SaSource {
    let Sa1099 {
        trustee_payer,
        recipient,
        origin,
        gross,
        box2_earnings,
        box4_value,
        box3_code,
        account,
    } = form;
    let form = match origin {
        HealthDocumentOrigin::Form1099Sa => crate::MedicalSaPaper::Form1099Sa(Form1099SaPaper {
corrected: false,
calendar_year: Default::default(),
            trustee_payer,
            recipient,
            gross_distribution: gross,
            earnings_on_excess_contributions: box2_earnings,
            fmv_on_date_of_death: box4_value,
            distribution_code: box3_code.map(|code| {
                [
                    Form1099SaDistributionCode::NormalDistributions,
                    Form1099SaDistributionCode::ExcessContributions,
                    Form1099SaDistributionCode::Disability,
                    Form1099SaDistributionCode::DeathDistributionOtherThanCode6,
                    Form1099SaDistributionCode::ProhibitedTransaction,
                    Form1099SaDistributionCode::DeathDistributionAfterYearOfDeathToNonspouseBeneficiary,
                ]
                .into_iter()
                .find(|candidate| candidate.printed_code() == code)
                .expect("the fixture must name a printed Form 1099-SA box 3 code")
            }),
            account,
            account_number: String::new(),
        }),
        HealthDocumentOrigin::TrusteeStatement => {
            crate::MedicalSaPaper::TrusteeStatement(crate::HsaTrusteeStatementPaper {
                trustee_payer,
                recipient,
                account_number: String::new(),
                amount: gross,
                account,
            })
        }
    };
    SaSource { form, origin, medicare_prior: None, medicare_from_spouse: None, payment: None }
}
