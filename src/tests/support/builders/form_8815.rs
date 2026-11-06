use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn savings_bonds_source(form: SavingsBonds) -> SavingsBondsSource {
    let SavingsBonds {
        source_issuer,
        source_recipient,
        proceeds,
        redemption_interest,
        earliest_issue_date,
        interest_statement,
        students,
    } = form;
    assert_eq!(source_issuer, PrintedParty::default(), "Form 8815 has no issuer block");
    SavingsBondsSource {
        form: Form8815Paper {
            taxpayer: source_recipient,
            line5: proceeds,
            line6: redemption_interest,
            ..Default::default()
        },
        redemption_interest: Some(redemption_interest),
        earliest_issue_date: Some(earliest_issue_date),
        answers: Some(SavingsBondAnswers {
            interest_statement,
            students: students.iter().map(Into::into).collect(),
        }),
    }
}

pub(in crate::tests) fn savings_bonds_document(source: SavingsBondsSource) -> SavingsBondsEntry {
    let form = source.entry();
    remember(
        IncomeSource::InterestDividend {
            value: InterestDividendDocument::SavingsBonds(form.clone()),
        },
        vec![DocumentAnswer::SavingsBondsRecords {
            value: source.answers.as_ref().map(Into::into),
        }],
    );
    form
}

pub(in crate::tests) fn printed_bond_redemption() -> crate::Form8818Paper {
    crate::Form8818Paper {
        name: "Synthetic Filer".into(),
        date_cashed: Some(20260820),
        line1: vec![crate::Form8818BondRow {
            serial_number: "EXAMPLEEE0001".into(),
            issue_date: Some(20100601),
            face_value: 600_000,
        }],
        line2: 600_000,
        line3: 300_000,
        line4: vec![crate::Form8818BondRow {
            serial_number: "EXAMPLEI0001".into(),
            issue_date: Some(20110701),
            face_value: 600_000,
        }],
        line5: 600_000,
        line6: 1_200_000,
        line7: 900_000,
        line8: 300_000,
    }
}
