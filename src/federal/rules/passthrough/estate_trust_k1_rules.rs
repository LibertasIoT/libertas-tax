// estate_trust_k1 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn estate_final_nol(d: &FederalDraft) -> Option<(i64, &FinalEstateTrustNol)> {
    d.income.iter().find_map(|entry| match &entry.income {
        FederalIncome::EstateTrustPortfolio { data } => {
            data.statements.final_nol.as_ref().map(|nol| (entry.id, nol))
        }
        _ => None,
    })
}

pub(super) fn collect_estate_trust_portfolio(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::EstateTrustSource,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    // These calculation issues concern the retained K-1 treatment, not its printed boxes.
    let context = context.clone().corrective(crate::TaxSection::PassThrough);
    #[cfg(test)]
    let source = src.source;
    let data = &crate::FederalEstateTrustPortfolioK1::from(data);
    if let Some(original) = d.income.iter().find(|entry| entry.id == e.id)
        && let Some(key) = crate::estate_trust::community_source_rule(d, original)
    {
        crate::issue_localized!(
            r,
            &context,
            &source,
            key,
            "This fiduciary adjustment needs its actual income and property-gain components before community allocation; a complete federal estimate cannot assume their attribution.",
        );
    }
    crate::require_localized!(
        r,
        data.complete_portfolio_only,
        &context,
        &source,
        "ISSUE_ESTATE_OTHER_SOURCE_REQUIRED",
        "This K-1 has unclassified income, foreign-beneficiary or inconsistent-return information. Sections 652 and 662 require its actual income character; known portfolio income is retained but a complete federal estimate needs the owning worksheet.",
    );
    for item in &data.unentered {
        if let Some(key) = item.refusal_key() {
            crate::issue_localized!(r, &context, &source, key, item.refusal().unwrap(),);
        }
    }
    crate::require_localized!(
        r,
        data.section199a_dividends.amount() == 0
            || (data.code_i_dividends_only
                && data.section199a_dividends.qualified() == data.section199a_dividends.amount()),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_BOX_14_CODE_I_STATEMENT_ALLOCATES_ONLY_QUALIFIED_SECTIO_E9402562",
        "Confirm the box 14 code I statement allocates only qualified section 199A REIT/RIC dividends to this beneficiary. Business, PTP, cooperative or other section 199A items need separate worksheets; an amount marked ineligible needs a corrected fiduciary statement.",
    );
    crate::require_localized!(
        r,
        data.box_10_interest_ird_estate_tax == 0
            || (data.box_10_interest_ird_only
                && data.interest > 0
                && data.niit_adjustment.is_none()
                && d.credits.as_ref().is_none_or(|credit| {
                    matches!(
                        &credit.foreign_tax,
                        crate::ForeignTaxClaim::No | crate::ForeignTaxClaim::Deduction { .. }
                    )
                })),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_ENTIRE_K_1_BOX_10_ESTATE_TAX_DEDUCTION_IS_ALLOCATED_BY_311E89D1",
        "Confirm the entire K-1 box 10 estate-tax deduction is allocated by the fiduciary to taxable IRD interest included in this K-1's box 1, without a box 14 code H adjustment that may already include it or a Form 1116 foreign-tax-credit claim. Mixed, noninvestment or capital-gain IRD needs a separate allocation and rate worksheet.",
    );
    crate::require_localized!(
        r,
        (data.final_nol.is_none()
            && data.final_67e_deduction == 0
            && data.final_personal_property_tax == 0
            && data.final_real_estate_tax == 0
            && data.final_state_income_tax == 0
            && data.final_state_sales_tax == 0
            && data.final_short_carry == 0
            && data.final_long_carry == 0)
            || data.final_successor(),
        &context,
        &source,
        "ISSUE_CONFIRM_A_FINAL_2026_FORM_1041_AND_THAT_THIS_BENEFICIARY_SUCCEEDS_T_5C3A8A61",
        "Confirm a final 2026 Form 1041 and that this beneficiary succeeds to the estate's or trust's property before claiming box 11 codes A through F.",
    );
    // The inherited NOL joins the return's NOL deduction, which
    // Form 1116 already takes on its own line (i1116, line 3b).
    if let Some(nol) = &data.final_nol {
        for (code, amount) in [
            ("11e_regular_nol", nol.years.iter().map(|year| year.regular_balance).sum()),
            ("11f_amt_nol", nol.years.iter().map(|year| year.alternative_balance).sum()),
        ] {
            if amount != 0 {
                line_cents(
                    r,
                    &alloc::format!("estate_trust_{}_{}", e.id, code),
                    amount,
                    &["income.estate_trust_portfolio.final_nol"],
                    "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
                    msg!("EXPLAIN_ESTATE_TRUST_N_N"),
                );
            }
        }
    }
    crate::require_localized!(
        r,
        (data.final_personal_property_tax == 0
            && data.final_real_estate_tax == 0
            && data.final_state_income_tax == 0
            && data.final_state_sales_tax == 0)
            || (data.final_taxes_only
                // Property and sales taxes join the itemized
                // deductions Form 1116 apportions on line 3a. An
                // elected state income tax is allocated by the
                // income it was paid on (i1116, line 3a state
                // and local taxes), which the beneficiary's own
                // state-tax records do not show for the estate's.
                && (data.final_state_income_tax == 0
                    || d.deductions
                        .as_ref()
                        .is_none_or(|v| !v.state_income_tax_elected())
                    || d.credits.as_ref().is_none_or(|credit| {
                        matches!(
                            &credit.foreign_tax,
                            crate::ForeignTaxClaim::No
                                | crate::ForeignTaxClaim::Deduction { .. }
                        )
                    }))),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_ENTIRE_FINAL_YEAR_BOX_11_CODE_B_AMOUNT_IS_ALLOCATED_STA_C1AF69AE",
        "Confirm the entire final-year box 11 code B amount is allocated state/local income or general sales tax without a Form 8960 share, noninvestment real-estate tax or nonbusiness personal-property tax. With a Form 1116 credit, an elected state income tax from the fiduciary needs the income it was paid on, which your state-tax records do not show.",
    );
    crate::require_localized!(
        r,
        data.final_state_income_tax == 0 || data.final_state_sales_tax == 0,
        &context,
        &source,
        "ISSUE_THE_SAME_FIDUCIARY_S_FINAL_YEAR_EXCESS_DEDUCTION_CANNOT_CONTAIN_BOT_34D28E4D",
        "The same fiduciary's final-year excess deduction cannot contain both state/local income tax and the elected general sales tax. Obtain a corrected source classification.",
    );
    crate::require_localized!(
        r,
        (data.final_short_carry == 0 && data.final_long_carry == 0)
            || data.final_character.is_some()
            || data.final_carry_same_tax_character,
        &context,
        &source,
        "ISSUE_BOX_11_CODES_C_D_MUST_RETAIN_THE_SAME_REGULAR_AMT_INVESTMENT_PROPER_536D3126",
        "Box 11 codes C/D must retain the same regular, AMT, investment-property and net-investment-income tax character; differences need separate worksheets.",
    );
    crate::require_localized!(
        r,
        // Form 1116 takes the section 67(e) adjustment with the
        // other Schedule 1 Part II adjustments on line 3b.
        data.final_67e_deduction == 0
            || data.final_character.is_some()
            || data.final_67e_no_niit_allocation,
        &context,
        &source,
        "ISSUE_CONFIRM_THE_ALLOCATED_BOX_11_CODE_A_SECTION_67_E_DEDUCTION_HAS_NO_N_30B57AA3",
        "Confirm the allocated box 11 code A section 67(e) deduction has no net-investment-income share, or enter that share in the final-year source character.",
    );
    // Source confirmations describe the whole fiduciary statement.
    // A valid component can project to zero on this return without
    // changing the accepted statement or its confirmation facts.
    let amt_source = d
        .income
        .iter()
        .find_map(|entry| {
            if entry.id == e.id
                && let FederalIncome::EstateTrustPortfolio { data } = &entry.income
            {
                Some(crate::FederalEstateTrustPortfolioK1::from(data))
            } else {
                None
            }
        })
        .unwrap_or_else(|| data.clone());
    // K-1 code A enters Form 6251 line 2j once. Codes B/C/D
    // identify disjoint rate and capital-gain components of A;
    // E/F only identify disjoint special-rate portions within D.
    let components = [amt_source.amt_box12b, amt_source.amt_box12c, amt_source.amt_box12d];
    let special = [amt_source.amt_box12e, amt_source.amt_box12f];
    let no_components = components.iter().chain(special.iter()).all(|v| *v == 0);
    let no_component_confirmations = !amt_source.amt_box12_components_confirmed
        && !amt_source.amt_box12e_included_in_d
        && !amt_source.amt_box12f_included_in_d;
    let no_foreign_credit = d.credits.as_ref().is_none_or(|credit| {
        matches!(
            &credit.foreign_tax,
            crate::ForeignTaxClaim::No | crate::ForeignTaxClaim::Deduction { .. }
        )
    });
    let sourced_reconciliation = amt_source.amt_reconciliation.is_some();
    let component_reconciliation = amt_source.amt_box12a != 0
        && amt_source.amt_box12_components_confirmed
        && !amt_source.amt_box12a_standalone
        && components.iter().sum::<i64>() == amt_source.amt_box12a
        && components.iter().all(|v| *v == 0 || v.signum() == amt_source.amt_box12a.signum())
        && ((amt_source.amt_box12e == 0 && !amt_source.amt_box12e_included_in_d)
            || (amt_source.amt_box12e != 0 && amt_source.amt_box12e_included_in_d))
        && ((amt_source.amt_box12f == 0 && !amt_source.amt_box12f_included_in_d)
            || (amt_source.amt_box12f != 0 && amt_source.amt_box12f_included_in_d))
        && special.iter().all(|v| *v == 0 || v.signum() == amt_source.amt_box12d.signum())
        && special.iter().map(|v| v.abs()).sum::<i64>() <= amt_source.amt_box12d.abs()
        && no_foreign_credit;
    crate::require_localized!(
        r,
        (no_components
            && no_component_confirmations
            && if amt_source.amt_box12a_standalone {
                no_foreign_credit
            } else {
                amt_source.amt_box12a == 0
            })
            || component_reconciliation
            || sourced_reconciliation,
        &context,
        &source,
        "ISSUE_CONFIRM_BOX_12_CODE_A_EITHER_STANDS_ALONE_OR_EXACTLY_EQUALS_THE_SUM_6EA75E82",
        "Confirm box 12 code A either stands alone or exactly equals the sum of its signed code B qualified-dividend, C short-term and D long-term components. Every nonzero component must have A's sign; confirmed codes E/F are disjoint same-sign portions of D, not additional AMTI. Other box 12 codes, missing or contradictory fiduciary attribution, related AMT items and Form 1116 foreign tax credits need separate worksheets.",
    );
    crate::require_localized!(
        r,
        sourced_reconciliation
            || amt_source.amt_box12c == 0
            || amt_source.short_gain + amt_source.amt_box12c >= 0,
        &context,
        &source,
        "ISSUE_THE_NARROW_BOX_12_CODE_C_PATH_REQUIRES_A_NONNEGATIVE_BENEFICIARY_AM_18D09457",
        "The narrow box 12 code C path requires a nonnegative beneficiary AMT short-term gain after applying the signed adjustment to box 3. A fiduciary AMT short-term loss needs its own allocation and carryover statement.",
    );
    crate::require_localized!(
        r,
        sourced_reconciliation
            || amt_source.amt_box12d == 0
            || (amt_source.long_gain + amt_source.amt_box12d >= 0
                && (amt_source.rate_28_gain == 0 || amt_source.amt_box12f != 0)
                && (amt_source.unrecaptured_gain == 0 || amt_source.amt_box12e != 0)),
        &context,
        &source,
        "ISSUE_THE_NARROW_BOX_12_CODE_D_PATH_REQUIRES_A_NONNEGATIVE_BENEFICIARY_AM_D8813A7E",
        "The narrow box 12 code D path requires a nonnegative beneficiary AMT long-term gain after applying the signed adjustment to box 4a. A 28%-rate gain needs matching code F; section 1250 gain needs matching code E. Other capital-character allocations need separate worksheets.",
    );
    crate::require_localized!(
        r,
        sourced_reconciliation
            || amt_source.amt_box12e == 0
            || (amt_source.unrecaptured_gain + amt_source.amt_box12e >= 0
                && amt_source.rate_28_gain
                    + amt_source.amt_box12f
                    + amt_source.unrecaptured_gain
                    + amt_source.amt_box12e
                    <= amt_source.long_gain + amt_source.amt_box12d),
        &context,
        &source,
        "ISSUE_THE_BOX_12_CODE_E_PATH_REQUIRES_NONNEGATIVE_AMT_UNRECAPTURED_SECTIO_91F4EE61",
        "The box 12 code E path requires nonnegative AMT unrecaptured section 1250 gain after adjustment, with both adjusted special-rate pools within adjusted code D long-term gain.",
    );
    crate::require_localized!(
        r,
        sourced_reconciliation
            || amt_source.amt_box12f == 0
            || (amt_source.rate_28_gain + amt_source.amt_box12f >= 0
                && amt_source.rate_28_gain
                    + amt_source.amt_box12f
                    + amt_source.unrecaptured_gain
                    + amt_source.amt_box12e
                    <= amt_source.long_gain + amt_source.amt_box12d
                && (amt_source.unrecaptured_gain == 0 || amt_source.amt_box12e != 0)),
        &context,
        &source,
        "ISSUE_THE_BOX_12_CODE_F_PATH_REQUIRES_NONNEGATIVE_AMT_28_RATE_GAIN_AFTER_17658F8F",
        "The box 12 code F path requires nonnegative AMT 28%-rate gain after adjustment, with both adjusted special-rate pools within adjusted code D long-term gain. Unrecaptured section 1250 gain also needs confirmed code E.",
    );
    crate::require_localized!(
        r,
        data.royalty_income == 0 || data.box_5_royalty_only,
        &context,
        &source,
        "ISSUE_CONFIRM_THE_ENTIRE_K_1_BOX_5_AMOUNT_IS_ORDINARY_DOMESTIC_NONBUSINES_49D1F6DD",
        "Confirm the entire K-1 box 5 amount is ordinary domestic nonbusiness royalties, with no unentered net-investment-income adjustment. Any limited box 9 code A depreciation, B cost depletion or C amortization below must belong to the same activity; annuities, income in respect of a decedent and other box 5 items need their own calculations.",
    );
    crate::require_localized!(
        r,
        (data.box_9_royalty_depreciation == 0 && !data.box_9_royalty_sound_recording_only)
            || (data.box_9_royalty_depreciation > 0
                && data.box_9_royalty_depreciation <= data.royalty_income
                && data.box_9_royalty_depletion == 0
                && data.box_9_royalty_amortization == 0
                && data.box_5_royalty_only
                && data.box_9_royalty_sound_recording_only
                && !data.final_successor()
                && data.niit_adjustment.is_none()),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_FIDUCIARY_S_COMPLETE_BOX_9_CODE_A_ALLOCATION_IS_SECTION_796AF61F",
        "Confirm the fiduciary's complete box 9 code A allocation is section 167 straight-line depreciation of one master sound recording licensed for box 5 domestic nonbusiness royalties, held by the fiduciary throughout 2026. Retain the per-asset basis, service-date, useful-life and beneficiary-allocation statement. The schedule began before 2026, with no section 181 expense, section 168(k) bonus, section 197 amortization or income-forecast look-back. The deduction must remain fully allowable and nonloss with identical regular, AMT and NIIT treatment, no box 9 B/C, box 12 code G or box 14 code H overlap. Other assets, methods, property succession or separate beneficiary limits need their own worksheets.",
    );
    crate::require_localized!(
        r,
        (data.box_9_royalty_depletion == 0 && !data.box_9_royalty_cost_depletion_only)
            || (data.box_9_royalty_depletion > 0
                && data.box_9_royalty_depletion <= data.royalty_income
                && data.box_9_royalty_depreciation == 0
                && data.box_9_royalty_amortization == 0
                && data.box_5_royalty_only
                && data.box_9_royalty_cost_depletion_only
                && !data.final_successor()
                && data.niit_adjustment.is_none()),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_FIDUCIARY_S_COMPLETE_BOX_9_CODE_B_ALLOCATION_IS_COST_DE_735381DF",
        "Confirm the fiduciary's complete box 9 code B allocation is cost depletion from one domestic nonbusiness mineral royalty property in box 5, held by the fiduciary throughout 2026. Retain its per-property section 611 basis, recoverable/sold-unit calculation and beneficiary allocation statement. Cost must be the allowable depletion method. The amount must remain fully allowable and nonloss for this beneficiary, with identical regular, AMT and NIIT treatment and no box 9 code C, box 12 code H or box 14 code H overlap. Percentage depletion, property succession or a separate beneficiary-level limit needs its own worksheet.",
    );
    crate::require_localized!(
        r,
        (data.box_9_royalty_amortization == 0 && !data.box_9_royalty_amortization_only)
            || (data.box_9_royalty_amortization > 0
                && data.box_9_royalty_amortization <= data.royalty_income
                && data.box_9_royalty_depreciation == 0
                && data.box_9_royalty_depletion == 0
                && data.box_5_royalty_only
                && data.box_9_royalty_amortization_only
                && data.niit_adjustment.is_none()),
        &context,
        &source,
        "ISSUE_CONFIRM_THE_FIDUCIARY_S_COMPLETE_BOX_9_CODE_C_ALLOCATION_IS_SECTION_0753B936",
        "Confirm the fiduciary's complete box 9 code C allocation is section 197 amortization from the same single domestic nonbusiness royalty activity in box 5, on a straight-line schedule begun before 2026. The amount must remain fully allowable and nonloss for this beneficiary, have identical regular, AMT and NIIT treatment, and not appear in box 5 or box 14 code H again. Other box 9 items, box 12 code I or a separate beneficiary limit need their own worksheets.",
    );
    if data.gross_farming_fishing_income != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_14f_gross_farming_fishing_income", e.id),
            data.gross_farming_fishing_income,
            &["income.estate_trust_portfolio"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_N_14F_GROSS_FARMING_FISHING_INCOME"),
        );
    }
    t.interest += data.interest;
    t.exempt += data.tax_exempt_interest;
    t.private_activity += data.private_activity_interest;
    if data.private_activity_interest != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_14a_private_activity", e.id),
            data.private_activity_interest,
            &["income.estate_trust_portfolio.private_activity_interest"],
            "https://www.irs.gov/pub/irs-dft/f6251--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_14A_PRIVATE_ACTIVITY"),
        );
    }
    t.dividends += data.ordinary_dividends;
    t.qualified += data.qualified_dividends;
    t.reit_dividends += data.section199a_dividends.qualified();
    t.retain_reit_owner(e, data.section199a_dividends.qualified());
    if data.section199a_dividends.amount() != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_14i_section199a_dividends", e.id),
            data.section199a_dividends.amount(),
            &["income.estate_trust_portfolio.section199a_dividends"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_14I_SECTION199A_DIVIDENDS"),
        );
    }
    t.add_short(data.short_gain);
    t.add_long(data.long_gain);
    t.schedule_d_k1[0] += data.short_gain;
    t.schedule_d_k1[1] += data.long_gain;
    t.investment_gains[0] += data.short_gain;
    t.investment_gains[1] += data.long_gain;
    t.collectible_long += data.rate_28_gain;
    t.unrecaptured_long += data.unrecaptured_gain;
    let royalty_deductions = data.box_9_royalty_depreciation
        + data.box_9_royalty_depletion
        + data.box_9_royalty_amortization;
    let box_5_royalties = data.royalty_income - royalty_deductions;
    t.estate_royalty_columns[0] += data.royalty_income;
    t.estate_royalty_columns[1] += royalty_deductions;
    t.royalties += exact::whole(box_5_royalties);
    t.estate_royalties += box_5_royalties;
    if data.royalty_income != 0 {
        // 2026 draft Schedule K-1 (Form 1041), beneficiary's table:
        // box 5 "Schedule E, line 33, column (f)".
        push_line(
            r,
            Some(FormLine::new(TaxForm::ScheduleE, "33f")),
            &alloc::format!("estate_trust_{}_line33f_royalties", e.id),
            data.royalty_income,
            &["income.estate_trust_portfolio"],
            "https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_LINE33F_ROYALTIES"),
        );
    }
    if royalty_deductions != 0 {
        push_line(
            r,
            Some(FormLine::new(TaxForm::ScheduleE, "33e")),
            &alloc::format!("estate_trust_{}_line33e_royalty_deductions", e.id),
            royalty_deductions,
            &["income.estate_trust_portfolio"],
            "https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_LINE33E_ROYALTY_DEDUCTIONS"),
        );
    }
    if data.box_9_royalty_depreciation > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_9a_royalty_depreciation", e.id),
            data.box_9_royalty_depreciation,
            &["income.estate_trust_portfolio.box_9_royalty_depreciation"],
            "https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_9A_ROYALTY_DEPRECIATION"),
        );
        r.review_notes.push(note_number(#[cfg(test)] alloc::format!("Estate/trust document {}, box 9 code A: retain the fiduciary's per-asset sound-recording basis, service-date, useful-life and beneficiary-allocation statement. The fiduciary continues the depreciation schedule; this App does not compute or transfer that basis.", e.id), "NOTE_ESTATE_BOX_9A_RECORDS", e.id));
    }
    if data.box_9_royalty_depletion > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_9b_royalty_depletion", e.id),
            data.box_9_royalty_depletion,
            &["income.estate_trust_portfolio.box_9_royalty_depletion"],
            "https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_9B_ROYALTY_DEPLETION"),
        );
        r.review_notes.push(note_number(#[cfg(test)] alloc::format!("Estate/trust document {}, box 9 code B: retain the fiduciary's per-property depletion basis, recoverable/sold-unit calculation and beneficiary allocation statement with this return. The fiduciary continues the property-basis account; this App does not compute depletion or transfer that basis.", e.id), "NOTE_ESTATE_BOX_9B_RECORDS", e.id));
    }
    if data.box_9_royalty_amortization > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_9c_royalty_amortization", e.id),
            data.box_9_royalty_amortization,
            &["income.estate_trust_portfolio.box_9_royalty_amortization"],
            "https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_9C_ROYALTY_AMORTIZATION"),
        );
    }
    t.estate_67e += data.final_67e_deduction;
    if let Some(finals) = &data.final_character {
        if !finals.section67e_in_h {
            t.estate_niit_excess_deduction += finals.section67e_nii;
        }
        // Box 11 code B investment interest keeps its character
        // (Reg. 1.642(h)-2(b)): it is Form 4952 line 1 interest
        // with the beneficiary's own, limited by the
        // beneficiary's net investment income (i1041sk1 box 11
        // code B, "the applicable line of Schedule A").
        for item in &finals.additional {
            if let crate::EstateFinalDeduction::InvestmentInterest { amount, .. } = item {
                for interest in &mut t.k1_interest {
                    *interest += amount;
                }
                t.estate_investment_interest += amount;
                t.injured_niit_deduction_cents.k1_interest
                    [usize::from(e.owner == crate::DocumentOwner::Spouses)] += amount;
            }
        }
    }
    t.estate_final_personal_property_tax += data.final_personal_property_tax;
    t.estate_final_real_estate_tax += data.final_real_estate_tax;
    t.estate_final_state_income_tax += data.final_state_income_tax;
    t.estate_final_state_sales_tax += data.final_state_sales_tax;
    if data.final_personal_property_tax != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_11b_personal_property_tax", e.id),
            data.final_personal_property_tax,
            &["income.estate_trust_portfolio.final_personal_property_tax"],
            "https://www.irs.gov/irb/2020-45_IRB",
            msg!("EXPLAIN_ESTATE_TRUST_N_11B_PERSONAL_PROPERTY_TAX"),
        );
    }
    if data.final_real_estate_tax != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_11b_real_estate_tax", e.id),
            data.final_real_estate_tax,
            &["income.estate_trust_portfolio.final_real_estate_tax"],
            "https://www.irs.gov/instructions/i1041sk1",
            msg!("EXPLAIN_ESTATE_TRUST_N_11B_REAL_ESTATE_TAX"),
        );
    }
    if data.final_state_income_tax != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_11b_state_income_tax", e.id),
            data.final_state_income_tax,
            &["income.estate_trust_portfolio.final_state_income_tax"],
            "https://www.irs.gov/irb/2020-45_IRB",
            msg!("EXPLAIN_ESTATE_TRUST_N_11B_STATE_INCOME_TAX"),
        );
    }
    if data.final_state_sales_tax != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_11b_state_sales_tax", e.id),
            data.final_state_sales_tax,
            &["income.estate_trust_portfolio.final_state_sales_tax"],
            "https://www.irs.gov/irb/2020-45_IRB",
            msg!("EXPLAIN_ESTATE_TRUST_N_11B_STATE_SALES_TAX"),
        );
    }
    let ird_tax = data.box_10_interest_ird_estate_tax;
    t.estate_interest_ird_tax += ird_tax;
    t.injured_niit_deduction_cents.estate_ird[owner] += ird_tax;
    if data.box_10_interest_ird_estate_tax != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_10_estate_tax", e.id),
            data.box_10_interest_ird_estate_tax,
            &["income.estate_trust_portfolio.box_10_interest_ird_estate_tax"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_N_ESTATE_TAX"),
        );
    }
    if let Some(reconciliation) = &data.amt_reconciliation {
        for (position, adjustment) in reconciliation.adjustments.iter().enumerate() {
            match &adjustment.treatment {
                crate::EstateAmtTreatment::Activity { included_in_a, .. } => {
                    if *included_in_a {
                        t.estate_amt_reallocated += adjustment.amount;
                    }
                }
                crate::EstateAmtTreatment::Independent { line, included_in_a } => {
                    if *included_in_a {
                        t.estate_amt_reallocated += adjustment.amount;
                    }
                    t.estate_amt_independent += adjustment.amount;
                    let printed = match line {
                        crate::EstateAmtLine::Depletion2d => "2d",
                        crate::EstateAmtLine::Depreciation2l => "2l",
                        crate::EstateAmtLine::Pre1987Depreciation3 => "3",
                    };
                    push_line(
                        r,
                        Some(FormLine::new(TaxForm::Form6251, printed)),
                        &alloc::format!("estate_trust_{}_amt_independent_{position}", e.id),
                        adjustment.amount,
                        &["income.estate_trust_portfolio.amt_reconciliation"],
                        "https://www.irs.gov/instructions/i6251",
                        msg!("EXPLAIN_ESTATE_AMT_INDEPENDENT"),
                    );
                }
                crate::EstateAmtTreatment::Unavailable => {
                    crate::issue_localized!(
                        r,
                        &context,
                        &source,
                        "ISSUE_ESTATE_AMT_SOURCE_UNAVAILABLE",
                        "The fiduciary's box 12 adjustment needs its actual regular/AMT source and applicable Form 6251 owner under sections 56 and 57; its added tax cannot be omitted.",
                    );
                }
                crate::EstateAmtTreatment::IncludedInA => {}
            }
        }
    }
    t.estate_amt_box12a += data.amt_box12a;
    t.estate_amt_box12b += data.amt_box12b;
    t.estate_amt_box12c += data.amt_box12c;
    t.estate_amt_box12d += data.amt_box12d;
    // Codes C and D refigure AMT Schedule D, while code A already
    // puts their income adjustments on Form 6251 line 2j.
    t.amt_sale_adjustment[0] += data.amt_box12c;
    t.amt_sale_adjustment[1] += data.amt_box12d;
    t.amt_investment_gain_adjustment[0] += data.amt_box12c;
    t.amt_investment_gain_adjustment[1] += data.amt_box12d;
    // 2026 Form 1041 K-1 box 12 code E is the unrecaptured section
    // 1250 subclass of code D. D refigures AMT Schedule D line 12;
    // E refigures line 11 of its section 1250 worksheet, whether E
    // is all or part of D. Code A alone owns the AMTI adjustment.
    // https://www.irs.gov/pub/irs-dft/f1041sk1--dft.pdf
    // https://www.irs.gov/instructions/i6251
    t.amt_unrecaptured_adjustment += data.amt_box12e;
    // Code F is a different code D subclass. It refigures line 4
    // of the AMT 28% Rate Gain Worksheet, whether all or part of
    // D; code A alone owns AMTI.
    t.amt_collectible_adjustment += data.amt_box12f;
    if data.amt_box12a != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12a_amt_adjustment", e.id),
            data.amt_box12a,
            &["income.estate_trust_portfolio.amt_box12a"],
            "https://www.irs.gov/pub/irs-dft/f6251--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_12A_AMT_ADJUSTMENT"),
        );
    }
    if data.amt_box12b != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12b_amt_qualified_dividends", e.id),
            data.amt_box12b,
            &["income.estate_trust_portfolio.amt_box12b"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_12B_AMT_QUALIFIED_DIVIDENDS"),
        );
    }
    if data.amt_box12c != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12c_amt_short_gain", e.id),
            data.amt_box12c,
            &["income.estate_trust_portfolio.amt_box12c"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_12C_AMT_SHORT_GAIN"),
        );
    }
    if data.amt_box12d != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12d_amt_long_gain", e.id),
            data.amt_box12d,
            &["income.estate_trust_portfolio.amt_box12d"],
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_ESTATE_TRUST_N_12D_AMT_LONG_GAIN"),
        );
    }
    if data.amt_box12e != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12e_amt_unrecaptured_gain", e.id),
            data.amt_box12e,
            &["income.estate_trust_portfolio.amt_box12e"],
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_ESTATE_TRUST_N_12E_AMT_UNRECAPTURED_GAIN"),
        );
    }
    if data.amt_box12f != 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_12f_amt_rate_28_gain", e.id),
            data.amt_box12f,
            &["income.estate_trust_portfolio.amt_box12f"],
            "https://www.irs.gov/instructions/i6251",
            msg!("EXPLAIN_ESTATE_TRUST_N_12F_AMT_RATE_N_GAIN"),
        );
    }
    if let Some(adjustment) = &data.niit_adjustment {
        t.estate_niit_line_7 += adjustment.line_7;
        t.estate_niit_magi += adjustment.modified_agi;
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_14h_niit_line_7", e.id),
            adjustment.line_7,
            &["income.estate_trust_portfolio.niit_adjustment"],
            "https://www.irs.gov/instructions/i8960",
            msg!("EXPLAIN_ESTATE_TRUST_N_14H_NIIT_LINE_N"),
        );
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_14h_niit_magi", e.id),
            adjustment.modified_agi,
            &["income.estate_trust_portfolio.niit_adjustment"],
            "https://www.irs.gov/instructions/i8960",
            msg!("EXPLAIN_ESTATE_TRUST_N_14H_NIIT_MAGI"),
        );
    }
    t.withholding_other += data.backup_withholding;
    emit_lines(
        r,
        &mut (),
        [
            ("1_interest", data.interest),
            ("2a_ordinary_dividends", data.ordinary_dividends),
            ("2b_qualified_dividends", data.qualified_dividends),
            ("3_short_gain", data.short_gain),
            ("4a_long_gain", data.long_gain),
            ("4b_rate_28_gain", data.rate_28_gain),
            ("4c_unrecaptured_gain", data.unrecaptured_gain),
            ("5_royalty_income", data.royalty_income),
            ("14a_tax_exempt_interest", data.tax_exempt_interest),
        ],
        |(box_name, amount), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("estate_trust_{}_{}", e.id, box_name),
                LineValue::MoneyCents(amount),
                &["income.estate_trust_portfolio"],
                "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
                msg!("EXPLAIN_ESTATE_TRUST_N_N_2"),
            ))
        },
    );
    if data.backup_withholding > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_backup_withholding", e.id),
            data.backup_withholding,
            &["income.estate_trust_portfolio.backup_withholding"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_BACKUP_WITHHOLDING"),
        );
        r.review_notes.push(note_number(#[cfg(test)] alloc::format!("Estate/trust document {}, backup withholding: retain the K-1 for attachment to a filed return. This App does not submit the return.", e.id), "NOTE_ESTATE_BACKUP_WITHHOLDING", e.id));
    }
    if data.allocated_estimated_tax > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_allocated_estimated_tax", e.id),
            data.allocated_estimated_tax,
            &["income.estate_trust_portfolio.allocated_estimated_tax"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_ALLOCATED_ESTIMATED_TAX"),
        );
    }
    if data.final_67e_deduction > 0 {
        line_cents(
            r,
            &alloc::format!("estate_trust_{}_11a_67e_deduction", e.id),
            data.final_67e_deduction,
            &["income.estate_trust_portfolio.final_67e_deduction"],
            "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
            msg!("EXPLAIN_ESTATE_TRUST_N_11A_67E_DEDUCTION"),
        );
    }
    for (box_name, amount) in
        [("11c_short_carry", data.final_short_carry), ("11d_long_carry", data.final_long_carry)]
    {
        if amount > 0 {
            line_cents(
                r,
                &alloc::format!("estate_trust_{}_{}", e.id, box_name),
                amount,
                &["income.estate_trust_portfolio"],
                "https://www.irs.gov/pub/irs-dft/i1041sk1--dft.pdf",
                msg!("EXPLAIN_ESTATE_TRUST_N_N_3"),
            );
        }
    }
    Ok(())
}
