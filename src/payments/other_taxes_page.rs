#[allow(unused_imports)]
use super::*;
// Other taxes: the stored records the page owns, one list
// item per kind (Schedule H employers, Trump accounts, Form 8828 homes).
pub(super) fn other_tax_items(d: &FederalDraft) -> Vec<OtherTax> {
    let mut items = Vec::new();
    if let Some(screening) = d.screening.as_ref() {
        if !screening.household_employment.is_empty() {
            items.push(OtherTax::HouseholdEmployees {
                employers: screening.household_employment.iter().map(Into::into).collect(),
            });
        }
        if !screening.trump_accounts.is_empty() {
            let mut accounts = crate::trump_account::project_accounts(d);
            for account in &mut accounts {
                account.returns.retain(|row| row.document_id.is_none());
            }
            items.push(OtherTax::TrumpAccounts {
                accounts: accounts.iter().map(Into::into).collect(),
            });
        }
    }
    if let Some(credits) = d.credits.as_ref()
        && !credits.mortgage_subsidies.is_empty()
    {
        items.push(OtherTax::MortgageSubsidyRecapture {
            homes: credits.mortgage_subsidies.iter().cloned().map(Into::into).collect(),
        });
    }
    if !d.coverdell_excess.is_empty() {
        items.push(OtherTax::CoverdellExcess {
            beneficiaries: d.coverdell_excess.iter().cloned().map(Into::into).collect(),
        });
    }
    items
}
// The list onto the records it owns: a kind absent from the list clears
// its field. Two items of the same kind keep the last one entered.
pub(super) fn apply_other_taxes(d: &mut FederalDraft, items: &[OtherTax]) {
    let mut employers = Vec::new();
    let mut accounts = Vec::new();
    let mut homes = Vec::new();
    let mut coverdell = Vec::new();
    for item in items {
        match item {
            OtherTax::CoverdellExcess { beneficiaries } => {
                coverdell = beneficiaries.iter().cloned().map(Into::into).collect();
            }
            OtherTax::HouseholdEmployees { employers: value } => {
                employers = value.iter().map(Into::into).collect();
            }
            OtherTax::TrumpAccounts { accounts: value } => {
                accounts = value.iter().map(Into::into).collect();
            }
            OtherTax::MortgageSubsidyRecapture { homes: value } => {
                homes = value.iter().cloned().map(Into::into).collect();
            }
        }
    }
    d.coverdell_excess = coverdell;
    if let Some(screening) = d.screening.as_mut() {
        screening.household_employment = employers;
        screening.trump_accounts = accounts;
    }
    if let Some(credits) = d.credits.as_mut() {
        credits.mortgage_subsidies = homes;
    }
}
// A Form 8828 sale names its home sale entry by a hidden ID: a row keeps
// an ID that is one of the return's home sales, and the others take the
// unused home sale closing on the row's disposition date, else the first
// unused one; a row with no home sale left keeps 0, which the estimate
// names (the sale is entered on Investment sales). Derived on every save
// (follow_home_sales), so a sale entered after the Form 8828 row binds.
pub(super) fn bind_home_sales(d: &FederalDraft, homes: &mut [MortgageSubsidyEntry]) {
    let sales: Vec<i64> = d
        .income
        .iter()
        .filter(|e| matches!(&e.income, FederalIncome::HomeSale { data } if !data.other_property()))
        .map(|e| e.id)
        .collect();
    let closed = |id: i64| {
        d.income.iter().find_map(|e| match &e.income {
            FederalIncome::HomeSale { data } if e.id == id => {
                crate::HomeSale::from(data).facts.map(|f| f.closed)
            }
            _ => None,
        })
    };
    let mut used: Vec<i64> = Vec::new();
    let mut kept = alloc::vec![false; homes.len()];
    for (home, kept) in homes.iter_mut().zip(kept.iter_mut()) {
        if let Some(id) = sale_id(home)
            && sales.contains(id)
            && !used.contains(id)
        {
            used.push(*id);
            *kept = true;
        }
    }
    for (home, kept) in homes.iter_mut().zip(kept) {
        let disposed = match &home.disposition {
            MortgageSubsidyDisposition::Ordinary { facts, .. } => Some(facts.disposed_on),
            _ => None,
        };
        let Some(id) = sale_id(home).filter(|_| !kept) else {
            continue;
        };
        let unused = || sales.iter().filter(|s| !used.contains(s));
        match unused()
            .find(|s| disposed.is_some() && closed(**s) == disposed)
            .or_else(|| unused().next())
        {
            Some(next) => {
                *id = *next;
                used.push(*next);
            }
            None => *id = 0,
        }
    }
}
// Form 8828's hidden home sale IDs follow the return's home sales (a
// derived value of every save).
pub(super) fn follow_home_sales(d: &mut FederalDraft) {
    let Some(mut homes) = d.credits.as_ref().map(|c| c.mortgage_subsidies.clone()) else {
        return;
    };
    bind_home_sales(d, &mut homes);
    if let Some(credits) = d.credits.as_mut() {
        credits.mortgage_subsidies = homes;
    }
}

impl Interview {
    pub(crate) fn other_taxes_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        if d.credits.is_none() || d.screening.is_none() {
            return problem_key("PROBLEM_SECTION_UNAVAILABLE_YET");
        }
        Reply::Edit(P::SaveOtherTaxes {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: OTHER_TAXES_PAGE,
            previous: self.previous_shown(TaxSection::OtherTaxes),
            review,
            estimate: self.estimate_now(TaxSection::OtherTaxes),
            warnings: Vec::new(),
            progress: Some(self.topic_progress(OTHER_TAXES_PAGE, None)),
            allowed_owners: self.allowed_owners(),
            mortgage_gift_owners: if self.joint() { alloc::vec![0, 1, 2] } else { Vec::new() },
            coverdell_corrections: self.coverdell_corrections(),
            value: OtherTaxes {
                taxes: other_tax_items(d),
                trump_returns: d
                    .income
                    .iter()
                    .filter_map(|entry| {
                        crate::trump_account::printed_excess(entry)?;
                        let data = entry.income.retirement()?;
                        let facts = d
                            .screening
                            .as_ref()?
                            .trump_accounts
                            .iter()
                            .filter(|a| a.owner == entry.owner.owned())
                            .flat_map(|a| &a.returns)
                            .find(|p| p.document_id == Some(entry.id));
                        Some(crate::TrumpExcessDocumentView {
                            id: entry.id,
                            label: entry.label.clone(),
                            principal: data.gross,
                            earnings: data.form_boxes.excess_contribution_earnings,
                            received: facts.map(|v| v.received),
                            deadline: facts.map(|v| v.deadline),
                            confirmed: facts.is_some_and(|v| v.confirmed),
                        })
                    })
                    .collect(),
            },
        })
    }
    // Other taxes: the list onto the records it owns
    // (apply_other_taxes), with the server twins of its client rules and
    // the owner lists (allowed_owners, never the echoed list). Other taxes is
    // a section, recorded once saved.
    pub(crate) fn other_taxes_candidate(
        &self,
        value: &OtherTaxes,
    ) -> Result<FederalDraft, crate::validation::SaveFailure> {
        let owners = self.allowed_owners();
        let owned = |owner: Owner| owners.contains(&(owner as i32));
        let coverdell: Vec<_> = value
            .taxes
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                OtherTax::CoverdellExcess { beneficiaries } => Some((index, beneficiaries)),
                _ => None,
            })
            .collect();
        // Tax structure: one whole-return Coverdell section owns all beneficiary
        // rows; a second section would overwrite the first in apply_other_taxes.
        if coverdell.len() > 1 {
            return Err(crate::validation::ValidationIssue::new("COVERDELL_TOPIC_TWICE")
                .at_field("beneficiaries")
                .in_list("taxes", coverdell[1].0)
                .in_field("value")
                .into());
        }
        if let Some((tax_index, values)) = coverdell.first() {
            if values.is_empty() {
                return Err(libertas_formatted_text("COVERDELL_INVALID_SOURCE", &[]).into());
            }
            let records: Vec<_> = values.iter().cloned().map(Into::into).collect();
            if let Some(issue) = crate::coverdell::issue(&records, &self.draft) {
                return Err(issue.in_list("taxes", *tax_index).in_field("value").into());
            }
        }
        for (tax_index, item) in value.taxes.iter().enumerate() {
            match item {
                OtherTax::HouseholdEmployees { employers } => {
                    if let Some(index) = employers.iter().position(|e| !owned(e.owner)) {
                        return Err(ValidationIssue::new(
                            "MESSAGE_SCHEDULE_H_YOUR_SPOUSE_S_HOUSEHOLD_EMPLOYMENT_IS_ENTERED_ONLY_ON_A_JOINT_RET",
                        ).at_field("owner").in_list("employers", index)
                            .in_list("taxes", tax_index).in_field("value").into());
                    }
                    if let Some(index) = employers
                        .iter()
                        .enumerate()
                        .position(|(i, e)| employers[..i].iter().any(|p| p.owner == e.owner))
                    {
                        return Err(ValidationIssue::new("HOUSEHOLD_EMPLOYER_TWICE")
                            .at_field("owner")
                            .in_list("employers", index)
                            .in_list("taxes", tax_index)
                            .in_field("value")
                            .into());
                    }
                    let records: Vec<crate::HouseholdEmployer> =
                        employers.iter().map(Into::into).collect();
                    if !crate::household_employment::valid(&records) {
                        return Err(crate::interview::status(
                            "MESSAGE_SCHEDULE_H_ENTER_2026_PAYMENT_DATES_TWO_LETTER_STATE_CODES_EACH_2025_QUARTER",
                        ).into());
                    }
                }
                OtherTax::TrumpAccounts { accounts } => {
                    if let Some(index) = accounts.iter().position(|a| !owned(a.owner)) {
                        return Err(ValidationIssue::new(
                            "MESSAGE_A_TRUMP_ACCOUNT_FOR_YOUR_SPOUSE_IS_ENTERED_ONLY_ON_A_JOINT_RETURN",
                        ).at_field("owner").in_list("accounts", index)
                            .in_list("taxes", tax_index).in_field("value").into());
                    }
                    let records: Vec<crate::TrumpAccount> =
                        accounts.iter().map(Into::into).collect();
                    if let Some(issue) = crate::trump_account::accounts_issue(&records) {
                        return Err(issue.in_list("taxes", tax_index).in_field("value").into());
                    }
                    if !crate::trump_account::valid_accounts(&records) {
                        return Err(crate::interview::status(
                            "MESSAGE_FORM_5329_A_RETURNED_EXCESS_CONTRIBUTION_NEEDS_A_RECEIPT_DATE_FROM_JULY_4_20",
                        ).into());
                    }
                }
                OtherTax::MortgageSubsidyRecapture { homes } => {
                    if let Some(index) = homes.iter().enumerate().position(|(i, h)| {
                        homes[..i].iter().any(|p| p.label.trim() == h.label.trim())
                    }) {
                        return Err(ValidationIssue::new("MORTGAGE_SUBSIDY_HOME_TWICE")
                            .at_field("label")
                            .in_list("homes", index)
                            .in_list("taxes", tax_index)
                            .in_field("value")
                            .into());
                    }
                    if let Some(index) = homes.iter().position(|home| !home.valid()) {
                        // Keep the source owner's existing validity check; identify
                        // the submitted name or disposition that needs correction.
                        let field = if homes[index].label.trim().is_empty() {
                            "label"
                        } else {
                            "disposition"
                        };
                        return Err(ValidationIssue::new(
                            "MESSAGE_FORM_8828_ENTER_A_2026_DISPOSITION_DATE_AN_ORIGINAL_LOAN_CLOSED_AFTER_1990_A",
                        ).at_field(field).in_list("homes", index)
                            .in_list("taxes", tax_index).in_field("value").into());
                    }
                }
                _ => {}
            }
        }
        let mut draft = self.draft.clone();
        if draft.credits.is_none() || draft.screening.is_none() {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        let mut items = value.taxes.clone();
        for (tax_index, item) in items.iter_mut().enumerate() {
            if let OtherTax::MortgageSubsidyRecapture { homes } = item {
                let mut records: Vec<MortgageSubsidyEntry> =
                    homes.iter().cloned().map(Into::into).collect();
                bind_home_sales(&draft, &mut records);
                *homes = records.into_iter().map(Into::into).collect();
                let sold: Vec<(usize, i64)> = homes
                    .iter()
                    .enumerate()
                    .filter_map(|(index, home)| home.home_sale_income_id().map(|id| (index, id)))
                    .collect();
                if let Some((index, _)) = sold.iter().enumerate().find_map(|(i, (index, id))| {
                    sold[..i].iter().any(|(_, prior)| prior == id).then_some((*index, *id))
                }) {
                    return Err(ValidationIssue::new("MORTGAGE_SUBSIDY_HOME_TWICE")
                        .at_list_item("homes", index)
                        .in_list("taxes", tax_index)
                        .in_field("value")
                        .into());
                }
            }
        }
        apply_other_taxes(&mut draft, &items);
        Self::document_answer_ids(
            &self
                .draft
                .income
                .iter()
                .filter(|entry| crate::trump_account::printed_excess(entry).is_some())
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            &value.trump_returns.iter().map(|row| row.id).collect::<Vec<_>>(),
            "trump_returns",
        )?;
        for (index, row) in value.trump_returns.iter().enumerate() {
            let source = self.draft.income.iter().find(|entry| entry.id == row.id).unwrap();
            let (principal, earnings) = crate::trump_account::printed_excess(source).unwrap();
            let failure = |key, field| {
                ValidationIssue::new(key)
                    .at_field(field)
                    .in_list("trump_returns", index)
                    .in_field("value")
            };
            let received =
                row.received.ok_or_else(|| failure("DOCUMENT_TRUMP_RETURN_DATE", "received"))?;
            if received / 10000 != self.draft.tax_year as u32 {
                return Err(failure("DOCUMENT_TRUMP_RETURN_DATE", "received").into());
            }
            let deadline = row
                .deadline
                .ok_or_else(|| failure("DOCUMENT_TRUMP_RETURN_DEADLINE", "deadline"))?;
            let account = draft
                .screening
                .as_mut()
                .unwrap()
                .trump_accounts
                .iter_mut()
                .find(|a| a.owner == source.owner.owned())
                .ok_or_else(|| failure("DOCUMENT_TRUMP_ACCOUNT_FACTS", "confirmed"))?;
            account.returns.push(crate::TrumpExcessReturn {
                document_id: Some(source.id),
                principal,
                earnings,
                received,
                deadline,
                confirmed: row.confirmed,
            });
        }
        finish_section(&mut draft, TaxSection::OtherTaxes);
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_other_taxes(
        &self,
        value: &OtherTaxes,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != OTHER_TAXES_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.other_taxes_candidate(value).and_then(|draft| {
            match crate::mortgage_recapture::gift_owner_save_rule(&draft, value) {
                Some(issue) => Err(issue.into()),
                None => Ok(draft),
            }
        })
    }
}
