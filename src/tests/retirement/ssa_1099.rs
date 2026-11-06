use super::*;

// Every printed box of a stored SSA-1099 lands once in the internal Social
// Security model; the party blocks and box 8 claim number reach no figure.
#[test]
fn ssa_1099_converter_carries_every_box_into_the_internal_model() {
    let paid = vec![SocialBenefitDescription {
        description: SocialBenefitDescriptionKind::MedicarePartB,
        amount: 240_000,
    }];
    let repaid = vec![SocialBenefitDescription {
        description: SocialBenefitDescriptionKind::PriorYear { year: 2025 },
        amount: 200_000,
    }];
    let form = Ssa1099 {
        benefits_paid: 2_000_000,
        paid_description: paid.clone(),
        benefits_repaid: 200_000,
        repaid_description: repaid.clone(),
        benefits: 1_800_000,
        withholding: 50_000,
        claim_number: "987-65-4320A".into(),
        ..Default::default()
    };
    assert_eq!(
        FederalSocial::from(&SocialBenefitStatement::Ssa1099(ssa_document(form))),
        FederalSocial {
            form_boxes: SocialPrintedBoxes {
                benefits_paid: 2_000_000,
                paid_description: paid,
                benefits_repaid: 200_000,
                repaid_description: repaid,
            },
            benefits: 1_800_000,
            withholding: 50_000,
        }
    );
}

#[test]
fn printed_ssa_1099_blank_amounts_and_printed_zero_survive_save_and_reopen() {
    use crate::tests::support::builders::assert_retirement_paper_reopens;
    for amount in [None, Some(0)] {
        let paper = crate::Ssa1099Paper {
            tax_year: 2026,
            benefits_paid: amount,
            benefits_repaid_to_ssa: amount,
            voluntary_federal_income_tax_withheld: amount,
            net_benefits: 0,
            ..Default::default()
        };
        assert_eq!(crate::FederalSocial::from(&paper).withholding, 0);
        assert_retirement_paper_reopens(crate::RetirementDocument::Ssa1099(paper));
    }
}
