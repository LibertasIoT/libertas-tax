// Coverage: Form 982 / 1099-C / 1099-A — Supported (unit T3-982, 2026-09-27; audited against
//   Form 982 (Rev. March 2018), i982 (Rev. December 2021, the current 2026 draft page), Pub. 4681
//   (2025), i1099ac (Rev. April 2025), f1099a/f1099c; OBBBA (P.L. 119-21) did not extend the
//   qualified principal residence exclusion, so line 1e applies in 2026 only to arrangements
//   made in writing before 2026).
//   Lines: Part I 1a (chapter 12/13), 1b (insolvency, limited to liabilities over asset value),
//   1c (qualified farm debt after insolvency, limited to adjusted tax attributes plus the basis
//   of qualified property held January 1, 2027), 1d (qualified real property business debt after
//   insolvency, limited to principal over the securing property's value net of other qualified
//   debt and to depreciable real property basis), 2. Part II after the 2026 tax, both tax
//   systems: 4 (buildings, securing property first), 5 (section 108(b)(5) election, Treas. Reg.
//   1.1017-1(c) order), 6 NOL (current, then prior by year), 7 general business credit (Form
//   8846 carryforwards, 33 1/3 cents), 8 minimum tax credit (33 1/3 cents), 9 capital loss
//   carryover, 10a basis in the 1.1017-1(a) category order with the 1017(b)(2) liability limit,
//   11a-c farm qualified property, 12 passive loss carryover (rental_suspended_loss), 13 foreign
//   tax credit carryovers (33 1/3 cents); tagged Form 982 lines debt_<id>_form982_line<n>;
//   per-property January 1, 2027 basis after reduction. Taxable personal 1099-C income (Schedule
//   1 line 8c), student-loan, gift, seller price adjustment and health-loan exceptions as before.
//   Unit B6b (2026-10-09): section 108(f)(4) health-service loan repayment is the Canceled debt
//   page's Student loan repaid by a health-service program kind (HealthLoanAssistance, one for
//   each NHSC or state program): confirmed Yes is excluded and its interest comes out of the
//   1098-E interest before the $2,500 cap and phase-out; No is income with
//   WARN_HEALTH_LOAN_ASSISTANCE_ASSUMED_TAXABLE and the interest stays deductible; program-paid
//   interest above the 1098-E total is refused with the Student loan page (src/tests/income/relief_income.rs
//   fixture B, src/tests/business/health_accounts.rs health_loan_assistance_excludes_payment_and_removes_
//   interest_before_cap_and_phaseout).
//   1099-A and 1099-C box 7: amount realized (recourse: smaller of value and debt; nonrecourse:
//   whole debt); personal-use loss not deductible, personal/investment gain or investment loss a
//   Form 8949 sale, business/farm/rental property matched to its own Form 4797 sale price. Unit B2
//   (2026-10-09): a Form 1099-A for a crop forfeited to the Commodity Credit Corporation names
//   the forfeited lot of a farm or farm rental (PropertyAcquisition.crop, the Acquisition1099A
//   page's AcquiredProperty::CccCrop chosen from the ccc_lots catalog; key = the farm's identity
//   x 1000 + the lot's position): the lot carries the forfeiture (Schedule F lines 5a-5c, Form
//   4835 lines 4a-4c) and the form is its information return, matched exactly instead of by
//   amount; the lot must be forfeited in 2026, have the box 2 balance as its principal and belong
//   to the same owner (ISSUE_A_FORM_1099_A_FOR_A_CROP_MUST_NAME_A_CCC_LOAN_FORFEITED_IN_CURRENT_YEAR_
//   WITH_THE_SAME_PRINCIPAL). Tests: src/tests/home/business.rs
//   ccc_1099a_forfeiture_is_counted_once_with_or_without_the_section_77_election;
//   src/tests/farm/farm_rental.rs a_form_1099a_names_its_forfeited_lot_exactly_even_when_two_lots_share_
//   a_principal, the_1099_a_and_1099_g_pages_keep_their_chosen_loan_and_farm.
//   A main home foreclosure with a gain is the home sale of the same price on the Sale of your main
//   home page (section 121 exclusion there). Refused by name: 1e/10b (pre-2026 written arrangements), line 3 election, chapter 7/11
//   estates, taxable business debt (Schedule C line 6, Schedule E line 3, Schedule F line 8, Form
//   4835 line 6) incl. any farm or real property debt above its limit, nonrecourse 1099-C,
//   abandonment/partial/mixed-use transfers, multiple
//   discharges per filer (1.1017-1(b)(2)), joint-return carryovers (1.108-7(d)), partial
//   reduction of mixed short/long capital carryovers, excess business loss carryforward, credit
//   or foreign-tax carrybacks reached by the reduction, minimum tax credit from 2026 AMT (Form
//   8801 for 2027), unlisted business/rental property. Part III (corporations) does not apply to
//   an individual return. The 1c limit is figured with the farm debt excluded (the remainder is
//   refused, not taxed). STORAGE_FORMAT 71.
//   Evidence: debt.rs reduce_attributes/event_reduction/apply_general, transfer_sale;
//   debt_page.rs; T/business.rs qualified_farm_debt_*, qualified_real_property_business_debt_*,
//   depreciable_first_election_precedes_basis_of_personal_property,
//   passive_loss_carryover_is_reduced_after_basis,
//   joint_carryovers_and_partial_mixed_capital_losses_are_refused_by_name,
//   foreclosed_personal_car_loss_is_not_deductible_and_investment_land_gain_is_taxed,
//   main_home_foreclosure_gain_boundary_and_business_property_needs_its_sale,
//   form_1099c_box_7_property_nonrecourse_and_unsupported_situations,
//   insolvency_rejects_unknown_snapshots_multiple_events_and_reduces_capital_loss; T/credits.rs
//   minimum_tax_carryforward_is_reduced_a_third_of_each_excluded_dollar; T/foreign.rs
//   foreign_retained_credit_balances_are_reduced_last_by_excluded_debt; T/investment.rs
//   bankruptcy_reduces_the_calculated_capital_loss_carryover; T/interview.rs canceled_debt_*;
//   form map tax-forms/form-982.md.

// Debt cancellation and secured-property transfers: IRC 61(a)(11), 102, 108,
// 1001, 1017; Pub. 4681; Form 982 (Rev. March 2018) with its instructions.
use crate::federal_rules::{issue_message, valid_birth};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{self, Exact};
use crate::kernel::ratio::{dollars, fraction};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{line, line_cents, push_line};
use crate::{
    FederalDraft, FederalIncome, FederalResult, FormLine, GainTerm, Message, MessageArg, Owner,
    TaxForm,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

const MAX: i64 = 100000000000;
fn money(v: i64) -> bool {
    (0..=MAX).contains(&v)
}
fn date_current_year(date: u32) -> bool {
    valid_birth(date) && date / 10000 == 2026
}

/// Debt canceled in 2026
/// One actual cancellation allocated to this filer. Reconcile the creditor's discharge and any Form 1099-C; a form is not an additional amount, and continued collection or a disputed discharge needs resolution. The absence of a form does not make canceled debt tax-free.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonalDebtCancellation {
    /// Interest included in the debt discharged — Form 1099-C box 3
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The printed box 3 interest is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: Option<i64>,
    /// Description of debt — Form 1099-C box 4
    pub debt_description: String,
    /// Identifiable event code — Form 1099-C box 6
    pub event_code: String,
    /// Printed fair market value — Form 1099-C box 7
    /// This value alone does not establish a transfer of property.
    /// [NullText]
    /// Box 7 is blank
    /// [SetText]
    /// The printed box 7 value is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub box7_fmv: Option<i64>,
    /// Actual cancellation date
    #[libertas_date_only]
    pub date: u32,
    /// This filer's canceled principal, interest and fees
    /// Include nondeductible personal interest even if separately shown in Form 1099-C box 3. Use only this filer's allocated discharge; if joint borrowers each received a form showing the whole debt, do not duplicate the whole amount. Exclude debt paid in the settlement and amounts still owed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Cancellation treatment
    /// The canceled amount is this filer's actual discharge and the selected treatment applies. The gift choice requires a genuine gift or inheritance, not a negotiated commercial write-off or payment for services.
    /// [NullText]
    /// The cancellation treatment is not established
    /// [SetText]
    /// The cancellation treatment is
    pub treatment: Option<PersonalDebtTreatment>,
    /// Personally liable for repayment
    /// Form 1099-C box 5. A debt secured only by property, with no personal liability, is nonrecourse.
    #[libertas_default(true)]
    pub personally_liable: bool,
    /// Property the creditor took
    /// The actual personal disposition, retained independently of the printed box 7 value.
    #[libertas_default($null)]
    pub transfer: Option<PropertyTransfer>,
}

/// Cancellation treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PersonalDebtTreatment {
    /// Insolvent immediately before cancellation
    /// Not a bankruptcy discharge. Use the full before-discharge assets and liabilities, including exempt assets. The excluded part reduces the tax attributes the return carries in the Form 982 order.
    Insolvent { data: PersonalInsolvency },
    /// Chapter 12 or 13 discharge
    /// Court-approved discharge while under title 11 jurisdiction. Chapters 7 and 11 and conversions involving a separate taxable estate need separate treatment.
    Bankruptcy { data: PersonalBankruptcy },
    /// Seller reduces the purchase price of personal-use property
    /// A solvent buyer's original seller-financed purchase debt, reduced by that seller while the buyer still owns the property. Not a bank refinance, assigned debt, interest write-off, bankruptcy or business/investment property. Use the income record label to identify the property. Only purchase principal is reduced; the property remains personal-use, with no other unentered basis change. Report a later 2026 sale here. Enter its basis after any earlier price reduction, without repeating a reduction. Home sales, business-use changes, installment sales and other special dispositions require their coordinated property worksheet.
    PriceReduction {
        /// Property's adjusted basis immediately before the reduction
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Fair market value of all this filer's assets immediately before reduction
        /// Include exempt assets and retirement accounts, not just this property.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        assets: i64,
        /// All this filer's liabilities immediately before reduction
        /// Include the seller debt before the reduction and all other liabilities.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        liabilities: i64,
        /// Later sale in 2026
        /// Leave absent if still owned at year end. Enter the sale here only, not again as an income-document sale.
        sale: Option<DebtPropertySale>,
    },
    /// Qualifying student-loan discharge
    StudentDischarge {
        /// Reason stated in the final discharge determination
        /// Retain the servicer's final determination and loan records. This is an actual qualifying discharge allocated to this filer, not a pending application, repayment plan or payment by an employer. Forgiven interest is not interest paid for a deduction. The broad 2021–2025 exclusion does not apply to an ordinary 2026 income-driven repayment discharge.
        reason: StudentDischargeReason,
    },
    /// Taxable cancellation without an exception or exclusion
    /// The borrower was solvent immediately before discharge and no bankruptcy or other exclusion applies. Ordinary personal borrowing produces income when forgiven; the canceled interest is not deductible merely because the creditor reports it separately.
    Taxable,
    /// Genuine gift, bequest or inheritance
    /// Retain the creditor's donative agreement or estate records. This exception does not require Form 982 tax-attribute reductions and does not create a deduction for the recipient.
    GiftOrBequest,
    /// Qualified farm debt
    /// Debt incurred directly in operating your farming business, canceled by a qualified lender. The insolvency exclusion applies first when you were insolvent.
    FarmDebt { data: FarmDebtExclusion },
    /// Qualified real property business debt
    /// Debt secured by real property used in your trade or business that you elect to exclude. The insolvency exclusion applies first when you were insolvent.
    RealPropertyDebt { data: RealPropertyDebtExclusion },
    /// A situation this return does not calculate
    Unsupported {
        /// Situation
        situation: UnsupportedDebt,
    },
}

/// Situation this return does not calculate
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum UnsupportedDebt {
    /// Canceled business, farm or rental debt that is income
    /// Solvent, outside bankruptcy and without the farm or real property business exclusion.
    BusinessDebt,
    /// Main-home mortgage under a written arrangement made before 2026
    /// The qualified principal residence exclusion.
    PrincipalResidence,
    /// Chapter 7 or chapter 11 bankruptcy
    /// Including a case converted from either chapter.
    LiquidationBankruptcy,
    /// Treat real property held for sale to customers as depreciable
    DealerPropertyElection,
    /// Something else
    /// Another exclusion, a debt paid by someone else as a benefit, or a disputed or reinstated debt.
    Other,
}
impl UnsupportedDebt {
    #[cfg(test)]
    fn refusal(self) -> &'static str {
        self.refusal_info().1
    }
    fn refusal_key(self) -> &'static str {
        self.refusal_info().0
    }
    fn refusal_info(self) -> (&'static str, &'static str) {
        match self {
            Self::BusinessDebt => (
                "ISSUE_DEBT_BUSINESS_DEBT",
                crate::fixture_english!(
                    "Canceled business debt included in income (Schedule C line 6, Schedule E line 3, Schedule F line 8 or Form 4835 line 6) is not calculated."
                ),
            ),
            Self::PrincipalResidence => (
                "ISSUE_DEBT_PRINCIPAL_RESIDENCE",
                crate::fixture_english!(
                    "Qualified principal residence debt canceled under a written arrangement made before 2026 (Form 982 lines 1e and 10b) is not calculated."
                ),
            ),
            Self::LiquidationBankruptcy => (
                "ISSUE_DEBT_LIQUIDATION_BANKRUPTCY",
                crate::fixture_english!(
                    "Debt discharged in a chapter 7 or chapter 11 case, with its separate bankruptcy estate (Form 982 line 1a), is not calculated."
                ),
            ),
            Self::DealerPropertyElection => (
                "ISSUE_DEBT_DEALER_PROPERTY_ELECTION",
                crate::fixture_english!(
                    "The election to treat real property held for sale to customers as depreciable property (Form 982 line 3) is not calculated."
                ),
            ),
            Self::Other => (
                "ISSUE_DEBT_OTHER",
                crate::fixture_english!(
                    "This canceled-debt situation (Pub. 4681) is not calculated; the canceled amount is not treated as tax-free."
                ),
            ),
        }
    }
}

/// Property the creditor took
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyTransfer {
    /// What the property was
    /// [NullText]
    /// The property is not established
    /// [SetText]
    /// The property was
    pub property: Option<TransferredProperty>,
}

/// What the property was
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TransferredProperty {
    /// Your main home
    /// A loss is not deductible. A gain is a home sale: enter it on the Sale of your main home page, with the amount realized that this entry shows as the price.
    MainHome {
        /// Adjusted basis
        /// What you paid plus improvements, less any casualty losses or earlier basis reductions.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
    },
    /// Other personal-use property
    /// A car, a second home or furnishings. A loss is not deductible; a gain is taxable.
    PersonalUse {
        /// Adjusted basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Holding period
        #[libertas_default(Long)]
        term: GainTerm,
        /// Collectible
        /// Art, antiques, gems, coins and similar items.
        #[libertas_default(false)]
        collectible: bool,
    },
    /// Investment property
    /// Land or other property held for investment, not used in a business or rental.
    Investment {
        /// Adjusted basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Holding period
        #[libertas_default(Long)]
        term: GainTerm,
        /// Collectible
        /// Art, antiques, gems, coins and similar items.
        #[libertas_default(false)]
        collectible: bool,
    },
    /// Business, farm or rental property
    /// Enter the property's sale on its business, farm or rental page, with the amount realized that this entry shows as the sale price. A crop forfeited to the Commodity Credit Corporation is chosen as that loan instead.
    Business,
    /// Something else
    /// Property you abandoned that the lender has not taken, a partial transfer, or property used partly for business.
    Other,
}
impl TransferredProperty {
    fn valid(&self) -> bool {
        match self {
            Self::MainHome { basis }
            | Self::PersonalUse { basis, .. }
            | Self::Investment { basis, .. } => money(*basis),
            Self::Business | Self::Other => true,
        }
    }
}
impl PropertyTransfer {
    fn valid(&self) -> bool {
        self.valid_with_mode(true)
    }
    fn valid_with_mode(&self, require_answers: bool) -> bool {
        (!require_answers || self.property.is_some())
            && self.property.as_ref().is_none_or(TransferredProperty::valid)
    }
}

// The pages where this owner's business, farm or rental property sale is
// entered: the owner's own activities, or all three when there are none.
pub(crate) fn business_sale_pages(d: &FederalDraft, id: i64) -> Vec<crate::TaxSection> {
    let owner = d.income.iter().find(|entry| entry.id == id).map(|entry| entry.owner);
    let mut sections = Vec::new();
    for entry in &d.income {
        if owner.is_some_and(|owner| owner != entry.owner) {
            continue;
        }
        let section = match &entry.income {
            FederalIncome::Business { data }
                if matches!(data.activity, crate::BusinessActivity::Farming { .. }) =>
            {
                crate::TaxSection::Farming
            }
            FederalIncome::FarmRental { .. } => crate::TaxSection::Farming,
            FederalIncome::Business { .. } => crate::TaxSection::SelfEmployment,
            FederalIncome::Rental { .. } => crate::TaxSection::Rentals,
            _ => continue,
        };
        if !sections.contains(&section) {
            sections.push(section);
        }
    }
    if sections.is_empty() {
        sections.extend([
            crate::TaxSection::SelfEmployment,
            crate::TaxSection::Farming,
            crate::TaxSection::Rentals,
        ]);
    }
    sections
}

// Sale prices of the business, farm and rental property sold in 2026, the
// Form 4797 dispositions a business property transfer must match.
pub(crate) fn business_dispositions(d: &FederalDraft) -> Vec<i64> {
    let mut prices = Vec::new();
    for e in &d.income {
        match &e.income {
            FederalIncome::Business { data } => {
                for asset in &data.assets {
                    if let crate::AssetDisposal::Disposed {
                        how: crate::DisposalKind::Sold { proceeds, .. },
                        ..
                    } = &asset.disposal
                    {
                        prices.push(*proceeds);
                    }
                }
            }
            FederalIncome::Rental { data } => {
                if let crate::RentalSale::Sold { price, .. } = &data.sale {
                    prices.push(*price);
                }
            }
            _ => (),
        }
    }
    prices
}

// Prices of the main homes sold in 2026, the sales a main home transfer's gain
// must match.
fn home_sale_prices(d: &FederalDraft) -> Vec<i64> {
    d.income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::HomeSale { data } if !data.other_property() => {
                Some(crate::HomeSale::from(data).price)
            }
            _ => None,
        })
        .collect()
}

// The forfeited CCC lot a Form 1099-A names, with the note that says where
// its forfeiture is counted. The lot must belong to the same owner, be
// forfeited in 2026 and have the box 2 balance as its principal.
fn crop_forfeiture(
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    key: i64,
    realized: i64,
    #[cfg(test)] section: &str,
) {
    let owner = d.income.iter().find(|entry| entry.id == id).map(|entry| entry.owner);
    let lot = crate::farm::ccc_lot(d, key).filter(|(source, _)| owner == Some(source.entry.owner));
    let included = lot.as_ref().and_then(|(source, index)| {
        let ccc = source.farm.ccc.as_ref()?;
        let index = *index;
        (ccc.lots.get(index)?.principal == realized)
            .then(|| ccc.forfeited_current_year(index))
            .flatten()
    });
    let Some(included) = included else {
        crate::issue_localized!(
            r,
            &crate::IssueContext::income_id(id),
            section,
            "ISSUE_A_FORM_1099_A_FOR_A_CROP_MUST_NAME_A_CCC_LOAN_FORFEITED_IN_CURRENT_YEAR_WITH_THE_SAME_PRINCIPAL",
            "A Form 1099-A for a crop forfeited to the Commodity Credit Corporation must name a loan of your farm or farm rental that was forfeited in 2026 and whose principal equals box 2.",
        );
        return;
    };
    let amount = dollars(realized);
    let rental = lot.as_ref().is_some_and(|(source, _)| source.rental);
    r.review_notes
        .push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)]
            match (included, rental) {
                (true, true) => alloc::format!(
                    "Form 1099-A entry {id}: this forfeited CCC loan was reported as income under the section 77 election, so the forfeiture is not counted again; your basis in the commodity equals the ${amount} reported (Form 4835 lines 4a-4c, Pub. 225 chapter 3)."
                ),
                (true, false) => alloc::format!(
                    "Form 1099-A entry {id}: this forfeited CCC loan was reported as income under the section 77 election, so the forfeiture is not counted again; your basis in the commodity equals the ${amount} reported (Schedule F lines 5a-5c, Pub. 225 chapter 3)."
                ),
                (false, true) => alloc::format!(
                    "Form 1099-A entry {id}: you did not elect to report this CCC loan as income, so the forfeiture is the sale and the ${amount} is income (Form 4835 lines 4b and 4c, Pub. 225 chapter 3); it is counted once, from the CCC loan record."
                ),
                (false, false) => alloc::format!(
                    "Form 1099-A entry {id}: you did not elect to report this CCC loan as income, so the forfeiture is the sale and the ${amount} is income (Schedule F lines 5b and 5c, Pub. 225 chapter 3); it is counted once, from the CCC loan record."
                ),
            },
            crate::Message::new(
                match (included, rental) {
                    (true, false) => "NOTE_CCC_FORFEITURE_ALREADY_REPORTED",
                    (false, false) => "NOTE_CCC_FORFEITURE_IS_THE_SALE",
                    (true, true) => "NOTE_CCC_FORFEITURE_ALREADY_REPORTED_RENTAL",
                    (false, true) => "NOTE_CCC_FORFEITURE_IS_THE_SALE_RENTAL",
                },
                alloc::vec![
                    crate::MessageArg::Number(id),
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: amount,
                    },
                ],
            ),
        ));
}

// The property side of a foreclosure, repossession or deed in lieu: the
// amount realized against the property's basis (Pub. 4681 chapter 2). A main
// home or other personal-use loss is not deductible; a personal or investment
// gain or investment loss is a Form 8949 sale; business property is the
// asset's own Form 4797 sale.
#[allow(clippy::too_many_arguments)]
fn transfer_sale(
    property: &TransferredProperty,
    realized: i64,
    d: &FederalDraft,
    r: &mut FederalResult,
    id: i64,
    prefix: &str,
    crop: Option<i64>,
    #[cfg(test)] section: &str,
) -> Option<crate::FederalSale> {
    line_cents(
        r,
        &alloc::format!("{prefix}_amount_realized"),
        realized,
        &["income.personal_debt"],
        "https://www.irs.gov/publications/p4681",
        msg!("EXPLAIN_N_AMOUNT_REALIZED"),
    );
    // A CCC loan is nonrecourse, so the whole principal is the amount
    // realized. The Form 1099-A names the forfeited lot of a farm or farm
    // rental; that lot already carries the forfeiture (Schedule F lines
    // 5a-5c, Form 4835 lines 4a-4c; Pub. 225 chapter 3, "CCC loans"): the
    // Form 1099-A is its information return, not a second sale.
    if let Some(key) = crop {
        crop_forfeiture(
            d,
            r,
            id,
            key,
            realized,
            #[cfg(test)]
            section,
        );
        return None;
    }
    let sale = |basis: i64, term: GainTerm, collectible: bool, use_kind| crate::FederalSale {
        amt: crate::SaleAmt::SameAsRegular,
        term,
        collectible,
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        proceeds: realized,
        basis: Some(basis),
        withholding: 0,
        use_kind,
        special: crate::SaleSpecial::NoneOfThese,
        adjustment: crate::SaleAdjustment::None,
        basis_reporting: crate::BasisReporting::NoForm1099B,
    };
    match property {
        TransferredProperty::MainHome { basis } => {
            // A gain is a home sale (Pub. 4681): it goes on the Sale of your main
            // home page, with the amount realized shown here as its price, so the
            // section 121 exclusion applies there.
            crate::require_localized!(
                r,
                realized <= *basis || home_sale_prices(d).contains(&realized),
                &crate::IssueContext::income_id(id)
                    .alternatives(&[crate::TaxSection::InvestmentSales]),
                section,
                "ISSUE_A_MAIN_HOME_FORECLOSURE_WITH_A_GAIN_NEEDS_THE_HOME_SALE_EXCLUSION_S_02E3DE73",
                "A main home foreclosure with a gain is entered on the Sale of your main home page, with the amount realized shown here as its price, so the home-sale exclusion applies (section 121; Pub. 4681).",
            );
            line_cents(
                r,
                &alloc::format!("{prefix}_nondeductible_loss"),
                phase::floor(*basis - realized, 0),
                &["income.personal_debt"],
                "https://www.irs.gov/publications/p4681",
                msg!("EXPLAIN_N_NONDEDUCTIBLE_LOSS"),
            );
            None
        }
        TransferredProperty::PersonalUse { basis, term, collectible } => {
            Some(sale(*basis, *term, *collectible, crate::SaleUse::Personal))
        }
        TransferredProperty::Investment { basis, term, collectible } => {
            Some(sale(*basis, *term, *collectible, crate::SaleUse::Investment { wash_sale: 0 }))
        }
        TransferredProperty::Business => {
            crate::federal_rules::require_message(
                r,
                business_dispositions(d).contains(&realized),
                &crate::IssueContext::income_id(id).alternatives(&business_sale_pages(d, id)),
                #[cfg(test)]
                section,
                #[cfg(test)]
                alloc::format!(
                    "Form 4797: enter this business, farm or rental property's 2026 sale on its own page with ${}.{:02} (the amount realized) as the sale price.",
                    realized / 100,
                    realized % 100
                ),
                crate::Message::new(
                    "ISSUE_TRANSFERRED_PROPERTY_SALE_PRICE",
                    alloc::vec![crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 2,
                        minor_units: realized,
                    }],
                ),
            );
            None
        }
        TransferredProperty::Other => {
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::Documents),
                section,
                "ISSUE_ABANDONED_PROPERTY_THE_LENDER_HAS_NOT_TAKEN_A_PARTIAL_TRANSFER_OR_M_1D9AC0DD",
                "Abandoned property the lender has not taken, a partial transfer or mixed-use property (Pub. 4681 chapters 2 and 3; Form 4797 or Form 8949) is not calculated.",
            );
            None
        }
    }
}

impl PersonalDebtCancellation {
    pub(crate) fn valid(&self) -> bool {
        self.valid_with_mode(true)
    }
    pub(crate) fn valid_with_mode(&self, require_answers: bool) -> bool {
        date_current_year(self.date)
            && money(self.amount)
            && self.interest.is_none_or(money)
            && self.box7_fmv.is_none_or(money)
            && (!require_answers || self.treatment.is_some())
            && (!require_answers || self.box7_fmv.is_none() || self.transfer.is_some())
            && (!require_answers || self.transfer.is_none() || self.box7_fmv.is_some())
            && self.transfer.as_ref().is_none_or(|transfer| {
                if require_answers { transfer.valid() } else { transfer.valid_with_mode(false) }
            })
            && self.treatment.as_ref().is_none_or(|treatment| match treatment {
                PersonalDebtTreatment::Insolvent { data } => data.valid(),
                PersonalDebtTreatment::Bankruptcy { data } => data.valid(),
                PersonalDebtTreatment::FarmDebt { data } => data.valid(),
                PersonalDebtTreatment::RealPropertyDebt { data } => data.valid(),
                PersonalDebtTreatment::PriceReduction {
                    basis, assets, liabilities, sale, ..
                } => {
                    [*basis, *assets, *liabilities].into_iter().all(money)
                        && sale.as_ref().is_none_or(|v| {
                            v.valid(if require_answers { self.date } else { v.date })
                        })
                }
                _ => true,
            })
    }
    // The canceled amount this record excludes under Form 982 Part I, before
    // the farm and real property limits that follow the current-year tax.
    fn excluded_before_limits(&self) -> Option<i64> {
        Some(match self.treatment.as_ref()? {
            PersonalDebtTreatment::Insolvent { data } => data.excluded(self.amount),
            PersonalDebtTreatment::Bankruptcy { .. }
            | PersonalDebtTreatment::FarmDebt { .. }
            | PersonalDebtTreatment::RealPropertyDebt { .. } => self.amount,
            _ => return None,
        })
    }
    pub(crate) fn taxable(
        &self,
        r: &mut FederalResult,
        id: i64,
        valid_ssn: bool,
        owner: Owner,
        #[cfg(test)] section: &str,
    ) -> i64 {
        // The availability owner withholds a return with missing personal records.
        let Some(treatment) = self.treatment.as_ref() else {
            return 0;
        };
        crate::require_localized!(
            r,
            self.personally_liable,
            &crate::IssueContext::Section(crate::TaxSection::Documents),
            section,
            "ISSUE_FORM_1099_C_BOX_5_IS_NOT_CHECKED_A_NONRECOURSE_DEBT_CANCELED_WITH_I_7768A375",
            "Form 1099-C box 5 is not checked: a nonrecourse debt canceled with its property is part of the amount realized, not canceled-debt income. Enter the property on a Form 1099-A entry with the whole debt as box 2. A nonrecourse debt reduced without a transfer (Pub. 4681) is not calculated.",
        );
        let taxable = match treatment {
            PersonalDebtTreatment::Insolvent { data } => {
                let taxable = self.amount - data.excluded(self.amount);
                crate::require_localized!(
                    r,
                    !data.business_debt || taxable == 0,
                    &crate::IssueContext::Section(crate::TaxSection::Documents),
                    "Insolvency tax attributes",
                    "ISSUE_THE_PART_OF_A_BUSINESS_FARM_OR_RENTAL_DEBT_ABOVE_YOUR_INSOLVENCY_IS_CFEE5ABB",
                    "The part of a business, farm or rental debt above your insolvency is Schedule C line 6, Schedule E line 3, Schedule F line 8 or Form 4835 line 6 income, which is not calculated.",
                );
                if data.business_debt { 0 } else { taxable }
            }
            PersonalDebtTreatment::PriceReduction { basis, assets, liabilities, sale } => {
                crate::require_localized!(
                    r,
                    assets >= liabilities
                        && *liabilities >= self.amount
                        && *basis >= self.amount
                        && (sale.is_none() || self.transfer.is_none()),
                    &crate::IssueContext::Section(crate::TaxSection::Documents),
                    "Seller purchase-price adjustment",
                    "ISSUE_CONFIRM_THE_ORIGINAL_SELLER_S_REDUCTION_OF_PURCHASE_PRINCIPAL_SOLVE_7D3164A4",
                    "Confirm the original seller's reduction of purchase principal, solvency, no bankruptcy or debt transfer, adequate property basis and retained personal ownership or the disclosed ordinary sale. Other cases need their cancellation or disposition worksheet.",
                );
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("price_reduction", self.amount),
                        ("remaining_basis", phase::floor(*basis - self.amount, 0)),
                    ],
                    |(suffix, value), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("debt_{id}_{suffix}"),
                            LineValue::MoneyCents(value),
                            &["income.personal_debt"],
                            "https://www.irs.gov/publications/p4681",
                            msg!("EXPLAIN_DEBT_N_N"),
                        ))
                    },
                );
                0
            }
            // Service terms that do not qualify: the discharge stays taxable.
            PersonalDebtTreatment::StudentDischarge { reason } if !reason.service_eligible() => {
                self.amount
            }
            PersonalDebtTreatment::StudentDischarge { reason } => {
                crate::require_localized!(
                    r,
                    *reason != StudentDischargeReason::DeathOrDisability || valid_ssn,
                    &crate::IssueContext::income_id(id).corrective(if owner == Owner::Taxpayer {
                        crate::TaxSection::People
                    } else {
                        crate::TaxSection::Spouse
                    }),
                    "Student-loan discharge",
                    "ISSUE_CONFIRM_THE_COMPLETED_QUALIFYING_DISCHARGE_AND_LOAN_CATEGORY_DEATH_E03BC664",
                    "Death/disability relief in 2026 requires this filer's employment-authorized SSN issued by the return deadline and included on the return. Missing qualification needs the applicable taxable-debt or other exclusion worksheet; do not assume relief.",
                );
                let safe_harbor = matches!(
                    reason,
                    StudentDischargeReason::SchoolDischarge
                        | StudentDischargeReason::SchoolSettlement
                );
                if safe_harbor {
                    r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!("Student-loan entry {id}: Rev. Proc. 2020-11 relief also prevents discharge-triggered recapture of prior education credits, student-loan interest and tuition deductions. Do not report that recapture solely because of this discharge. Separate cash refunds or other recoveries need their own treatment; retain the final discharge or settlement and eligible-school records."), crate::Message::new("NOTE_STUDENT_LOAN_ENTRY_ID_REV_PROC_2020_11_RELIEF_ALSO_PREVE_F831B367", alloc::vec![crate::MessageArg::Number(id)])));
                }
                line_cents(
                    r,
                    &alloc::format!("debt_{id}_student_exclusion"),
                    self.amount,
                    &["income.personal_debt"],
                    if safe_harbor {
                        "https://www.irs.gov/irb/2020-06_IRB#REV-PROC-2020-11"
                    } else {
                        "https://www.irs.gov/publications/p4681"
                    },
                    msg!("EXPLAIN_DEBT_N_STUDENT_EXCLUSION"),
                );
                0
            }
            PersonalDebtTreatment::Taxable => self.amount,
            PersonalDebtTreatment::Unsupported { situation } => {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Documents),
                    section,
                    situation.refusal_key(),
                    situation.refusal(),
                );
                0
            }
            // Excluded in full here; the attribute stage after the current-year
            // tax refuses any part above the farm or real property limit.
            PersonalDebtTreatment::GiftOrBequest
            | PersonalDebtTreatment::Bankruptcy { .. }
            | PersonalDebtTreatment::FarmDebt { .. }
            | PersonalDebtTreatment::RealPropertyDebt { .. } => 0,
        };
        line(
            r,
            &alloc::format!("debt_cancellation_{id}_taxable"),
            dollars(taxable),
            &["income.personal_debt"],
            "https://www.irs.gov/publications/p4681",
            msg!("EXPLAIN_DEBT_CANCELLATION_N_TAXABLE"),
        );
        taxable
    }
    // The sales this record adds to Form 8949: the later sale after a seller's
    // price adjustment, and an established transfer using the canonical box 7 value.
    pub(crate) fn sales(
        &self,
        d: &FederalDraft,
        r: &mut FederalResult,
        id: i64,
        #[cfg(test)] section: &str,
    ) -> Vec<crate::FederalSale> {
        let mut sales = Vec::new();
        if let Some(sale) = self.sale() {
            sales.push(sale);
        }
        if let Some(transfer) = &self.transfer
            && let Some(property) = &transfer.property
            && let Some(value) = self.box7_fmv
            && let Some(sale) = transfer_sale(
                property,
                value,
                d,
                r,
                id,
                &alloc::format!("debt_{id}_transfer"),
                None,
                #[cfg(test)]
                section,
            )
        {
            sales.push(sale);
        }
        sales
    }
}

/// Property a lender acquired in 2026
/// A foreclosure, repossession or deed in lieu of foreclosure reported on Form 1099-A. When the lender also canceled debt, enter its Form 1099-C separately, without box 7.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PropertyAcquisition {
    /// Description of property — Form 1099-A box 6
    pub description: String,
    /// Date the lender acquired the property
    #[libertas_date_only]
    pub date: u32,
    /// Balance of principal outstanding
    /// Form 1099-A box 2.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Fair market value of the property
    /// Form 1099-A box 4.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Personally liable for repayment
    /// Form 1099-A box 5.
    pub personally_liable: bool,
    /// What the property was
    /// [NullText]
    /// The property is not established
    /// [SetText]
    /// The property was
    pub property: Option<TransferredProperty>,
    /// Forfeited CCC loan
    /// The record identity of the CCC loan lot of a farm or farm rental that this form reports. Set only for a crop; the property is then a business, farm or rental property.
    #[libertas_number(min = 1)]
    #[libertas_default($null)]
    pub crop: Option<i64>,
}
impl PropertyAcquisition {
    pub(crate) fn valid(&self) -> bool {
        self.valid_with_mode(true)
    }
    pub(crate) fn valid_with_mode(&self, require_answers: bool) -> bool {
        date_current_year(self.date)
            && money(self.balance)
            && money(self.value)
            && (!require_answers || self.property.is_some())
            && self.property.as_ref().is_none_or(TransferredProperty::valid)
            && self.crop.is_none_or(|key| {
                key >= 1
                    && match self.property.as_ref() {
                        Some(TransferredProperty::Business) => true,
                        None => !require_answers,
                        _ => false,
                    }
            })
    }
    // Pub. 4681 Table 1-1 and chapter 2: recourse debt realizes the smaller
    // of fair market value and the debt; nonrecourse debt the whole debt.
    pub(crate) fn realized(&self) -> i64 {
        if self.personally_liable { phase::cap(self.value, [self.balance]) } else { self.balance }
    }
    pub(crate) fn sale(
        &self,
        d: &FederalDraft,
        r: &mut FederalResult,
        id: i64,
        #[cfg(test)] section: &str,
    ) -> Option<crate::FederalSale> {
        let property = self.property.as_ref()?;
        if self.personally_liable && self.balance > self.value {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!(
                    "Form 1099-A entry {id}: the ${} of recourse debt above the property's value is not income until the lender cancels it; enter that lender's Form 1099-C when it does.",
                    dollars(self.balance - self.value)
                ),
                crate::Message::new(
                    "NOTE_RECOURSE_DEBT_ABOVE_PROPERTY_VALUE",
                    alloc::vec![
                        crate::MessageArg::Number(id),
                        crate::MessageArg::Currency {
                            currency_code: "USD".into(),
                            scale: 0,
                            minor_units: dollars(self.balance - self.value),
                        },
                    ],
                ),
            ));
        }
        transfer_sale(
            property,
            self.realized(),
            d,
            r,
            id,
            &alloc::format!("acquisition_{id}"),
            self.crop,
            #[cfg(test)]
            section,
        )
    }
}

/// Qualifying student-loan discharge reason
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum StudentDischargeReason {
    /// Other qualifying service-based student-loan cancellation
    Service {
        /// Lender and program category
        lender: StudentServiceLender,
        /// Loan financed attendance at an eligible educational organization
        /// A school with a regular faculty and curriculum and a regularly enrolled student body under section 170(b)(1)(A)(ii). For refinancing, the original loan financed this attendance. No makes the cancellation taxable.
        eligible_school: bool,
        /// Loan terms required a service period in qualifying professions for a broad employer class
        /// The actual cancellation satisfies those original terms, not a later discretionary employer award. A service requirement for just one named employer does not meet the broad-class rule. Your promissory note or program agreement shows it. No makes the cancellation taxable.
        service_terms: bool,
        /// Cancellation pays for services to a lending school or tax-exempt refinancing organization
        /// Include services to the organization funding that school's loan. Choose No for qualifying government employment where the lender is the government itself; that alone is not the prohibited school-service arrangement. Yes makes the cancellation taxable.
        prohibited_services: bool,
    },
    /// Public Service Loan Forgiveness or Temporary Expanded PSLF
    /// Final federal program forgiveness after the required qualifying service and payments, not ordinary income-driven repayment forgiveness or an employer's payment.
    PublicService,
    /// Federal Teacher Loan Forgiveness
    /// Final federal Teacher Loan Forgiveness program approval, not a school canceling its own loan in exchange for services to that school or funding organization.
    Teacher,
    /// Federal Closed School or Borrower Defense discharge
    /// Final Department of Education discharge through the Closed School or Defense to Repayment process for attendance at a nonprofit institution under 20 USC 1001(a)/(b) or proprietary institution under 20 USC 1002(b). Includes qualifying parent borrowing. A school closure, complaint or pending application alone is insufficient.
    SchoolDischarge,
    /// Private school-loan discharge under a legal settlement
    /// Discharged private education debt under a settlement of a legal cause of action alleging unlawful practices by the school or its education lenders, within Rev. Proc. 2020-11. The loan financed attendance at a nonprofit institution under 20 USC 1001(a)/(b) or proprietary institution under 20 USC 1002(b). Not a hardship compromise or ordinary private-loan forgiveness.
    SchoolSettlement,
    /// Student's death or total and permanent disability
    /// Final discharge on this ground under a qualifying government/educational student loan within section 108(f)(2), or a qualifying private education loan under Truth in Lending Act section 140(a)(7). Not a home-equity loan, credit card or ordinary personal loan used for tuition. This filer's qualifying SSN must be included on the return. Do not substitute temporary disability or a pending application.
    DeathOrDisability,
}

/// Lender category for service-based student-loan relief
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum StudentServiceLender {
    /// U.S., state, territory, District of Columbia, agency or political subdivision
    Government,
    /// Tax-exempt public benefit corporation operating a former government hospital
    /// A section 501(c)(3) public benefit corporation that assumed control of a state, county or municipal hospital and whose employees are public employees under state law.
    PublicHospital,
    /// Eligible school lending funds supplied by a qualifying government or hospital entity
    /// A funding agreement with an entity described by either preceding category; not any private school loan.
    FundedSchool,
    /// Eligible school's unmet-needs service program
    /// Encourages students into occupations or areas with unmet needs, with services for or under the direction of government or a tax-exempt section 501(c)(3) organization.
    SchoolProgram,
    /// Educational or tax-exempt organization's qualifying refinancing program
    /// Refinances an education-attendance loan through a program with the same unmet-needs and government/501(c)(3) service direction requirements. Not ordinary bank refinancing.
    RefinancingProgram,
}
impl StudentDischargeReason {
    fn service_eligible(&self) -> bool {
        match self {
            Self::Service { eligible_school, service_terms, prohibited_services, .. } => {
                *eligible_school && *service_terms && !*prohibited_services
            }
            _ => true,
        }
    }
}

/// Ordinary personal-property sale after a seller price adjustment
/// No home exclusion, business use, installment payments, related-party arrangement or other special treatment. The earlier price adjustment supplies the basis; do not enter this sale a second time elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DebtPropertySale {
    /// Sale date after the price adjustment
    #[libertas_date_only]
    pub date: u32,
    /// Holding period from original acquisition
    pub term: crate::GainTerm,
    /// Collectible capital asset
    /// Includes art, antiques, gems and similar collectibles, not self-created work.
    pub collectible: bool,
    /// Net sale proceeds after selling costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Federal income tax withheld on this sale
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}
impl DebtPropertySale {
    fn valid(&self, adjustment_date: u32) -> bool {
        date_current_year(self.date)
            && self.date >= adjustment_date
            && [self.proceeds, self.withholding].into_iter().all(money)
    }
}
impl PersonalDebtCancellation {
    pub(crate) fn sale(&self) -> Option<crate::FederalSale> {
        if let Some(PersonalDebtTreatment::PriceReduction { basis, sale: Some(v), .. }) =
            &self.treatment
        {
            Some(crate::FederalSale {
                digital_asset: false,
                market_discount: 0,
                several_sales: false,
                amt: crate::SaleAmt::SameAsRegular,
                term: v.term,
                collectible: v.collectible,
                proceeds: v.proceeds,
                basis: Some(phase::floor(*basis - self.amount, 0)),
                withholding: v.withholding,
                use_kind: crate::SaleUse::Personal,
                special: crate::SaleSpecial::NoneOfThese,
                adjustment: crate::SaleAdjustment::None,
                basis_reporting: crate::BasisReporting::NoForm1099B,
            })
        } else {
            None
        }
    }
}

/// Insolvency and basis-reduction facts
/// One discharge event per filer. Combine debts canceled in that same event, using one snapshot. Basis reduction occurs in 2027; retain the results for later dispositions and possible ordinary-income recapture.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonalInsolvency {
    /// Total liabilities immediately before cancellation
    /// Include the canceled debt and all other enforceable liabilities under the IRS insolvency worksheet, allocated to this filer. Settlement payments already made are not still owed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub liabilities: i64,
    /// Fair market value of all assets immediately before cancellation
    /// Include everything owned, including cash, retirement accounts, exempt assets and collateral, not just property creditors can seize. Use this filer's share of jointly owned assets. Fair market value is not tax basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub assets: i64,
    /// Total property tax basis plus money immediately after cancellation
    /// Use the separate post-discharge snapshot; it is not the fair market value above. Remaining liabilities are the before-discharge total less canceled debt.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_and_cash_after: i64,
    /// Alternative minimum tax basis plus money immediately after cancellation
    /// Leave empty when your property's alternative minimum tax basis equals its regular basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_basis_and_cash_after: Option<i64>,
    /// Property held at the beginning of January 1, 2027
    /// List every asset once, with a distinct label and its adjusted basis. Exclude money.
    #[libertas_unordered]
    /// ----
    /// Property
    pub property: Vec<DebtProperty>,
    /// Canceled debt you elect to apply first to depreciable property
    /// Form 982 line 5. Leave zero unless you make this election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciable_first: i64,
    /// Business, farm or rental debt
    #[libertas_default(false)]
    pub business_debt: bool,
}
/// Property for debt-related basis reduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DebtProperty {
    /// Distinct asset label
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    pub label: String,
    /// Kind of property
    #[libertas_default(PersonalUse)]
    pub kind: DebtPropertyKind,
    /// Adjusted basis before the January 1, 2027 reduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Alternative minimum tax adjusted basis
    /// Leave empty when it equals the regular basis.
    /// [NullText]
    /// AMT adjusted basis equals regular adjusted basis
    /// [SetText]
    /// Different AMT adjusted basis is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub amt_basis: Option<i64>,
}
/// Kind of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DebtPropertyKind {
    /// Personal-use property
    /// Your home, car, furniture and other things not used in a business or held for investment.
    PersonalUse,
    /// Building or improvement
    /// Depreciable real property used in a business or rental, or held for investment.
    Building {
        /// Secured the canceled debt
        #[libertas_default(false)]
        secured: bool,
    },
    /// Equipment or vehicle
    /// Depreciable or amortizable business or investment property other than buildings.
    Equipment {
        /// Secured the canceled debt
        #[libertas_default(false)]
        secured: bool,
    },
    /// Land
    /// Used in a business or rental, or held for investment.
    Land {
        /// Secured the canceled debt
        #[libertas_default(false)]
        secured: bool,
        /// Used in your farming business
        #[libertas_default(false)]
        farming: bool,
    },
    /// Stocks or other investments
    /// Nondepreciable investment property other than land.
    Investment {
        /// Secured the canceled debt
        #[libertas_default(false)]
        secured: bool,
    },
    /// Inventory or receivables
    /// Including real estate held for sale to customers.
    Inventory,
}
impl DebtPropertyKind {
    // Treas. Reg. 1.1017-1(a): secured real property, secured personal
    // property, other business and investment property, inventory and
    // receivables, then personal-use property.
    fn general_rank(&self) -> Option<usize> {
        Some(match self {
            Self::Building { secured: true } | Self::Land { secured: true, .. } => 0,
            Self::Equipment { secured: true } | Self::Investment { secured: true } => 1,
            Self::Building { .. }
            | Self::Land { .. }
            | Self::Equipment { .. }
            | Self::Investment { .. } => 2,
            Self::Inventory => 3,
            Self::PersonalUse => 4,
        })
    }
    // Treas. Reg. 1.1017-1(c): the section 108(b)(5) order among depreciable
    // property.
    fn depreciable_rank(&self) -> Option<usize> {
        match self {
            Self::Building { secured: true } => Some(0),
            Self::Equipment { secured: true } => Some(1),
            Self::Building { .. } | Self::Equipment { .. } => Some(2),
            _ => None,
        }
    }
    // Section 1017(b)(4): qualified property for farm debt, Form 982 lines
    // 11a (depreciable), 11b (farm land) and 11c (other).
    fn farm_rank(&self) -> Option<usize> {
        match self {
            Self::Building { .. } | Self::Equipment { .. } => Some(0),
            Self::Land { farming: true, .. } => Some(1),
            Self::Land { .. } | Self::Investment { .. } | Self::Inventory => Some(2),
            Self::PersonalUse => None,
        }
    }
    // Treas. Reg. 1.1017-1(c)(1): the qualified real property securing the
    // debt, then other depreciable real property (Form 982 line 4).
    fn real_rank(&self) -> Option<usize> {
        match self {
            Self::Building { secured: true } => Some(0),
            Self::Building { .. } => Some(1),
            _ => None,
        }
    }
}
impl DebtProperty {
    fn basis_for(&self, alternative: bool) -> i64 {
        if alternative { self.amt_basis.unwrap_or(self.basis) } else { self.basis }
    }
}
impl PersonalInsolvency {
    fn valid(&self) -> bool {
        [self.liabilities, self.assets, self.basis_and_cash_after, self.depreciable_first]
            .into_iter()
            .all(money)
            && self.amt_basis_and_cash_after.is_none_or(money)
            && valid_properties(&self.property)
    }
    fn excluded(&self, amount: i64) -> i64 {
        phase::cap(amount, [phase::floor(self.liabilities - self.assets, 0)])
    }
}

/// Chapter 12 or 13 discharge facts
/// One discharge event per filer. Do not use for a chapter 7 or 11 separate taxable estate or a case converted from either chapter. This filer was under the court's jurisdiction and the debt was discharged by the court or its approved plan, not merely subject to a petition or payment plan. Keep reduced basis and potential recapture records for later dispositions.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonalBankruptcy {
    /// Total property tax basis plus money immediately after discharge
    /// All property and money belonging to this filer, not fair market value.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_and_cash_after: i64,
    /// Alternative minimum tax basis plus money immediately after discharge
    /// Leave empty when your property's alternative minimum tax basis equals its regular basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_basis_and_cash_after: Option<i64>,
    /// Total liabilities remaining immediately after discharge
    /// Include debts that survive the court's discharge; exclude discharged amounts and settlement payments already made.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub liabilities_after: i64,
    /// Property held at the beginning of January 1, 2027
    /// List every asset once, with a distinct label and its adjusted basis. Exclude money.
    #[libertas_unordered]
    /// ----
    /// Property
    pub property: Vec<DebtProperty>,
    /// Canceled debt you elect to apply first to depreciable property
    /// Form 982 line 5. Leave zero unless you make this election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciable_first: i64,
}
impl PersonalBankruptcy {
    fn valid(&self) -> bool {
        [self.basis_and_cash_after, self.liabilities_after, self.depreciable_first]
            .into_iter()
            .all(money)
            && self.amt_basis_and_cash_after.is_none_or(money)
            && valid_properties(&self.property)
    }
}

/// Insolvency immediately before the cancellation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DebtInsolvency {
    /// Total liabilities immediately before cancellation
    /// Include the canceled debt and all other enforceable liabilities.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub liabilities: i64,
    /// Fair market value of all assets immediately before cancellation
    /// Include everything owned, including retirement accounts and exempt assets.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub assets: i64,
    /// Total property tax basis plus money immediately after cancellation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_and_cash_after: i64,
    /// Alternative minimum tax basis plus money immediately after cancellation
    /// Leave empty when your property's alternative minimum tax basis equals its regular basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_basis_and_cash_after: Option<i64>,
}
impl DebtInsolvency {
    fn valid(&self) -> bool {
        [self.liabilities, self.assets, self.basis_and_cash_after].into_iter().all(money)
            && self.amt_basis_and_cash_after.is_none_or(money)
    }
    fn excluded(&self, amount: i64) -> i64 {
        phase::cap(amount, [phase::floor(self.liabilities - self.assets, 0)])
    }
}

/// Qualified farm debt facts
/// The debt was incurred directly in operating your farming business; at least 50% of your gross receipts for 2023 through 2025 came from farming; the lender is regularly in the business of lending or a government agency, and is not related to you, the seller of the property or someone paid because of your investment in it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmDebtExclusion {
    /// Insolvent immediately before the cancellation
    /// Leave empty when your assets were worth at least your liabilities.
    #[libertas_default($null)]
    pub insolvency: Option<DebtInsolvency>,
    /// Property held at the beginning of January 1, 2027
    /// List every asset once, with a distinct label and its adjusted basis. Exclude money.
    #[libertas_unordered]
    /// ----
    /// Property
    pub property: Vec<DebtProperty>,
    /// Canceled debt you elect to apply first to depreciable property
    /// Form 982 line 5. Leave zero unless you make this election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciable_first: i64,
}
impl FarmDebtExclusion {
    fn valid(&self) -> bool {
        self.insolvency.as_ref().is_none_or(DebtInsolvency::valid)
            && money(self.depreciable_first)
            && valid_properties(&self.property)
    }
}

/// Qualified real property business debt facts
/// The debt was incurred or assumed to buy, build or substantially improve real property used in your trade or business (or refinances such debt, or was incurred before 1993), is secured by that property, is not farm debt, and you elect to exclude it. Real property held for sale to customers and a rental you also use as a home do not qualify.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RealPropertyDebtExclusion {
    /// Insolvent immediately before the cancellation
    /// Leave empty when your assets were worth at least your liabilities.
    #[libertas_default($null)]
    pub insolvency: Option<DebtInsolvency>,
    /// Outstanding principal of this debt immediately before the cancellation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub principal: i64,
    /// Fair market value of the real property securing the debt
    /// Immediately before the cancellation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// Other qualified real property business debt secured by that property
    /// Its outstanding principal immediately before the cancellation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_secured: i64,
    /// Adjusted basis of all your depreciable real property immediately before the cancellation
    /// Exclude property acquired in contemplation of the cancellation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depreciable_basis: i64,
    /// Property held at the beginning of January 1, 2027
    /// List every asset once, with a distinct label and its adjusted basis. Exclude money.
    #[libertas_unordered]
    /// ----
    /// Property
    pub property: Vec<DebtProperty>,
    /// Canceled debt you elect to apply first to depreciable property
    /// Form 982 line 5, for the part excluded because of insolvency. Leave zero unless you make this election.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciable_first: i64,
}
impl RealPropertyDebtExclusion {
    fn valid(&self) -> bool {
        self.insolvency.as_ref().is_none_or(DebtInsolvency::valid)
            && [
                self.principal,
                self.value,
                self.other_secured,
                self.depreciable_basis,
                self.depreciable_first,
            ]
            .into_iter()
            .all(money)
            && valid_properties(&self.property)
    }
    // Section 108(c)(2): the principal over the securing property's value net
    // of other qualified debt, and the depreciable real property's basis.
    fn value_limit(&self) -> i64 {
        phase::floor(self.principal - phase::floor(self.value - self.other_secured, 0), 0)
    }
}
fn valid_properties(property: &[DebtProperty]) -> bool {
    property.len() <= 100
        && property.iter().enumerate().all(|(i, p)| {
            !p.label.trim().is_empty()
                && money(p.basis)
                && p.amt_basis.is_none_or(money)
                && !property[..i].iter().any(|other| other.label == p.label)
        })
}

// Whole-dollar balances after current tax. Historical origin/owner allocation
// is not implied by these aggregate balances and still needs its own ledger.
#[derive(Clone, Copy)]
pub(crate) struct LossCarryovers {
    pub current_nol: i64,
    pub excess_business_loss: i64,
}
impl LossCarryovers {
    pub(crate) fn total_nol(self) -> i64 {
        self.current_nol + self.excess_business_loss
    }
}
// Cents reduced only after the current-year tax has been determined.
#[derive(Clone, Copy, Default)]
pub(crate) struct NolReduction {
    pub current: i64,
    pub prior: i64,
}
impl NolReduction {
    fn total(self) -> i64 {
        self.current + self.prior
    }
}
#[derive(Clone, Copy, Default)]
pub(crate) struct DebtReduction {
    pub regular: NolReduction,
    pub alternative: NolReduction,
}

// One tax system's attribute balances after the current-year tax, in cents,
// reduced as Form 982 Part II applies the excluded debt.
#[derive(Clone, Default)]
struct Balances {
    nol: [Exact; 2],
    excess_business: i64,
    gbc: i64,
    gbc_carryback: i64,
    mtc: i64,
    mtc_unknown: bool,
    capital: [i64; 2],
    passive: i64,
    ftc: i64,
    ftc_carryback: i64,
}
impl Balances {
    // Section 108(g)(3)(B): adjusted tax attributes, credits at three dollars
    // for each dollar of credit.
    fn adjusted(&self) -> Exact {
        &self.nol[0]
            + &self.nol[1]
            + exact::whole(self.capital[0])
            + exact::whole(self.capital[1])
            + exact::whole(self.passive)
            + exact::whole(3)
                * (exact::whole(self.gbc) + exact::whole(self.mtc) + exact::whole(self.ftc))
    }
}
fn cents(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount)
}
fn balances(
    r: &FederalResult,
    alternative: bool,
    amt: i64,
    losses: LossCarryovers,
    prior_nol: &Exact,
) -> Balances {
    let p = if alternative { "amt_" } else { "" };
    let system = if alternative { "amt" } else { "regular" };
    let foreign = |suffix: &str| {
        ["general", "passive"]
            .iter()
            .map(|c| cents(r, &alloc::format!("foreign_{c}_{suffix}")))
            .sum::<i64>()
    };
    Balances {
        nol: [exact::whole(losses.current_nol) * exact::whole(100), prior_nol * exact::whole(100)],
        excess_business: losses.excess_business_loss * 100,
        gbc: cents(r, "tip_fica_credit_carryforward_to_next_year")
            + cents(r, "tip_fica_current_credit_carryforward_to_next_year"),
        gbc_carryback: cents(r, "tip_fica_current_credit_carryback_to_prior_year"),
        mtc: cents(r, "minimum_tax_credit_carryforward"),
        mtc_unknown: amt > 0,
        capital: [
            cents(r, &alloc::format!("{p}capital_short_carryforward")),
            cents(r, &alloc::format!("{p}capital_long_carryforward")),
        ],
        passive: cents(r, &alloc::format!("{p}rental_suspended_loss")),
        ftc: foreign(if alternative { "amt_unused_tax" } else { "unused_tax" })
            + foreign(&alloc::format!("{system}_retained_carryforward")),
        ftc_carryback: foreign(&alloc::format!("{system}_carryback_prior_year")),
    }
}

// Excluded debt applied, per Form 982 Part II line, in cents.
#[derive(Clone, Copy, Default)]
struct Applied {
    real_property: i64,
    election: i64,
    nol: NolReduction,
    gbc: i64,
    mtc: i64,
    capital: [i64; 2],
    basis: i64,
    farm: [i64; 3],
    passive: i64,
    ftc: i64,
}

// One tax system's Part II worksheet for one discharge event.
struct Chain<'a> {
    alternative: bool,
    bal: Balances,
    applied: Applied,
    property: &'a [DebtProperty],
    reduced: Vec<i64>,
}
impl Chain<'_> {
    fn left(&self, i: usize) -> i64 {
        crate::kernel::carry::native_balance(
            self.property[i].basis_for(self.alternative),
            self.reduced[i],
        )
    }
    // Reduces the basis of the property `rank` places, rank by rank, each in
    // proportion to its remaining adjusted basis (labels fix the rounding
    // order); returns the reduction by rank.
    fn reduce(
        &mut self,
        amount: i64,
        rank: impl Fn(&DebtPropertyKind) -> Option<usize>,
    ) -> [i64; 5] {
        let mut used = [0; 5];
        let mut remaining = amount;
        for (level, used) in used.iter_mut().enumerate() {
            let mut items: Vec<usize> = (0..self.property.len())
                .filter(|i| rank(&self.property[*i].kind) == Some(level))
                .collect();
            items.sort_by(|a, b| self.property[*a].label.cmp(&self.property[*b].label));
            let basis: Vec<i64> = items.iter().map(|i| self.left(*i)).collect();
            let total: i64 = basis.iter().sum();
            let take = phase::cap(remaining, [total]);
            if take <= 0 {
                continue;
            }
            let mut prefix = crate::kernel::ratio::Prefix::default();
            for (i, b) in items.iter().zip(&basis) {
                let share = prefix.next(*b, |cumulative| fraction(take, cumulative, total));
                self.reduced[*i] += share;
            }
            *used = take;
            remaining = crate::kernel::carry::native_balance(remaining, take);
        }
        used
    }
}

// The facts of one discharge event, per Form 982 Part I.
struct Event<'a> {
    id: i64,
    #[cfg(test)]
    section: &'static str,
    amount: i64,
    // Box 1a or 1b amount, and whether it is the title 11 case.
    general: i64,
    title11: bool,
    // 1017(b)(2) limit on the general basis reduction, per tax system.
    room: [Option<i64>; 2],
    farm: bool,
    real_property: Option<&'a RealPropertyDebtExclusion>,
    property: &'a [DebtProperty],
    election: i64,
    /// Check the program approval, payment statement and servicer interest allocation.
    confirmed: bool,
}
fn event_of(id: i64, data: &PersonalDebtCancellation) -> Option<Event<'_>> {
    let snapshot = |i: &DebtInsolvency, amount: i64| {
        let after = i.liabilities - amount;
        [
            Some(phase::floor(i.basis_and_cash_after - after, 0)),
            Some(phase::floor(
                i.amt_basis_and_cash_after.unwrap_or(i.basis_and_cash_after) - after,
                0,
            )),
        ]
    };
    let amount = data.amount;
    Some(match data.treatment.as_ref()? {
        PersonalDebtTreatment::Insolvent { data: facts } => {
            let after = facts.liabilities - amount;
            Event {
                id,
                #[cfg(test)]
                section: "Insolvency tax attributes",
                amount,
                general: facts.excluded(amount),
                title11: false,
                room: [
                    Some(phase::floor(facts.basis_and_cash_after - after, 0)),
                    Some(phase::floor(
                        facts.amt_basis_and_cash_after.unwrap_or(facts.basis_and_cash_after)
                            - after,
                        0,
                    )),
                ],
                farm: false,
                real_property: None,
                property: &facts.property,
                election: facts.depreciable_first,
                confirmed: facts.liabilities >= amount,
            }
        }
        PersonalDebtTreatment::Bankruptcy { data: facts } => Event {
            id,
            #[cfg(test)]
            section: "Bankruptcy tax attributes",
            amount,
            general: amount,
            title11: true,
            room: [
                Some(phase::floor(facts.basis_and_cash_after - facts.liabilities_after, 0)),
                Some(phase::floor(
                    facts.amt_basis_and_cash_after.unwrap_or(facts.basis_and_cash_after)
                        - facts.liabilities_after,
                    0,
                )),
            ],
            farm: false,
            real_property: None,
            property: &facts.property,
            election: facts.depreciable_first,
            confirmed: true,
        },
        PersonalDebtTreatment::FarmDebt { data: facts } => Event {
            id,
            #[cfg(test)]
            section: "Qualified farm debt",
            amount,
            general: facts.insolvency.as_ref().map_or(0, |i| i.excluded(amount)),
            title11: false,
            room: facts.insolvency.as_ref().map_or([None; 2], |i| snapshot(i, amount)),
            farm: true,
            real_property: None,
            property: &facts.property,
            election: facts.depreciable_first,
            confirmed: facts.insolvency.as_ref().is_none_or(|i| i.liabilities >= amount),
        },
        PersonalDebtTreatment::RealPropertyDebt { data: facts } => Event {
            id,
            #[cfg(test)]
            section: "Qualified real property business debt",
            amount,
            general: facts.insolvency.as_ref().map_or(0, |i| i.excluded(amount)),
            title11: false,
            room: facts.insolvency.as_ref().map_or([None; 2], |i| snapshot(i, amount)),
            farm: false,
            real_property: Some(facts),
            property: &facts.property,
            election: facts.depreciable_first,
            confirmed: facts.principal >= amount
                && facts.insolvency.as_ref().is_none_or(|i| i.liabilities >= amount),
        },
        _ => return None,
    })
}

// The ordering context shared by every step of one tax system.
struct Context {
    joint: bool,
    retained_business: bool,
    #[cfg(test)]
    section: &'static str,
}
fn stop(
    r: &mut FederalResult,
    #[cfg(test)] section: &str,
    key: &str,
    #[cfg(test)] english: &str,
    args: Vec<MessageArg>,
) -> bool {
    issue_message(
        r,
        &crate::IssueContext::Section(crate::TaxSection::Documents),
        #[cfg(test)]
        section,
        #[cfg(test)]
        english,
        Message::new(key, args),
    );
    false
}
// Section 108(b)(2) and (b)(5): the excluded amount reduces, in order, the
// elected depreciable basis (line 5), NOL (6), general business credit (7,
// 33 1/3 cents a dollar), minimum tax credit (8, 33 1/3 cents), capital loss
// (9), basis (10a, or 11a-c for farm debt), passive loss (12) and foreign tax
// credit (13, 33 1/3 cents). Returns false after refusing an attribute the
// return does not carry.
fn apply_general(
    chain: &mut Chain,
    remaining: &mut i64,
    election: &mut i64,
    room: &mut Option<i64>,
    farm: bool,
    c: &Context,
    r: &mut FederalResult,
) -> bool {
    use crate::kernel::carry::native_balance;
    let joint_refusal = |r: &mut FederalResult, line: i64| {
        stop(
            r,
            #[cfg(test)]
            c.section,
            "ISSUE_DEBT_JOINT_CARRYOVER_SHARE",
            #[cfg(test)]
            &alloc::format!(
                "A joint return needs each spouse's share of the carryover this canceled debt reduces (Treas. Reg. 1.108-7(d), Form 982 line {line}); it is not calculated."
            ),
            alloc::vec![MessageArg::Number(line)],
        )
    };
    let elected = phase::cap(*election, [*remaining]);
    if elected > 0 {
        let used: i64 = chain.reduce(elected, DebtPropertyKind::depreciable_rank).iter().sum();
        chain.applied.election += used;
        *remaining = native_balance(*remaining, used);
        *election = native_balance(*election, elected);
    }
    let b = &mut chain.bal;
    let a = &mut chain.applied;
    if *remaining > 0 && &b.nol[0] + &b.nol[1] > exact::whole(0) {
        if c.joint {
            return joint_refusal(r, 6);
        }
        let current = exact::rounded(&phase::cap(exact::whole(*remaining), [b.nol[0].clone()]))
            .expect("current loss debit is bounded by native excluded debt");
        let prior =
            exact::rounded(&phase::cap(exact::whole(*remaining - current), [b.nol[1].clone()]))
                .expect("prior loss debit is bounded by remaining native excluded debt");
        a.nol.current += current;
        a.nol.prior += prior;
        b.nol[0] -= exact::whole(current);
        b.nol[1] -= exact::whole(prior);
        *remaining = native_balance(*remaining, current + prior);
    }
    if *remaining > 0 && b.excess_business > 0 {
        return stop(
            r,
            #[cfg(test)]
            c.section,
            "ISSUE_DEBT_EXCESS_BUSINESS_LOSS",
            #[cfg(test)]
            "An excess business loss carryforward (Form 461 line 16) reached by canceled debt (Form 982 line 6) is not calculated.",
            alloc::vec![],
        );
    }
    for (credit, carryback, line) in [(&mut b.gbc, b.gbc_carryback, 7), (&mut b.mtc, 0, 8)] {
        #[cfg(test)]
        let name = if line == 7 { "general business credit" } else { "minimum tax credit" };
        if *remaining == 0 {
            break;
        }
        if line == 8 && b.mtc_unknown {
            return stop(
                r,
                #[cfg(test)]
                c.section,
                "ISSUE_DEBT_MINIMUM_TAX_CREDIT_UNKNOWN",
                #[cfg(test)]
                "The minimum tax credit from 2026 alternative minimum tax (Form 8801 for 2027) that canceled debt reduces (Form 982 line 8) is not calculated.",
                alloc::vec![],
            );
        }
        if *credit == 0 {
            continue;
        }
        if c.joint {
            return joint_refusal(r, line);
        }
        if carryback > 0 {
            return stop(
                r,
                #[cfg(test)]
                c.section,
                if line == 7 { "ISSUE_DEBT_GBC_CARRYBACK" } else { "ISSUE_DEBT_MTC_CARRYBACK" },
                #[cfg(test)]
                &alloc::format!(
                    "A {name} carried back to 2025 that canceled debt reduces (Form 982 line {line}) is not calculated."
                ),
                alloc::vec![],
            );
        }
        let applied = phase::cap(*remaining, [3 * *credit]);
        *credit = native_balance(*credit, phase::cap(fraction(applied, 1, 3), [*credit]));
        *remaining = native_balance(*remaining, applied);
        if line == 7 {
            a.gbc += applied;
        } else {
            a.mtc += applied;
        }
    }
    let capital = b.capital[0] + b.capital[1];
    if *remaining > 0 && capital > 0 {
        if c.joint {
            return joint_refusal(r, 9);
        }
        let applied = phase::cap(*remaining, [capital]);
        if applied < capital && b.capital[0] > 0 && b.capital[1] > 0 {
            return stop(
                r,
                #[cfg(test)]
                c.section,
                "ISSUE_DEBT_PARTIAL_MIXED_CAPITAL",
                #[cfg(test)]
                "Canceled debt that reduces only part of a mixed short- and long-term capital loss carryover (Form 982 line 9) needs the year-of-origin ledger; it is not calculated.",
                alloc::vec![],
            );
        }
        let short = phase::cap(applied, [b.capital[0]]);
        let long = applied - short;
        a.capital[0] += short;
        a.capital[1] += long;
        b.capital = [native_balance(b.capital[0], short), native_balance(b.capital[1], long)];
        *remaining = native_balance(*remaining, applied);
    }
    if *remaining > 0 {
        if c.retained_business
            && !chain.property.iter().any(|p| p.kind != DebtPropertyKind::PersonalUse)
        {
            return stop(
                r,
                #[cfg(test)]
                c.section,
                "ISSUE_DEBT_BUSINESS_PROPERTY_MISSING",
                #[cfg(test)]
                "List the business, farm and rental property you hold on January 1, 2027 under Property, with its basis: canceled debt reduces it before personal-use property (Form 982 line 10a).",
                alloc::vec![],
            );
        }
        if farm {
            let used = chain.reduce(*remaining, DebtPropertyKind::farm_rank);
            for (total, used) in chain.applied.farm.iter_mut().zip(used) {
                *total += used;
                *remaining = native_balance(*remaining, used);
            }
        } else {
            let take = room.map_or(*remaining, |v| phase::cap(v, [*remaining]));
            let used: i64 = chain.reduce(take, DebtPropertyKind::general_rank).iter().sum();
            chain.applied.basis += used;
            *remaining = native_balance(*remaining, used);
            if let Some(v) = room {
                *v = native_balance(*v, used);
            }
        }
    }
    let b = &mut chain.bal;
    let a = &mut chain.applied;
    if *remaining > 0 && b.passive > 0 {
        if c.joint {
            return joint_refusal(r, 12);
        }
        let applied = phase::cap(*remaining, [b.passive]);
        a.passive += applied;
        b.passive = native_balance(b.passive, applied);
        *remaining = native_balance(*remaining, applied);
    }
    if *remaining > 0 && b.ftc > 0 {
        if c.joint {
            return joint_refusal(r, 13);
        }
        if b.ftc_carryback > 0 {
            return stop(
                r,
                #[cfg(test)]
                c.section,
                "ISSUE_DEBT_FTC_CARRYBACK",
                #[cfg(test)]
                "A foreign tax credit carried back to 2025 that canceled debt reduces (Form 982 line 13) is not calculated.",
                alloc::vec![],
            );
        }
        let applied = phase::cap(*remaining, [3 * b.ftc]);
        b.ftc = native_balance(b.ftc, phase::cap(fraction(applied, 1, 3), [b.ftc]));
        a.ftc += applied;
        *remaining = native_balance(*remaining, applied);
    }
    true
}

// Form 982 Part II after the current-year tax: every filer's single
// discharge event, its Part I exclusions and the ordered attribute
// reductions in both tax systems. Returns the NOL reductions the carryforward
// ledgers apply.
pub(crate) fn reduce_attributes(
    d: &FederalDraft,
    r: &mut FederalResult,
    amt: i64,
    losses: [LossCarryovers; 2],
    prior_nol: [Exact; 2],
) -> DebtReduction {
    let mut result = DebtReduction::default();
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
    // Business, farm and rental property the return still holds must be
    // listed when excluded debt reaches basis.
    let retained_business = d.income.iter().any(|e| match &e.income {
        FederalIncome::Business { data } => {
            data.assets.iter().any(|a| a.disposal == crate::AssetDisposal::No)
        }
        // A rental exchanged for another rental is still held.
        FederalIncome::Rental { data } => match &data.sale {
            crate::RentalSale::No => true,
            crate::RentalSale::Exchanged(exchange) => {
                matches!(exchange.replacement.used, crate::ReplacementUse::Rental { .. })
            }
            crate::RentalSale::Sold { .. } | crate::RentalSale::Condemned { .. } => false,
        },
        FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
            ..
        } => !assets.is_empty(),
        _ => false,
    });
    let mut base = [
        balances(r, false, amt, losses[0], &prior_nol[0]),
        balances(r, true, amt, losses[1], &prior_nol[1]),
    ];
    for entry in &d.income {
        let FederalIncome::PersonalDebt { data } = &entry.income else {
            continue;
        };
        let data = &PersonalDebtCancellation::from(data);
        if !data.personally_liable {
            continue;
        }
        let Some(event) = event_of(entry.id, data) else {
            continue;
        };
        let count = d
            .income
            .iter()
            .filter(|other| {
                other.owner == entry.owner
                    && matches!(&other.income, FederalIncome::PersonalDebt { data }
                        if PersonalDebtCancellation::from(data).excluded_before_limits().is_some())
            })
            .count();
        if !require_ok(
            r,
            count == 1,
            #[cfg(test)]
            event.section,
            "ISSUE_DEBT_MULTIPLE_DISCHARGES",
            #[cfg(test)]
            "Several discharges with exclusions for one filer in 2026 need the multiple-discharge basis allocation (Treas. Reg. 1.1017-1(b)(2), Form 982 Part II); it is not calculated.",
            alloc::vec![],
        ) || !require_ok(
            r,
            event.confirmed,
            #[cfg(test)]
            event.section,
            "ISSUE_DEBT_FACTS_UNCONFIRMED",
            #[cfg(test)]
            "Confirm the discharge facts, including a liability or principal snapshot that includes the canceled debt, and the property held on January 1, 2027, after completing the 2026 return.",
            alloc::vec![],
        ) {
            continue;
        }
        let Some(reduction) = event_reduction(&event, &mut base, joint, retained_business, r)
        else {
            continue;
        };
        result.regular.current += reduction.regular.current;
        result.regular.prior += reduction.regular.prior;
        result.alternative.current += reduction.alternative.current;
        result.alternative.prior += reduction.alternative.prior;
    }
    result
}
fn require_ok(
    r: &mut FederalResult,
    ok: bool,
    #[cfg(test)] section: &str,
    key: &str,
    #[cfg(test)] english: &str,
    args: Vec<MessageArg>,
) -> bool {
    if !ok {
        issue_message(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Documents),
            #[cfg(test)]
            section,
            #[cfg(test)]
            english,
            Message::new(key, args),
        );
    }
    ok
}

// One event through both tax systems; None after a refusal.
fn event_reduction(
    event: &Event,
    base: &mut [Balances; 2],
    joint: bool,
    retained_business: bool,
    r: &mut FederalResult,
) -> Option<DebtReduction> {
    let context = Context {
        joint,
        retained_business,
        #[cfg(test)]
        section: event.section,
    };
    let mut excluded = [event.general, 0, 0];
    let mut chains: Vec<Chain> = [false, true]
        .into_iter()
        .map(|alternative| Chain {
            alternative,
            bal: base[usize::from(alternative)].clone(),
            applied: Applied::default(),
            property: event.property,
            reduced: alloc::vec![0; event.property.len()],
        })
        .collect();
    let mut elections = [event.election; 2];
    for (i, chain) in chains.iter_mut().enumerate() {
        let mut remaining = event.general;
        let mut room = event.room[i];
        if !apply_general(chain, &mut remaining, &mut elections[i], &mut room, false, &context, r) {
            return None;
        }
        if i == 0 {
            // Section 108(c)/(g) limits follow the insolvency reductions.
            let rest = event.amount - event.general;
            if event.farm && rest > 0 {
                let qualified: i64 = (0..event.property.len())
                    .filter(|p| event.property[*p].kind != DebtPropertyKind::PersonalUse)
                    .map(|p| chain.left(p))
                    .sum();
                let limit = chain.bal.adjusted() + exact::whole(qualified);
                if exact::whole(rest) > limit
                    && !require_ok(
                        r,
                        false,
                        #[cfg(test)]
                        event.section,
                        if chain.bal.mtc_unknown {
                            "ISSUE_DEBT_FARM_LIMIT_MTC_UNKNOWN"
                        } else {
                            "ISSUE_DEBT_FARM_LIMIT_EXCEEDED"
                        },
                        #[cfg(test)]
                        &alloc::format!(
                            "Canceled farm debt above the Form 982 line 1c limit (adjusted tax attributes plus the basis of qualified property, ${}) is Schedule F line 8 income, which is not calculated.{}",
                            exact::dollars(&limit)
                                .expect("a failing farm limit is below native excluded debt"),
                            if chain.bal.mtc_unknown {
                                " The limit also needs the minimum tax credit from 2026 alternative minimum tax (Form 8801 for 2027)."
                            } else {
                                ""
                            }
                        ),
                        alloc::vec![MessageArg::usd_dollars(
                            exact::dollars(&limit)
                                .expect("a failing farm limit is below native excluded debt")
                        )],
                    )
                {
                    return None;
                }
                excluded[1] = rest;
            }
            if let Some(real) = event.real_property
                && rest > 0
            {
                let reduced_real: i64 = (0..event.property.len())
                    .filter(|p| {
                        matches!(event.property[*p].kind, DebtPropertyKind::Building { .. })
                    })
                    .map(|p| chain.reduced[p])
                    .sum();
                let limit = phase::cap(
                    real.value_limit(),
                    [phase::floor(real.depreciable_basis - reduced_real, 0)],
                );
                if !require_ok(
                    r,
                    rest <= limit,
                    #[cfg(test)]
                    event.section,
                    "ISSUE_DEBT_REAL_PROPERTY_LIMIT_EXCEEDED",
                    #[cfg(test)]
                    &alloc::format!(
                        "Canceled real property business debt above the Form 982 line 1d limit (${}) is Schedule C line 6, Schedule E line 3 or Schedule F line 8 income, which is not calculated.",
                        dollars(limit)
                    ),
                    alloc::vec![MessageArg::usd_dollars(dollars(limit))],
                ) {
                    return None;
                }
                excluded[2] = rest;
            }
        }
        if excluded[1] > 0 {
            let mut remaining = excluded[1];
            let mut none = None;
            if !apply_general(
                chain,
                &mut remaining,
                &mut elections[i],
                &mut none,
                true,
                &context,
                r,
            ) {
                return None;
            }
        }
        if excluded[2] > 0 {
            let used: i64 = chain.reduce(excluded[2], DebtPropertyKind::real_rank).iter().sum();
            if !require_ok(
                r,
                used == excluded[2],
                #[cfg(test)]
                event.section,
                "ISSUE_DEBT_REAL_PROPERTY_LIST_MISSING",
                #[cfg(test)]
                "List the depreciable real property that secured the debt, and your other buildings, under Property with their January 1, 2027 basis (Form 982 line 4).",
                alloc::vec![],
            ) {
                return None;
            }
            chain.applied.real_property += used;
        }
    }
    for (i, chain) in chains.iter().enumerate() {
        base[i] = chain.bal.clone();
    }
    report(event, excluded, &chains, r);
    Some(DebtReduction { regular: chains[0].applied.nol, alternative: chains[1].applied.nol })
}

const I982: &str = "https://www.irs.gov/instructions/i982";
fn report(event: &Event, excluded: [i64; 3], chains: &[Chain], r: &mut FederalResult) {
    let id = event.id;
    let inputs = ["income.personal_debt", "screening.nol", "screening.atnol"];
    let (general_id, general_box) =
        if event.title11 { ("bankruptcy_exclusion", "1a") } else { ("insolvency_exclusion", "1b") };
    for (suffix, amount, explanation) in [
        (
            general_id,
            excluded[0],
            if event.title11 { msg!("EXPLAIN_DEBT_N_N_2") } else { msg!("EXPLAIN_DEBT_N_N_3") },
        ),
        ("farm_exclusion", excluded[1], msg!("EXPLAIN_DEBT_N_N_FARM_EXCLUSION")),
        ("real_property_exclusion", excluded[2], msg!("EXPLAIN_DEBT_N_N_REAL_PROPERTY_EXCLUSION")),
    ] {
        if amount > 0 || suffix == general_id && !event.farm && event.real_property.is_none() {
            line_cents(
                r,
                &alloc::format!("debt_{id}_{suffix}"),
                amount,
                &inputs,
                I982,
                explanation,
            );
        }
    }
    let regular = &chains[0].applied;
    let form = |r: &mut FederalResult, number: &str, amount: i64, explanation: crate::Message| {
        push_line(
            r,
            Some(FormLine::new(TaxForm::Form982, number)),
            &alloc::format!("debt_{id}_form982_line{number}"),
            dollars(amount) * 100,
            &inputs,
            I982,
            explanation,
        );
    };
    for (number, amount, explanation) in [
        (general_box, excluded[0], msg!("EXPLAIN_DEBT_N_FORM982_LINE")),
        ("1c", excluded[1], msg!("EXPLAIN_DEBT_N_FORM982_LINE_1C")),
        ("1d", excluded[2], msg!("EXPLAIN_DEBT_N_FORM982_LINE_1D")),
    ] {
        if amount > 0 {
            form(r, number, amount, explanation);
        }
    }
    form(r, "2", excluded.iter().sum(), msg!("EXPLAIN_DEBT_N_FORM982_LINE2"));
    for (number, amount, explanation) in [
        ("4", regular.real_property, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N")),
        ("5", regular.election, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_2")),
        ("6", regular.nol.total(), msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_3")),
        ("7", regular.gbc, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_4")),
        ("8", regular.mtc, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_5")),
        ("9", regular.capital[0] + regular.capital[1], msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_6")),
        ("10a", regular.basis, msg!("EXPLAIN_DEBT_N_FORM982_LINE_10A")),
        ("11a", regular.farm[0], msg!("EXPLAIN_DEBT_N_FORM982_LINE_11A")),
        ("11b", regular.farm[1], msg!("EXPLAIN_DEBT_N_FORM982_LINE_11B")),
        ("11c", regular.farm[2], msg!("EXPLAIN_DEBT_N_FORM982_LINE_11C")),
        ("12", regular.passive, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_7")),
        ("13", regular.ftc, msg!("EXPLAIN_DEBT_N_FORM982_LINE_N_8")),
    ] {
        if amount > 0 {
            form(r, number, amount, explanation);
        }
    }
    for chain in chains {
        let a = &chain.applied;
        let b = &chain.bal;
        let (name, prefix) = if chain.alternative { ("amt", "amt_") } else { ("regular", "") };
        line_cents(
            r,
            &alloc::format!("debt_{id}_{name}_nol_reduction"),
            a.nol.total(),
            &inputs,
            I982,
            msg!("EXPLAIN_DEBT_N_N_NOL_REDUCTION"),
        );
        line_cents(
            r,
            &alloc::format!("debt_{id}_{prefix}basis_reduction"),
            a.basis,
            &inputs,
            I982,
            msg!("EXPLAIN_DEBT_N_0BASIS_REDUCTION"),
        );
        for (suffix, amount, balance, explanation) in [
            (
                "election_basis_reduction",
                a.election,
                None,
                msg!("EXPLAIN_DEBT_N_N_ELECTION_BASIS_REDUCTION"),
            ),
            (
                "farm_basis_reduction",
                a.farm.iter().sum::<i64>(),
                None,
                msg!("EXPLAIN_DEBT_N_N_FARM_BASIS_REDUCTION"),
            ),
            (
                "real_property_basis_reduction",
                a.real_property,
                None,
                msg!("EXPLAIN_DEBT_N_N_REAL_PROPERTY_BASIS_REDUCTION"),
            ),
            (
                "capital_reduction",
                a.capital[0] + a.capital[1],
                Some(b.capital[0] + b.capital[1]),
                msg!("EXPLAIN_DEBT_N_N_CAPITAL_REDUCTION"),
            ),
            (
                "passive_reduction",
                a.passive,
                Some(b.passive),
                msg!("EXPLAIN_DEBT_N_N_PASSIVE_REDUCTION"),
            ),
            (
                "ftc_reduction",
                fraction(a.ftc, 1, 3),
                Some(b.ftc),
                msg!("EXPLAIN_DEBT_N_N_FTC_REDUCTION"),
            ),
        ] {
            if amount > 0 {
                line_cents(
                    r,
                    &alloc::format!("debt_{id}_{prefix}{suffix}"),
                    amount,
                    &inputs,
                    I982,
                    explanation,
                );
                if let Some(balance) = balance {
                    line_cents(
                        r,
                        &alloc::format!(
                            "debt_{id}_{prefix}{}_after",
                            suffix.trim_end_matches("_reduction")
                        ),
                        balance,
                        &inputs,
                        I982,
                        msg!("EXPLAIN_DEBT_N_N_AFTER"),
                    );
                }
            }
        }
        if !chain.alternative {
            for (suffix, amount, balance, explanation) in [
                ("gbc", a.gbc, b.gbc, msg!("EXPLAIN_DEBT_N_N_REDUCTION_GBC")),
                ("mtc", a.mtc, b.mtc, msg!("EXPLAIN_DEBT_N_N_REDUCTION_MTC")),
            ] {
                if amount > 0 {
                    line_cents(
                        r,
                        &alloc::format!("debt_{id}_{suffix}_reduction"),
                        fraction(amount, 1, 3),
                        &inputs,
                        I982,
                        explanation,
                    );
                    line_cents(
                        r,
                        &alloc::format!("debt_{id}_{suffix}_after"),
                        balance,
                        &inputs,
                        I982,
                        msg!("EXPLAIN_DEBT_N_N_AFTER"),
                    );
                }
            }
        }
        for (index, p) in chain.property.iter().enumerate() {
            emit_lines(
                r,
                &mut (),
                [("reduction", chain.reduced[index]), ("remaining_basis", chain.left(index))],
                |(suffix, value), _, _| {
                    Some(calculation_row(
                        None,
                        &alloc::format!("debt_{id}_{prefix}property_{index}_{suffix}"),
                        LineValue::MoneyCents(value),
                        &["income.personal_debt"],
                        "https://www.irs.gov/publications/p4681",
                        crate::Message::new(
                            "EXPLAIN_DEBT_N_0PROPERTY_N_N",
                            alloc::vec![crate::MessageArg::Text(p.label.clone())],
                        ),
                    ))
                },
            );
        }
    }
}

/// Health-service education loan repayment assistance
/// One program's actual 2026 assistance for this filer. Do not also enter it as taxable wages, canceled debt or another receipt. Use the program and servicer records to allocate interest; an employer's ordinary loan benefit is a different rule.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HealthLoanAssistance {
    /// Qualifying program
    pub program: HealthLoanProgram,
    /// Assistance received or paid on this filer's behalf
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Portion paying interest included in the student-loan interest entry
    /// Enter only interest also included in Adjustments, including reimbursed interest. If no deduction is requested or the servicer total there excludes this interest, enter zero. The App subtracts this overlap before applying the deduction cap and income phaseout. Do not repeat interest across program entries.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub interest: i64,
    /// Program eligibility, assistance and interest allocation confirmed
    /// Retain program approval and payment records. This is the selected statutory health-service program, not an application, ordinary employer benefit or unsupported service award. No unentered part of the program-paid interest is being claimed as a deduction. No treats the assistance as taxable, with a notice.
    /// Check the program approval, payment statement and servicer interest allocation.
    pub confirmed: bool,
}
/// Tax-free health-service loan assistance program
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthLoanProgram {
    /// National Health Service Corps Loan Repayment Program
    National,
    /// State education loan repayment program eligible for Public Health Service Act funds
    PublicHealthState,
    /// Other state program increasing health services in underserved or shortage areas
    /// The state determines the underserved or health professional shortage area; ordinary state loan forgiveness is not enough.
    UnderservedState,
}
impl HealthLoanAssistance {
    // What the assistance adds to income: nothing when confirmed as excluded.
    pub(crate) fn on_return(&self) -> i64 {
        if self.confirmed { 0 } else { self.amount }
    }
    // The page's client rule, repeated on save.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (self.interest > self.amount).then_some(
            crate::validation::ValidationIssue::new("HEALTH_LOAN_INTEREST_ABOVE_ASSISTANCE")
                .at_field("interest"),
        )
    }
    pub(crate) fn valid(&self) -> bool {
        money(self.amount) && (0..=self.amount).contains(&self.interest)
    }
    pub(crate) fn report(&self, r: &mut FederalResult, id: i64) {
        line_cents(
            r,
            &alloc::format!("health_loan_{id}_excluded"),
            self.amount,
            &["income.health_loan_assistance"],
            "https://www.irs.gov/publications/p970",
            msg!("EXPLAIN_HEALTH_LOAN_N_EXCLUDED"),
        );
    }
}
