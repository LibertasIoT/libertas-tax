// Coverage: Form 8828 — Supported (audited 2026-09-28, u5-credits).
//   Lines: Part I facts from the issuer notice, lines 9-23 for an ordinary sale (matched HomeSale
//   entry) or a nonspouse gift (FMV as line 9), line 13 QSML gain share, line 15 modified AGI,
//   line 18 income percentage (rounded), line 19 6.25%, line 20 issuer percentage or the
//   early-repayment worksheet, line 23 to 2026 Schedule 2 line 13b; exemptions (section 1041,
//   home improvement loan, pre-1991 loan, on-site casualty replacement) and the nine-year period.
//   Page: the Other taxes list's Form 8828 item (OtherTax::MortgageSubsidyRecapture,
//   credits.mortgage_subsidies); a sale's hidden home sale ID is derived on every save
//   (follow_home_sales -> bind_home_sales: a kept match, else the unused home sale closing on the
//   disposition date, else the first unused one; 0 without one, which the estimate names). u6-home
//   (2026-09-28): the home sale has its page (Investment sales, MainHomeSale); its closing date
//   (1099-S box 1) must equal the disposition date. Tests: src/tests/home/home_sale_page.rs
//   form_8828_recapture_on_a_sale_through_both_pages,
//   form_8828_sale_entered_before_the_home_sale_binds_on_the_next_save.
//   Re-verified against Form 8828 (Rev. November 2024, the 2026 draft posting): no change.
//   Evidence: src/tests/home/investment.rs (the engine cases); src/tests/credits/other_taxes.rs
//   form_8828_gift_through_the_other_taxes_page, form_8828_nine_year_period_boundary,
//   form_8828_income_percentage_boundaries, form_8828_half_the_gain_limits_the_tax,
//   form_8828_homes_are_named_once.
use crate::kernel::lines::line;
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use crate::{
    FederalDraft, FederalIncome, FederalResult, HomeExclusion, MortgageSubsidyDisposition,
    MortgageSubsidyEntry, MortgageSubsidyPayoff, MortgageSubsidySpouseRecipient,
    MortgageSubsidyTransfer,
};
use alloc::format;

const SOURCE: &str = "https://www.irs.gov/pub/irs-pdf/f8828.pdf";

// Whole-dollar native tax results retained at their actual source, once.
// A Both answer identifies shared ownership; it does not invent equal shares.
#[derive(Clone, Copy, Default)]
pub(crate) struct MortgageRecaptureSources {
    pub(crate) joint: i64,
    pub(crate) exclusive: [i64; 2],
    pub(crate) shared: i64,
    pub(crate) complete: bool,
}

pub(crate) fn calculate_all(
    entries: &[MortgageSubsidyEntry],
    draft: &FederalDraft,
    agi: i64,
    tax_exempt_interest: i64,
    result: &mut FederalResult,
) -> MortgageRecaptureSources {
    let mut sources = MortgageRecaptureSources { complete: true, ..Default::default() };
    for (index, entry) in entries.iter().enumerate() {
        let first_line = result.lines.len();
        let first_note = result.review_notes.len();
        let tax = calculate_one(&entry.disposition, draft, agi, tax_exempt_interest, result);
        sources.joint += tax;
        // Failed or unreviewed native input must not become a proven zero.
        if !result.lines[first_line..]
            .iter()
            .any(|line| line.id == "mortgage_subsidy_recapture_tax")
        {
            sources.complete = false;
        }
        if tax != 0 {
            let owner = match &entry.disposition {
                MortgageSubsidyDisposition::Ordinary { transfer, .. } => match transfer {
                    MortgageSubsidyTransfer::Sale { home_sale_income_id, .. } => draft
                        .income
                        .iter()
                        .find(|source| {
                            source.id == *home_sale_income_id
                                && matches!(source.income, FederalIncome::HomeSale { .. })
                        })
                        .map(|source| source.owner)
                        .map(|owner| match owner {
                            crate::Owner::Taxpayer => crate::MortgageSubsidyGiftOwners::Taxpayer,
                            crate::Owner::Spouse => crate::MortgageSubsidyGiftOwners::Spouse,
                        }),
                    MortgageSubsidyTransfer::GiftToOther { owners, .. } => {
                        if draft.setup.as_ref().is_some_and(|setup| {
                            setup.filing_status() != crate::FederalStatus::Joint
                        }) {
                            Some(crate::MortgageSubsidyGiftOwners::Taxpayer)
                        } else {
                            *owners
                        }
                    }
                },
                _ => None,
            };
            match owner {
                Some(crate::MortgageSubsidyGiftOwners::Taxpayer) => sources.exclusive[0] += tax,
                Some(crate::MortgageSubsidyGiftOwners::Spouse) => sources.exclusive[1] += tax,
                Some(crate::MortgageSubsidyGiftOwners::Both) => sources.shared += tax,
                None => sources.complete = false,
            }
        }
        let prefix = format!("mortgage_subsidy_{}", index + 1);
        for calculated in &mut result.lines[first_line..] {
            calculated.id = calculated.id.replacen("mortgage_subsidy", &prefix, 1);
            calculated.rule = format!("US-2026.{}", calculated.id);
            for input in &mut calculated.inputs {
                if input.starts_with("mortgage_subsidy_") {
                    *input = input.replacen("mortgage_subsidy", &prefix, 1);
                }
            }
        }
        // The breakdown labels each line by its numbered id
        // ("Mortgage subsidy 1 ..."), in the order of the entries.
        for note in &mut result.review_notes[first_note..] {
            #[cfg(test)]
            let english = format!("{}: {}", entry.label, note.english);
            let message = core::mem::replace(
                &mut note.display,
                crate::Message::new(
                    "NOTE_WITH_LABEL",
                    alloc::vec![crate::MessageArg::Text(entry.label.clone())],
                ),
            );
            note.display.args.push(crate::MessageArg::Nested(alloc::boxed::Box::new(message)));
            #[cfg(test)]
            {
                note.english = english;
            }
        }
    }
    if !entries.is_empty() {
        let child_ids: alloc::vec::Vec<_> = entries
            .iter()
            .enumerate()
            .map(|(index, _)| format!("mortgage_subsidy_{}_recapture_tax", index + 1))
            .collect();
        let child_inputs: alloc::vec::Vec<_> =
            child_ids.iter().map(alloc::string::String::as_str).collect();
        line(
            result,
            "mortgage_subsidy_recapture_tax",
            sources.joint,
            &child_inputs,
            SOURCE,
            msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX"),
        );
    }
    sources
}

// Review reuses the actual estimate so a later income correction that makes
// a gift taxable returns the person to its source page, without another ledger.
pub(crate) fn gift_owner_missing(draft: &FederalDraft, result: &FederalResult) -> bool {
    crate::interview::joint(draft)
        && draft.credits.as_ref().is_some_and(|credits| {
            credits.mortgage_subsidies.iter().enumerate().any(|(index, home)| {
                matches!(
                    &home.disposition,
                    MortgageSubsidyDisposition::Ordinary {
                        transfer: MortgageSubsidyTransfer::GiftToOther { owners: None, .. },
                        ..
                    }
                ) && result.lines.iter().any(|line| {
                    line.id == format!("mortgage_subsidy_{}_recapture_tax", index + 1)
                        && line.amount > 0
                })
            })
        })
}

// Save validates this source fact. A running estimate may still display
// the unchanged joint tax while the person completes its ownership answer.
pub(crate) fn gift_owner_save_rule(
    draft: &FederalDraft,
    value: &crate::OtherTaxes,
) -> Option<crate::validation::ValidationIssue> {
    let joint = draft
        .setup
        .as_ref()
        .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Joint);
    let missing_gift_owner = value.taxes.iter().any(|item| match item {
        crate::OtherTax::MortgageSubsidyRecapture { homes } => homes.iter().any(|home| {
            matches!(
                &home.disposition,
                MortgageSubsidyDisposition::Ordinary {
                    transfer: MortgageSubsidyTransfer::GiftToOther { owners: None, .. },
                    ..
                }
            )
        }),
        _ => false,
    });
    let native = (joint && missing_gift_owner).then(|| crate::federal_rules::estimate(draft));
    for (tax_index, item) in value.taxes.iter().enumerate() {
        let crate::OtherTax::MortgageSubsidyRecapture { homes } = item else {
            continue;
        };
        for (home_index, home) in homes.iter().enumerate() {
            let MortgageSubsidyDisposition::Ordinary {
                transfer: MortgageSubsidyTransfer::GiftToOther { owners, .. },
                ..
            } = &home.disposition
            else {
                continue;
            };
            // apply_other_taxes supplies the accepted whole list. Locate the
            // exact submitted gift in that list, not by label or amount.
            let positive = draft
                .credits
                .as_ref()
                .and_then(|credits| {
                    credits.mortgage_subsidies.iter().position(|saved| saved == home)
                })
                .is_some_and(|index| {
                    native.as_ref().is_some_and(|result| {
                        result.lines.iter().any(|line| {
                            line.id == format!("mortgage_subsidy_{}_recapture_tax", index + 1)
                                && line.amount > 0
                        })
                    })
                });
            let key = if !joint && owners.is_some() {
                "MORTGAGE_GIFT_OWNER_NOT_JOINT"
            } else if positive && owners.is_none() {
                "MORTGAGE_GIFT_OWNER_REQUIRED"
            } else {
                continue;
            };
            return Some(
                crate::validation::ValidationIssue::new(key)
                    .at_field("owners")
                    .in_field("transfer")
                    .in_field("disposition")
                    .in_list("homes", home_index)
                    .in_list("taxes", tax_index)
                    .in_field("value"),
            );
        }
    }
    None
}

fn rounded_up_years(start: u32, end: u32) -> i64 {
    i64::from(end / 10000 - start / 10000) + i64::from(end % 10000 > start % 10000)
}

// The issuer notice supplies the family-size/holding-period table inputs.
// This calculation intentionally does not invent an issuer table or a
// different history for the same home disposition.
fn calculate_one(
    disposition: &MortgageSubsidyDisposition,
    draft: &FederalDraft,
    agi: i64,
    tax_exempt_interest: i64,
    result: &mut FederalResult,
) -> i64 {
    let (transfer, facts) = match disposition {
        MortgageSubsidyDisposition::ExemptSection1041 {
            recipient,
            qualifies_without_recognized_gain_or_loss,
            ..
        } => {
            crate::require_localized!(
                result,
                *qualifies_without_recognized_gain_or_loss,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_A_DIRECT_SECTION_1041_TRANSFER_TO_A_SPOUSE_OR_FORMER_SPOUSE_4835C064",
                "Confirm a direct section 1041 transfer to a spouse or former spouse incident to divorce with no recognized gain or loss; other transfers require Form 8828 review.",
            );
            if !*qualifies_without_recognized_gain_or_loss {
                return 0;
            }
            line(
                result,
                "mortgage_subsidy_recapture_tax",
                0,
                &["credits.mortgage_subsidies"],
                "https://www.govinfo.gov/link/uscode/26/143",
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_2"),
            );
            let recipient_name = match recipient {
                MortgageSubsidySpouseRecipient::CurrentSpouse => "current spouse",
                MortgageSubsidySpouseRecipient::FormerSpouseIncidentToDivorce => {
                    "former spouse incident to divorce"
                }
            };
            result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] format!(
                "The confirmed section 1041 transfer to a {recipient_name} has no Form 8828 recapture."
            ), crate::Message::new("NOTE_THE_CONFIRMED_SECTION_1041_TRANSFER_TO_A_RECIPIENT_NAME_66A5C8A8", alloc::vec![crate::MessageArg::Text(alloc::string::String::from(recipient_name))])));
            return 0;
        }
        MortgageSubsidyDisposition::ExemptHomeImprovementLoan {
            qualified_sole_subsidy, ..
        } => {
            crate::require_localized!(
                result,
                *qualified_sole_subsidy,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_SOLE_FEDERAL_MORTGAGE_SUBSIDY_WAS_A_QMB_FUNDED_QUALIFIE_A1937640",
                "Confirm the sole federal mortgage subsidy was a QMB-funded qualified home improvement loan of no more than $15,000 used for eligible improvements; other subsidized financing needs separate review.",
            );
            if !*qualified_sole_subsidy {
                return 0;
            }
            line(
                result,
                "mortgage_subsidy_recapture_tax",
                0,
                &["credits.mortgage_subsidies"],
                "https://www.irs.gov/pub/irs-pdf/i8828.pdf",
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_3"),
            );
            result.review_notes.push(
                crate::federal_model::ReviewNote::localized(#[cfg(test)] "The confirmed qualified home improvement loan is not subject to Form 8828 recapture.", crate::Message::new("NOTE_THE_CONFIRMED_QUALIFIED_HOME_IMPROVEMENT_LOAN_IS_NOT_SUBJECT_TO_7A20C395", alloc::vec![])),
            );
            return 0;
        }
        MortgageSubsidyDisposition::ExemptBefore1991 { no_later_subsidy, .. } => {
            crate::require_localized!(
                result,
                *no_later_subsidy,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_ORIGINAL_FEDERALLY_SUBSIDIZED_LOAN_WAS_PROVIDED_BEFORE_483CDD01",
                "Confirm the original federally subsidized loan was provided before 1991 and no later qualifying QMB loan or MCC became subsidized debt on this home.",
            );
            if !*no_later_subsidy {
                return 0;
            }
            line(
                result,
                "mortgage_subsidy_recapture_tax",
                0,
                &["credits.mortgage_subsidies"],
                SOURCE,
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_4"),
            );
            result.review_notes.push(
                crate::federal_model::ReviewNote::localized(#[cfg(test)] "The confirmed pre-1991 federal mortgage subsidy has no Form 8828 requirement.", crate::Message::new("NOTE_THE_CONFIRMED_PRE_1991_FEDERAL_MORTGAGE_SUBSIDY_HAS_NO_FORM_8828_5B0389D0", alloc::vec![])),
            );
            return 0;
        }
        MortgageSubsidyDisposition::ExemptReplacedCasualty {
            qualified_replacement_and_casualty_tax_resolved,
            ..
        } => {
            crate::require_localized!(
                result,
                *qualified_replacement_and_casualty_tax_resolved,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_CASUALTY_DESTROYED_HOME_WAS_ALREADY_REPLACED_ON_ITS_ORI_A5D55F58",
                "Confirm the casualty-destroyed home was already replaced on its original site for use as your main home within the ordinary deadline, and any separate casualty gain or loss is accounted for.",
            );
            if !*qualified_replacement_and_casualty_tax_resolved {
                return 0;
            }
            line(
                result,
                "mortgage_subsidy_recapture_tax",
                0,
                &["credits.mortgage_subsidies"],
                "https://www.irs.gov/pub/irs-pdf/i8828.pdf",
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_5"),
            );
            result.review_notes.push(
                crate::federal_model::ReviewNote::localized(#[cfg(test)] "The confirmed on-site casualty replacement avoids current Form 8828 recapture; separate casualty tax is handled elsewhere in the return.", crate::Message::new("NOTE_THE_CONFIRMED_ON_SITE_CASUALTY_REPLACEMENT_AVOIDS_CURRENT_FORM_8_2BA1F30C", alloc::vec![])),
            );
            return 0;
        }
        MortgageSubsidyDisposition::Unreviewed => {
            crate::issue_localized!(
                result,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_REVIEW_THE_DISPOSITION_AND_ORIGINAL_SUBSIDIZED_LOAN_BEFORE_CALCULAT_6879B419",
                "Review the disposition and original subsidized loan before calculating Form 8828.",
            );
            return 0;
        }
        MortgageSubsidyDisposition::Ordinary { transfer, facts, .. } => (transfer, facts),
    };
    crate::require_localized!(
        result,
        facts.confirmed,
        &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
        "Mortgage subsidy recapture",
        "ISSUE_CONFIRM_THE_ORDINARY_TRANSFER_QUALIFIED_MCC_OR_QMB_SOURCE_ISSUER_NO_F0B1C88E",
        "Confirm the ordinary transfer, qualified MCC or QMB source, issuer notice, owner share and original loan. A gift to a spouse or ex-spouse incident to divorce, casualty replacement and subordinate-loan gain sharing need separate treatment.",
    );
    if !facts.confirmed {
        return 0;
    }
    let (gain, included_gain) = match transfer {
        MortgageSubsidyTransfer::Sale { home_sale_income_id, qsml_gain_share } => {
            let Some((entry, home)) = draft.income.iter().find_map(|entry| {
                if entry.id == *home_sale_income_id {
                    match &entry.income {
                        FederalIncome::HomeSale { data } => Some((entry, data)),
                        _ => None,
                    }
                } else {
                    None
                }
            }) else {
                crate::issue_localized!(
                    result,
                    &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
                    "Mortgage subsidy recapture",
                    "ISSUE_THE_FORM_8828_SALE_MUST_IDENTIFY_ONE_MATCHING_HOMESALE_INCOME_ENTRY_917D600A",
                    "The Form 8828 sale must identify one matching HomeSale income entry.",
                );
                return 0;
            };
            // The sale page's closing date (1099-S box 1), or the
            // surviving-spouse sale date, is the disposition date.
            let sale_date_matches = match (&home.facts, &home.exclusion) {
                (Some(sale), _) => sale.closed == facts.disposed_on,
                (None, HomeExclusion::Widowed { sale_date, .. }) => *sale_date == facts.disposed_on,
                (None, _) => true,
            };
            crate::require_localized!(
                result,
                home.ordinary,
                &crate::interview::income_issue_context(entry)
                    .corrective(crate::TaxSection::InvestmentSales),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_MATCHING_ORDINARY_HOMESALE_AND_SALE_DATE_BEFORE_CALCULA_B0801AE7",
                "Confirm the matching ordinary HomeSale and sale date before calculating Form 8828.",
            );
            crate::require_localized!(
                result,
                sale_date_matches,
                &crate::interview::income_issue_context(entry).alternatives(&[
                    crate::TaxSection::OtherTaxes,
                    crate::TaxSection::InvestmentSales,
                ]),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_MATCHING_ORDINARY_HOMESALE_AND_SALE_DATE_BEFORE_CALCULA_B0801AE7",
                "Confirm the matching ordinary HomeSale and sale date before calculating Form 8828.",
            );
            if !home.ordinary || !sale_date_matches {
                return 0;
            }
            let sale_gain = home.price - home.expenses - home.basis;
            // No: the share is not subtracted, which keeps the
            // larger recapture gain.
            let share = qsml_gain_share.as_ref().filter(|share| share.qualified_and_distinct);

            let gain = if let Some(share) = share {
                let paid_share_is_valid = share.paid_on >= facts.disposed_on
                    && sale_gain > 0
                    && share.paid_gain_share <= sale_gain;
                crate::require_localized!(
                    result,
                    paid_share_is_valid,
                    &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
                    "Mortgage subsidy recapture",
                    "ISSUE_CONFIRM_A_QUALIFIED_SUBORDINATE_MORTGAGE_LOAN_GOVERNMENTAL_LENDER_W_65400021",
                    "Confirm a qualified subordinate mortgage-loan governmental lender was actually paid at or after the sale, the HomeSale expenses exclude this payment, and the paid share does not exceed the sale gain.",
                );
                if !paid_share_is_valid {
                    return 0;
                }
                line(
                    result,
                    "mortgage_subsidy_qsml_paid_gain_share",
                    dollars(share.paid_gain_share),
                    &["credits.mortgage_subsidies", "income.home_sale"],
                    SOURCE,
                    msg!("EXPLAIN_MORTGAGE_SUBSIDY_QSML_PAID_GAIN_SHARE"),
                );
                dollars(sale_gain - share.paid_gain_share)
            } else {
                dollars(sale_gain)
            };
            let included_gain = if gain >= 0 {
                let gain_line = format!("home_{home_sale_income_id}_taxable_gain");
                let Some(line) = result.lines.iter().find(|line| line.id == gain_line) else {
                    crate::issue_localized!(
                        result,
                        &crate::interview::income_issue_context(entry)
                            .corrective(crate::TaxSection::InvestmentSales),
                        "Mortgage subsidy recapture",
                        "ISSUE_THE_MATCHING_HOME_SALE_TAXABLE_GAIN_MUST_BE_CALCULATED_BEFORE_FORM_1C4CCE73",
                        "The matching home-sale taxable gain must be calculated before Form 8828 modified income.",
                    );
                    return 0;
                };
                dollars(line.amount)
            } else {
                0
            };
            (gain, included_gain)
        }
        MortgageSubsidyTransfer::GiftToOther {
            fair_market_value,
            adjusted_basis,
            no_consideration_or_debt_relief,
            ..
        } => {
            crate::require_localized!(
                result,
                *no_consideration_or_debt_relief,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes).manual(),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THIS_WAS_A_PURE_NONSPOUSE_GIFT_WITH_NO_CONSIDERATION_MORTGA_B1961919",
                "Confirm this was a pure nonspouse gift with no consideration, mortgage assumption or other debt relief to the donor; a mixed sale and gift needs coordinated recognized-income treatment.",
            );
            if !*no_consideration_or_debt_relief {
                return 0;
            }
            (dollars(fair_market_value - adjusted_basis), 0)
        }
    };
    let disposition_inputs: &[&str] = match transfer {
        MortgageSubsidyTransfer::Sale { .. } => &["credits.mortgage_subsidies", "income.home_sale"],
        MortgageSubsidyTransfer::GiftToOther { .. } => &["credits.mortgage_subsidies"],
    };
    let result_inputs: &[&str] = match transfer {
        MortgageSubsidyTransfer::Sale { .. } => {
            &["credits.mortgage_subsidies", "income.home_sale", "agi"]
        }
        MortgageSubsidyTransfer::GiftToOther { .. } => &["credits.mortgage_subsidies", "agi"],
    };
    line(
        result,
        "mortgage_subsidy_disposition_gain",
        gain,
        disposition_inputs,
        SOURCE,
        msg!("EXPLAIN_MORTGAGE_SUBSIDY_DISPOSITION_GAIN"),
    );
    // Source facts remain required even when the nine-year period removes recapture.
    if facts.disposed_on >= facts.original_closed_on + 90000 {
        line(
            result,
            "mortgage_subsidy_recapture_tax",
            0,
            disposition_inputs,
            SOURCE,
            msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_6"),
        );
        return 0;
    }
    if gain < 0 {
        line(
            result,
            "mortgage_subsidy_recapture_tax",
            0,
            &["mortgage_subsidy_disposition_gain"],
            SOURCE,
            msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_7"),
        );
        return 0;
    }
    let modified_income = agi + dollars(tax_exempt_interest) - included_gain;
    let adjusted_income = dollars(facts.adjusted_qualifying_income);
    let excess_income = phase::floor(modified_income - adjusted_income, 0);
    let income_percentage = fraction(phase::cap(excess_income, [5000]), 100, 5000);
    let subsidized_amount = dollars(fraction(facts.highest_subsidized_principal, 625, 10000));
    let holding_percentage = match facts.payoff {
        MortgageSubsidyPayoff::AtDispositionOrReissued => i64::from(facts.holding_percentage),
        MortgageSubsidyPayoff::Earlier { repaid_on }
            if repaid_on <= facts.original_closed_on + 40000 =>
        {
            let repayment_years = rounded_up_years(facts.original_closed_on, repaid_on);
            let post_repayment_years = rounded_up_years(repaid_on, facts.disposed_on);
            let repayment_percentage = (repayment_years * 20).min(80);
            let post_repayment_percentage = (120 - post_repayment_years * 20).clamp(0, 100);
            fraction(repayment_percentage, post_repayment_percentage, 100)
        }
        MortgageSubsidyPayoff::Earlier { .. } => i64::from(facts.holding_percentage),
        MortgageSubsidyPayoff::Unreviewed => {
            crate::issue_localized!(
                result,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
                "Mortgage subsidy recapture",
                "ISSUE_CONFIRM_THE_ORIGINAL_SUBSIDIZED_LOAN_S_FULL_PAYOFF_OR_QUALIFYING_RE_77D8D2D2",
                "Confirm the original subsidized loan's full-payoff or qualifying reissue history for Form 8828 line 20.",
            );
            return 0;
        }
    };
    let holding_amount = fraction(subsidized_amount, holding_percentage, 100);
    let amount = fraction(holding_amount, income_percentage, 100);
    let tax = phase::cap(fraction(phase::floor(gain, 0), 50, 100), [amount]);
    emit_lines(
        result,
        &mut (),
        [
            (
                "mortgage_subsidy_modified_income",
                modified_income,
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_MODIFIED_INCOME"),
            ),
            (
                "mortgage_subsidy_adjusted_qualifying_income",
                adjusted_income,
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_ADJUSTED_QUALIFYING_INCOME"),
            ),
            ("mortgage_subsidy_amount", subsidized_amount, msg!("EXPLAIN_MORTGAGE_SUBSIDY_AMOUNT")),
            (
                "mortgage_subsidy_recapture_tax",
                tax,
                msg!("EXPLAIN_MORTGAGE_SUBSIDY_RECAPTURE_TAX_8"),
            ),
        ],
        |(id, value, description), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(value),
                result_inputs,
                SOURCE,
                description,
            ))
        },
    );
    result.review_notes.push(crate::federal_model::ReviewNote::localized(
        #[cfg(test)] format!(
            "Form 8828 income percentage: {income_percentage}%; applied holding-period percentage: {holding_percentage}% (issuer table or early-repayment worksheet). The on-screen recapture calculation uses the issuer notice; the Form 8828 attachment is not prepared."
        ),
        crate::Message::new(
            "NOTE_FORM_8828_INCOME_AND_HOLDING_PERCENTAGES",
            alloc::vec![
                crate::MessageArg::Number(income_percentage),
                crate::MessageArg::Number(holding_percentage),
            ],
        ),
    ));
    tax
}
