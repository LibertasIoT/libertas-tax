// schedule_e items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

impl FederalDraft {
    // Qualification remains a personal fact. For section 469 a joint return
    // is one taxpayer once either spouse separately qualifies (Reg. 1.469-9(c)(4)).
    // This calculation context never replaces either person's source record.
    pub(crate) fn professional(&self, owner: Owner) -> Option<crate::ProfessionalRentals> {
        let people = self.people.as_ref()?;
        if !self.professional_joint() {
            return people.professional(owner).cloned();
        }
        if !people.any_professional() {
            return None;
        }
        let elected = [Owner::Taxpayer, Owner::Spouse]
            .into_iter()
            .any(|o| people.professional(o).is_some_and(|p| p.single_activity));
        Some(crate::ProfessionalRentals {
            single_activity: elected,
            // The documented safe harbor covers the combined activity only
            // with the election; otherwise retain the owner's per-rental answer.
            over_500_hours: if elected {
                [Owner::Taxpayer, Owner::Spouse]
                    .into_iter()
                    .any(|o| people.professional(o).is_some_and(|p| p.over_500_hours))
            } else {
                people.professional(owner).is_some_and(|p| p.over_500_hours)
            },
        })
    }
    pub(crate) fn professional_joint(&self) -> bool {
        self.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Joint)
    }
    pub(crate) fn professional_group_owner(&self, owner: Owner) -> Owner {
        let owner = crate::community::professional_owner(self.setup.as_ref(), owner);
        if self.professional_joint() && self.professional(owner).is_some_and(|p| p.single_activity)
        {
            Owner::Taxpayer
        } else {
            owner
        }
    }
    // Keep the existing elected person's editor authoritative. A newly
    // qualified spouse's default false is never an election revocation.
    pub(crate) fn professional_election_editor(&self, opening: Owner) -> Owner {
        if !self.professional_joint() {
            return opening;
        }
        let Some(people) = &self.people else {
            return opening;
        };
        [Owner::Taxpayer, Owner::Spouse]
            .into_iter()
            .find(|o| people.professional(*o).is_some_and(|p| p.single_activity))
            .or_else(|| {
                [Owner::Taxpayer, Owner::Spouse]
                    .into_iter()
                    .find(|o| people.professional(*o).is_some())
            })
            .unwrap_or(opening)
    }
    pub(crate) fn professional_elections(&self, opening: Owner) -> alloc::vec::Vec<i32> {
        if self.professional_election_editor(opening) == opening {
            alloc::vec![0, 1]
        } else {
            alloc::vec![i32::from(self.professional(opening).is_some_and(|p| p.single_activity))]
        }
    }
    pub(crate) fn reconcile_professional_election(&mut self, before: &Self, edited: Owner) {
        if !before.professional_joint() {
            return;
        }
        let prior = before.professional(edited).is_some_and(|p| p.single_activity);
        let authoritative = before.professional_election_editor(edited) == edited;
        let elected = if authoritative {
            self.people
                .as_ref()
                .and_then(|p| p.professional(edited))
                .map_or(prior, |p| p.single_activity)
        } else {
            prior
        };
        // Removing qualification transfers an existing joint election to the
        // remaining qualifier. It does not erase that spouse's qualification
        // or either person's own safe-harbor/history facts.
        let previous_editor = before.professional_election_editor(edited);
        let editor =
            if self.people.as_ref().is_some_and(|p| p.professional(previous_editor).is_some()) {
                previous_editor
            } else {
                self.professional_election_editor(edited)
            };
        let Some(people) = self.people.as_mut() else {
            return;
        };
        if let Some(p) = people.taxpayer.real_estate_professional.as_mut() {
            p.single_activity = elected && editor == Owner::Taxpayer;
        }
        if let Some(p) = people.spouse.as_mut().and_then(|p| p.real_estate_professional.as_mut()) {
            p.single_activity = elected && editor == Owner::Spouse;
        }
    }
    pub(crate) fn any_professional(&self) -> bool {
        self.people.as_ref().is_some_and(|p| p.any_professional())
    }
}

pub(super) struct RoyaltySourcesOut {
    pub(super) royalty_regular: Vec<crate::royalties::Depletion>,
    pub(super) royalty_amt: Vec<crate::royalties::Depletion>,
    pub(super) royalty_total: Exact,
    pub(super) business_depletion_adjustment: i64,
}

pub(super) fn royalty_sources_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    partner_pass: &PartnerPass,
) -> core::ops::ControlFlow<(), RoyaltySourcesOut> {
    let &Pass { d, allocated_entries, .. } = pass;
    let royalty_regular = partner_pass.depletion_regular.clone();
    let royalty_amt = partner_pass.depletion_amt.clone();
    let royalty_total = royalty_regular
        .iter()
        .filter(|v| {
            allocated_entries
                .iter()
                .any(|e| e.id == v.id && matches!(e.income, FederalIncome::Royalty { .. }))
        })
        .map(|v| exact::whole(v.allowance))
        .sum::<Exact>();
    t.royalties -= &royalty_total;
    for entry in allocated_entries {
        if matches!(entry.income, FederalIncome::Royalty { .. }) {
            // Retain the same final depletion allowance that reduced the
            // native joint pool. A missing result is not a zero allowance.
            let amount =
                royalty_regular.iter().find(|row| row.id == entry.id).map(|row| -row.allowance);
            t.injured_other_amount(entry.id, amount);
            t.record_income_component(
                entry.id,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                crate::injured_spouse::income_categories::IncomeCategory::Royalties,
                amount,
                amount,
            );
        }
    }
    let mut business_depletion_adjustment = 0;
    for row in &royalty_regular {
        if allocated_entries
            .iter()
            .any(|e| e.id == row.id && matches!(e.income, FederalIncome::Business { .. }))
        {
            let delta = royalty_amt
                .iter()
                .find(|v| v.id == row.id)
                .map_or(0, |v| row.allowance - v.allowance);
            business_depletion_adjustment += delta;
            t.business_depreciation_adjustment += delta;
            *t.business_amt_adjustments.entry(row.id).or_default() += delta;
        }
    }
    if !crate::royalties::report(d, &royalty_regular, false, r)
        || !crate::royalties::report(d, &royalty_amt, true, r)
    {
        return core::ops::ControlFlow::Break(());
    }
    core::ops::ControlFlow::Continue(RoyaltySourcesOut {
        royalty_regular,
        royalty_amt,
        royalty_total,
        business_depletion_adjustment,
    })
}

pub(super) struct ScheduleEOutputIn<'a> {
    pub(super) royalty_regular: &'a [crate::royalties::Depletion],
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) royalties: i64,
    pub(super) estate_royalties: i64,
    pub(super) k1_royalty_losses: i64,
    pub(super) estate_royalty_columns: [i64; 2],
}

pub(super) fn schedule_e_output_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: ScheduleEOutputIn<'_>,
) -> Option<()> {
    let Pass { d, allocated_entries, .. } = *pass;
    let ScheduleEOutputIn {
        royalty_regular,
        rental,
        royalties,
        estate_royalties,
        k1_royalty_losses,
        estate_royalty_columns,
    } = input;
    let has_rental =
        allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Rental { .. }));
    // Line 26 totals Part I (lines 24 and 25); a K-1 (Form 1041) box 5
    // royalty is on line 33 column (f) instead.
    let part_one_royalties = royalties - dollars(estate_royalties);
    let has_royalties = royalty_regular.iter().any(|row| {
        allocated_entries.iter().any(|entry| {
            entry.id == row.id && matches!(entry.income, FederalIncome::Royalty { .. })
        })
    }) || r.lines.iter().any(|row| row.id.ends_with("_line4_royalties"));
    if has_rental || has_royalties {
        let sum = |line: &str, only_positive: bool| -> i64 {
            r.lines
                .iter()
                .filter(|row| {
                    row.form_line
                        .as_ref()
                        .is_some_and(|f| f.form == TaxForm::ScheduleE && f.line == line)
                })
                .map(|row| if only_positive { positive(row.amount) } else { row.amount })
                .sum()
        };
        let totals = [
            ("23a", "schedule_e_line23a_rents", sum("3", false)),
            ("23b", "schedule_e_line23b_royalties", sum("4", false)),
            ("23c", "schedule_e_line23c_mortgage_interest", sum("12", false)),
            ("23d", "schedule_e_line23d_depreciation_depletion", sum("18", false)),
            ("23e", "schedule_e_line23e_expenses", sum("20", false)),
            ("24", "schedule_e_line24_income", sum("21", true)),
            ("25", "schedule_e_line25_rental_losses", sum("22", false)),
        ];
        emit_lines(r, &mut (), totals, |(printed, id, amount), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::ScheduleE, printed)),
                id,
                LineValue::MoneyCents(amount),
                &["income.rental", "income.royalty"],
                "https://www.irs.gov/instructions/i1040se",
                msg!("EXPLAIN_SCHEDULE_E_TOTALS"),
            ))
        });
        if has_royalties {
            let Some(royalty_losses) = exact::rounded(
                &(crate::royalties::losses(d, royalty_regular) + exact::whole(k1_royalty_losses)),
            ) else {
                crate::extended_rules::expense_output_unavailable(r, None);
                return None;
            };
            push_line(
                r,
                Some(FormLine::new(TaxForm::ScheduleE, "25")),
                "schedule_e_line25_royalty_losses",
                royalty_losses,
                &["income.royalty", "income.partnership", "income.scorp"],
                "https://www.irs.gov/instructions/i1040se",
                msg!("EXPLAIN_SCHEDULE_E_TOTALS"),
            );
        }
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleE, "26"),
            "schedule_e_rental_royalty_total",
            rental.part_one + part_one_royalties,
            &["income.rental", "rental_income"],
            "https://www.irs.gov/instructions/i1040se",
            msg!("EXPLAIN_SCHEDULE_E_RENTAL_ROYALTY_TOTAL"),
        );
    }
    if allocated_entries
        .iter()
        .any(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
    {
        // Keep the actual allowed Part III columns, rather than deriving
        // deductions from the return's aggregate net income.
        let [passive_loss, passive_income, mut other_loss, mut other_income] =
            rental.estate_columns.map(|amount| amount * 100);
        other_income += estate_royalty_columns[0];
        other_loss += estate_royalty_columns[1];
        let income = passive_income + other_income;
        let losses = passive_loss + other_loss;
        emit_lines(
            r,
            &mut (),
            [
                ("34a(d)", "schedule_e_line34a_passive_income", passive_income),
                ("34a(f)", "schedule_e_line34a_other_income", other_income),
                ("34b(c)", "schedule_e_line34b_passive_losses", passive_loss),
                ("34b(e)", "schedule_e_line34b_other_losses", other_loss),
                ("35", "schedule_e_line35_estate_income", income),
                ("36", "schedule_e_line36_estate_losses", -losses),
                ("37", "schedule_e_line37_estate_net", income - losses),
            ],
            |(printed, id, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::ScheduleE, printed)),
                    id,
                    LineValue::MoneyCents(amount),
                    &["income.estate_trust_portfolio"],
                    "https://www.irs.gov/pub/irs-dft/f1040se--dft.pdf",
                    msg!("EXPLAIN_SCHEDULE_E_ESTATE_TOTALS"),
                ))
            },
        );
    }
    // Schedule E Part V (2026 draft): line 40 is each farm rental's
    // Form 4835 line 32 or 34c; line 42 reconciles the farming and
    // fishing gross income that the farmer estimated-tax test reads, from
    // every source the form names: Form 4835 line 7, Schedule K-1 (Form
    // 1065) box 14 code B and Schedule K-1 (Form 1041) box 14 code F.
    let has_farm_rental =
        allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::FarmRental { .. }));
    if has_farm_rental {
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleE, "40"),
            "schedule_e_farm_rental_total",
            rental.farm_rental,
            &["income.farm_rental", "rental_income"],
            "https://www.irs.gov/instructions/i1040se",
            msg!("EXPLAIN_SCHEDULE_E_FARM_RENTAL_TOTAL"),
        );
    }
    let farming_sources: alloc::vec::Vec<i64> = allocated_entries
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::FarmRental { .. } => Some(
                r.lines
                    .iter()
                    .find(|l| l.id == alloc::format!("farm_rental_{}_gross_income", e.id))
                    .map_or(0, |l| positive(l.amount)),
            ),
            FederalIncome::EstateTrustPortfolio { data } => {
                let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                Some(data.gross_farming_fishing_income)
            }
            FederalIncome::Partnership { data } => match &data.se_gross {
                Some(crate::PartnerSeGross::Farm { gross }) => Some(*gross),
                _ => None,
            },
            FederalIncome::PartnershipActivities { data } => data
                .ordinary_self_employment()
                .ok()
                .flatten()
                .filter(|source| source.category == 0)
                .map(|source| source.gross),
            _ => None,
        })
        .collect();
    let farming_gross: i64 = farming_sources.iter().sum();
    if has_farm_rental || farming_gross != 0 {
        push_line(
            r,
            Some(FormLine::new(TaxForm::ScheduleE, "42")),
            "schedule_e_farming_fishing_gross",
            farming_gross,
            &["income.farm_rental", "income.partnership", "income.estate_trust_portfolio"],
            "https://www.irs.gov/instructions/i1040se",
            msg!("EXPLAIN_SCHEDULE_E_FARMING_FISHING_GROSS"),
        );
    }
    Some(())
}

pub(super) struct RoyaltyIncomeReportingIn<'a> {
    pub(super) royalty_routing: &'a Option<crate::investment_interest::InvestmentRouting>,
}

pub(super) struct RoyaltyIncomeReportingOut {
    pub(super) royalty_interest: [i64; 2],
    pub(super) royalties: i64,
    pub(super) personal_rent: i64,
    pub(super) personal_rent_expenses: i64,
    pub(super) property_net: i64,
}

pub(super) fn royalty_income_reporting_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    input: RoyaltyIncomeReportingIn<'_>,
) -> core::ops::ControlFlow<(), RoyaltyIncomeReportingOut> {
    let &Pass { d, .. } = pass;
    let RoyaltyIncomeReportingIn { royalty_routing } = input;
    let royalty_interest = royalty_routing.as_ref().map_or([0; 2], |v| v.royalties());
    let royalties =
        match exact::dollars(&t.royalties).filter(|amount| amount.checked_mul(100).is_some()) {
            Some(amount) => amount,
            None => {
                crate::extended_rules::expense_output_unavailable(r, None);
                return core::ops::ControlFlow::Break(());
            }
        } - royalty_interest[0];
    if let Some(routing) = royalty_routing {
        for (&source, amounts) in &routing.sources {
            if source == 0 {
                continue;
            }
            // The native ScheduleE charge is already whole dollars. Keep
            // its exact source destination and convert that charge once
            // into the category's cents; do not deduct the raw loan again.
            t.injured_other_amount(source, Some(-amounts[0][0] * 100));
            if let Some(entry) = d.income.iter().find(|entry| entry.id == source) {
                t.record_income_component(
                    source,
                    usize::from(entry.owner == crate::DocumentOwner::Spouses),
                    crate::injured_spouse::income_categories::IncomeCategory::Royalties,
                    Some(-amounts[0][0] * 100),
                    Some(-amounts[0][0] * 100),
                );
            }
            form_line(
                r,
                FormLine::new(TaxForm::ScheduleE, "13"),
                &alloc::format!("royalty_{source}_line13_interest"),
                amounts[0][0],
                &["deductions.investment_interest", "income.nonbusiness_property"],
                "https://www.irs.gov/pub/irs-dft/f4952--dft.pdf",
                msg!("EXPLAIN_INVESTMENT_SOURCE_ROUTING"),
            );
        }
    }
    let personal_rent = dollars(t.personal_rent);
    let personal_rent_expenses = dollars(t.personal_rent_expenses);
    let property_net = royalties + personal_rent - personal_rent_expenses;
    emit_lines(
        r,
        &mut (),
        [
            ("nonbusiness_personal_rent", personal_rent, msg!("EXPLAIN_NONBUSINESS_PERSONAL_RENT")),
            (
                "nonbusiness_personal_rent_depreciation",
                dollars(t.personal_rent_depreciation),
                msg!("EXPLAIN_NONBUSINESS_PERSONAL_RENT_DEPRECIATION"),
            ),
            (
                "nonbusiness_personal_rent_expenses_paid",
                dollars(t.personal_rent_expenses - t.personal_rent_depreciation),
                msg!("EXPLAIN_NONBUSINESS_PERSONAL_RENT_EXPENSES_PAID"),
            ),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income.nonbusiness_property"],
                "https://www.irs.gov/taxtopics/tc414",
                explanation,
            ))
        },
    );
    line(
        r,
        "nonbusiness_royalty_income",
        royalties,
        &["income.nonbusiness_property"],
        "https://www.irs.gov/instructions/i1040se",
        msg!("EXPLAIN_NONBUSINESS_ROYALTY_INCOME"),
    );
    core::ops::ControlFlow::Continue(RoyaltyIncomeReportingOut {
        royalty_interest,
        royalties,
        personal_rent,
        personal_rent_expenses,
        property_net,
    })
}
