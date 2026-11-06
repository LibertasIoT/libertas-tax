// payments items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// The review's "Credits" count: care, education, the saver's credit and any
// more-credits entry.
// The accepted return without Form 2210: the lines the payments pages show
// or derive (withholding, Schedule AI column (d)) do not depend on the
// penalty, and a Form 2210 answer still being entered (box C before its
// periods) must not hide them.
pub(super) fn return_before_penalty(d: &FederalDraft) -> FederalResult {
    let mut d = d.clone();
    if let Some(pay) = d.payments.as_mut() {
        pay.prior_year = None;
    }
    estimate(&d)
}
// Form 1040 lines 25a-25d of the accepted return, in cents: the payments
// page shows them and the box D periods must add up to line 25d.
pub(super) fn withheld(d: &FederalDraft) -> Option<Withheld> {
    return_before_penalty(d).payment_withholding
}
// Schedule AI column (d) from the accepted return (M3-P14): AGI (Form 1040
// line 11), the itemized deductions taken (line 12e, 0 with the standard
// deduction), the Schedule 1-A deductions (line 13a), the QBI deduction
// (line 13b), and the net profit and Social Security wages of the person
// with self-employment tax.
pub(super) fn full_year_period(d: &FederalDraft) -> AnnualizedPeriod {
    let result = return_before_penalty(d);
    AnnualizedPeriod {
        agi: form_1040_line(&result, "11a / 11b"),
        itemized: calculation_line(&result, "schedule_ai_full_year_itemized"),
        qbi: form_1040_line(&result, "13b"),
        additional: form_1040_line(&result, "13a"),
        self_employment: calculation_line(&result, "schedule_ai_full_year_self_employment"),
        social_wages: calculation_line(&result, "schedule_ai_full_year_social_wages"),
    }
}
// The full year spread evenly through a period ending after `months`: the
// page's starting values before the filer enters the actual amounts, which
// annualize back to the full year, so the election changes nothing until
// they are entered.
pub(super) fn even_period(full: &AnnualizedPeriod, months: i64) -> AnnualizedPeriod {
    use crate::kernel::ratio::fraction;
    AnnualizedPeriod {
        agi: fraction(full.agi, months, 12),
        itemized: fraction(full.itemized, months, 12),
        qbi: fraction(full.qbi, months, 12),
        additional: fraction(full.additional, months, 12),
        self_employment: fraction(full.self_employment, months, 12),
        social_wages: fraction(full.social_wages, months, 12),
    }
}
// Box C (the annualized income installment method) is chosen.
pub(super) fn box_c(d: &FederalDraft) -> bool {
    d.payments.as_ref().is_some_and(|p| p.annualized)
}
// Column (d) follows the return: every accepted change re-derives it, so
// the stored Schedule AI record always reconciles to the return.
pub(super) fn follow_full_year(d: &mut FederalDraft) {
    if d.payments.as_ref().is_some_and(|p| p.annualized_income.is_some()) {
        let full = full_year_period(d);
        if let Some(periods) = d.payments.as_mut().and_then(|p| p.annualized_income.as_mut()) {
            periods.full_year = full;
        }
    }
}
pub(super) fn waiver_record(waiver: &Waiver) -> Option<PenaltyWaiver> {
    match waiver {
        Waiver::None => None,
        Waiver::Full { reason, explanation } => {
            Some(PenaltyWaiver::Full { reason: *reason, explanation: explanation.clone() })
        }
        Waiver::Partial { reason, explanation, amount } => Some(PenaltyWaiver::Partial {
            reason: *reason,
            explanation: explanation.clone(),
            amount: *amount,
        }),
    }
}
pub(super) fn waiver_item(waiver: &Option<PenaltyWaiver>) -> Waiver {
    match waiver {
        None => Waiver::None,
        Some(PenaltyWaiver::Full { reason, explanation }) => {
            Waiver::Full { reason: *reason, explanation: explanation.clone() }
        }
        Some(PenaltyWaiver::Partial { reason, explanation, amount }) => {
            Waiver::Partial { reason: *reason, explanation: explanation.clone(), amount: *amount }
        }
    }
}
// The stored Form 2210 facts as the page's penalty choice; None when the
// record holds a shape this page could not have written.
// The stored 2027 estimate as the payments page shows it (T2-49c): the
// farmer and fisherman answer inside the set the page offers, "Not needed"
// when 2026 qualifies, and a stored "Not needed" that no longer applies
// constructed as an unanswered required choice.
pub(super) fn next_year_item(
    stored: &crate::NextYearEstimateRecord,
    farming_return_year: bool,
) -> Option<NextYearEstimate> {
    if !farming_return_year
        && matches!(
            NextYearEstimate::from(stored.clone()),
            NextYearEstimate::SameAsThisReturn {
                farmer_fisherman_next_year: FarmerNextYear::NotNeeded
            } | NextYearEstimate::WithChanges {
                farmer_fisherman_next_year: FarmerNextYear::NotNeeded,
                ..
            }
        )
    {
        return None;
    }
    let shown = |answer: FarmerNextYear| match (farming_return_year, answer) {
        (true, _) => FarmerNextYear::NotNeeded,
        (false, FarmerNextYear::NotNeeded) => FarmerNextYear::NotNeeded,
        (false, answer) => answer,
    };
    let mut item = NextYearEstimate::from(stored.clone());
    match &mut item {
        NextYearEstimate::NotNeeded => {}
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year }
        | NextYearEstimate::WithChanges { farmer_fisherman_next_year, .. } => {
            *farmer_fisherman_next_year = shown(*farmer_fisherman_next_year)
        }
    }
    Some(item)
}
pub(super) fn penalty_item(
    pay: &FederalPayments,
    farming_return_year: bool,
) -> Option<PenaltyChoice> {
    let Some(prior) = &pay.prior_year else {
        let untouched = !pay.withholding_by_date
            && pay.withholding_periods.is_none()
            && !pay.annualized
            && pay.annualized_income.is_none()
            && !pay.farmer_fisherman_prior_year
            && !pay.farmer_paid_by_march
            && pay.waiver.is_none();
        return untouched.then_some(PenaltyChoice::IrsFigures);
    };
    let withholding = match (pay.withholding_by_date, &pay.withholding_periods) {
        (false, None) => WithholdingTiming::Evenly,
        (true, Some(p)) => WithholdingTiming::ByPeriod {
            first: p.first,
            second: p.second,
            third: p.third,
            fourth: p.fourth,
        },
        _ => return None,
    };
    let income = match (pay.annualized, &pay.annualized_income) {
        (true, _) => IncomeTiming::ByPeriod,
        (false, None) => IncomeTiming::Evenly,
        (false, Some(_)) => return None,
    };
    // 2026 figured from this return decides first; the 2025 answer is shown
    // only when 2026 does not qualify, and the March 1 answer only with Yes.
    let farmer = match (farming_return_year, pay.farmer_fisherman_prior_year) {
        (true, _) => Farmer::ThisReturn { paid_by_march_1: pay.farmer_paid_by_march },
        (false, false) => Farmer::No,
        (false, true) => Farmer::Yes { paid_by_march_1: pay.farmer_paid_by_march },
    };
    Some(PenaltyChoice::Figure {
        prior_tax: prior.tax,
        prior_agi: prior.agi,
        full_year: prior.full_year,
        prior_return: match prior.spouse_tax {
            0 => PriorReturn::SameReturn,
            spouse_tax => PriorReturn::SeparateReturns { spouse_tax },
        },
        withholding,
        income,
        farmer,
        waiver: waiver_item(&pay.waiver),
    })
}
// The page preserves the selected estimated-payment method and its sources;
// IrsFigures leaves every Form 2210 fact empty; box C keeps the
// income by period already entered, Evenly erases it. Home kerosene is not
// on this page and is kept, and so are the refund choices, asked on the
// review.
pub(super) fn payments_record(value: &Payments, kept: &FederalPayments) -> FederalPayments {
    let mut pay = FederalPayments {
        credit_applied_next_year: kept.credit_applied_next_year,
        credit_recipient: kept.credit_recipient,
        estimated: value.estimated.accepted(),
        extension: value.extension,
        home_kerosene: kept.home_kerosene.clone(),
        extension_payer: value.more.extension_payer,
        extension_refund_property: value
            .more
            .extension_refund_property
            .as_ref()
            .map(crate::RefundSourceProperty::accepted),
        next_year: value.more.next_year.clone().into(),
        ..FederalPayments::none()
    };
    if let PenaltyChoice::Figure {
        prior_tax,
        prior_agi,
        full_year,
        prior_return,
        withholding,
        income,
        farmer,
        waiver,
    } = &value.more.penalty
    {
        pay.prior_year = Some(PriorYearTax {
            full_year: *full_year,
            tax: *prior_tax,
            agi: *prior_agi,
            spouse_tax: match prior_return {
                PriorReturn::SameReturn => 0,
                PriorReturn::SeparateReturns { spouse_tax } => *spouse_tax,
            },
        });
        if let WithholdingTiming::ByPeriod { first, second, third, fourth } = withholding {
            pay.withholding_by_date = true;
            pay.withholding_periods = Some(PeriodWithholding {
                first: *first,
                second: *second,
                third: *third,
                fourth: *fourth,
            });
        }
        if *income == IncomeTiming::ByPeriod {
            pay.annualized = true;
            pay.annualized_income = kept.annualized_income.clone();
        }
        // The 2025 answer is Yes only when chosen; with 2026 from this return
        // it is not asked and stays No.
        match farmer {
            Farmer::No => {}
            Farmer::Yes { paid_by_march_1 } => {
                pay.farmer_fisherman_prior_year = true;
                pay.farmer_paid_by_march = *paid_by_march_1;
            }
            Farmer::ThisReturn { paid_by_march_1 } => {
                pay.farmer_paid_by_march = *paid_by_march_1;
            }
        }
        pay.waiver = waiver_record(waiver);
    }
    pay
}
pub(super) fn payments_count(pay: &FederalPayments) -> usize {
    pay.estimated.count() + usize::from(pay.extension != 0)
}

impl Interview {
    // Which Form 2210 answers this return offers: box E (separate 2025
    // returns) only on a joint return; the farmer and fisherman choices by the
    // 2026 test (allowed_farmer), on every return.
    pub(super) fn allowed_prior_returns(&self) -> Vec<i32> {
        let mut choices =
            alloc::vec![libertas_macros::variant_index!(PriorReturn::SameReturn) as i32];
        if self.joint() {
            choices.push(libertas_macros::variant_index!(PriorReturn::SeparateReturns) as i32);
        }
        choices
    }
    // Form 2210's farmer and fisherman choices (T2-49b): only "Yes, for
    // 2026" when this return shows two-thirds of 2026 gross income from
    // farming or fishing; otherwise the 2025 question.
    pub(super) fn allowed_farmer(&self) -> Vec<i32> {
        if self.farming_return_year() {
            alloc::vec![libertas_macros::variant_index!(Farmer::ThisReturn) as i32]
        } else {
            alloc::vec![
                libertas_macros::variant_index!(Farmer::No) as i32,
                libertas_macros::variant_index!(Farmer::Yes) as i32,
            ]
        }
    }
    // Form 1040-ES's 2027 farmer and fisherman choices (T2-49c): only "Not
    // needed" when this return already meets the two-thirds test for 2026;
    // otherwise the 2027 question.
    pub(super) fn allowed_farmer_next_year(&self) -> Vec<i32> {
        if self.farming_return_year() {
            alloc::vec![FarmerNextYear::NotNeeded as i32]
        } else {
            alloc::vec![FarmerNextYear::Yes as i32, FarmerNextYear::No as i32,]
        }
    }
    // The farmer and fisherman two-thirds test for 2026 as the estimate
    // figures it from this return; not met when the estimate is refused.
    pub(super) fn farming_return_year(&self) -> bool {
        let r = estimate(&self.draft);
        let line = |id: &str| r.lines.iter().find(|l| l.id == id).map(|l| l.amount);
        match (line("farming_fishing_gross_income_return_year"), line("gross_income_return_year")) {
            (Some(farming), Some(gross)) => {
                crate::underpayment::TwoThirds { farming, gross }.qualifies()
            }
            _ => false,
        }
    }
    // Accepted methods reopen losslessly. A first visit requires an explicit
    // estimated-payment method; constructor absence is not an answered No.
    // Withholding is derived from the current return.
    pub(super) fn payments_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let pay = d.payments.clone().unwrap_or_else(FederalPayments::none);
        let farming = self.farming_return_year();
        let penalty = penalty_item(&pay, farming);
        let Some(penalty) = penalty else {
            return problem_key("PROBLEM_UNDISPLAYABLE_PAYMENTS_RECORD");
        };
        let next_year = next_year_item(&pay.next_year, farming);
        let result = estimate(d);
        let initial_next_year = match &NextYearEstimate::from(pay.next_year.clone()) {
            NextYearEstimate::NotNeeded => crate::federal_model::NextYearInitial::NotNeeded,
            NextYearEstimate::SameAsThisReturn { .. } => {
                crate::federal_model::NextYearInitial::SameAsThisReturn
            }
            NextYearEstimate::WithChanges { agi, deductions, credits, withholding, .. } => {
                crate::federal_model::NextYearInitial::WithChanges {
                    agi: *agi,
                    deductions: *deductions,
                    credits: *credits,
                    withholding: *withholding,
                }
            }
        };
        let answered = d.finished_sections.contains(&TaxSection::Payments);
        if next_year.is_none() || !answered {
            return Reply::Response(P::BeginPayments {
                cookie: d.cookie.clone(),
                revision: d.revision,
                page: PAYMENTS_PAGE,
                previous: self.previous_shown(TaxSection::Payments),
                review,
                estimate: self.estimate_so_far(d, &result, TaxSection::Payments).map(Into::into),
                warnings: Vec::new(),
                progress: self.progress(PAYMENTS_PAGE),
                allowed_prior_returns: self.allowed_prior_returns(),
                allowed_farmer: self.allowed_farmer(),
                allowed_farmer_next_year: self.allowed_farmer_next_year(),
                allowed_refund_management: crate::injured_spouse_payments::allowed_management(
                    crate::injured_spouse::texas(d),
                ),
                value: crate::federal_model::PaymentsInitial {
                    withheld: result.payment_withholding.clone(),
                    estimated: (answered || pay.estimated.count() != 0)
                        .then(|| pay.estimated.page()),
                    extension: pay.extension,
                    more: crate::federal_model::PaymentsMoreInitial {
                        extension_payer: pay.extension_payer,
                        extension_refund_property: pay
                            .extension_refund_property
                            .as_ref()
                            .map(crate::RefundSourcePropertyFacts::page),
                        penalty,
                        next_year: initial_next_year,
                    },
                },
            });
        }
        Reply::Edit(P::SavePayments {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: PAYMENTS_PAGE,
            previous: self.previous_shown(TaxSection::Payments),
            review,
            estimate: self.estimate_so_far(d, &result, TaxSection::Payments),
            warnings: Vec::new(),
            progress: self.progress(PAYMENTS_PAGE),
            allowed_prior_returns: self.allowed_prior_returns(),
            allowed_farmer: self.allowed_farmer(),
            allowed_farmer_next_year: self.allowed_farmer_next_year(),
            allowed_refund_management: crate::injured_spouse_payments::allowed_management(
                crate::injured_spouse::texas(d),
            ),
            value: Payments {
                withheld: result.payment_withholding.clone(),
                estimated: pay.estimated.page(),
                extension: pay.extension,
                more: crate::PaymentsMore {
                    extension_payer: pay.extension_payer,
                    extension_refund_property: pay
                        .extension_refund_property
                        .as_ref()
                        .map(crate::RefundSourcePropertyFacts::page),
                    penalty,
                    next_year: next_year.expect("answered next-year estimate"),
                },
            },
        })
    }
    // The income by period page (M3-P14): the stored periods, or the full
    // year spread evenly the first time, so it opens complete; column (d)
    // is derived from the return. Back returns to the payments page.
    pub(super) fn income_by_period_page(&self, review: bool) -> Reply {
        let d = &self.draft;
        let full_year = full_year_period(d);
        let stored = d.payments.as_ref().and_then(|p| p.annualized_income.as_ref());
        let (through_march, through_may, through_august) = match stored {
            Some(v) => (v.through_march.clone(), v.through_may.clone(), v.through_august.clone()),
            None => {
                (even_period(&full_year, 3), even_period(&full_year, 5), even_period(&full_year, 8))
            }
        };
        Reply::Edit(P::SaveIncomeByPeriod {
            user: self.user,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: INCOME_BY_PERIOD_PAGE,
            previous: PAYMENTS_PAGE,
            review,
            estimate: self.estimate_now(TaxSection::IncomeByPeriod),
            warnings: Vec::new(),
            progress: self.progress(INCOME_BY_PERIOD_PAGE),
            value: IncomeByPeriod { through_march, through_may, through_august, full_year },
        })
    }
    // Validate payment dates, the box D total against actual withholding,
    // and box E and farmer choices against the offered answers.
    pub(super) fn payments_candidate(&self, value: &Payments) -> Result<FederalDraft, SaveFailure> {
        if (value.extension == 0) != value.more.extension_payer.is_none() {
            return Err(ValidationIssue::new("INJURED_SPOUSE_EXTENSION_PAYER_REQUIRED")
                .at_field("extension_payer")
                .in_field("more")
                .in_field("value")
                .into());
        }
        let allowed = crate::injured_spouse_payments::allowed_management(
            crate::injured_spouse::texas(&self.draft),
        );
        if let Some(issue) = value.estimated.accepted().issue(&allowed) {
            return Err(issue.in_field("estimated").in_field("value").into());
        }
        if let Some(mut issue) = value
            .more
            .extension_refund_property
            .as_ref()
            .and_then(|property| property.accepted().issue(value.extension, &allowed))
        {
            issue.key = "INJURED_SPOUSE_COMMUNITY_ABOVE_PAYMENT";
            return Err(issue
                .in_field("extension_refund_property")
                .in_field("more")
                .in_field("value")
                .into());
        }
        // The 2027 estimate's expected amounts (T2-49): the schema's ranges,
        // checked again on save.
        if let NextYearEstimate::WithChanges { agi, deductions, credits, withholding, .. } =
            &value.more.next_year
        {
            let issue = |field| {
                ValidationIssue::new(
                    "MESSAGE_AN_EXPECTED_2027_AMOUNT_IS_OUTSIDE_THE_SUPPORTED_RANGE",
                )
                .at_field(field)
                .in_field("next_year")
                .in_field("more")
                .in_field("value")
            };
            if agi.is_some_and(|v| v.abs() > 100_000_000_000) {
                return Err(issue("agi").into());
            }
            for (field, amount) in
                [("deductions", deductions), ("credits", credits), ("withholding", withholding)]
            {
                if amount.is_some_and(|v| !(0..=100_000_000_000).contains(&v)) {
                    return Err(issue(field).into());
                }
            }
        }
        if let NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year }
        | NextYearEstimate::WithChanges { farmer_fisherman_next_year, .. } =
            &value.more.next_year
            && !self.allowed_farmer_next_year().contains(&(*farmer_fisherman_next_year as i32))
        {
            return Err(ValidationIssue::new(
                "MESSAGE_CHOOSE_NOT_NEEDED_FOR_2027_FARMING_OR_FISHING_ONLY_WHEN_THIS_RETURN_SHOWS_AT",
            ).at_field("farmer_fisherman_next_year").in_field("next_year").in_field("more").in_field("value").into());
        }
        if let PenaltyChoice::Figure { prior_return, withholding, income, farmer, waiver, .. } =
            &value.more.penalty
        {
            if let WithholdingTiming::ByPeriod { first, second, third, fourth } = withholding {
                let actual = withheld(&self.draft).ok_or_else(|| {
                    ValidationIssue::new("PAYMENT_WITHHOLDING_UNAVAILABLE")
                        .at_field("withholding")
                        .in_field("penalty")
                        .in_field("more")
                        .in_field("value")
                })?;
                if crate::kernel::ratio::checked_sum([*first, *second, *third, *fourth])
                    != Some(actual.total)
                {
                    return Err(ValidationIssue::new("WITHHOLDING_PERIODS_NOT_TOTAL")
                        .at_field("withholding")
                        .in_field("penalty")
                        .in_field("more")
                        .in_field("value")
                        .into());
                }
            }
            // Form 2210-F has one payment, due January 15, 2027: a farmer or
            // fisher has neither box C nor box D.
            if !matches!(farmer, Farmer::No)
                && (matches!(withholding, WithholdingTiming::ByPeriod { .. })
                    || *income == IncomeTiming::ByPeriod)
            {
                let field = if matches!(withholding, WithholdingTiming::ByPeriod { .. }) {
                    "withholding"
                } else {
                    "income"
                };
                return Err(ValidationIssue::new("FARMER_EVEN_TIMING")
                    .at_field(field)
                    .in_field("penalty")
                    .in_field("more")
                    .in_field("value")
                    .into());
            }
            let prior_index = match prior_return {
                PriorReturn::SameReturn => {
                    libertas_macros::variant_index!(PriorReturn::SameReturn) as i32
                }
                PriorReturn::SeparateReturns { .. } => {
                    libertas_macros::variant_index!(PriorReturn::SeparateReturns) as i32
                }
            };
            if !self.allowed_prior_returns().contains(&prior_index) {
                return Err(ValidationIssue::new(
                    "MESSAGE_YOUR_SPOUSE_S_SEPARATE_2025_TAX_FORM_2210_BOX_E_APPLIES_ONLY_WHEN_YOU_FILE_J",
                ).at_field("prior_return").in_field("penalty").in_field("more").in_field("value").into());
            }
            let farmer_index = match farmer {
                Farmer::No => libertas_macros::variant_index!(Farmer::No) as i32,
                Farmer::Yes { .. } => libertas_macros::variant_index!(Farmer::Yes) as i32,
                Farmer::ThisReturn { .. } => {
                    libertas_macros::variant_index!(Farmer::ThisReturn) as i32
                }
            };
            if !self.allowed_farmer().contains(&farmer_index) {
                return Err(ValidationIssue::new(
                    "MESSAGE_CHOOSE_YES_FOR_2026_ONLY_WHEN_THIS_RETURN_SHOWS_AT_LEAST_TWO_THIRDS_OF_YOUR",
                ).at_field("farmer").in_field("penalty").in_field("more").in_field("value").into());
            }
            if let Some(field) = waiver_record(waiver).and_then(|w| w.invalid_field()) {
                return Err(ValidationIssue::new(
                    "MESSAGE_A_PENALTY_WAIVER_REQUEST_NEEDS_AN_EXPLANATION_OF_UP_TO_2_000_CHARACTERS",
                ).at_field(field).in_field("waiver").in_field("penalty").in_field("more").in_field("value").into());
            }
        }
        let mut draft = self.draft.clone();
        let kept = draft.payments.clone().unwrap_or_else(FederalPayments::none);
        draft.payments =
            Some(FederalPayments { joint: self.joint(), ..payments_record(value, &kept) });
        finish_section(&mut draft, TaxSection::Payments);
        Ok(draft)
    }
    // The review's refund choices as they open: the stored plan and the
    // overpayment available for it. A retained invalid plan stays editable
    // against the ordinary bound the final-result cleanup withholds.
    pub(super) fn refund_choice(&self) -> RefundChoice {
        let d = &self.draft;
        let pay = d.payments.clone().unwrap_or_else(FederalPayments::none);
        let mut ordinary = d.clone();
        ordinary.payments = Some(FederalPayments { credit_applied_next_year: 0, ..pay.clone() });
        RefundChoice {
            credit_applied_next_year: pay.credit_applied_next_year,
            credit_recipient: pay.credit_recipient,
            credit_capacity: estimate(&ordinary).credit_capacity,
        }
    }
    // The refund choices merged into the accepted return (Form 1040 line
    // 36): checked against the ordinary settlement before the plan applies,
    // so an unadmitted plan never invalidates its own bound.
    pub(super) fn refund_candidate(
        &self,
        value: &RefundChoice,
    ) -> Result<FederalDraft, SaveFailure> {
        let mut draft = self.draft.clone();
        let joint = self.joint();
        let pay = draft
            .payments
            .get_or_insert_with(|| FederalPayments { joint, ..FederalPayments::none() });
        pay.credit_applied_next_year = 0;
        pay.credit_recipient = value.credit_recipient;
        if value.credit_applied_next_year < 0 {
            return Err(ValidationIssue::new("PAYMENT_APPLIED_CREDIT_NEGATIVE")
                .at_field("credit_applied_next_year")
                .in_field("refund")
                .into());
        }
        if value.credit_applied_next_year > 0 {
            if let Some(maximum) = crate::federal_rules::estimate(&draft).credit_capacity
                && value.credit_applied_next_year > maximum
            {
                let issue = ValidationIssue::new("PAY_CREDIT_ABOVE_CAPACITY")
                    .at_field("credit_applied_next_year")
                    .in_field("refund");
                return Err(SaveFailure {
                    message: Message::new(issue.key, alloc::vec![MessageArg::usd_cents(maximum)])
                        .encode(),
                    field: issue.field,
                });
            }
            if value.credit_recipient.is_none() {
                return Err(ValidationIssue::new("PAY_CREDIT_RECIPIENT_NEEDED")
                    .at_field("credit_recipient")
                    .in_field("refund")
                    .into());
            }
            // Unknown capacity is not a contradiction. Retain this planned
            // intent; the ordinary estimate withholds an unverified settlement.
        }
        draft.payments.as_mut().expect("candidate payments").credit_applied_next_year =
            value.credit_applied_next_year;
        Ok(draft)
    }
    // The income by period page (M3-P14) onto FederalPayments.annualized_income:
    // the three entered columns and column (d) derived from the return; the
    // echoed column (d) is ignored.
    pub(super) fn income_by_period_candidate(
        &self,
        value: &IncomeByPeriod,
    ) -> Result<FederalDraft, Vec<u8>> {
        if !box_c(&self.draft) {
            return Err(crate::interview::status("PROBLEM_SECTION_UNAVAILABLE_YET"));
        }
        let periods = AnnualizedIncome {
            through_march: value.through_march.clone(),
            through_may: value.through_may.clone(),
            through_august: value.through_august.clone(),
            full_year: full_year_period(&self.draft),
        };
        if !periods.valid() {
            return Err(crate::interview::status(
                "MESSAGE_ENTER_EACH_PERIOD_S_AMOUNTS_WITHIN_THE_SUPPORTED_RANGE",
            ));
        }
        let mut draft = self.draft.clone();
        if let Some(pay) = draft.payments.as_mut() {
            pay.annualized_income = Some(periods);
        }
        Ok(draft)
    }
}

impl Interview {
    pub(super) fn save_payments(
        &self,
        value: &Payments,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != PAYMENTS_PAGE || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.payments_candidate(value)
    }

    pub(super) fn save_income_by_period(
        &self,
        value: &IncomeByPeriod,
        page: i32,
    ) -> Result<FederalDraft, SaveFailure> {
        if page != INCOME_BY_PERIOD_PAGE || !box_c(&self.draft) || !self.reached(page) {
            return Err(status("PROBLEM_SECTION_UNAVAILABLE_YET").into());
        }
        self.income_by_period_candidate(value).map_err(Into::into)
    }
}
