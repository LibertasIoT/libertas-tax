// schedule_d items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
pub(crate) fn preferential_tax(income: i64, preferential: i64, status: FederalStatus) -> i64 {
    preferential_tax_with_collectibles(income, preferential, 0, status)
}
// Schedule D Tax Worksheet without a Form 4952 election.
// `preferential` includes all net long-term gain and qualified dividends;
// `collectibles` is the loss-adjusted 28% component, capped by net capital gain upstream.
pub(crate) fn preferential_tax_with_collectibles(
    income: i64,
    preferential: i64,
    collectibles: i64,
    status: FederalStatus,
) -> i64 {
    preferential_tax_with_special_gains(income, preferential, collectibles, 0, status)
}
pub(crate) fn preferential_tax_with_special_gains(
    income: i64,
    preferential: i64,
    collectibles: i64,
    unrecaptured: i64,
    status: FederalStatus,
) -> i64 {
    let preferential = positive(preferential);
    let unrecaptured = phase::cap(positive(unrecaptured), [preferential]);
    let collectibles = phase::cap(positive(collectibles), [preferential - unrecaptured]);
    schedule_d_tax(
        income,
        preferential,
        collectibles + unrecaptured,
        unrecaptured,
        collectibles > 0,
        status,
    )
}
// Schedule D lines 12 and 35 are distinct inputs for the Form 8615 family
// worksheet: allocated special gains versus original Schedule D line 19 totals.
pub(crate) fn schedule_d_tax(
    income: i64,
    preferential: i64,
    special: i64,
    original_unrecaptured: i64,
    has_collectibles: bool,
    status: FederalStatus,
) -> i64 {
    crate::income_tax::Rates::with_method(2026, status, true).unwrap().schedule_d(
        income,
        preferential,
        special,
        original_unrecaptured,
        has_collectibles,
    )
}

impl Totals {
    // Schedule D amounts, a gain or a loss: the net and, for Form 1116 line 3e, the gain before
    // any loss (the positive part of each contribution).
    pub(crate) fn add_short(&mut self, amount: i64) {
        self.try_add_capital(amount, false).expect("admitted short-term capital totals");
    }
    pub(crate) fn add_long(&mut self, amount: i64) {
        self.try_add_capital(amount, true).expect("admitted long-term capital totals");
    }
    // Form 1116 lines 3d/3e take gains before losses. The native K-1 owners
    // have already subtracted allowed retained losses from their Schedule D
    // amounts; restore only the gross operands, keeping those nets intact.
    // https://www.irs.gov/instructions/i1116
    pub(super) fn restore_k1_capital_gross(
        &mut self,
        net: [[i64; 2]; 2],
        gross: [[i64; 2]; 2],
        has_k3: bool,
    ) -> bool {
        let mut restored = 0_i128;
        let mut net_adjustment = 0_i128;
        let mut gross_adjustment = 0_i128;
        for term in 0..2 {
            net_adjustment += i128::from(net[1][term]) - i128::from(net[0][term]);
            if !has_k3 {
                restored += i128::from(gross[0][term]) - i128::from(positive(net[0][term]));
                gross_adjustment += i128::from(gross[1][term]) - i128::from(gross[0][term]);
            }
        }
        if restored == 0 && net_adjustment == 0 && gross_adjustment == 0 {
            return true;
        }
        let candidates = [
            i128::from(self.capital_gross) + restored,
            i128::from(self.netted_losses) + restored,
            i128::from(self.amt_k1_capital_net_adjustment) + net_adjustment,
            i128::from(self.amt_k1_capital_gross_adjustment) + gross_adjustment,
        ];
        let [Ok(capital_gross), Ok(netted_losses), Ok(net_delta), Ok(gross_delta)] =
            candidates.map(i64::try_from)
        else {
            return false;
        };
        // Include the source's following add_short/add_long calls before
        // mutating either restored operand or the independent AMT deltas.
        let added_gross =
            if has_k3 { 0 } else { net[0].iter().map(|v| i128::from(positive(*v))).sum() };
        let added_losses: i128 = net[0].iter().map(|v| (-i128::from(*v)).max(0)).sum();
        if i64::try_from(i128::from(capital_gross) + added_gross).is_err()
            || i64::try_from(i128::from(netted_losses) + added_losses).is_err()
            || i64::try_from(i128::from(capital_gross) + added_gross + i128::from(gross_delta))
                .is_err()
        {
            return false;
        }
        self.capital_gross = capital_gross;
        self.netted_losses = netted_losses;
        self.amt_k1_capital_net_adjustment = net_delta;
        self.amt_k1_capital_gross_adjustment = gross_delta;
        true
    }
    // An exclusion from gross income of a gain already added (section 1202(a), section
    // 1400Z-2(a)(1)(A)): it leaves the net and Form 1116 line 3e together.
    pub(super) fn exclude_short(&mut self, amount: i64) {
        self.short -= amount;
        self.capital_gross -= amount;
    }
    pub(super) fn exclude_long(&mut self, amount: i64) {
        self.long -= amount;
        self.capital_gross -= amount;
    }
    // A K-1 gain whose Schedule K-3 gross income Form 1116 line 3e takes instead.
    pub(crate) fn add_short_outside_line_3e(&mut self, amount: i64) {
        self.try_add_k1_capital(amount, false, true).expect("admitted short-term capital totals");
    }
    pub(crate) fn add_long_outside_line_3e(&mut self, amount: i64) {
        self.try_add_k1_capital(amount, true, true).expect("admitted long-term capital totals");
    }

    pub(crate) fn amt_capital_operands(&self, r: &mut FederalResult) -> Option<[i64; 2]> {
        let gross = crate::kernel::ratio::checked_sum([
            self.capital_gross,
            self.amt_k1_capital_gross_adjustment,
        ]);
        // Three signed64 operands fit i128, including subtracting signed MIN.
        let residual = i64::try_from(
            i128::from(self.amt_sale_adjustment[0]) + i128::from(self.amt_sale_adjustment[1])
                - i128::from(self.amt_k1_capital_net_adjustment),
        )
        .ok();
        if let (Some(gross), Some(residual)) = (gross, residual) {
            return Some([gross, residual]);
        }
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign tax credit",
            crate::k1_activity::ordinary::RANGE_KEY,
            "Review the document amounts; they must be within the supported input range.",
        );
        None
    }
}

pub(super) struct CapitalPriorIn {
    pub(super) modify_income: bool,
}

pub(super) struct CapitalPriorOut {
    pub(super) capital_in: [i64; 2],
    pub(super) amt_capital_in: [i64; 2],
    pub(super) estate_carry_cents: [i64; 2],
    pub(super) capital_work_prior: [i64; 2],
    pub(super) filed_prior: [i64; 2],
    pub(super) estate_investment_carry: [Option<[i64; 2]>; 2],
    pub(super) estate_business_carry: [Option<[i64; 2]>; 2],
    pub(super) investment_capital_in: [[i64; 2]; 2],
    pub(super) regular_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) portfolio_capital: i64,
    pub(super) amt_capital_amounts: crate::capital::CapitalAmounts,
}

pub(super) fn capital_prior_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: CapitalPriorIn,
) -> CapitalPriorOut {
    let &Pass { d, allocated_entries, separate, .. } = pass;
    let CapitalPriorIn { modify_income } = i;
    let [mut capital_in, mut amt_capital_in] = crate::extended_rules::prior_capital(d, r);
    let estate_carry_cents = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                Some([data.final_short_carry, data.final_long_carry])
            }
            _ => None,
        })
        .fold([0i64; 2], |mut total, amounts| {
            total[0] += amounts[0];
            total[1] += amounts[1];
            total
        });
    let estate_amt_carry_cents = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::EstateTrustPortfolio { data } => Some({
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                data.final_character
                    .as_ref()
                    .map_or([data.final_short_carry, data.final_long_carry], |f| {
                        [f.capital.alternative_short, f.capital.alternative_long]
                    })
            }),
            _ => None,
        })
        .fold([0i64; 2], |mut total, amounts| {
            total[0] += amounts[0];
            total[1] += amounts[1];
            total
        });
    // Prior filed-return carries are the existing owner's settled dollar
    // amounts. Current K1 final carries retain their admitted source cents.
    let capital_work_prior =
        core::array::from_fn(|term| capital_in[term] * 100 + estate_carry_cents[term]);
    // The filer's own carryovers from the prior-year Capital Loss Carryover
    // Worksheet stay on Schedule D lines 6 and 14. A beneficiary's K-1 (1041)
    // box 11 codes C and D print on lines 5 and 12 with the other K-1 amounts
    // (Instructions for Schedule K-1 (Form 1041) for a Beneficiary, box 11).
    let filed_prior = capital_in;
    for term in 0..2 {
        capital_in[term] += dollars(estate_carry_cents[term]);
        amt_capital_in[term] += dollars(estate_amt_carry_cents[term]);
    }
    // Independent source axes: neither Form8960's difference nor total
    // ScheduleD carryovers establish Form4952 or Form172 character.
    let estate_investment_carry = [
        crate::estate_trust::capital_character(
            d,
            false,
            crate::estate_trust::CapitalLossUse::Investment,
        ),
        crate::estate_trust::capital_character(
            d,
            true,
            crate::estate_trust::CapitalLossUse::Investment,
        ),
    ];
    let estate_business_carry = [
        crate::estate_trust::capital_character(
            d,
            false,
            crate::estate_trust::CapitalLossUse::Business,
        ),
        crate::estate_trust::capital_character(
            d,
            true,
            crate::estate_trust::CapitalLossUse::Business,
        ),
    ];
    let mut investment_capital_in = [capital_in, amt_capital_in];
    for (index, total) in [estate_carry_cents, estate_amt_carry_cents].iter().enumerate() {
        let parts = estate_investment_carry[index].unwrap_or([0; 2]);
        for term in 0..2 {
            investment_capital_in[index][term] += dollars(parts[term]) - dollars(total[term]);
        }
    }
    let regular_capital_amounts = crate::capital::CapitalAmounts {
        short: dollars(t.short),
        long: dollars(t.long),
        collectibles: dollars(t.collectible_long),
        unrecaptured: dollars(t.unrecaptured_long),
    };
    let portfolio_capital =
        regular_capital_amounts.calculate(capital_in, separate, modify_income).income;
    let amt_capital_amounts = crate::capital::CapitalAmounts {
        short: dollars(t.short + t.amt_sale_adjustment[0]),
        long: dollars(t.long + t.amt_sale_adjustment[1]),
        collectibles: dollars(t.collectible_long + t.amt_collectible_adjustment),
        unrecaptured: dollars(t.unrecaptured_long + t.amt_unrecaptured_adjustment),
    };
    CapitalPriorOut {
        capital_in,
        amt_capital_in,
        estate_carry_cents,
        capital_work_prior,
        filed_prior,
        estate_investment_carry,
        estate_business_carry,
        investment_capital_in,
        regular_capital_amounts,
        portfolio_capital,
        amt_capital_amounts,
    }
}

pub(super) struct ScheduleDSourceReportingIn<'a> {
    pub(super) unrecaptured_gain: i64,
    pub(super) collectible_gain: i64,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
}

pub(super) fn schedule_d_source_reporting_phase(
    r: &mut FederalResult,
    t: &Totals,
    i: ScheduleDSourceReportingIn<'_>,
) {
    let ScheduleDSourceReportingIn { unrecaptured_gain, collectible_gain, property_character } = i;
    line(
        r,
        "unrecaptured_section_1250_gain",
        unrecaptured_gain,
        &[
            "income.dividends",
            "income.undistributed_gain",
            "income.home_sale",
            "income.sales",
            "screening.capital_history",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_UNRECAPTURED_SECTION_N_GAIN"),
    );
    line(
        r,
        "rate_28_gain",
        collectible_gain,
        &[
            "income.sales",
            "income.dividends",
            "income.undistributed_gain",
            "income.qualified_small_business_sale",
            "screening.capital_history",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_RATE_N_GAIN"),
    );
    // Form 8949 box totals (A-F), Schedule D lines 1a/1b/2/3 short-term and
    // 8a/8b/9/10 long-term, with lines 4 and 11: every short/long contributor
    // has its line, so lines 1a to 4 sum to the short-term total and lines
    // 8a to 11 to the long-term total before carryovers (the footing test in
    // src/tests/investment/investment_kinds.rs). Form 2439 box 1a, Form 6781 and Form 6252 are
    // lines 4 and 11.
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "1a"),
        "capital_gain_1a",
        t.sale_line_1a,
        &["income.sales"],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_1A"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "1b"),
        "capital_gain_1b",
        t.sale_line_1b,
        &[
            "income.sales",
            "income.employee_stock_sale",
            "income.espp_sale",
            "income.iso_sale",
            "income.nua_sale",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_1B"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "2"),
        "capital_gain_2",
        t.sale_line_2,
        &["income.sales"],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_N"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "3"),
        "capital_gain_3",
        t.sale_line_3,
        &[
            "income.sales",
            "income.home_sale",
            "income.small_business_stock",
            "income.worthless_security",
            "income.nonbusiness_bad_debt",
            "income.return_of_capital",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_N_2"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "4"),
        "capital_gain_4",
        t.schedule_d_line_4,
        &[
            "screening.installment_payments",
            "income.installment_sale",
            "screening.related_exchanges",
            "income.section_1256",
            "deductions.personal_casualties",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_4"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "8a"),
        "capital_gain_8a",
        t.sale_line_8a,
        &["income.sales"],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_8A"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "8b"),
        "capital_gain_8b",
        t.sale_line_8b,
        &[
            "income.sales",
            "income.employee_stock_sale",
            "income.espp_sale",
            "income.iso_sale",
            "income.nua_sale",
            "income.qualified_small_business_sale",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_8B"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "9"),
        "capital_gain_9",
        t.sale_line_9,
        &["income.sales", "income.qualified_small_business_sale"],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_N_3"),
    );
    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "10"),
        "capital_gain_10",
        t.sale_line_10,
        &[
            "income.sales",
            "income.home_sale",
            "income.qualified_small_business_sale",
            "income.small_business_stock",
            "income.worthless_security",
            "income.return_of_capital",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_N_4"),
    );

    crate::capital::schedule_d_source_cell(
        r,
        FormLine::new(TaxForm::ScheduleD, "11"),
        "capital_gain_11",
        // Form 4797 Part I gain (section1231.rs long_term, return dollars) joins the
        // Form 6252, 8824, 6781, 2439 and 4684 parts of the printed line.
        t.schedule_d_line_11 + property_character.as_ref().map_or(0, |v| v.long_term * 100),
        &[
            "screening.installment_payments",
            "income.installment_sale",
            "screening.related_exchanges",
            "income.section_1256",
            "income.undistributed_gain",
            "income.business.assets.disposal",
            "income.rental.sale",
            "screening.section1231",
            "deductions.personal_casualties",
        ],
        "https://www.irs.gov/instructions/i1040sd",
        msg!("EXPLAIN_CAPITAL_GAIN_11"),
    );
    // Schedule D columns (d) proceeds, (e) cost and (g) adjustments of lines 1a to 3 and 8a to 10
    // (column (h) is the line itself). Lines 1a and 8a have no column (g): Exception 1 lots have
    // no adjustment. A line shows its columns when any is not zero.
    for (index, line) in crate::capital::SALE_LINES.iter().enumerate() {
        let columns = &t.sale_columns[index * 3..index * 3 + 3];
        if columns.iter().all(|v| *v == 0) {
            continue;
        }
        let summary = matches!(*line, "1a" | "8a");
        for (column, suffix, name, explanation) in [
            (0, "proceeds", "(d)", msg!("EXPLAIN_CAPITAL_GAIN_COLUMN_D")),
            (1, "basis", "(e)", msg!("EXPLAIN_CAPITAL_GAIN_COLUMN_E")),
            (2, "adjustment", "(g)", msg!("EXPLAIN_CAPITAL_GAIN_COLUMN_G")),
        ] {
            if summary && column == 2 {
                continue;
            }
            crate::capital::schedule_d_source_cell(
                r,
                FormLine::new(TaxForm::ScheduleD, &alloc::format!("{line}{name}")),
                &alloc::format!("capital_gain_{line}_{suffix}"),
                columns[column],
                &["income.sales"],
                "https://www.irs.gov/instructions/i1040sd",
                explanation,
            );
        }
    }
}

pub(super) struct CapitalCarryReportingIn {
    pub(super) additional: [i64; 3],
    pub(super) capital_in: [i64; 2],
    pub(super) itemized_reduction: i64,
    pub(super) short: i64,
    pub(super) long: i64,
    pub(super) capital: i64,
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) qbi_ded: i64,
}

pub(super) fn capital_carry_reporting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: CapitalCarryReportingIn,
) {
    let &Pass { d, repayment, separate, .. } = pass;
    let CapitalCarryReportingIn {
        additional,
        capital_in,
        itemized_reduction,
        short,
        long,
        capital,
        agi,
        deduction,
        qbi_ded,
    } = input;
    line(
        r,
        "itemized_deduction_reduction",
        itemized_reduction,
        &["agi", "deductions"],
        "https://www.irs.gov/publications/p505",
        msg!("EXPLAIN_ITEMIZED_DEDUCTION_REDUCTION"),
    );
    let carry = crate::extended_rules::capital_carryover(
        short,
        long,
        positive(-capital),
        agi - deduction - additional.iter().sum::<i64>() - qbi_ded,
    );
    crate::section1256::report(
        d,
        &crate::section1256::Netting {
            short,
            long,
            deducted: positive(-capital),
            taxable: agi - deduction - additional.iter().sum::<i64>() - qbi_ded,
            carry_in: capital_in[0] + capital_in[1],
            carry_1256: crate::extended_rules::prior_section_1256(d),
            separate,
        },
        r,
    );
    let reported_carry = repayment.future_capital.map_or(carry, |v| v[0]);
    emit_lines(
        r,
        &mut (),
        [
            ("capital_short_carryforward", reported_carry[0]),
            ("capital_long_carryforward", reported_carry[1]),
            ("capital_loss_carryforward", reported_carry[0] + reported_carry[1]),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["capital_gain", "deductions", "agi"],
                "https://www.irs.gov/instructions/i1040sd",
                msg!("EXPLAIN_FEDERAL_RULES_10"),
            ))
        },
    );
}

pub(super) struct CapitalNettingIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) amt_property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) regular_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) amt_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) capital_in: [i64; 2],
    pub(super) amt_capital_in: [i64; 2],
    pub(super) filed_prior: [i64; 2],
    pub(super) estate_carry_cents: [i64; 2],
    pub(super) modify_income: bool,
    pub(super) qualified: i64,
    pub(super) loan_gain: i64,
    pub(super) amt_loan_gain: i64,
    pub(super) property_ordinary: i64,
    pub(super) amt_property_ordinary: i64,
}

pub(super) struct CapitalNettingOut {
    pub(super) regular_capital: crate::capital::CapitalResult,
    pub(super) amt_capital_result: crate::capital::CapitalResult,
    pub(super) short: i64,
    pub(super) long: i64,
    pub(super) capital: i64,
    pub(super) amt_short: i64,
    pub(super) amt_long: i64,
    pub(super) amt_capital: i64,
    pub(super) amt_capital_independent_adjustment: i64,
    pub(super) amt_net_capital_gain: i64,
    pub(super) amt_schedule_d_18: i64,
    pub(super) amt_schedule_d_19: i64,
    pub(super) pref: i64,
    pub(super) net_capital_gain: i64,
    pub(super) schedule_d_18: i64,
    pub(super) schedule_d_19: i64,
    pub(super) unrecaptured_gain: i64,
    pub(super) collectible_gain: i64,
}

pub(super) fn capital_netting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    i: CapitalNettingIn<'_>,
) -> core::ops::ControlFlow<(), CapitalNettingOut> {
    let &Pass { separate, .. } = pass;
    let CapitalNettingIn {
        rental,
        amt_rental,
        property_character,
        amt_property_character,
        regular_capital_amounts,
        amt_capital_amounts,
        capital_in,
        amt_capital_in,
        filed_prior,
        estate_carry_cents,
        modify_income,
        qualified,
        loan_gain,
        amt_loan_gain,
        property_ordinary,
        amt_property_ordinary,
    } = i;
    let regular_capital = crate::capital::CapitalAmounts {
        short: dollars(
            match crate::kernel::ratio::checked_sum([t.short, rental.k1_cuts.short.get()]) {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(r);
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            },
        ),
        long: dollars(
            match crate::kernel::ratio::checked_sum([t.long, rental.k1_cuts.long.get()]) {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(r);
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            },
        ) + property_character.as_ref().map_or(0, |v| v.long_term),
        unrecaptured: regular_capital_amounts.unrecaptured
            + property_character.as_ref().map_or(0, |v| v.unrecaptured),
        ..regular_capital_amounts
    }
    .calculate(capital_in, separate, modify_income);
    let amt_capital_result = crate::capital::CapitalAmounts {
        short: dollars(
            match crate::kernel::ratio::checked_sum([
                t.short,
                t.amt_sale_adjustment[0],
                amt_rental.k1_cuts.short.get(),
            ]) {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(r);
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            },
        ),
        long: dollars(
            match crate::kernel::ratio::checked_sum([
                t.long,
                t.amt_sale_adjustment[1],
                amt_rental.k1_cuts.long.get(),
            ]) {
                Some(value) => value,
                None => {
                    CollectFailure::Broker(crate::IssueContext::Section(
                        crate::TaxSection::InvestmentSales,
                    ))
                    .report(r);
                    withhold_required_source_result(r);
                    return core::ops::ControlFlow::Break(());
                }
            },
        ) + amt_property_character.as_ref().map_or(0, |v| v.long_term),
        unrecaptured: amt_capital_amounts.unrecaptured
            + amt_property_character.as_ref().map_or(0, |v| v.unrecaptured),
        ..amt_capital_amounts
    }
    .calculate(amt_capital_in, separate, modify_income);
    let short = regular_capital.short;
    let long = regular_capital.long;
    let capital = regular_capital.income;
    let amt_short = amt_capital_result.short;
    let amt_long = amt_capital_result.long;
    let amt_capital = amt_capital_result.income;
    // Codes C and D are already in K-1 code A (Form 6251 line 2j), and also
    // refigure AMT Schedule D. Code E refigures only its section 1250
    // worksheet. The residual difference after subtracting C and D is the
    // separate line 2k effect, including the AMT capital-loss limit.
    let amt_capital_independent_adjustment =
        amt_capital - capital - dollars(t.estate_amt_box12c + t.estate_amt_box12d);
    // Disposition amounts stay on Form 6251 line 2k even when basis,
    // at-risk or passive limits determine their allowed amounts. The AMT
    // base below already consumes these components; this row reports their
    // combined difference without adding it to income a second time.
    form_line(
        r,
        FormLine::new(TaxForm::Form6251, "2k"),
        "amt_disposition_adjustment",
        amt_loan_gain - loan_gain + amt_property_ordinary - property_ordinary
            + amt_capital_independent_adjustment,
        &["income", "screening.capital_history", "screening.section1231"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_DISPOSITION_ADJUSTMENT"),
    );
    // NOL modification runs are internal worksheets, not the filed return.
    // Project only the actual regular result, without re-netting its amounts.
    if !modify_income {
        regular_capital.emit_summary(
            filed_prior,
            // The netting takes each estate carry in whole dollars, so lines
            // 5 and 12 subtract the same dollars and Schedule D keeps footing.
            [
                t.schedule_d_k1[0] + rental.k1_cuts.short.get()
                    - dollars(estate_carry_cents[0]) * 100,
                t.schedule_d_k1[1] + rental.k1_cuts.long.get()
                    - dollars(estate_carry_cents[1]) * 100,
            ],
            t.schedule_d_distributions,
            r,
        );
    }
    let amt_net_capital_gain = amt_capital_result.net_gain;
    let amt_schedule_d_18 = amt_capital_result.line18;
    let amt_schedule_d_19 = amt_capital_result.line19;
    let pref = qualified + regular_capital.net_gain;
    let net_capital_gain = regular_capital.net_gain;
    let schedule_d_18 = regular_capital.line18;
    let schedule_d_19 = regular_capital.line19;
    let unrecaptured_gain = regular_capital.unrecaptured_gain;
    let collectible_gain = regular_capital.collectible_gain;
    core::ops::ControlFlow::Continue(CapitalNettingOut {
        regular_capital,
        amt_capital_result,
        short,
        long,
        capital,
        amt_short,
        amt_long,
        amt_capital,
        amt_capital_independent_adjustment,
        amt_net_capital_gain,
        amt_schedule_d_18,
        amt_schedule_d_19,
        pref,
        net_capital_gain,
        schedule_d_18,
        schedule_d_19,
        unrecaptured_gain,
        collectible_gain,
    })
}
