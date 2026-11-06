// form_4952 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct PostQbiDeductionIn<'a> {
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) royalty_routing: Option<&'a crate::investment_interest::InvestmentRouting>,
    pub(super) use_itemized: bool,
    pub(super) investment_unavailable: bool,
    pub(super) additional: [i64; 3],
    pub(super) elected: [i64; 2],
    pub(super) amt_elected: [i64; 2],
    pub(super) portfolio_interest: [i64; 2],
    pub(super) royalty_interest: [i64; 2],
    pub(super) investment_paid: [i64; 2],
    pub(super) investment_interest: [[i64; 2]; 2],
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) itemized: i64,
    pub(super) before_qbi_signed: i64,
    pub(super) qbi_ded: i64,
    pub(super) investment_income: i64,
    pub(super) investment_gain: i64,
    pub(super) investment_net_capital: i64,
    pub(super) investment_expenses: i64,
    pub(super) royalty_investment_expenses: i64,
    pub(super) qualified: i64,
    pub(super) amt_investment_adjustment: i64,
    pub(super) amt_investment_gain: i64,
    pub(super) amt_investment_net_capital: i64,
    pub(super) amt_qualified: i64,
}

pub(super) struct PostQbiDeductionOut {
    pub(super) unlimited_deduction: i64,
    pub(super) itemized_reduction: i64,
    pub(super) investment_interest_deduction: i64,
    pub(super) deduction: i64,
    pub(super) taxable: i64,
}

pub(super) fn post_qbi_deduction_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: PostQbiDeductionIn<'_>,
) -> PostQbiDeductionOut {
    let &Pass { d, s, screen, repayment, allocated_entries, .. } = pass;
    let PostQbiDeductionIn {
        investment_facts,
        royalty_routing,
        use_itemized,
        investment_unavailable,
        additional,
        elected,
        amt_elected,
        portfolio_interest,
        royalty_interest,
        investment_paid,
        investment_interest,
        agi,
        deduction,
        itemized,
        before_qbi_signed,
        qbi_ded,
        investment_income,
        investment_gain,
        investment_net_capital,
        investment_expenses,
        royalty_investment_expenses,
        qualified,
        amt_investment_adjustment,
        amt_investment_gain,
        amt_investment_net_capital,
        amt_qualified,
    } = input;
    crate::require_localized!(
        r,
        repayment.deduction == 0
            || !use_itemized
            || (!screen.nol.present()
                && !screen.atnol.present()
                && estate_final_nol(d).is_none()
                && agi >= deduction + additional.iter().sum::<i64>()),
        &crate::IssueContext::Section(crate::TaxSection::Deductions),
        "Repayment comparison",
        "ISSUE_REPAYMENT_METHODS_INVOLVING_PRIOR_NOLS_OR_POTENTIAL_CURRENT_LOSSES_6AE75F10",
        "Repayment methods involving prior NOLs or potential current losses require the section 1341 loss carryback/carryover computation.",
    );
    let unlimited_deduction = deduction;
    let itemized_reduction = if use_itemized {
        phase::remaining_fraction(
            phase::cap(
                itemized,
                [positive(before_qbi_signed - qbi_ded + itemized - brackets(s.filing_status())[5])],
            ),
            2,
            37,
        )
    } else {
        0
    };
    // Section 68(b) applies after other itemized limits, including Form 4952.
    // Its reduction never increases a section 163(d) interest carryforward.
    let investment_interest_deduction = if use_itemized { portfolio_interest[0] } else { 0 };
    if !investment_unavailable {
        emit_lines(
            r,
            &mut (),
            [
                ("investment_interest_deduction", investment_interest_deduction),
                ("investment_interest_carryforward", investment_interest[0][1]),
                (
                    "amt_investment_interest_deduction",
                    if use_itemized { portfolio_interest[1] } else { 0 },
                ),
                ("amt_investment_interest_carryforward", investment_interest[1][1]),
            ],
            |(id, amount), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["deductions.investment_interest", "income"],
                    "https://www.irs.gov/pub/irs-pdf/f4952.pdf",
                    msg!("EXPLAIN_FEDERAL_RULES_8"),
                ))
            },
        );
    }
    if let Some(facts) = &investment_facts {
        let line4c = investment_income
            - elected.iter().sum::<i64>()
            - (investment_gain - investment_net_capital);
        let allowed_expenses =
            if use_itemized { investment_expenses } else { royalty_investment_expenses };
        emit_lines(
            r,
            &mut (),
            [
                ("1", dollars(facts.paid)),
                ("2", dollars(facts.carryover)),
                ("3", dollars(facts.paid) + dollars(facts.carryover)),
                ("4a", line4c + qualified),
                ("4b", qualified),
                ("4c", line4c),
                ("4d", investment_gain),
                ("4e", investment_net_capital),
                ("4f", investment_gain - investment_net_capital),
                ("4g", elected.iter().sum()),
                ("4h", investment_income),
                ("5", allowed_expenses),
                ("6", positive(investment_income - allowed_expenses)),
                ("7", investment_interest[0][1]),
                ("8", investment_interest[0][0]),
            ],
            |(number, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form4952, number)),
                    &alloc::format!("investment_form_line{number}"),
                    LineValue::MoneyDollars(amount),
                    &["deductions.investment_interest", "income"],
                    "https://www.irs.gov/pub/irs-dft/f4952--dft.pdf",
                    msg!("EXPLAIN_INVESTMENT_SOURCE_ROUTING"),
                ))
            },
        );
        let amt_income = investment_income + amt_investment_adjustment;
        let amt_line4c = amt_income
            - amt_elected.iter().sum::<i64>()
            - (amt_investment_gain - amt_investment_net_capital);
        emit_lines(
            r,
            &mut (),
            [
                ("1", dollars(investment_paid[1]) + dollars(facts.private_activity)),
                ("2", dollars(facts.amt_carryover)),
                (
                    "3",
                    dollars(investment_paid[1])
                        + dollars(facts.private_activity)
                        + dollars(facts.amt_carryover),
                ),
                ("4a", amt_line4c + amt_qualified),
                ("4b", amt_qualified),
                ("4c", amt_line4c),
                ("4d", amt_investment_gain),
                ("4e", amt_investment_net_capital),
                ("4f", amt_investment_gain - amt_investment_net_capital),
                ("4g", amt_elected.iter().sum()),
                ("4h", amt_income),
                ("5", royalty_investment_expenses),
                ("6", positive(amt_income - royalty_investment_expenses)),
                ("7", investment_interest[1][1]),
                ("8", investment_interest[1][0]),
            ],
            |(number, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form4952, number)),
                    &alloc::format!("amt_investment_form_line{number}"),
                    LineValue::MoneyDollars(amount),
                    &["deductions.investment_interest", "income"],
                    "https://www.irs.gov/pub/irs-pdf/i6251.pdf",
                    msg!("EXPLAIN_INVESTMENT_SOURCE_ROUTING"),
                ))
            },
        );
        if let Some(routing) = &royalty_routing {
            for (&source, amounts) in &routing.sources {
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("allowed", amounts[0][0]),
                        ("carryforward", amounts[0][1]),
                        ("amt_allowed", amounts[1][0]),
                        ("amt_carryforward", amounts[1][1]),
                    ],
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("investment_source_{source}_{suffix}"),
                            LineValue::MoneyDollars(amount),
                            &["deductions.investment_interest", "income.nonbusiness_property"],
                            "https://www.irs.gov/pub/irs-dft/f4952--dft.pdf",
                            msg!("EXPLAIN_INVESTMENT_SOURCE_ROUTING"),
                        ))
                    },
                );
            }
        }
    }
    crate::capital::emit_summary_choices(r, qualified);
    form_line(
        r,
        FormLine::new(TaxForm::Form6251, "2c"),
        "amt_investment_interest_adjustment",
        royalty_interest[0] - royalty_interest[1]
            + if use_itemized { portfolio_interest[0] - portfolio_interest[1] } else { 0 },
        &["deductions.investment_interest", "income.nonbusiness_property"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_INVESTMENT_SOURCE_ROUTING"),
    );
    if investment_facts.as_ref().is_some()
        && allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        && r.issues.is_empty()
    {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("K-1 investment-interest worksheet: regular allowed ${}; carryforward ${}; AMT allowed ${}; carryforward ${}. These are Form 4952 limits before the itemized-deduction election and section 68; other return eligibility checks remain separate.", investment_interest[0][0], investment_interest[0][1], investment_interest[1][0], investment_interest[1][1]),
            Message::new("NOTE_K1_INVESTMENT_INTEREST", alloc::vec![
                MessageArg::usd_dollars(investment_interest[0][0]), MessageArg::usd_dollars(investment_interest[0][1]),
                MessageArg::usd_dollars(investment_interest[1][0]), MessageArg::usd_dollars(investment_interest[1][1]),
            ]),
        ));
    }
    let deduction = deduction - itemized_reduction;
    let taxable = positive(before_qbi_signed - qbi_ded + itemized_reduction);
    PostQbiDeductionOut {
        unlimited_deduction,
        itemized_reduction,
        investment_interest_deduction,
        deduction,
        taxable,
    }
}

pub(super) struct InvestmentSourcesOut {
    pub(super) royalty_investment_expenses: i64,
    pub(super) royalty_investment_income: i64,
    pub(super) royalty_routing: Option<crate::investment_interest::InvestmentRouting>,
    pub(super) investment_unavailable: bool,
    pub(super) investment_paid: [i64; 2],
    pub(super) k1_interest_record: Option<crate::InvestmentInterest>,
}

pub(super) fn investment_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
) -> core::ops::ControlFlow<(), InvestmentSourcesOut> {
    let &Pass { d, ded, screen, c, allocated_entries, .. } = pass;
    let force_itemized = pass.context.force_itemized;
    // Direct royalty income is gross on Form 4952 line 4a; its already
    // allowed noninterest costs enter line 5 even with a standard deduction.
    let royalty_expense_cents = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::Royalty { data } => {
                Some(crate::RentalExpense::total_exact(&data.expenses))
            }
            _ => None,
        })
        .sum::<Exact>();
    let royalty_investment_expenses = match exact::dollars(&royalty_expense_cents)
        .filter(|amount| amount.checked_mul(100).is_some())
    {
        Some(amount) => amount,
        None => {
            crate::extended_rules::expense_output_unavailable(r, None);
            return core::ops::ControlFlow::Break(());
        }
    };
    // Reconstruct source gross in cents, then round the printed line once.
    // Adding separately rounded net and expenses can invent a dollar.
    let royalty_investment_income = match exact::dollars(&(&t.royalties + &royalty_expense_cents))
        .filter(|amount| amount.checked_mul(100).is_some())
    {
        Some(amount) => amount,
        None => {
            crate::extended_rules::expense_output_unavailable(r, None);
            return core::ops::ControlFlow::Break(());
        }
    };
    let royalty_borrowing = ded
        .investment_interest
        .as_ref()
        .is_some_and(|facts| facts.loans.iter().any(|loan| loan.source != 0));
    // Nonportfolio interest affects AGI. Finalize it before the existing
    // AGI-dependent owners, using their already collected source amounts.
    // Contexts needing later capital/passive/SALT/education projections are
    // explicitly unavailable here; never perform a whole-return retry.
    let royalty_routing =
        ded.investment_interest.as_ref().filter(|_| royalty_borrowing).and_then(|facts| {
            let early_income_complete = screen.capital_history.is_empty()
                && screen.installment_payments.is_empty()
                && !d
                    .income
                    .iter()
                    .any(|entry| matches!(entry.income, FederalIncome::InstallmentSale { .. }))
                && screen.related_exchanges.is_empty()
                && ded.personal_casualties.is_empty()
                && t.investment_gains == [0; 2]
                && t.amt_investment_gain_adjustment == [0; 2]
                && c.tax_credit_bonds.is_empty()
                && allocated_entries.iter().all(|entry| match &entry.income {
                    FederalIncome::Wage { .. } | FederalIncome::Interest { .. } => true,
                    FederalIncome::Dividend { data } => {
                        data.form.capital_distributions.unwrap_or(0) == 0
                    }
                    FederalIncome::Royalty { data } => data.ordinary,
                    _ => false,
                });
            let expenses_complete = ded.investment_property_taxes() == 0
                && ded.no_investment_income_tax()
                && !ded.other_itemized_lines().iter().any(|v| {
                    matches!(v,
                    crate::OtherItemizedLine::BondPremium { amount, .. } if *amount != 0)
                });
            if !early_income_complete
                || !expenses_complete
                || facts.election != crate::InvestmentElection::None
            {
                return None;
            }
            let income = dollars(t.interest) + dollars(t.dividends) - dollars(t.qualified)
                + royalty_investment_income;
            let foreign_taxes = crate::extended_rules::direct_foreign_tax_deduction(d);
            let itemizing = force_itemized.or(match d.deduction_choice() {
                DeductionChoice::Itemized => Some(true),

                DeductionChoice::Automatic => None,
            });
            let known_foreign_expense =
                if itemizing == Some(true) { dollars(foreign_taxes.portfolio) } else { 0 };
            let possible_foreign_expense = if itemizing == Some(false) {
                0
            } else {
                dollars(foreign_taxes.portfolio + foreign_taxes.unallocated_tax)
            };
            let forms = facts.aggregate().calculate(
                [facts.aggregate().paid; 2],
                income,
                royalty_investment_expenses + known_foreign_expense,
                dollars(t.private_activity),
                royalty_investment_expenses,
            );
            let endpoint = facts.aggregate().calculate(
                [facts.aggregate().paid; 2],
                income,
                royalty_investment_expenses + possible_foreign_expense,
                dollars(t.private_activity),
                royalty_investment_expenses,
            );
            // Known itemizing taxes can be used before AGI. An automatic
            // route or unresolved expense may proceed only if both endpoints
            // preserve every allowance/carry, avoiding a return-level retry.
            if forms != endpoint {
                return None;
            }
            facts.route(forms)
        });
    let investment_character_unavailable = ded.investment_interest().is_some()
        && ded.other_itemized_lines().iter().any(|v| {
            matches!(v,
            crate::OtherItemizedLine::BondPremium { amount, .. } if *amount != 0)
        });
    let investment_unavailable =
        (royalty_borrowing && royalty_routing.is_none()) || investment_character_unavailable;
    if investment_character_unavailable {
        note_once_scoped(
            r,
            msg!("NOTE_INVESTMENT_PREMIUM_CHARACTER"),
            crate::IssueContext::Section(crate::TaxSection::OtherItemized),
        );
    }
    if royalty_borrowing && royalty_routing.is_none() {
        note_once_scoped(
            r,
            msg!("NOTE_INVESTMENT_DESTINATION_UNAVAILABLE"),
            crate::IssueContext::Section(crate::TaxSection::OtherItemized),
        );
    }
    let own_investment = ded.investment_interest();
    let investment_paid =
        t.k1_interest.map(|amount| own_investment.as_ref().map_or(0, |facts| facts.paid) + amount);
    let k1_interest_record: Option<crate::InvestmentInterest> = {
        let own = own_investment;
        if t.k1_interest == [0; 2] {
            own
        } else {
            Some(match own {
                Some(mut record) => {
                    record.paid = investment_paid[0];
                    record
                }
                None => crate::InvestmentInterest {
                    paid: investment_paid[0],
                    carryover: 0,
                    amt_carryover: 0,
                    private_activity: 0,
                    election: crate::InvestmentElection::None,
                    income_use: crate::InvestmentInterestUse::Ordinary,
                },
            })
        }
    };
    core::ops::ControlFlow::Continue(InvestmentSourcesOut {
        royalty_investment_expenses,
        royalty_investment_income,
        royalty_routing,
        investment_unavailable,
        investment_paid,
        k1_interest_record,
    })
}

pub(super) struct InvestmentSourcePoolsIn<'a> {
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) amt_property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) property_sources_known_absent: bool,
    pub(super) estate_investment_carry: [Option<[i64; 2]>; 2],
    pub(super) investment_capital_in: [[i64; 2]; 2],
    pub(super) pure_loan_gain: i64,
    pub(super) pure_amt_loan_gain: i64,
    pub(super) k1_investment_ordinary: [i64; 2],
    pub(super) stock_loss: i64,
}

pub(super) struct InvestmentSourcePoolsOut {
    pub(super) scorp_investment: [i64; 2],
    pub(super) property_investment: [[i64; 2]; 2],
    pub(super) regular_property_investment_known_zero: bool,
    pub(super) investment_carry_max: [[i64; 2]; 2],
    pub(super) investment_carry_known: bool,
    pub(super) investment_gain: i64,
    pub(super) investment_net_capital: i64,
    pub(super) maximum_gain: i64,
    pub(super) maximum_net_capital: i64,
    pub(super) amt_investment_gain: i64,
    pub(super) amt_investment_net_capital: i64,
    pub(super) maximum_amt_gain: i64,
    pub(super) maximum_amt_net_capital: i64,
}

pub(super) fn investment_source_pools_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: InvestmentSourcePoolsIn<'_>,
) -> InvestmentSourcePoolsOut {
    let &Pass { d, allocated_entries, screen, context, .. } = pass;
    let source_claims = context.source_claims;
    let InvestmentSourcePoolsIn {
        investment_facts,
        rental,
        amt_rental,
        property_character,
        amt_property_character,
        property_sources_known_absent,
        estate_investment_carry,
        investment_capital_in,
        pure_loan_gain,
        pure_amt_loan_gain,
        k1_investment_ordinary,
        stock_loss,
    } = input;
    // The K-1 box 13 (12) code H amounts are Form 4952 line 1
    // interest of this return, with the filer's own and no 2025 carryover
    // of their own.
    let mut scorp_investment = [0; 2];
    if investment_facts.as_ref().is_some() {
        for entry in allocated_entries {
            if let FederalIncome::Scorp { data } = &entry.income {
                let data = &crate::ScorpK1::from(data);
                let regular = data.investment_interest_income();
                let alternative = if data.trade_kind == crate::ScorpTradeKind::Ordinary
                    || data.participation.as_ref().map(|v| v.material())
                        == Some(crate::participation::Participation::Yes)
                {
                    Ok(0)
                } else {
                    data.amt.as_ref().ok_or(("ISSUE_SCORP_AMT_INVESTMENT_INTEREST_RECORDS", crate::fixture_english!("Complete separate AMT trading records for the investment-interest limit.")))
                        .and_then(|v| v.source(data))
                        .and_then(|v| v.investment_interest_income())
                };
                for (i, amount) in [regular, alternative].into_iter().enumerate() {
                    match amount {
                        Ok(amount) => scorp_investment[i] += amount,
                        Err((key, _message)) => crate::issue_localized!(
                            r,
                            &if key
                                == "ISSUE_SCORP_TRADING_INCOME_WITH_RELEASED_PRIOR_PASSIVE_8CD8F8E2"
                            {
                                crate::interview::income_issue_context(entry).manual()
                            } else {
                                crate::interview::income_issue_context(entry)
                            },
                            "K-1 investment interest",
                            key,
                            _message,
                        ),
                    }
                }
            }
        }
    }
    let mut property_investment = [[0; 2]; 2];
    // Physical property can be complete without contributing investment gain.
    // Retain the actual regular native result, not an initialized zero after
    // a failed source call. The final owner packet is checked at consumption.
    let mut regular_property_investment_known_zero = property_sources_known_absent;
    if investment_facts.as_ref().is_some() {
        for (index, pool, character) in
            [(0, rental, property_character), (1, amt_rental, amt_property_character)]
        {
            if let Some(character) = character {
                match crate::section1231::investment_interest_amounts_with_claims(
                    d,
                    pool,
                    character,
                    index == 1,
                    source_claims,
                ) {
                    Ok(amounts) => {
                        property_investment[index] = amounts;
                        if index == 0 {
                            regular_property_investment_known_zero = amounts == [0; 2];
                        }
                    }
                    Err(error) => {
                        let (key, _message) = error.detail;
                        crate::issue_localized!(
                            r,
                            &error.context,
                            "Property investment interest",
                            key,
                            _message,
                        )
                    }
                }
            }
        }
    }
    // The ordinary Schedule D pool retains every capital loss. Form 4952
    // consumes only actual investment portions, with independent AMT facts.
    let mut investment_carry_min =
        estate_investment_carry.map(|parts| parts.unwrap_or([0; 2]).map(dollars));
    let mut investment_carry_max = investment_capital_in;
    for history in &screen.capital_history {
        if let Some(parts) = crate::extended_rules::investment_capital_parts(history) {
            let caps = crate::extended_rules::capital_caps(history);
            for (basis, full) in [
                [
                    crate::kernel::ratio::trunc_i64(caps.short, 100),
                    crate::kernel::ratio::trunc_i64(caps.long, 100),
                ],
                [
                    crate::kernel::ratio::trunc_i64(caps.amt_short, 100),
                    crate::kernel::ratio::trunc_i64(caps.amt_long, 100),
                ],
            ]
            .iter()
            .enumerate()
            {
                for (term, cap) in full.iter().enumerate() {
                    investment_carry_min[basis][term] += parts[basis][term];
                    investment_carry_max[basis][term] -= cap - parts[basis][term];
                }
            }
        }
    }
    let investment_carry_known = investment_carry_min == investment_carry_max;
    // One disposition-pool projection is shared by both tax systems and
    // both source-attribution endpoints. It is not another tax engine.
    let investment_gain_pool = |carry: [i64; 2], alternative: bool| {
        let basis = usize::from(alternative);
        let short = dollars(
            t.investment_gains[0]
                + if alternative { t.amt_investment_gain_adjustment[0] } else { 0 },
        ) - carry[0];
        let long = dollars(
            t.investment_gains[1]
                + if alternative { t.amt_investment_gain_adjustment[1] } else { 0 },
        ) - carry[1]
            + property_investment[basis][1];
        crate::investment_interest::gain_pool(
            short,
            long,
            if alternative {
                pure_amt_loan_gain + k1_investment_ordinary[1]
            } else {
                pure_loan_gain + k1_investment_ordinary[0]
            },
            stock_loss,
            property_investment[basis][0],
        )
    };
    let [investment_gain, investment_net_capital] =
        investment_gain_pool(investment_carry_max[0], false);
    let [maximum_gain, maximum_net_capital] = investment_gain_pool(investment_carry_min[0], false);
    let [amt_investment_gain, amt_investment_net_capital] =
        investment_gain_pool(investment_carry_max[1], true);
    let [maximum_amt_gain, maximum_amt_net_capital] =
        investment_gain_pool(investment_carry_min[1], true);
    InvestmentSourcePoolsOut {
        scorp_investment,
        property_investment,
        regular_property_investment_known_zero,
        investment_carry_max,
        investment_carry_known,
        investment_gain,
        investment_net_capital,
        maximum_gain,
        maximum_net_capital,
        amt_investment_gain,
        amt_investment_net_capital,
        maximum_amt_gain,
        maximum_amt_net_capital,
    }
}

pub(super) struct InvestmentElectionsIn<'a> {
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) royalty_routing: &'a Option<crate::investment_interest::InvestmentRouting>,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) amt_rental: &'a crate::extended_rules::RentalResult,
    pub(super) scorp_investment: [i64; 2],
    pub(super) investment_carry_known: bool,
    pub(super) investment_gain: i64,
    pub(super) investment_net_capital: i64,
    pub(super) maximum_net_capital: i64,
    pub(super) amt_investment_gain: i64,
    pub(super) amt_investment_net_capital: i64,
    pub(super) maximum_amt_net_capital: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) qualified: i64,
    pub(super) amt_qualified: i64,
    pub(super) net_capital_gain: i64,
    pub(super) amt_net_capital_gain: i64,
    pub(super) able_income: i64,
    pub(super) education_income: i64,
    pub(super) royalty_investment_income: i64,
    pub(super) k1_amt_interest_adjustment: i64,
    pub(super) k1_amt_dividend_adjustment: i64,
    pub(super) k1_amt_qualified_adjustment: i64,
    pub(super) amt_schedule_d_18: i64,
    pub(super) amt_schedule_d_19: i64,
    pub(super) investment_paid: [i64; 2],
    pub(super) investment_expenses: i64,
    pub(super) royalty_investment_expenses: i64,
}

pub(super) struct InvestmentElectionsOut {
    pub(super) capital_election_unavailable: bool,
    pub(super) elected: [i64; 2],
    pub(super) tax_net_capital: i64,
    pub(super) investment_ordinary_income: i64,
    pub(super) investment_income: i64,
    pub(super) amt_elected: [i64; 2],
    pub(super) amt_tax_net_capital: i64,
    pub(super) amt_tax_pref: i64,
    pub(super) amt_tax_unrecaptured: i64,
    pub(super) amt_tax_collectible: i64,
    pub(super) amt_investment_adjustment: i64,
    pub(super) investment_interest: [[i64; 2]; 2],
    pub(super) portfolio_interest: [i64; 2],
}

pub(super) fn investment_elections_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: InvestmentElectionsIn<'_>,
) -> InvestmentElectionsOut {
    let &Pass { allocated_entries, .. } = pass;
    let InvestmentElectionsIn {
        investment_facts,
        royalty_routing,
        rental,
        amt_rental,
        scorp_investment,
        investment_carry_known,
        investment_gain,
        investment_net_capital,
        maximum_net_capital,
        amt_investment_gain,
        amt_investment_net_capital,
        maximum_amt_net_capital,
        interest,
        dividends,
        qualified,
        amt_qualified,
        net_capital_gain,
        amt_net_capital_gain,
        able_income,
        education_income,
        royalty_investment_income,
        k1_amt_interest_adjustment,
        k1_amt_dividend_adjustment,
        k1_amt_qualified_adjustment,
        amt_schedule_d_18,
        amt_schedule_d_19,
        investment_paid,
        investment_expenses,
        royalty_investment_expenses,
    } = input;
    // A child's qualified dividends and capital gain distributions reported on Form 8814 are not
    // investment income unless chosen, but the line 4g election may include all or part of them
    // (Pub. 550, "Investment income of child reported on parent's return"; section 163(d)(4)(B)).
    let capital_election_unavailable = !investment_carry_known
        && investment_facts.as_ref().is_some_and(|v| {
            let low = v.election.amounts(investment_net_capital, qualified);
            let high = v.election.amounts(maximum_net_capital, qualified);
            low != high
                || low.is_some_and(|e| {
                    v.election.amt_amounts(e, amt_investment_net_capital, amt_qualified)
                        != v.election.amt_amounts(e, maximum_amt_net_capital, amt_qualified)
                })
        });
    let elected = investment_facts.as_ref().map_or(Some([0; 2]), |v| {
        if capital_election_unavailable {
            Some([0; 2])
        } else {
            v.election.amounts(investment_net_capital, qualified)
        }
    });
    crate::require_localized!(
        r,
        elected.is_some(),
        &(crate::IssueContext::Section(crate::TaxSection::OtherItemized)).manual(),
        "Investment interest",
        "ISSUE_ELECTED_GAINS_AND_DIVIDENDS_CANNOT_EXCEED_THE_CORRESPONDING_INVESTM_2D32B42A",
        "Elected gains and dividends cannot exceed the corresponding investment income available for election.",
    );
    let elected = elected.unwrap_or([0; 2]);
    // Schedule D worksheet lines 6, 9, 10, 12 and 35. QBI below deliberately
    // retains the unmodified Form 1040/Schedule D amounts (Form 8995 line 12).
    let tax_net_capital = positive(net_capital_gain - elected[0]);
    let estate_investment_adjustment = allocated_entries
        .iter()
        .filter_map(|entry| match &entry.income {
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                crate::estate_trust::investment_adjustment(data)
            }
            _ => None,
        })
        .fold([0i64; 2], |mut total, amounts| {
            total[0] += amounts[0];
            total[1] += amounts[1];
            total
        });
    let investment_ordinary_income = interest + dividends - qualified
        + dollars(t.k1_investment_income)
        + able_income
        + education_income
        + dollars(estate_investment_adjustment[0])
        + t.child.investment
        + dollars(t.nonqualified_annuity)
        + royalty_investment_income
        + dollars(t.investment_rent[0])
        + rental.ptp_income
        + investment_gain
        - investment_net_capital;
    let investment_income =
        investment_ordinary_income + scorp_investment[0] + elected.iter().sum::<i64>();
    // Form6251 line2c step3 reuses the actual regular election, bounded
    // independently by the AMT pools. Missing source endpoints never invent
    // a different gain/dividend allocation.
    let amt_elected = investment_facts.as_ref().map_or(Some([0; 2]), |v| {
        v.election.amt_amounts(elected, amt_investment_net_capital, amt_qualified)
    });
    crate::require_localized!(
        r,
        amt_elected.is_some(),
        &crate::IssueContext::Section(crate::TaxSection::OtherItemized),
        "AMT investment election",
        "ISSUE_THE_ELECTED_CAPITAL_GAIN_EXCEEDS_THE_AVAILABLE_AMT_INVESTMENT_CAPIT_6CBAA2D7",
        "The elected capital gain or dividends exceeds its available AMT pool; review the actual election and AMT records.",
    );
    let amt_elected = amt_elected.unwrap_or([0; 2]);
    let amt_tax_net_capital = positive(amt_net_capital_gain - amt_elected[0]);
    let amt_tax_pref = positive(amt_qualified - amt_elected[1]) + amt_tax_net_capital;
    let amt_tax_unrecaptured = phase::cap(amt_tax_net_capital, [amt_schedule_d_19]);
    let amt_tax_collectible =
        phase::cap(amt_tax_net_capital - amt_tax_unrecaptured, [amt_schedule_d_18]);
    let amt_investment_adjustment = scorp_investment[1] - scorp_investment[0]
        + k1_amt_interest_adjustment
        + k1_amt_dividend_adjustment
        - k1_amt_qualified_adjustment
        + dollars(estate_investment_adjustment[1] - estate_investment_adjustment[0])
        + dollars(t.amt_royalty_adjustment)
        + dollars(t.private_activity)
        + dollars(t.investment_rent[1])
        - dollars(t.investment_rent[0])
        + amt_rental.ptp_income
        - rental.ptp_income
        + amt_investment_gain
        - amt_investment_net_capital
        - (investment_gain - investment_net_capital)
        + amt_elected.iter().sum::<i64>()
        - elected.iter().sum::<i64>();
    let mut investment_interest = royalty_routing.as_ref().map_or_else(
        || {
            investment_facts.as_ref().map_or([[0; 2]; 2], |v| {
                v.calculate(
                    investment_paid,
                    investment_income,
                    investment_expenses,
                    amt_investment_adjustment,
                    royalty_investment_expenses,
                )
            })
        },
        |routing| routing.forms,
    );
    // Reg. 1.642(h)-2(a) allows a final-year excess deduction only in the
    // beneficiary's year in which the estate or trust terminates. Whether
    // the part the section 163(d)(1) limit disallows still carries forward
    // under section 163(d)(2) is not settled, so that part stays off Form
    // 4952 line 7 and the estimate is refused on the K-1.
    if t.estate_investment_interest > 0 && investment_interest.iter().any(|tax| tax[1] > 0) {
        for tax in &mut investment_interest {
            tax[1] = positive(tax[1] - dollars(t.estate_investment_interest));
        }
        for entry in allocated_entries {
            if let FederalIncome::EstateTrustPortfolio { data } = &entry.income
                && data.statements.final_character.as_ref().is_some_and(|f| {
                    f.additional.iter().any(|item| {
                        matches!(item, crate::EstateFinalDeduction::InvestmentInterest { .. })
                    })
                })
            {
                crate::issue_localized!(
                    r,
                    &crate::interview::income_issue_context(entry),
                    "Final-year investment interest",
                    "ISSUE_ESTATE_FINAL_INTEREST_LIMITED",
                    "Your net investment income doesn't absorb all the final-year box 11 code B investment interest. Regulations section 1.642(h)-2(a) allows this deduction only in the year the estate or trust ends, and whether the part section 163(d) disallows carries forward isn't settled, so this return can't be completed.",
                );
            }
        }
    }
    let portfolio_interest = royalty_routing
        .as_ref()
        .map_or([investment_interest[0][0], investment_interest[1][0]], |v| v.portfolio());
    InvestmentElectionsOut {
        capital_election_unavailable,
        elected,
        tax_net_capital,
        investment_ordinary_income,
        investment_income,
        amt_elected,
        amt_tax_net_capital,
        amt_tax_pref,
        amt_tax_unrecaptured,
        amt_tax_collectible,
        amt_investment_adjustment,
        investment_interest,
        portfolio_interest,
    }
}

pub(super) struct InvestmentInterestRefigureIn<'a> {
    pub(super) investment_facts: Option<&'a crate::InvestmentInterest>,
    pub(super) royalty_routing: &'a Option<crate::investment_interest::InvestmentRouting>,
    pub(super) foreign_tax_deduction: &'a crate::extended_rules::ForeignTaxDeduction,
    pub(super) use_itemized: bool,
    pub(super) capital_election_unavailable: bool,
    pub(super) investment_carry_known: bool,
    pub(super) investment_interest: [[i64; 2]; 2],
    pub(super) portfolio_interest: [i64; 2],
    pub(super) investment_paid: [i64; 2],
    pub(super) elected: [i64; 2],
    pub(super) amt_elected: [i64; 2],
    pub(super) investment_gain: i64,
    pub(super) investment_net_capital: i64,
    pub(super) maximum_gain: i64,
    pub(super) maximum_net_capital: i64,
    pub(super) amt_investment_gain: i64,
    pub(super) amt_investment_net_capital: i64,
    pub(super) maximum_amt_gain: i64,
    pub(super) maximum_amt_net_capital: i64,
    pub(super) investment_ordinary_income: i64,
    pub(super) investment_income: i64,
    pub(super) amt_investment_adjustment: i64,
    pub(super) investment_expenses: i64,
    pub(super) royalty_investment_expenses: i64,
    pub(super) investment_property_taxes: i64,
    pub(super) investment_tax_bounds: [i64; 2],
    pub(super) foreign_portfolio_tax: i64,
}

pub(super) struct InvestmentInterestRefigureOut {
    pub(super) investment_interest: [[i64; 2]; 2],
    pub(super) portfolio_interest: [i64; 2],
}

pub(super) fn investment_interest_refigure_phase(
    r: &mut FederalResult,
    input: InvestmentInterestRefigureIn<'_>,
) -> InvestmentInterestRefigureOut {
    let InvestmentInterestRefigureIn {
        investment_facts,
        royalty_routing,
        foreign_tax_deduction,
        use_itemized,
        capital_election_unavailable,
        investment_carry_known,
        mut investment_interest,
        mut portfolio_interest,
        investment_paid,
        elected,
        amt_elected,
        investment_gain,
        investment_net_capital,
        maximum_gain,
        maximum_net_capital,
        amt_investment_gain,
        amt_investment_net_capital,
        maximum_amt_gain,
        maximum_amt_net_capital,
        investment_ordinary_income,
        investment_income,
        amt_investment_adjustment,
        investment_expenses,
        royalty_investment_expenses,
        investment_property_taxes,
        investment_tax_bounds,
        foreign_portfolio_tax,
    } = input;
    if let Some(interest_facts) = investment_facts.as_ref() {
        if !use_itemized && royalty_routing.is_none() {
            investment_interest = interest_facts.calculate(
                investment_paid,
                investment_income,
                royalty_investment_expenses,
                amt_investment_adjustment,
                royalty_investment_expenses,
            );
            portfolio_interest = [investment_interest[0][0], investment_interest[1][0]];
        }
        // Missing portions are benefit-only uncertainty. Compare the opposite
        // complete loss endpoint, including both allowance and future carry.
        let maximum_income_delta =
            maximum_gain - maximum_net_capital - (investment_gain - investment_net_capital);
        let maximum_amt_delta = maximum_amt_gain
            - maximum_amt_net_capital
            - (amt_investment_gain - amt_investment_net_capital);
        let other_endpoint = interest_facts.calculate(
            investment_paid,
            investment_income + maximum_income_delta,
            if use_itemized { investment_expenses } else { royalty_investment_expenses },
            amt_investment_adjustment + maximum_amt_delta - maximum_income_delta,
            royalty_investment_expenses,
        );
        if capital_election_unavailable
            || (!investment_carry_known && investment_interest != other_endpoint)
        {
            note_once_scoped(
                r,
                msg!("NOTE_INVESTMENT_CAPITAL_PARTS"),
                crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
            );
        }

        emit_lines(
            r,
            &mut (),
            [
                ("investment_elected_capital_gain", elected[0]),
                ("investment_elected_qualified_dividends", elected[1]),
                ("amt_investment_elected_capital_gain", amt_elected[0]),
                ("amt_investment_elected_qualified_dividends", amt_elected[1]),
                ("investment_interest_net_gain", investment_gain),
                ("investment_interest_net_capital_gain", investment_net_capital),
                ("investment_interest_ordinary_income", investment_ordinary_income),
            ],
            |(id, amount), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["income", "screening.capital_history"],
                    "https://www.irs.gov/pub/irs-pdf/f4952.pdf",
                    msg!("EXPLAIN_FEDERAL_RULES_5"),
                ))
            },
        );
        // Only actual allowed deductions enter line 5. Compare complete
        // allowance/carry endpoints; use the conservative benefit when
        // unresolved and preserve ordinary income/tax.
        let lower_expense_interest = interest_facts.calculate(
            investment_paid,
            investment_income,
            royalty_investment_expenses
                + investment_property_taxes
                + investment_tax_bounds[0]
                + foreign_portfolio_tax,
            amt_investment_adjustment,
            royalty_investment_expenses,
        );
        // Unlike known portfolio taxes, unclassified K-3/carry taxes cannot
        // be assigned to line 5. Compare both endpoints through the same owner;
        // unresolved classification must not increase interest or its carry.
        let foreign_expense_endpoint = interest_facts.calculate(
            investment_paid,
            investment_income,
            investment_expenses + dollars(foreign_tax_deduction.unallocated_tax),
            amt_investment_adjustment,
            royalty_investment_expenses,
        );
        crate::require_localized!(
            r,
            !use_itemized || investment_interest[0] == foreign_expense_endpoint[0],
            &(crate::IssueContext::Section(crate::TaxSection::OtherItemized)).manual(),
            "Investment interest",
            "ISSUE_INCOME_TAX_EXPENSE_ATTRIBUTION_COULD_CHANGE_THE_FORM_4952_DEDUCTION_E4F6979E",
            "Income-tax expense attribution could change the Form 4952 deduction or carryforward and needs additional coordination.",
        );
        if use_itemized && investment_interest[0] != lower_expense_interest[0] {
            note_once_scoped(
                r,
                msg!("NOTE_INVESTMENT_TAX_ALLOCATION"),
                crate::IssueContext::Section(crate::TaxSection::Taxes),
            );
        }
    }
    InvestmentInterestRefigureOut { investment_interest, portfolio_interest }
}
