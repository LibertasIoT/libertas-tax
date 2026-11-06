extern crate std;

use super::paper_documents::remember;
use crate::tests::*;

pub(in crate::tests) fn estate_snapshot(source: &EstateTrustSource) -> K1EstateTrust {
    source.page()
}

pub(in crate::tests) fn estate_source(data: FederalEstateTrustPortfolioK1) -> EstateTrustSource {
    let FederalEstateTrustPortfolioK1 {
        final_character,
        amt_reconciliation,
        investment_income,
        reported_credits,
        foreign_statement,
        investment_property,
        ein: _,
        interest: _,
        ordinary_dividends: _,
        qualified_dividends: _,
        section199a_dividends,
        code_i_dividends_only,
        short_gain: _,
        long_gain: _,
        rate_28_gain: _,
        unrecaptured_gain: _,
        royalty_income: _,
        box_5_royalty_only,
        box_9_royalty_depreciation: _,
        box_9_royalty_sound_recording_only,
        box_9_royalty_depletion: _,
        box_9_royalty_cost_depletion_only,
        box_9_royalty_amortization: _,
        box_9_royalty_amortization_only,
        activities,
        qbi,
        gross_farming_fishing_income: _,
        box_10_interest_ird_estate_tax: _,
        box_10_interest_ird_only,
        final_67e_deduction: _,
        final_67e_no_niit_allocation,
        final_personal_property_tax,
        final_real_estate_tax,
        final_state_income_tax,
        final_state_sales_tax,
        final_taxes_only,
        final_short_carry: _,
        final_long_carry: _,
        final_year_successor,
        final_k1: _,
        final_mortgages,
        final_carry_same_tax_character,
        final_nol,
        amt_box12a: _,
        amt_box12a_standalone,
        amt_box12b: _,
        amt_box12_components_confirmed,
        amt_box12c: _,
        amt_box12d: _,
        amt_box12e: _,
        amt_box12e_included_in_d,
        amt_box12f: _,
        amt_box12f_included_in_d,
        backup_withholding: _,
        allocated_estimated_tax: _,
        form_1041t_timely: _,
        tax_exempt_interest: _,
        private_activity_interest,
        niit_adjustment,
        unentered,
        complete_portfolio_only,
        entity_period,
        minimum_tax_exclusion: _,
    } = &data;
    let mut source = EstateTrustSource {
        form: crate::K1EstateTrustPaper::from(&data),
        statements: EstateTrustStatements {
            final_character: final_character.clone(),
            amt_reconciliation: amt_reconciliation.clone(),
            investment_income: investment_income.clone(),
            reported_credits: reported_credits.iter().map(Into::into).collect(),
            foreign_statement: foreign_statement.clone(),
            investment_property: investment_property.clone(),
            section199a_dividends: section199a_dividends.clone(),
            code_i_dividends_only: *code_i_dividends_only,
            box_5_royalty_only: *box_5_royalty_only,
            box_9_royalty_sound_recording_only: *box_9_royalty_sound_recording_only,
            box_9_royalty_cost_depletion_only: *box_9_royalty_cost_depletion_only,
            box_9_royalty_amortization_only: *box_9_royalty_amortization_only,
            activities: activities.activities.clone(),
            qbi: qbi.clone(),
            box_10_interest_ird_only: *box_10_interest_ird_only,
            final_67e_no_niit_allocation: *final_67e_no_niit_allocation,
            final_personal_property_tax: *final_personal_property_tax,
            final_real_estate_tax: *final_real_estate_tax,
            final_state_income_tax: *final_state_income_tax,
            final_state_sales_tax: *final_state_sales_tax,
            final_taxes_only: *final_taxes_only,
            final_carry_same_tax_character: *final_carry_same_tax_character,
            amt_box12a_standalone: *amt_box12a_standalone,
            amt_box12_components_confirmed: *amt_box12_components_confirmed,
            amt_box12e_included_in_d: *amt_box12e_included_in_d,
            amt_box12f_included_in_d: *amt_box12f_included_in_d,
            private_activity_interest: *private_activity_interest,
            final_nol: final_nol.clone(),
            niit_adjustment: niit_adjustment.as_ref().map(|a| a.modified_agi),
            unentered: unentered.clone(),
        },
        answers: EstateTrustAnswers {
            final_year_successor: *final_year_successor,
            final_mortgages: final_mortgages.clone(),
            complete_portfolio_only: *complete_portfolio_only,
            entity_period: entity_period.as_ref().map(|p| p.accounting.clone()),
        },
    };
    estate_fixture_markers(&mut source);
    source
}

pub(in crate::tests) fn estate_fixture_markers(source: &mut EstateTrustSource) {
    fn add(rows: &mut Vec<EstateTrustCodeRow>, code: &str, amount: i64) {
        if !rows.iter().any(|row| row.code == code) {
            rows.push(EstateTrustCodeRow { code: code.into(), amount });
        }
    }
    fn amounts(rows: impl Iterator<Item = (&'static str, i64)>) -> Vec<(&'static str, i64)> {
        let mut values: Vec<(&'static str, i64)> = Vec::new();
        for (code, amount) in rows {
            if let Some((_, total)) = values.iter_mut().find(|(known, _)| *known == code) {
                *total = crate::kernel::ratio::checked_sum([*total, amount])
                    .expect("estate fixture code amount overflow");
            } else {
                values.push((code, amount));
            }
        }
        values
    }
    for (code, amount) in amounts(
        source
            .statements
            .reported_credits
            .iter()
            .map(|credit| (credit.code_name(), credit.paper.amount)),
    ) {
        add(&mut source.form.credits, code, amount);
    }
    if let Some(reconciliation) = &source.statements.amt_reconciliation {
        for (code, amount) in amounts(reconciliation.adjustments.iter().map(|row| {
            (
                match row.code {
                    EstateAmtCode::DepreciationG => "G",
                    EstateAmtCode::DepletionH => "H",
                    EstateAmtCode::AmortizationI => "I",
                },
                row.amount,
            )
        })) {
            add(&mut source.form.amt, code, amount);
        }
    }
    if let Some(statement) = &source.statements.foreign_statement {
        add(&mut source.form.information, "B", statement.tax);
    }
    if let Some(statement) = &source.statements.investment_income {
        add(&mut source.form.information, "E", statement.regular);
    }
    for (code, amount) in
        amounts(source.statements.investment_property.iter().map(|row| match &row.property {
            K1InvestmentProperty::RehabilitationRental { expenditures, .. }
            | K1InvestmentProperty::RehabilitationOther { expenditures, .. } => {
                ("C", *expenditures)
            }
            K1InvestmentProperty::Energy { basis, .. } => ("D", *basis),
            K1InvestmentProperty::CleanElectricity { basis, .. } => ("M", *basis),
        }))
    {
        add(&mut source.form.information, code, amount);
    }
    for item in &source.statements.unentered {
        let code = match item {
            EstateTrustK1Item::ForeignTaxes => "B",
            EstateTrustK1Item::Rehabilitation => "C",
            EstateTrustK1Item::EnergyProperty => "D",
            EstateTrustK1Item::InvestmentIncome => "E",
            EstateTrustK1Item::ForeignTradingGrossReceipts => "G",
            EstateTrustK1Item::AdvancedCoalGasification => "J",
            EstateTrustK1Item::AdvancedEnergy => "K",
            EstateTrustK1Item::AdvancedManufacturing => "L",
            EstateTrustK1Item::CleanElectricity => "M",
            EstateTrustK1Item::OtherInformation => "ZZ",
            EstateTrustK1Item::CreditRecapture => {
                add(&mut source.form.credits, "R", 0);
                continue;
            }
            EstateTrustK1Item::Credits | EstateTrustK1Item::AmtItems => continue,
        };
        // Presence-only synthetic evidence uses a blank printed amount. This
        // marker identifies the code; it does not recover an unknown tax amount.
        add(&mut source.form.information, code, 0);
    }
}

std::thread_local! {
    static REOPENED_ESTATE: core::cell::RefCell<Vec<K1EstateTrustDocument>> = const { core::cell::RefCell::new(Vec::new()) };
}

type EstateFixture = (K1EstateTrustDocument, EstateTrustSource, Option<K1EstateTrust>, bool);

std::thread_local! {
    static ESTATE_ANSWERS: core::cell::RefCell<Vec<EstateFixture>> =
        const { core::cell::RefCell::new(Vec::new()) };
}

/// A document reopened from a session is a new value; its fixture answers are the stored
/// source's own, unless a fixture already registered this exact document.
pub(in crate::tests) fn remember_reopened_estate_document(
    document: K1EstateTrustDocument,
    source: EstateTrustSource,
) {
    let reopened = REOPENED_ESTATE.with(|rows| rows.borrow().contains(&document));
    let known =
        ESTATE_ANSWERS.with(|rows| rows.borrow().iter().any(|(given, _, _, _)| given == &document));
    if reopened || !known {
        let document = remember_estate_answers(document, source, None, false);
        REOPENED_ESTATE.with(|rows| rows.borrow_mut().push(document));
    }
}

pub(in crate::tests) fn remember_estate_document(
    document: K1EstateTrustDocument,
    source: EstateTrustSource,
) -> K1EstateTrustDocument {
    remember_estate_fixture(document, source, None)
}

pub(super) fn remember_estate_fixture(
    document: K1EstateTrustDocument,
    source: EstateTrustSource,
    page: Option<K1EstateTrust>,
) -> K1EstateTrustDocument {
    remember_estate_answers(document, source, page, true)
}

pub(super) fn remember_estate_answers(
    document: K1EstateTrustDocument,
    source: EstateTrustSource,
    page: Option<K1EstateTrust>,
    supplied: bool,
) -> K1EstateTrustDocument {
    remember(
        IncomeSource::K1 { value: K1Document::K1EstateTrust(document.clone()) },
        source.moved_questions().into_iter().map(|(_, _, answer, _)| answer).collect(),
    );
    REOPENED_ESTATE.with(|rows| rows.borrow_mut().retain(|given| given != &document));
    ESTATE_ANSWERS.with(|rows| {
        let mut rows = rows.borrow_mut();
        rows.retain(|(given, _, _, _)| given != &document);
        rows.push((document.clone(), source, page, supplied));
    });
    document
}

pub(in crate::tests) fn estate_fixture_source(
    document: &K1EstateTrustDocument,
) -> EstateTrustSource {
    ESTATE_ANSWERS.with(|rows| {
        rows.borrow()
            .iter()
            .rev()
            .find(|(given, _, _, _)| given == document)
            .map_or_else(|| document.source(), |(_, source, _, _)| source.clone())
    })
}

pub(super) fn estate_fixture_page(document: &K1EstateTrustDocument) -> Option<K1EstateTrust> {
    ESTATE_ANSWERS.with(|rows| {
        rows.borrow()
            .iter()
            .rev()
            .find(|(given, _, _, _)| given == document)
            .and_then(|(_, _, page, _)| page.clone())
    })
}

fn prepare_estate_document(source: &mut EstateTrustSource) {
    estate_fixture_markers(source);
    for item in &source.statements.unentered {
        match item {
            EstateTrustK1Item::Credits => assert!(
                source.form.credits.iter().any(|row| !matches!(row.code.as_str(), "A" | "B" | "R")
                    && !source
                        .statements
                        .reported_credits
                        .iter()
                        .any(|credit| credit.code_name() == row.code)),
                "estate fixture unentered Credits has no exact unentered box 13 code and amount; supply the actual printed row"
            ),
            EstateTrustK1Item::AmtItems => assert!(
                source.statements.amt_reconciliation.is_none()
                    && source
                        .form
                        .amt
                        .iter()
                        .any(|row| matches!(row.code.as_str(), "G" | "H" | "I")),
                "estate fixture unentered AmtItems has no exact unentered box 12 G/H/I row; supply the actual printed code and amount"
            ),
            _ => {}
        }
    }
}

pub(in crate::tests) trait EstatePaperFixture {
    fn estate_document(self) -> K1EstateTrustDocument;
}

impl EstatePaperFixture for K1EstateTrustDocument {
    fn estate_document(self) -> K1EstateTrustDocument {
        self
    }
}

impl EstatePaperFixture for EstateTrustSource {
    fn estate_document(mut self) -> K1EstateTrustDocument {
        prepare_estate_document(&mut self);
        remember_estate_document(K1EstateTrustDocument::from_source(&self), self)
    }
}

impl EstatePaperFixture for K1EstateTrust {
    fn estate_document(self) -> K1EstateTrustDocument {
        let mut source = estate_source(self.record());
        source.form.estate_trust = self.estate_trust.clone();
        source.form.fiduciary = self.fiduciary.clone();
        source.form.beneficiary = self.beneficiary.clone();
        prepare_estate_document(&mut source);
        let mut document = K1EstateTrustDocument::from_source(&source);
        document.estate_activities = self.estate_activities.clone();
        document.estate_mortgages = self.estate_mortgages.clone();
        remember_estate_fixture(document, source, Some(self))
    }
}

pub(in crate::tests) fn estate_document(source: impl EstatePaperFixture) -> K1EstateTrustDocument {
    source.estate_document()
}

pub(super) fn fixture_estate_answers_supplied(document: &K1EstateTrustDocument) -> bool {
    ESTATE_ANSWERS.with(|rows| {
        rows.borrow().iter().any(|(given, _, _, supplied)| given == document && *supplied)
    })
}

pub(super) fn rebind_estate_fixture_answers(
    before: &K1EstateTrustDocument,
    after: &K1EstateTrustDocument,
) {
    let mut source = estate_fixture_source(before);
    source.form = after.form.clone();
    let page = estate_fixture_page(before).map(|mut page| {
        page.estate_trust = after.form.estate_trust.clone();
        page.fiduciary = after.form.fiduciary.clone();
        page.beneficiary = after.form.beneficiary.clone();
        page
    });
    remember_estate_fixture(after.clone(), source, page);
}
