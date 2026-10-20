// Deduction-specific Save requiredness uses shared person attribution.
// The native financial source owners retain these final actual operands.
// Final native operands retained for Save requiredness. None is an unavailable
// native component, not a known zero. This packet never reruns an expense cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpenseRoleDollars {
    pub(crate) itemizing: bool,
    pub(crate) itemized_dollars: i64,
    pub(crate) property_dollars: i64,
    pub(crate) interest_dollars: Option<i64>,
    pub(crate) connected_dollars: Option<i64>,
    pub(crate) salt_dollars: i64,
    pub(crate) income_tax_category_dollars: i64,
    pub(crate) foreign_dollars: i64,
    pub(crate) current_paid_dollars: Option<i64>,
    pub(crate) retained_carry_dollars: Option<i64>,
}
impl ExpenseRoleDollars {
    fn needs_split(&self, draft: &crate::FederalDraft) -> bool {
        draft
            .setup
            .as_ref()
            .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Joint)
            && self.itemizing
            && self.itemized_dollars > 0
            // A joint NIIT calculation consumes the joint expense only.
            // Canonical people are required when a per-person split is used.
            && draft.injured_spouse.is_some()
    }
    // A current question and an independently retained attribute have separate
    // source identities even when their numeric amounts happen to be equal.
    pub(crate) fn missing(
        &self,
        draft: &crate::FederalDraft,
        section: crate::TaxSection,
    ) -> Option<MissingExpenseOwner> {
        if !self.needs_split(draft) {
            return None;
        }
        let deductions = draft.deductions.as_ref()?;
        match section {
            crate::TaxSection::OtherItemized => {
                if self.interest_dollars.is_some_and(|v| v > 0) {
                    for (index, loan) in
                        deductions.investment_interest.iter().flat_map(|v| &v.loans).enumerate()
                    {
                        if loan.source != 0 {
                            continue;
                        }
                        if self.current_paid_dollars.is_some_and(|v| v > 0)
                            && loan.paid > 0
                            && loan.paid_owners.is_none()
                        {
                            return Some(MissingExpenseOwner::CurrentInterest(index));
                        }
                        if self.retained_carry_dollars.is_some_and(|v| v > 0)
                            && loan.carry > 0
                            && loan.carry_owners.is_none()
                        {
                            return Some(MissingExpenseOwner::RetainedInterest(index));
                        }
                    }
                }
                if self.property_dollars > 0
                    && deductions.investment_property_tax_owners().is_none()
                {
                    return Some(MissingExpenseOwner::PropertyTax);
                }
            }
            crate::TaxSection::Taxes
                if (self.connected_dollars.is_some_and(|v| v > 0)
                    || (draft.injured_spouse.is_some()
                        && self.salt_dollars > 0
                        && self.income_tax_category_dollars > 0))
                    && deductions.income_tax_owners().is_none() =>
            {
                return Some(MissingExpenseOwner::ConnectedTax);
            }
            crate::TaxSection::MoreCredits if self.foreign_dollars > 0 => {
                if let crate::ForeignTaxClaim::Deduction { taxes } =
                    &draft.credits.as_ref()?.foreign_tax
                    && let Some(index) = taxes.iter().position(|tax| {
                        tax.tax > 0
                            && tax.owners.is_none()
                            && matches!(
                                tax.income,
                                crate::ForeignTaxIncome::Investment
                                    | crate::ForeignTaxIncome::Passive
                            )
                    })
                {
                    return Some(MissingExpenseOwner::ForeignTax(index));
                }
            }
            _ => {}
        }
        None
    }
}

pub(crate) enum MissingExpenseOwner {
    CurrentInterest(usize),
    RetainedInterest(usize),
    PropertyTax,
    ConnectedTax,
    ForeignTax(usize),
}
impl MissingExpenseOwner {
    pub(crate) fn issue(
        self,
        foreign_credit_index: Option<usize>,
    ) -> crate::validation::ValidationIssue {
        use crate::validation::ValidationIssue;
        let issue = ValidationIssue::new("EXPENSE_PERSON_OWNER_REQUIRED");
        match self {
            Self::CurrentInterest(index) => issue
                .at_field("paid_owners")
                .in_list("loans", index)
                .in_field("investment_interest"),
            Self::RetainedInterest(index) => issue
                .at_field("carry_owners")
                .in_list("loans", index)
                .in_field("investment_interest"),
            Self::PropertyTax => issue.at_field("investment_property_tax_owners"),
            Self::ConnectedTax => issue.at_field("income_tax_owners"),
            Self::ForeignTax(index) => issue
                .at_field("owners")
                .in_list("taxes", index)
                .in_list("credits", foreign_credit_index.expect("submitted foreign tax credit")),
        }
        .in_field("value")
    }
}

// Cheap source-presence check before invoking the one native calculation for
// requiredness. This does not decide whether a paid amount is allowed or round
// an individual record; the settled packet decides that below.
pub(crate) fn has_unanswered(draft: &crate::FederalDraft, section: crate::TaxSection) -> bool {
    if !draft
        .setup
        .as_ref()
        .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Joint)
    {
        return false;
    }
    let Some(deductions) = &draft.deductions else {
        return false;
    };
    match section {
        crate::TaxSection::OtherItemized => {
            (deductions.investment_property_taxes() > 0 && deductions.investment_property_tax_owners().is_none())
                || deductions.investment_interest.iter().flat_map(|borrowing| &borrowing.loans).any(|loan| loan.source == 0
                    && ((loan.paid > 0 && loan.paid_owners.is_none()) || (loan.carry > 0 && loan.carry_owners.is_none())))
        }
        crate::TaxSection::Taxes => deductions.income_tax_paid() > 0 && deductions.income_tax_owners().is_none(),
        crate::TaxSection::MoreCredits => draft.credits.as_ref().is_some_and(|credits| matches!(&credits.foreign_tax,
            crate::ForeignTaxClaim::Deduction { taxes } if taxes.iter().any(|tax| tax.tax > 0 && tax.owners.is_none()
                && matches!(tax.income, crate::ForeignTaxIncome::Investment | crate::ForeignTaxIncome::Passive)))),
        _ => false,
    }
}
