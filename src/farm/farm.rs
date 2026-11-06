// Current scope (2026-10-11, unit B12): partial direct Schedule F/Form 4835.
// Header/source twins, canonical land/production identities, supported sections 126/175
// allocation/basis, current section 180 election, held production costs, regular/AMT
// supplies and actual CCC/crop source histories are integrated. Current poultry
// supports documented whole-month ratable periods; no partial-month rule is guessed.
// Optional section 175/Schedule J/crop benefits with unavailable source allocations are left out
// with named notices. Required property/basis/timing unknowns remain explicit.
// Full K-1/property/risk histories, released inventory/ADS/interest, farmer-gross
// source reconciliation and nonnumeric form cells still need their actual owners.
// The audit below records historical scope; its old refusals are not current
// claims about every supported subcase. See the owning source modules and tests.
// Coverage: Schedule F — Supported (audited 2026-09-24; unit T2-45, 2026-09-26).
//   Lines: Part I cash lines 1a–1c, 2, 3a/3b cooperative distributions from Form 1099-PATR
//   documents (T2-45: boxes 1, 2, 3 and 5 on 3a; 3b after the personal-purchase dividends; Part
//   III 38a/38b on the accrual method), 4a/4b agricultural program payments (fully taxable
//   receipts and Form 1099-G box 7; the section 126 cost-sharing receipt FarmReceipt::CostSharing
//   is refused by name on line 4b), 5 CCC loans, 6 crop insurance/disaster incl. deferral
//   election, 7 custom hire, 8 other; Part II expenses via shared business expenses/179/
//   depreciation; Part III accrual (sales, inventories, purchases); CRP SE exclusion;
//   prepaid-supplies limit; section 175 conservation; farm-land sales/1062 deferral; Schedule J
//   averaging; optional farm SE. The 1099-PATR document (federal_model.rs Patr1099/PatrMore onto
//   FederalIncome::Cooperative { FederalCooperative }) enters on the Self-employment page and is
//   routed through the related component keys on the receiving activity
//   (an owner's only eligible activity when unclaimed, a correction
//   note when several; unit B2 replaced the lowest-id rule) with the 1099-G box 7 payments
//   (federal_rules.rs routed_farm, farm_cooperative, farm_routed_income); its taxable
//   amount joins the farm's profit, SE earnings, Schedule J base and QBI; box 8 qualified items
//   below the taxable amount remove the excess from the farm's QBI; box 7 qualified payments
//   compute a zero Form 8995-A Schedule D patron reduction when the farm has no W-2 wages; box 4
//   is Form 1040 line 25b. Rules PATR_WITHHOLDING_ABOVE_DISTRIBUTIONS,
//   PATR_PERSONAL_ABOVE_DISTRIBUTIONS and PATR_QUALIFIED_ITEMS_ABOVE_DISTRIBUTIONS on both sides.
//   Refused by name: box 6 section 199A(g) deduction (Form 8995-A Part IV line 38), box 7 with
//   farm W-2 wages (Form 8995-A Schedule D), box 9 SSTB items, boxes 10–12 credits (Forms
//   3468/5884/3800), dividends on capital or depreciable property (basis reduction), a
//   non-farm cooperative statement (Schedule C line 6), distributions without a farm for the
//   owner, section 175 with cooperative distributions on the same farm; section 126
//   cost-sharing exclusion (line 4b); the "Farm income" message, a method change whose section 481(a) adjustment is not entered (Form 3115), farming syndicate (section 464), UNICAP
//   (section 263A), farm debt cancellation (Form 982); farm NOL carryback (only a valid
//   waiver computes; "Farm loss" Unsupported). STORAGE_FORMAT 52.
//   Section 175 pool (fix unit F2, 2026-10-09): gross income from farming counts raised and resold
//   products, ordinary program payments, a CCC loan treated as income (section 77) and crop
//   insurance proceeds (section 451(f)) for farms and farm rentals (section 175(b)(1), Reg.
//   1.175-5, Schedule F instructions line 12); custom hire, cost-sharing, other operating income
//   and CRP still stop the pool (conservation_source_ready).
//   Unit 481a (2026-10-09): a section 481(a) adjustment of Form 3115 (FederalBusiness.method_changes,
//   method_change.rs) is Schedule F line 8 other income (Part III line 43 accrual, inside line 9) when
//   positive and a line 32a-32f other expense when negative, untagged like those lines, with the
//   rows farm_<id>_method_change_<n> and ..._remaining; the "No accounting-method change" confirmation
//   now excludes only a change not entered there. Tests: src/tests/cost_recovery/method_change.rs.
//   T2-45p (2026-09-27): the Farming income topic enters every record above (interview.rs
//   T2-45p block, farm_page.rs types); the 1099-PATR kind moved to it; RareSituation::Farm
//   removed. Tagged (F14): 5a/5b (Part III 40a/40b, farm_ccc.rs advances and
//   forfeiture_reported), 6a/6b/6d (41a/41b/41d, farm_insurance.rs; 6b is the new
//   crop_insurance_current_taxable row, current proceeds less the 2027 deferral), Part III 37, 45,
//   46 and 48; lines 7 and 8 stay untagged until F5 settles CRP (unit T2-45x). Test:
//   src/tests/farm/interview.rs farm_ccc_and_crop_insurance_lines_are_tagged.
//   Evidence: farm.rs, farm_ccc.rs, farm_crp.rs, farm_insurance.rs, farm_supplies.rs,
//   farm_conservation.rs, farm_land.rs, farm_deferral.rs, farm_averaging.rs, section1231.rs;
//   src/tests/farm/farm.rs farm_cash_sources_flow_once_to_schedule_one_se_qbi_and_return,
//   farm_accrual_inventory_and_income_flow_through_the_full_return,
//   farm_nol_requires_explicit_valid_waiver_and_preserves_the_carryback_choice,
//   farm_averaging_recomputes_base_year_rate_schedules_not_filing_tables,
//   farm_cooperative_distributions_reach_schedule_f_line_3_se_qbi_and_the_return,
//   farm_cooperative_box_8_below_the_taxable_distributions_leaves_the_farm_qbi,
//   farm_cooperative_boxes_and_missing_farm_are_refused_by_name,
//   farm_program_payments_are_tagged_4a_4b_and_cost_sharing_is_refused_by_name;
//   src/tests/farm/interview.rs patr_document_maps_onto_the_cooperative_record_and_back,
//   patr_box_rules_reject_on_both_sides, patr_fold_boxes_refuse_by_name; sources i1040sf (2025;
//   2026 draft form created 4/24/26 keeps the lines), i1099ptr, i8995a, f4835, Pub 225, i1040sj,
//   26 CFR 1.77-2.
//   T2-45x (2026-09-27, map checked against the 2026 draft i1040sf of 9/17/26, the 2025
//   i1040sf and Pub 225 (2025)): CRP payments print on lines 4a/4b (Pub 225 ch. 12; Schedule
//   SE line 1b), CCC lot market gain on 4a (4b when the loan was not included in income) and a
//   redeemed CCC commodity sold on 1a/1b (loan included; basis = redemption payment) or 2;
//   lines 9 and 34 include the line 8 fuel-tax credit/recovery income the Business record
//   carries (b.receipts(), as Schedule 1 line 6 and SE); a farm's expense categories print as
//   Schedule F Part II rows farm_<id>_expense_line<n>_<name> (EXPENSE_LINES: 15, 20, 21a, 21c,
//   22 wages + contract labor, 24a, 24b, 25, 28, 29, 30 tagged; advertising, commissions,
//   legal, office, travel, meals and labelled other on 32a–32e untagged), never the Schedule C
//   rows; standard-mileage car and truck is line 10. No total changed. Tests: src/tests/farm/farm.rs
//   farm_expenses_print_on_schedule_f_part_ii_lines_not_schedule_c,
//   farm_standard_mileage_is_schedule_f_line_10_not_schedule_c_line_9,
//   farm_crp_and_ccc_market_gain_are_schedule_f_line_4_and_ccc_sales_lines_1_and_2,
//   farm_fuel_tax_recovery_is_in_schedule_f_lines_9_and_34.
//   Farm defects (2026-10-08, 2026 drafts f1040sf created 4/24/26 and i1040sf of 9/17/26; f1040sc created
//   5/15/26): standard-mileage vehicles' business-use loan interest prints on its own Schedule F line 21b
//   (return line business_<id>_vehicle_loan_interest; Schedule C line 16b), line 10 holds the rest, the
//   total is unchanged, other interest is 21c (Schedule C 16c). A Form 1099-A whose balance equals a CCC
//   lot forfeited in 2026 is the CCC loan record's information return (debt.rs transfer_sale): section 77
//   loans are not counted again, loan-method forfeitures are the sale on 5b/5c, each with a NOTE_CCC_*.
//   Tests: src/tests/farm/farm.rs farm_vehicle_loan_interest_is_schedule_f_line_21b_with_totals_unchanged;
//   src/tests/business/business.rs schedule_c_vehicle_loan_interest_is_line_16b_and_other_interest_16c_with_totals_unchanged,
//   ccc_1099a_forfeiture_is_counted_once_with_or_without_the_section_77_election.
//   Unit B2 (2026-10-09): a farm rental (Form 4835, farm_rental.rs) joins the owner's farms in the
//   shared records: FarmForm names the printed lines of the CCC, crop-insurance, conservation and
//   land-sale reports, sources() lists both kinds, the section 175 pool, the one-method CCC
//   check and the farming-NOL test span both. Tests: src/tests/farm/farm_rental.rs.
//   Line audit QBI/F (unit F12, 2026-10-10): a standard-mileage vehicle's property tax is line 29 (line 10
//   holds the rest); the start-up deduction and amortization, the prepaid-supply deduction and a CCC paid
//   deficiency are rows farm_<id>_expense_line32_{startup_amortization,prepaid_supplies,ccc_deficiency}
//   so Part II adds to line 33 (src/tests/cross/line_audit_qbi_f.rs assert_schedule_f_part_ii_foots); an accrual
//   farm's crop insurance is the single Part III line 41, line 37 holds the net sale of a CCC-pledged
//   crop and its program income is lines 39a/39b; the section 175 pool (farm_conservation.rs), the
//   Schedule J farm income and the at-risk loss test read one farm figure that includes the routed
//   1099-G box 7 and 1099-PATR income (federal_rules.rs farm_receipts). Box 8 of the 1099-PATR is the only
//   QBI part of the distributions (nonqualified), and a box 7 patron files Form 8995-A with Schedule D line 6.
//   T2-45y (2026-09-27, 2026 draft i1040sf lines 11–31): ten farm-only BusinessExpenseCategory
//   variants (Chemicals 11, CustomHire 13, Feed 16, Fertilizers 17, Freight 18, Fuel 19,
//   EmployeePension 23 (employees' plans; the owner's own plan stays BusinessRetirement,
//   Schedule 1 line 16), Seeds 26, Storage 27, Veterinary 31) print tagged on their own lines;
//   the Farming page writes them instead of labelled Other. They are ordinary deductible
//   expenses through FederalBusinessExpense::total (profit, SE, QBI, section 179 income
//   limit). A non-farm business storing one is refused by name (federal_rules.rs, "Farm-only
//   expense line"). STORAGE_FORMAT 63. Tests: src/tests/farm/farm.rs
//   farm_expenses_print_on_schedule_f_part_ii_lines_not_schedule_c,
//   schedule_c_business_with_a_farm_only_expense_line_is_refused_by_name; src/tests/farm/interview.rs
//   farm_page_maps_every_branch_onto_the_stored_farm_and_back.

// Schedule F source facts; shared business stages own SE/QBI and deductions.
pub use crate::farm_nol_source::FarmNolElection;
use crate::federal_rules::farm_1099g_program;
use crate::line_emission::push_line;
use crate::{BusinessInventory, FederalBusiness, FederalDraft, FederalResult, Owner};
use crate::{FormLine, TaxForm};
use alloc::{format, vec::Vec};
use libertas_macros::*;

/// Farm income and accounting method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmIncome {
    /// Related accepted document-component identities
    pub related: Vec<i64>,
    /// Direct special source facts, original elections and retained history
    pub specials: crate::FarmSpecials,
    /// Canonical parcel opening/acquisition basis and exclusion history
    pub land_basis: Vec<crate::FarmLandBasis>,
    /// Schedule F header; absent only for the shared Form 4835 payload
    #[libertas_default($null)]
    pub header: Option<crate::FarmHeader>,
    /// Accounting records
    pub accounting: FarmAccounting,
    /// Crop-insurance and federal crop-disaster records
    pub insurance: crate::farm_insurance_paper::FarmInsuranceRecords,
    /// Source-keyed facts absent from related crop-insurance statements
    pub insurance_papers: crate::farm_insurance_paper::CropInsuranceFacts,
    /// CCC marketing-assistance loan records
    pub ccc: Option<crate::FarmCcc>,
    /// Conservation Reserve Program payments
    pub crp: Vec<crate::CrpPayment>,
    /// Cash-method prepaid production supplies and retained undeducted costs
    pub supplies: Option<crate::FarmSupplies>,
    /// Soil and water conservation election and retained costs
    pub conservation: Option<crate::FarmConservation>,
    /// Direct farm-land sale worksheets
    /// Enter closing and retained basis records. Do not repeat proceeds in farm receipts or investment sales.
    pub land_sales: Vec<crate::FarmLandSale>,
}

/// Farm accounting records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmAccounting {
    /// Cash method
    Cash {
        /// Farm receipts and purchased-product costs
        /// Record amounts received in 2026, including barter value. Do not duplicate a 1099 payment. Operating costs belong in business expense records.
        receipts: Vec<FarmReceipt>,
    },
    /// Accrual method
    /// Use consistent tax inventory valuations and recognized income/expense records for 2026, not cash collections. Include all required inventory classes, including raised products. Do not also enter Schedule C inventory or cash-method resale basis.
    Accrual {
        /// Sales recognized in 2026
        /// All livestock and products raised or purchased for sale; exclude equipment, land and depreciable livestock dispositions. Include earned receivables once, not again when collected. Exclude sales entered in CCC lot records; the App adds those separately.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        sales: i64,
        /// Opening farm inventory
        /// Retained prior-year closing tax valuation, including required raised and purchased products and all crops pledged for or redeemed from loan-method CCC advances. Exclude depreciable assets and costs already deducted outside inventory.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: i64,
        /// Closing farm inventory
        /// Use the same established permitted valuation method and all required inventory classes, including unsold crops pledged for or redeemed from loan-method CCC advances. Raised inventory may increase without purchased livestock/product costs; the App does not cap closing inventory at purchases plus opening inventory.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        closing: i64,
        /// Livestock and products purchased during 2026
        /// Costs recognized under the accrual method, including unsold products included in closing inventory. Do not repeat these costs in operating expenses or include depreciable asset purchases.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        purchases: i64,
        /// Custom hire and machine-work income recognized
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        custom_hire: i64,
        /// Fully taxable ordinary agricultural-program income recognized
        /// Exclude cooperative distributions, CCC loans, conservation-reserve payments, insurance and special subsidy treatments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        program: i64,
        /// Other ordinary farm operating income recognized
        /// Exclude separately reported rent, interest, asset gains, debt discharge and tax-benefit recoveries needing another worksheet.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        other: i64,
    },
}

/// Farm cash receipt
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmReceipt {
    /// Products or market livestock you raised
    /// Exclude sales of equipment, land, breeding/dairy livestock or other business assets.
    Raised {
        /// Proceeds received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Purchased livestock or products resold
    /// Include only products sold in 2026. Do not deduct unsold purchases or duplicate these costs as operating expenses.
    Resale {
        /// Proceeds received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Cost or other tax basis of the products sold
        /// Use paid purchase records, including a purchase in a prior year; exclude costs already deducted or recovered. This is basis, not a computed profit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        cost: i64,
    },
    /// Custom hire or machine work
    CustomHire {
        /// Receipts received for farm work
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Fully taxable ordinary agricultural-program payment
    /// Exclude conservation-reserve payments, cooperative distributions, commodity-credit loans and payments subject to exclusion or deferral.
    Program {
        /// Payment received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Cost-sharing payment under a conservation program eligible for the section 126 exclusion
    /// Enter the full payment received. The App reports it on line 4a but does not compute the excludable portion; a fully taxable payment belongs under agricultural program payment instead.
    CostSharing {
        /// Payment received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Other ordinary farm operating income
    /// Include incidental operating income, not rent, interest, debt discharge, asset gains or tax-benefit recoveries needing another worksheet.
    Other {
        /// Receipt received
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
impl FarmReceipt {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::Raised { amount }
            | Self::Resale { amount, .. }
            | Self::CustomHire { amount }
            | Self::Program { amount }
            | Self::CostSharing { amount }
            | Self::Other { amount } => *amount,
        }
    }
    fn cost(&self) -> i64 {
        match self {
            Self::Resale { cost, .. } => *cost,
            _ => 0,
        }
    }
}
impl FarmIncome {
    pub(crate) fn valid(&self) -> bool {
        self.specials_rule().is_none()
            && self.header.as_ref().is_none_or(|h| h.rule().is_none())
            && self.supplies.as_ref().is_none_or(crate::FarmSupplies::valid)
            && self.land_sales.len() <= 400
            && self.land_sales.iter().all(crate::FarmLandSale::valid)
            && self.conservation.as_ref().is_none_or(crate::FarmConservation::valid)
            && self.insurance.valid()
            && self.crp.len() <= 400
            && self.crp.iter().all(crate::CrpPayment::valid)
            && self.ccc.as_ref().is_none_or(crate::FarmCcc::valid)
            && match &self.accounting {
                FarmAccounting::Cash { receipts } => {
                    receipts.len() <= 400
                        && receipts.iter().all(|v| {
                            (0..=100000000000).contains(&v.amount())
                                && (0..=100000000000).contains(&v.cost())
                        })
                }
                FarmAccounting::Accrual {
                    sales,
                    opening,
                    closing,
                    purchases,
                    custom_hire,
                    program,
                    other,
                } => [sales, opening, closing, purchases, custom_hire, program, other]
                    .into_iter()
                    .all(|v| (0..=100000000000).contains(v)),
            }
    }
    pub(crate) fn gross(&self) -> i64 {
        let operating = match &self.accounting {
            FarmAccounting::Cash { receipts } => {
                receipts.iter().map(|v| v.amount() - v.cost()).sum()
            }
            FarmAccounting::Accrual {
                sales,
                opening,
                closing,
                purchases,
                custom_hire,
                program,
                other,
            } => sales + custom_hire + program + other + closing - opening - purchases,
        };
        operating
            + self
                .specials
                .income_adjustment(matches!(self.accounting, FarmAccounting::Cash { .. }))
            + self.crp.iter().map(|p| p.amount).sum::<i64>()
            + self.ccc.as_ref().map_or(0, crate::FarmCcc::income)
            + self.insurance.income(matches!(self.accounting, FarmAccounting::Cash { .. }))
    }
    // Whether every receipt of this farm is gross income derived from farming
    // (section 175(b)(1); Reg. 1.175-5: income from the production of crops,
    // fish, fruits, other agricultural products or livestock; the Schedule F
    // instructions, line 12: 25% of gross income from farming, excluding gains
    // from selling assets such as farm machinery and land). Products raised
    // or resold, ordinary program payments, a Commodity Credit Corporation
    // loan treated as income (section 77) and crop insurance proceeds
    // (section 451(f) treats them as the sale of the crop) are production
    // income and count in `gross()`. Custom hire, cost-sharing payments,
    // conservation-reserve payments and other operating income may be
    // need actual economic character. Pure custom-hire services are known
    // nonproduction income and stay out of this specific legal gross pool.
    pub(crate) fn conservation_source_ready(&self) -> bool {
        self.crp.is_empty()
            && self.specials.other_income.is_empty()
            && self.specials.cost_sharing.is_empty()
            && match &self.accounting {
                FarmAccounting::Cash { receipts } => receipts.iter().all(|v| {
                    matches!(
                        v,
                        FarmReceipt::Raised { .. }
                            | FarmReceipt::Resale { .. }
                            | FarmReceipt::Program { .. }
                            | FarmReceipt::CustomHire { .. }
                    ) || v.amount() == 0
                }),
                FarmAccounting::Accrual { other, .. } => *other == 0,
            }
    }
    // Distinct from Schedule J and section6654 gross income. Do not change
    // the ordinary receipt owner or reuse its total as legal production gross.
    pub(crate) fn conservation_gross(&self) -> Option<i64> {
        if !self.conservation_source_ready() {
            return None;
        }
        let services = match &self.accounting {
            FarmAccounting::Cash { receipts } => receipts
                .iter()
                .filter_map(|receipt| {
                    if let FarmReceipt::CustomHire { amount } = receipt {
                        Some(*amount)
                    } else {
                        None
                    }
                })
                .sum(),
            FarmAccounting::Accrual { custom_hire, .. } => *custom_hire,
        };
        Some(self.gross() - services)
    }

    pub(crate) fn report_with_section179_with_claims(
        &self,
        form_context: crate::related_forms::RelatedFormContext<'_>,
        b: &FederalBusiness,
        id: i64,
        r: &mut FederalResult,
        selected: Option<&crate::Section179PreparedAllocation>,

        home: Option<&crate::home_office::HomeContext>,
    ) -> Result<(), crate::Section179PreparedIssue> {
        let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
            form_context;
        let header_context = d
            .income
            .iter()
            .find(|entry| entry.id == id)
            .map(crate::interview::income_issue_context)
            .unwrap_or(crate::IssueContext::Section(crate::TaxSection::Farming));
        crate::farm_specials::report_header(self.header.as_ref(), id, header_context.clone(), r);
        self.specials.report(
            matches!(self.accounting, FarmAccounting::Cash { .. }),
            id,
            FarmForm::ScheduleF,
            header_context.clone(),
            r,
        );
        self.report_parcels_with_claims(d, id, FarmForm::ScheduleF, r, source_claims);
        self.report_property_history(&b.assets, header_context.clone(), r);

        let death = owner_death_date(d, id);
        for (index, sale) in self.land_sales.iter().enumerate() {
            crate::require_localized!(
                r,
                death.is_none_or(|date| sale.sold <= date),
                &crate::IssueContext::Section(crate::TaxSection::Farming),
                "Farm land final-return allocation",
                "ISSUE_A_SALE_AFTER_THIS_FARM_OWNER_S_DEATH_DOES_NOT_BELONG_TO_THE_OWNER_S_E146F4F3",
                "A sale after this farm owner's death does not belong to the owner's final individual return. Resolve the estate or beneficiary's ownership and reporting; a surviving spouse's own farm follows that spouse's dates.",
            );
            sale.report_with_claims(
                crate::related_forms::RelatedFormContext::original(d, source_claims),
                self,
                id,
                index,
                FarmForm::ScheduleF,
                r,
            );
        }
        if let Some(conservation) = &self.conservation {
            conservation.report_with_claims(
                self.gross()
                    + crate::method_change::increase(&b.method_changes)
                    + crate::farm_conservation::routed_to_with_claims(d, id, source_claims)?,
                d,
                id,
                FarmForm::ScheduleF,
                r,
                source_claims,
            );
        }
        if let Some(supplies) = &self.supplies {
            supplies.report_with_claims(d, b, id, r, home, source_claims);
        }
        crate::farm_crp::report(&self.crp, id, r);
        crate::farm_insurance_paper::resolved(d, id, source_claims)
            .unwrap_or_else(|| self.insurance.kernel())
            .report(
                matches!(self.accounting, FarmAccounting::Cash { .. }),
                id,
                FarmForm::ScheduleF,
                r,
            );
        if let Some(ccc) = &self.ccc {
            ccc.report(
                matches!(self.accounting, FarmAccounting::Cash { .. }),
                id,
                FarmForm::ScheduleF,
                r,
            );
        }
        // Form 1099-G box 7 agriculture-program payments for this owner are
        // routed to the lowest-id farm (`farm_1099g_program`) for SE tax and
        // QBI; include the same amount in this farm's own displayed gross
        // income and net profit so the per-farm breakdown agrees with those
        // totals without double-counting them (the totals add this amount
        // separately from `self.gross()`, which this function does not change).
        let owner = d
            .income
            .iter()
            .find(|e| e.id == id)
            .map_or(0, |e| usize::from(e.owner == crate::DocumentOwner::Spouses));
        let program_1099g = farm_1099g_program(d, owner, id, source_claims);
        // Form 1099-PATR distributions for this owner are routed the same way
        // (`farm_cooperative`): line 3a/3b of the lowest-id farm.
        let cooperative =
            crate::federal_rules::farm_cooperative_with_claims(d, owner, id, source_claims)?;
        // `b.receipts()` is this farm's gross income plus the line 8 fuel-tax
        // credit or recovery income the Business record carries, the same
        // amount Schedule 1 line 6 and Schedule SE use.
        let grants = crate::information_returns::farm_grants_with_claims(d, id, source_claims);
        let gross = b.receipts()
            + program_1099g
            + cooperative.taxable
            + grants
            + crate::farm_insurance_paper::income_delta(d, id, source_claims);
        let profit = gross
            - crate::section179_return::expenses_with_claims(
                d,
                id,
                b,
                selected,
                home,
                source_claims,
            )?;
        let cost_sharing: i64 = match &self.accounting {
            FarmAccounting::Cash { receipts } => receipts
                .iter()
                .filter(|v| matches!(v, FarmReceipt::CostSharing { .. }))
                .map(FarmReceipt::amount)
                .sum(),
            FarmAccounting::Accrual { .. } => 0,
        };
        crate::require_localized!(
            r,
            cost_sharing == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm program payments",
            "ISSUE_SCHEDULE_F_LINE_4B_THE_SECTION_126_EXCLUSION_FOR_A_CONSERVATION_COS_9EAC6444",
            "Schedule F line 4b: the section 126 exclusion for a conservation cost-sharing payment (the present-value computation of Pub 225 chapter 3 and its attached statement) is not calculated. Enter a payment that is fully taxable as an agricultural program payment instead.",
        );
        crate::require_localized!(
            r,
            matches!(b.inventory, BusinessInventory::None),
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm inventory",
            "ISSUE_FARM_PRODUCT_BASIS_AND_INVENTORY_ARE_ALREADY_ACCOUNTED_FOR_IN_THE_F_63EC9604",
            "Farm product basis and inventory are already accounted for in the farm records; do not also deduct Schedule C inventory costs.",
        );
        require_farm_depreciation(r, &b.assets);
        // CCC lot amounts that print on Part I lines 1a, 1b, 2, 4a and 4b.
        let [ccc_market, ccc_taxable_market, ccc_resale, ccc_basis, ccc_raised] =
            self.ccc.as_ref().map_or([0; 5], crate::FarmCcc::part_one);
        let crp: i64 = self.crp.iter().map(|p| p.amount).sum();
        let weather =
            self.specials.weather_lines(matches!(self.accounting, FarmAccounting::Cash { .. }));
        let mut rows = match &self.accounting {
            FarmAccounting::Cash { receipts } => alloc::vec![
                (
                    "raised_sales",
                    receipts
                        .iter()
                        .filter(|v| matches!(v, FarmReceipt::Raised { .. }))
                        .map(FarmReceipt::amount)
                        .sum::<i64>()
                        + ccc_raised
                        + weather[0],
                    msg!("EXPLAIN_FARM_N_RAISED_SALES")
                ),
                (
                    "resale_proceeds",
                    receipts
                        .iter()
                        .filter(|v| matches!(v, FarmReceipt::Resale { .. }))
                        .map(FarmReceipt::amount)
                        .sum::<i64>()
                        + ccc_resale
                        + weather[1],
                    msg!("EXPLAIN_FARM_N_RESALE_PROCEEDS")
                ),
                (
                    "resale_basis",
                    receipts.iter().map(FarmReceipt::cost).sum::<i64>() + ccc_basis,
                    msg!("EXPLAIN_FARM_N_RESALE_BASIS")
                ),
            ],
            FarmAccounting::Accrual {
                sales,
                opening,
                closing,
                purchases,
                custom_hire,
                program: _,
                other,
            } => alloc::vec![
                (
                    "accrual_sales",
                    // Line 37 also holds the sale of a CCC-pledged crop, net of
                    // the redemption payment that is the basis of a loan
                    // included in income (the lot's sale_net).
                    *sales + ccc_raised + ccc_resale - ccc_basis,
                    msg!("EXPLAIN_FARM_N_ACCRUAL_SALES")
                ),
                ("opening_inventory", *opening, msg!("EXPLAIN_FARM_N_OPENING_INVENTORY")),
                ("closing_inventory", *closing, msg!("EXPLAIN_FARM_N_CLOSING_INVENTORY")),
                ("product_purchases", *purchases, msg!("EXPLAIN_FARM_N_PRODUCT_PURCHASES")),
                (
                    "other_accrual_income",
                    custom_hire + other,
                    msg!("EXPLAIN_FARM_N_OTHER_ACCRUAL_INCOME")
                ),
            ],
        };
        let cash = matches!(self.accounting, FarmAccounting::Cash { .. });
        rows.extend([
            (
                "cooperative_distributions",
                cooperative.distributions,
                msg!("EXPLAIN_FARM_N_COOPERATIVE_DISTRIBUTIONS"),
            ),
            (
                "cooperative_taxable",
                cooperative.taxable,
                msg!("EXPLAIN_FARM_N_COOPERATIVE_TAXABLE"),
            ),
        ]);
        {
            // Lines 4a/4b on the cash method, Part III lines 39a/39b on the
            // accrual method (its recognized program income is the accrual
            // "program" amount).
            let program: i64 = match &self.accounting {
                FarmAccounting::Cash { receipts } => receipts
                    .iter()
                    .filter(|v| matches!(v, FarmReceipt::Program { .. }))
                    .map(FarmReceipt::amount)
                    .sum(),
                FarmAccounting::Accrual { program, .. } => *program,
            };
            rows.extend([
                (
                    "program_payments",
                    program
                        + cost_sharing
                        + program_1099g
                        + crp
                        + ccc_market
                        + self.specials.cost_sharing_gross(),
                    msg!("EXPLAIN_FARM_N_PROGRAM_PAYMENTS"),
                ),
                (
                    "program_taxable",
                    program
                        + program_1099g
                        + crp
                        + ccc_taxable_market
                        + self.specials.cost_sharing_income(),
                    msg!("EXPLAIN_FARM_N_PROGRAM_TAXABLE"),
                ),
            ]);
        }
        if cash {
            let resale_proceeds = rows
                .iter()
                .find(|(name, _, _)| *name == "resale_proceeds")
                .map_or(0, |(_, value, _)| *value);
            let resale_basis = rows
                .iter()
                .find(|(name, _, _)| *name == "resale_basis")
                .map_or(0, |(_, value, _)| *value);
            rows.push((
                "resale_net",
                resale_proceeds - resale_basis,
                msg!("EXPLAIN_FARM_N_RESALE_NET"),
            ));
        }
        let (custom, other) = match &self.accounting {
            FarmAccounting::Cash { receipts } => (
                receipts
                    .iter()
                    .filter(|v| matches!(v, FarmReceipt::CustomHire { .. }))
                    .map(FarmReceipt::amount)
                    .sum::<i64>(),
                receipts
                    .iter()
                    .filter(|v| matches!(v, FarmReceipt::Other { .. }))
                    .map(FarmReceipt::amount)
                    .sum::<i64>(),
            ),
            FarmAccounting::Accrual { custom_hire, other, .. } => (*custom_hire, *other),
        };
        rows.push(("custom_hire_income", custom, msg!("EXPLAIN_FARM_CUSTOM_HIRE")));
        rows.push((
            "other_income",
            other + grants + self.specials.other_income() + b.receipts() - self.gross(),
            msg!("EXPLAIN_FARM_OTHER_INCOME"),
        ));
        if let FarmAccounting::Accrual { opening, purchases, closing, .. } = self.accounting {
            rows.push((
                "accrual_gross_before_inventory",
                gross + opening + purchases - closing,
                msg!("EXPLAIN_FARM_ACCRUAL_SUBTOTAL"),
            ));
            rows.push((
                "product_cost",
                opening + purchases - closing,
                msg!("EXPLAIN_FARM_PRODUCT_COST"),
            ));
            rows.push(("accrual_gross_income", gross, msg!("EXPLAIN_FARM_N_GROSS_INCOME")));
        }
        rows.extend([
            ("gross_income", gross, msg!("EXPLAIN_FARM_N_GROSS_INCOME")),
            (
                "expenses",
                crate::section179_return::expenses_with_claims(
                    d,
                    id,
                    b,
                    selected,
                    home,
                    source_claims,
                )?,
                msg!("EXPLAIN_FARM_N_EXPENSES"),
            ),
            ("net_profit", profit, msg!("EXPLAIN_FARM_N_NET_PROFIT")),
        ]);
        rows.push((
            "depreciation",
            crate::federal_rules::business_depreciation_with_section179_with_claims(
                d,
                id,
                &b.assets,
                selected,
                home,
                source_claims,
            )?,
            msg!("EXPLAIN_FARM_N_DEPRECIATION"),
        ));
        for (name, value, note) in rows {
            // Schedule F lines: Part I cash receipts 1a, 1b and 2 (with CCC
            // commodity sales), cooperative distributions 3a/3b (Part III
            // 38a/38b on the accrual method), program payments 4a/4b with CRP
            // and CCC market gain (cash only), gross income 9,
            // depreciation 14, total expenses 33 and net profit 34, and the
            // accrual Part III lines 37, 45, 46 and 48. The combined
            // other-income row is not tagged.
            let line = match name {
                "custom_hire_income" => Some(if cash { "7" } else { "42" }),
                "other_income" => Some(if cash { "8" } else { "43" }),
                "accrual_gross_before_inventory" => Some("44"),
                "product_cost" => Some("49"),
                "accrual_gross_income" => Some("50"),
                "resale_proceeds" => Some("1a"),
                "resale_net" => Some("1c"),
                // Part III lines 37, 45, 46 and 48 (T2-45p, question F14).
                "accrual_sales" => Some("37"),
                "opening_inventory" => Some("45"),
                "product_purchases" => Some("46"),
                "closing_inventory" => Some("48"),
                "resale_basis" => Some("1b"),
                "raised_sales" => Some("2"),
                "cooperative_distributions" if cash => Some("3a"),
                "cooperative_taxable" if cash => Some("3b"),
                "cooperative_distributions" => Some("38a"),
                "cooperative_taxable" => Some("38b"),
                "program_payments" => Some(if cash { "4a" } else { "39a" }),
                "program_taxable" => Some(if cash { "4b" } else { "39b" }),
                "gross_income" => Some("9"),
                "depreciation" => Some("14"),
                "expenses" => Some("33"),
                "net_profit" => Some("34"),
                _ => None,
            };
            push_line(
                r,
                line.map(|line| FormLine::new(TaxForm::ScheduleF, line)),
                &format!("farm_{id}_{name}"),
                value,
                &["income.business.activity.farming"],
                "https://www.irs.gov/instructions/i1040sf",
                note,
            );
        }
        Ok(())
    }
}

// Pre-2018 farm equipment keeps its 150% or straight-line method (question F9).
// ThirtyNine is fixed straight-line real property, not farm equipment.
pub(crate) fn require_farm_depreciation(r: &mut FederalResult, assets: &[crate::BusinessAsset]) {
    crate::require_localized!(
        r,
        assets.iter().all(|a| {
            a.placed >= 20180101
                || matches!(
                    a.class,
                    crate::EquipmentClass::Fifteen
                        | crate::EquipmentClass::Twenty
                        | crate::EquipmentClass::ThirtyNine
                )
                || a.method != crate::EquipmentMethod::Regular
        }),
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "Farm depreciation",
        "ISSUE_PRE_2018_FARM_EQUIPMENT_REQUIRES_ITS_APPLICABLE_150_OR_STRAIGHT_LIN_D2ED857C",
        "Pre-2018 farm equipment requires its applicable 150% or straight-line depreciation method, not ordinary 200% declining balance.",
    );
}

// Schedule F Part II line of each `BusinessExpenseCategory`, in its `index()`
// order: (return-line suffix, printed line, explanation). The Schedule C
// categories go to the Schedule F line the i1040sf text gives them; the ten
// farm-only categories are their own lines. Categories without a Schedule F
// line of their own are "other expenses" on lines 32a–32e, one row each and
// untagged (which of the five lines is a layout choice). Wages and contract
// labor share line 22.
const EXPENSE_LINES: [(&str, Option<&str>, &str); crate::BusinessExpenseCategory::COUNT] = [
    ("line32_advertising", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_ADVERTISING"),
    ("line32_commissions_and_fees", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_COMMISSIONS_AND_FEES"),
    ("line22_labor_hired", Some("22"), "EXPLAIN_FARM_N_EXPENSE_LINE22_LABOR_HIRED"),
    (
        "line15_employee_benefit_programs",
        Some("15"),
        "EXPLAIN_FARM_N_EXPENSE_LINE15_EMPLOYEE_BENEFIT_PROGRAMS",
    ),
    ("line20_insurance", Some("20"), "EXPLAIN_FARM_N_EXPENSE_LINE20_INSURANCE"),
    ("line21a_mortgage_interest", Some("21a"), "EXPLAIN_FARM_N_EXPENSE_LINE21A_MORTGAGE_INTEREST"),
    ("line21c_other_interest", Some("21c"), "EXPLAIN_FARM_N_EXPENSE_LINE21C_OTHER_INTEREST"),
    ("line32_legal_and_professional", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_LEGAL_AND_PROFESSIONAL"),
    ("line32_office", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_OFFICE"),
    (
        "line23_pension_and_profit_sharing",
        Some("23"),
        "EXPLAIN_FARM_N_EXPENSE_LINE23_PENSION_AND_PROFIT_SHARING",
    ),
    (
        "line24a_rent_vehicles_equipment",
        Some("24a"),
        "EXPLAIN_FARM_N_EXPENSE_LINE24A_RENT_VEHICLES_EQUIPMENT",
    ),
    ("line24b_rent_other", Some("24b"), "EXPLAIN_FARM_N_EXPENSE_LINE24B_RENT_OTHER"),
    ("line25_repairs", Some("25"), "EXPLAIN_FARM_N_EXPENSE_LINE25_REPAIRS"),
    ("line28_supplies", Some("28"), "EXPLAIN_FARM_N_EXPENSE_LINE28_SUPPLIES"),
    ("line29_taxes", Some("29"), "EXPLAIN_FARM_N_EXPENSE_LINE29_TAXES"),
    ("line32_travel", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_TRAVEL"),
    ("line32_meals", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_MEALS"),
    ("line30_utilities", Some("30"), "EXPLAIN_FARM_N_EXPENSE_LINE30_UTILITIES"),
    ("line22_labor_hired", Some("22"), "EXPLAIN_FARM_N_EXPENSE_LINE22_LABOR_HIRED"),
    ("line32_other", None, "EXPLAIN_FARM_N_EXPENSE_LINE32_OTHER"),
    ("line11_chemicals", Some("11"), "EXPLAIN_FARM_N_EXPENSE_LINE11_CHEMICALS"),
    ("line13_custom_hire", Some("13"), "EXPLAIN_FARM_N_EXPENSE_LINE13_CUSTOM_HIRE"),
    ("line16_feed", Some("16"), "EXPLAIN_FARM_N_EXPENSE_LINE16_FEED"),
    (
        "line17_fertilizers_and_lime",
        Some("17"),
        "EXPLAIN_FARM_N_EXPENSE_LINE17_FERTILIZERS_AND_LIME",
    ),
    (
        "line18_freight_and_trucking",
        Some("18"),
        "EXPLAIN_FARM_N_EXPENSE_LINE18_FREIGHT_AND_TRUCKING",
    ),
    (
        "line19_gasoline_fuel_and_oil",
        Some("19"),
        "EXPLAIN_FARM_N_EXPENSE_LINE19_GASOLINE_FUEL_AND_OIL",
    ),
    ("line26_seeds_and_plants", Some("26"), "EXPLAIN_FARM_N_EXPENSE_LINE26_SEEDS_AND_PLANTS"),
    (
        "line27_storage_and_warehousing",
        Some("27"),
        "EXPLAIN_FARM_N_EXPENSE_LINE27_STORAGE_AND_WAREHOUSING",
    ),
    (
        "line31_veterinary_breeding_and_medicine",
        Some("31"),
        "EXPLAIN_FARM_N_EXPENSE_LINE31_VETERINARY_BREEDING_AND_MEDICINE",
    ),
];

// The Schedule F line explanation of one expense category.
pub(crate) fn expense_explanation(category: &crate::BusinessExpenseCategory) -> crate::Message {
    crate::Message::new(EXPENSE_LINES[category.index()].2, alloc::vec![])
}

// One farm's Part II rows from its business expense records, one per
// Schedule F line (categories sharing a line are added together).
pub(crate) fn expense_lines(r: &mut FederalResult, id: i64, b: &FederalBusiness) {
    let mut totals = crate::FederalBusinessExpense::category_totals(&b.expenses);
    if let crate::BusinessActivity::Farming { data } = &b.activity {
        totals[crate::BusinessExpenseCategory::Fertilizers.index()] +=
            data.specials.fertilizer_deduction();
    }
    if let crate::BusinessActivity::Farming { data } = &b.activity {
        for (category, amount) in crate::farm_production::category_totals(
            &data.specials,
            matches!(data.accounting, crate::FarmAccounting::Cash { .. }),
        ) {
            totals[category.index()] += amount;
        }
    }
    let mut rows: Vec<(&str, Option<&str>, &str, i64)> = Vec::new();
    for ((suffix, line, explanation), amount) in EXPENSE_LINES.into_iter().zip(totals) {
        match rows.iter_mut().find(|row| row.0 == suffix) {
            Some(row) => row.3 += amount,
            None => rows.push((suffix, line, explanation, amount)),
        }
    }
    // Line 29 also holds the business share of a standard-mileage vehicle's
    // personal property tax (2026 i1040sf line 10); line 10 holds the rest.
    if let Some(row) = rows.iter_mut().find(|row| row.0 == "line29_taxes") {
        row.3 += crate::BusinessMileage::property_tax(&b.vehicles);
    }
    // Lines 32a-32f also hold the start-up cost deduction and amortization
    // and the section 197 amortization (2026 i1040sf lines 32a-32f).
    rows.push((
        "line32_startup_amortization",
        None,
        "EXPLAIN_FARM_N_EXPENSE_LINE32_STARTUP_AMORTIZATION",
        b.amortization_cents(),
    ));
    if let crate::BusinessActivity::Farming { data } = &b.activity {
        let capital = crate::farm_production::uniform_capital(
            &data.specials,
            matches!(data.accounting, crate::FarmAccounting::Cash { .. }),
        );
        rows.push((
            "line32f_production_capitalized",
            Some("32f"),
            "EXPLAIN_FARM_PRODUCTION_CAPITALIZED",
            -capital,
        ));
    }
    for (suffix, line, explanation, amount) in rows {
        if amount != 0 {
            push_line(
                r,
                line.map(|line| FormLine::new(TaxForm::ScheduleF, line)),
                &format!("farm_{id}_expense_{suffix}"),
                amount,
                &["income.business.expenses"],
                "https://www.irs.gov/instructions/i1040sf",
                crate::Message::new(explanation, alloc::vec![]),
            );
        }
    }
}

/// How the 2026 farming loss meets the section 172(b)(1)(B) two-year carryback.
/// Keeping the carryback changes no 2026 line: the refund for 2024 and 2025 is
/// claimed on Form 1045 or Form 1040-X, and only the carryforward it leaves is
/// used later (Form 172 instructions), so the estimate notes it instead of refusing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FarmLossRoute {
    NotNeeded,
    Waived,
    CarriedBack,
    Missing,
}

// The ordinary loss is deductible without a farm carryback worksheet when neither
// regular nor alternative-tax NOL arises. Do not turn every Schedule F loss into a blocker.
//
// `rentals` is the allowed income or loss of the Form 4835 farm rentals in
// cents, regular and alternative minimum tax: a farm rental loss is a farming
// loss (section 172(b)(1)(B)), so a return whose NOL includes it still needs
// the explicit choice.

pub(crate) fn check_nol_with_section179_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    regular_nol: i64,
    alternative_nol: i64,
    rentals: [i64; 2],
    selected_pair: Option<&[crate::section179_return::Selected; 2]>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<FarmLossRoute, crate::Section179PreparedIssue> {
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    let selected = selected_pair.map(|pair| &pair[0].allocation);
    let mut regular = rentals[0];
    let mut alternative = rentals[1];
    for e in &d.income {
        if let crate::FederalIncome::Business { data } = &e.income
            && matches!(data.activity, crate::BusinessActivity::Farming { .. })
        {
            let owner = usize::from(e.owner == crate::DocumentOwner::Spouses);
            let profit = data.receipts()
                + crate::federal_rules::farm_routed_income_with_claims(
                    d,
                    owner,
                    e.id,
                    source_claims,
                )?
                - crate::section179_return::expenses_with_claims(
                    d,
                    e.id,
                    data,
                    selected,
                    home,
                    source_claims,
                )?;
            regular += profit;
            let existing = crate::depreciation::business_amt_adjustment(
                data,
                crate::depreciation::mid_quarter(d),
            ) + crate::farm_conservation::business_amt_adjustment_with_claims(
                d,
                e.id,
                source_claims,
            ) + crate::farm_supplies::business_amt_adjustment_with_claims(
                d,
                e.id,
                source_claims,
            ) + crate::farm_production::business_amt_adjustment(d, e.id);
            alternative +=
                profit + crate::section179_return::amt_adjustment(e.id, existing, selected_pair)?;
        }
    }
    let needed = regular < 0 && regular_nol > 0 || alternative < 0 && alternative_nol > 0;
    let election = d.screening.as_ref().and_then(|s| s.farm_nol.as_ref());
    let route = match election {
        _ if !needed => FarmLossRoute::NotNeeded,
        Some(FarmNolElection::Waive) => FarmLossRoute::Waived,
        Some(FarmNolElection::Carryback) => FarmLossRoute::CarriedBack,
        None => FarmLossRoute::Missing,
    };
    crate::require_localized!(
        r,
        route != FarmLossRoute::Missing,
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "Farm loss",
        "ISSUE_A_FARMING_NOL_REQUIRES_AN_EXPLICIT_VALID_CARRYBACK_WAIVER_OR_THE_SE_96CD586D",
        "A farming NOL requires an explicit carryback choice: keep the two-year carryback or waive it. A missing choice never becomes an automatic waiver.",
    );
    Ok(route)
}

// Use the linked business owner, never any death on the joint return.
pub(crate) fn owner_death_date(d: &FederalDraft, id: i64) -> Option<u32> {
    let entry = d.income.iter().find(|e| e.id == id)?;
    let people = d.people.as_ref()?;
    people.person(entry.owner.owned())?.death_date()
}

// Section 77(b): one current-year CCC method for every farm and farm rental an
// owner has.
pub(crate) fn require_same_ccc_method(
    d: &FederalDraft,
    owner: Owner,
    ccc: &crate::FarmCcc,
    r: &mut FederalResult,
) {
    crate::require_localized!(
        r,
        sources(d)
            .filter(|other| other.entry.owner == crate::DocumentOwner::from(owner))
            .all(|other| other.farm.ccc.as_ref().is_none_or(|v| v.method == ccc.method)),
        &crate::IssueContext::Section(crate::TaxSection::Farming),
        "CCC loan election",
        "ISSUE_USE_THE_SAME_CURRENT_YEAR_CCC_METHOD_FOR_EVERY_FARM_OWNED_BY_THIS_T_C9BBF517",
        "Use the same current-year CCC method for every farm owned by this taxpayer.",
    );
}

/// The printed form of a farm's report: Schedule F or, for a farm rental,
/// Form 4835. The shared CCC, crop-insurance, conservation and land-sale
/// reports name their lines and inputs through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FarmForm {
    ScheduleF,
    Form4835,
}
impl FarmForm {
    pub(crate) fn tax_form(self) -> TaxForm {
        match self {
            Self::ScheduleF => TaxForm::ScheduleF,
            Self::Form4835 => TaxForm::Form4835,
        }
    }
    // The input path of the whole stored farm record.
    pub(crate) fn root(self) -> &'static str {
        match self {
            Self::ScheduleF => "income.business.activity.farming",
            Self::Form4835 => "income.farm_rental.farm",
        }
    }
    // The input path of one stored farm record.
    pub(crate) fn input(self, tail: &str) -> alloc::string::String {
        match self {
            Self::ScheduleF => format!("income.business.activity.farming.{tail}"),
            Self::Form4835 => format!("income.farm_rental.farm.{tail}"),
        }
    }
}

/// One record of farm income on the return: a Schedule F farm or a Form 4835
/// farm rental.
pub(crate) struct FarmSource<'a> {
    pub entry: &'a crate::FederalIncomeEntry,
    pub farm: &'a FarmIncome,
    /// A Form 4835 farm rental.
    pub rental: bool,
}

impl FarmSource<'_> {
    /// Gross income from farming for the section 175(b) limit, in cents: the
    /// farm's own gross income plus a Schedule F farm's section 481(a)
    /// increase (F3, 2026-10-09). The fuel-tax recovery is not income from
    /// producing farm products and stays out.
    pub(crate) fn gross(&self) -> i64 {
        self.farm.gross()
            + match &self.entry.income {
                crate::FederalIncome::Business { data } => {
                    crate::method_change::increase(&data.method_changes)
                }
                _ => 0,
            }
    }
}

// Every farm and farm rental of the return, in entry order.
pub(crate) fn sources(d: &FederalDraft) -> impl Iterator<Item = FarmSource<'_>> {
    d.income.iter().filter_map(|entry| match &entry.income {
        crate::FederalIncome::Business { data } => match &data.activity {
            crate::BusinessActivity::Farming { data } => {
                Some(FarmSource { entry, farm: data, rental: false })
            }
            _ => None,
        },
        crate::FederalIncome::FarmRental { data } => {
            Some(FarmSource { entry, farm: &data.farm, rental: true })
        }
        _ => None,
    })
}

// The farm or farm rental holding the CCC loan lot a Form 1099-A names by the
// lot's record identity, with the lot's position in that farm's loan list.
// Identities are assigned when a farm is saved and are unique on the return, so
// editing a farm's loan list never changes which lot a stored key names.
pub(crate) fn ccc_lot(d: &FederalDraft, key: i64) -> Option<(FarmSource<'_>, usize)> {
    if key <= 0 {
        return None;
    }
    sources(d).find_map(|source| {
        let index = source.farm.ccc.as_ref()?.lots.iter().position(|lot| lot.id == key)?;
        Some((source, index))
    })
}
// The identities of the CCC loan lots in a stored income record, for the
// save that assigns them.
pub(crate) fn ccc_lot_ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    let farm = match income {
        crate::FederalIncome::Business { data } => match &mut data.activity {
            crate::BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        crate::FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    };
    farm.and_then(|farm| farm.ccc.as_mut())
        .map(|ccc| ccc.lots.iter_mut().map(|lot| &mut lot.id).collect())
        .unwrap_or_default()
}
pub(crate) fn ccc_lot_ids(income: &crate::FederalIncome) -> Vec<i64> {
    let farm = match income {
        crate::FederalIncome::Business { data } => match &data.activity {
            crate::BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        crate::FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    };
    farm.and_then(|farm| farm.ccc.as_ref())
        .map(|ccc| ccc.lots.iter().map(|lot| lot.id).collect())
        .unwrap_or_default()
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmIncome {
    amounts: [
        specials,
        land_basis,
        accounting,
        insurance,
        ccc,
        crp,
        supplies,
        conservation,
        land_sales
    ],
    gains: [],
    facts: [header, related, insurance_papers],
});

crate::community::divide_enum!(FarmAccounting {
    records: [
        Cash { amounts: [receipts], facts: [] },
        Accrual {
            amounts: [sales, opening, closing, purchases, custom_hire, program, other],
            facts: []
        }
    ],
    units: [],
    tuples: [],
});

crate::community::divide_enum!(FarmReceipt {
    records: [
        Raised { amounts: [amount], facts: [] },
        Resale { amounts: [amount, cost], facts: [] },
        CustomHire { amounts: [amount], facts: [] },
        Program { amounts: [amount], facts: [] },
        CostSharing { amounts: [amount], facts: [] },
        Other { amounts: [amount], facts: [] }
    ],
    units: [],
    tuples: [],
});
