use super::paper_documents::remember;
use crate::tests::*;
use alloc::string::String;

pub(in crate::tests) fn q1099_paper(form: Q1099) -> Form1099QPaper {
    let Q1099 { payer_trustee, recipient, gross, account, transfer, recipient_not_beneficiary } =
        form;
    let (distribution_from, earnings, basis, fair_market_value) = match account {
        EducationAccount::Qtp { earnings, basis } => {
            (Form1099QDistributionFrom::StateQtp, earnings, basis, None)
        }
        EducationAccount::Coverdell { amounts } => match amounts {
            CoverdellAmounts::Reported { earnings, basis } => {
                (Form1099QDistributionFrom::CoverdellEsa, earnings, basis, None)
            }
            CoverdellAmounts::ValueOnly { value } => (
                Form1099QDistributionFrom::CoverdellEsa,
                0,
                0,
                Some(Form1099QFairMarketValue { value }),
            ),
        },
    };
    let (trustee_to_trustee, qtp_to_roth_ira, distribution_code) = match transfer {
        Q1099Transfer::TrusteeTransfer => (true, false, None),
        Q1099Transfer::RothIra => (false, true, None),
        Q1099Transfer::ExcessThisYear => {
            (false, false, Some(Form1099QDistributionCode::ExcessThisYear))
        }
        Q1099Transfer::ExcessLastYear => {
            (false, false, Some(Form1099QDistributionCode::ExcessLastYear))
        }
        Q1099Transfer::ProhibitedTransaction => {
            (false, false, Some(Form1099QDistributionCode::ProhibitedTransaction))
        }
        Q1099Transfer::Withdrawal { reason } => (
            false,
            false,
            Some(match reason {
                DistributionReason::Withdrawal => Form1099QDistributionCode::Distribution,
                DistributionReason::Disability => Form1099QDistributionCode::Disability,
                DistributionReason::Death => Form1099QDistributionCode::Death,
            }),
        ),
    };
    Form1099QPaper {
        calendar_year: 2026,
        corrected: false,
        payer_trustee,
        payer_telephone: String::new(),
        recipient,
        account_number: String::new(),
        gross_distribution: gross,
        earnings,
        basis,
        trustee_to_trustee,
        qtp_to_roth_ira,
        distribution_from,
        recipient_not_beneficiary,
        fair_market_value,
        distribution_code,
    }
}

pub(in crate::tests) fn education_distribution_source(
    data: FederalEducationDistribution,
) -> EducationDistributionSource {
    let FederalEducationDistribution {
        gross,
        account,
        transfer,
        recipient_not_beneficiary,
        beneficiary,
        coverdell_basis_start,
        coverdell_contributions,
        redeposited,
        taxable_transfer_part,
        roth_rollover,
        late_rollovers,
    } = data;
    EducationDistributionSource {
        form: q1099_paper(Q1099 {
            payer_trustee: Default::default(),
            recipient: Default::default(),
            gross,
            account,
            transfer,
            recipient_not_beneficiary,
        }),
        beneficiary,
        coverdell_basis_start,
        coverdell_contributions,
        redeposited,
        taxable_transfer_part,
        roth_rollover,
        late_rollovers,
    }
}

pub(in crate::tests) fn education_distribution_document(
    source: EducationDistributionSource,
) -> Form1099QPaper {
    use crate::DocumentAnswer as A;
    let form = source.form;
    remember(
        IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Q1099(form.clone()) },
        vec![
            A::EducationBeneficiary { value: source.beneficiary.as_ref().map(Into::into) },
            A::CoverdellRecords {
                opening: source.coverdell_basis_start,
                contributions: source.coverdell_contributions,
                late_rollovers: source.late_rollovers,
            },
            A::EducationRedeposited { value: source.redeposited },
            A::EducationTaxableTransfer { value: source.taxable_transfer_part },
            A::EducationRothRecords { value: source.roth_rollover },
        ],
    );
    form
}
