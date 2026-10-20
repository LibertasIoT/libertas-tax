pub(in crate::tests) mod business_credit_owner_sources {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    use crate::tests::support::assert::injured_credit::business_credit_owner_sources::{
        foreign_statement_money, foreign_statement_source_links, money, payee_limit_money,
        payee_limit_sources, residual_money, residual_sources, source,
    };
    use crate::tests::support::builders::injured_credit::business_credit_owner_sources::{
        PayeeLimitCase, ResidualFamily, fixture, foreign_statement_fixture, payee_limit_fixture,
        residual_fixture,
    };
    use crate::tests::*;
    pub(in crate::tests) fn saved(
        owner: Owner,
        injured: Owner,
        reverse: bool,
        stale_answer: Option<i64>,
    ) -> FederalResult {
        let mut draft = fixture(owner, injured, reverse);
        if let Some(injured) = stale_answer {
            draft.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                component: "INJURED_COMPONENT_NONREFUNDABLE".into(),
                injured,
            });
        }
        let mut session = restart(draft);
        let ids: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
        let original_source = source(&session.draft, owner);
        for _ in 0..2 {
            save_native_credit_sources(
                &mut session,
                &[TaxSection::Wages, TaxSection::SelfEmployment],
            );
            money(&session.draft);
            assert_eq!(source(&session.draft, owner), original_source);
            assert_eq!(session.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>(), ids);
            let draft = roundtrip_native_draft(session.draft.clone());
            session = restart(draft);
            money(&session.draft);
            assert_eq!(source(&session.draft, owner), original_source);
        }
        money(&session.draft)
    }

    pub(in crate::tests) fn observations() -> Vec<(Owner, Owner, bool, FederalResult)> {
        let mut observations = vec![];
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    observations.push((
                        owner,
                        injured,
                        reverse,
                        saved(owner, injured, reverse, None),
                    ));
                }
            }
        }
        observations
    }

    pub(in crate::tests) fn residual_saved_with_payer(
        family: ResidualFamily,
        owner: Owner,
        payer: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let draft = if matches!(family, ResidualFamily::ForeignTax) {
            let mut draft = foreign_statement_fixture(owner, payer, injured, reverse);
            foreign_statement_money(&draft);
            foreign_statement_source_links(&draft, owner, payer);
            draft.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
                component: family.old_key().into(),
                injured: family.answer(),
            });
            draft
        } else {
            assert_eq!(owner, payer);
            residual_fixture(family, owner, injured, reverse)
        };
        let mut s = restart(draft);
        let ids: Vec<_> = s.draft.income.iter().map(|e| e.id).collect();
        let source_credits = s.draft.credits.clone();
        for _ in 0..2 {
            super::super::income::save_native_income_topics(
                &mut s,
                &[TaxSection::Wages, TaxSection::SelfEmployment],
            );
            if matches!(family, ResidualFamily::ForeignTax) {
                let page = open_topic(&mut s, TaxSection::InterestDividends);
                accepted(&send(&mut s, page));
            } else {
                let Reply::Edit(page @ P::SaveMarketplace { .. }) =
                    from_review(&mut s, TaxSection::Marketplace)
                else {
                    panic!("actual Marketplace editor")
                };
                accepted(&send(&mut s, page));
            }
            save_native_credit_sources(&mut s, &[]);
            residual_money(&s.draft, family);
            residual_sources(&s.draft, family, owner, payer);
            assert_eq!(s.draft.income.iter().map(|e| e.id).collect::<Vec<_>>(), ids);
            assert_eq!(s.draft.credits, source_credits);
            let draft = roundtrip_native_draft(s.draft.clone());
            s = restart(draft);
            residual_money(&s.draft, family);
            residual_sources(&s.draft, family, owner, payer);
        }
        s.draft
    }

    pub(in crate::tests) fn residual_compatibility() {
        let family = ResidualFamily::PremiumTax;
        let mut profiles = vec![];
        // Complete every native financial/public source Save/Avro/restart profile
        // before judging compatibility or ownership of the independent residual.
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                for reverse in [false, true] {
                    profiles.push((
                        owner,
                        injured,
                        reverse,
                        residual_saved(family, owner, injured, reverse),
                    ));
                }
            }
        }
        for (owner, injured, _reverse, d) in profiles {
            let r = residual_money(&d, family);
            let plan = r.injured_spouse.as_ref().unwrap();
            assert!(
                plan.accepted
                    .iter()
                    .all(|row| row.component != family.key() && row.component != family.old_key())
            );
            let row = plan.required.iter().find(|row| row.component == family.key()).unwrap();
            assert_eq!(
                (row.minimum, row.maximum, row.joint),
                (0, family.remainder(), family.remainder())
            );
            assert!((row.minimum..=row.maximum).contains(&family.answer()));
            let construction = d.injured_spouse.as_ref().unwrap().construction(&d, &r);
            assert!(
                construction
                    .allocations
                    .iter()
                    .all(|row| row.component != family.key() && row.component != family.old_key())
            );
            assert!(construction.new_allocations.iter().any(|row| row.component == family.key()));

            let mut s = restart(d);
            let Reply::Edit(mut obsolete) = native_editor(
                from_review(&mut s, TaxSection::InjuredSpouse),
                &[(family.key(), family.answer())],
            ) else {
                panic!()
            };
            let P::SaveInjuredSpouse { value, .. } = &mut obsolete else { panic!() };
            let [answer] = value.allocations.as_mut_slice() else { panic!() };
            answer.component = family.old_key().into();
            assert_eq!(
                crate::injured_spouse::allocation_rule(value, &s.draft, &r),
                Err("INJURED_SPOUSE_ALLOCATION_OUTSIDE_COMPONENT"),
            );
            let before = s.draft.clone();
            let dirty = s.dirty;
            match send(&mut s, obsolete) {
                Reply::Response(P::BeginInjuredSpouse { value, .. }) => {
                    assert!(value.allocations.is_empty());
                    let [row] = value.new_allocations.as_slice() else { panic!() };
                    assert_eq!(row.component, family.key());
                    assert_eq!((row.minimum, row.maximum), (0, family.remainder()));
                }
                Reply::Response(P::Problem { .. }) => {}
                other => panic!("obsolete aggregate answer must not save: {other:?}"),
            }
            assert_eq!(s.draft, before);
            assert_eq!(s.draft.revision, before.revision);
            assert_eq!(s.dirty, dirty);
            residual_money(&s.draft, family);
            let Reply::Edit(request) = native_editor(
                from_review(&mut s, TaxSection::InjuredSpouse),
                &[(family.key(), family.answer())],
            ) else {
                panic!()
            };
            // Explicit native completed request, not evidence of untouched client
            // initialization. The existing actual bridge owns that separate proof.
            accepted(&send(&mut s, request));
            let accepted_result = residual_money(&s.draft, family);
            let line = "17";
            let known = 398700;
            let expected = if owner == injured {
                [known + family.answer(), family.remainder() - family.answer()]
            } else {
                [family.answer(), known + family.remainder() - family.answer()]
            };
            assert_eq!(
                ["b", "c"].map(|column| accepted_result
                    .lines
                    .iter()
                    .find(|row| row.id == alloc::format!("form_8379_line_{line}_{column}"))
                    .unwrap()
                    .amount),
                expected
            );
            assert_eq!(s.draft.injured_spouse.as_ref().unwrap().allocations.len(), 1);
            assert_eq!(
                s.draft.injured_spouse.as_ref().unwrap().allocations[0].component,
                family.key()
            );
            let stored = s.draft.clone();
            let bytes = native_draft_records(&stored);
            let draft = draft_from_native_records(stored.tax_year, &bytes);
            assert_eq!(draft, stored);
            assert_eq!(native_draft_records(&draft), bytes);
            let mut s = restart(draft);
            residual_money(&s.draft, family);
            let Reply::Edit(request) =
                native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
            else {
                panic!()
            };
            accepted(&send(&mut s, request));
            residual_money(&s.draft, family);
        }
    }

    pub(in crate::tests) fn payee_limit_saved(
        case: PayeeLimitCase,
        business_owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut s = restart(payee_limit_fixture(case, business_owner, injured, reverse));
        let ids: Vec<_> = s.draft.income.iter().map(|entry| entry.id).collect();
        let sources = payee_limit_sources(&s.draft, case, business_owner);
        let credits = s.draft.credits.clone();
        for _ in 0..2 {
            save_native_credit_sources(
                &mut s,
                &[TaxSection::Wages, TaxSection::SelfEmployment, TaxSection::InterestDividends],
            );
            payee_limit_money(&s.draft, case);
            assert_eq!(payee_limit_sources(&s.draft, case, business_owner), sources);
            assert_eq!(s.draft.income.iter().map(|entry| entry.id).collect::<Vec<_>>(), ids);
            assert_eq!(s.draft.credits, credits);
            let draft = roundtrip_native_draft(s.draft.clone());
            s = restart(draft);
            payee_limit_money(&s.draft, case);
            assert_eq!(payee_limit_sources(&s.draft, case, business_owner), sources);
        }
        s.draft
    }

    pub(in crate::tests) fn residual_saved(
        family: ResidualFamily,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        residual_saved_with_payer(family, owner, owner, injured, reverse)
    }
}

pub(in crate::tests) mod elective_credit_mixed_sources {
    use crate::tests::cross::interview::{accepted, from_review};
    use crate::tests::support::assert::injured_credit::elective_credit_mixed_sources::{
        money, source_links,
    };
    use crate::tests::support::builders::injured_credit::elective_credit_mixed_sources::{
        Family, USED, fixture,
    };
    use crate::tests::*;
    use alloc::boxed::Box;
    pub(in crate::tests) fn persist(family: Family, injured: Owner, order: u8) -> FederalDraft {
        let mut s = restart(fixture(family, injured, order));
        let ids: Vec<_> = s.draft.income.iter().map(|e| e.id).collect();
        let links = source_links(&s.draft, family);
        for _ in 0..2 {
            save_native_credit_sources(&mut s, &[TaxSection::Wages, TaxSection::SelfEmployment]);
            money(&s.draft, family);
            assert_eq!(source_links(&s.draft, family), links);
            assert_eq!(s.draft.income.iter().map(|e| e.id).collect::<Vec<_>>(), ids);
            let draft = roundtrip_native_draft(s.draft.clone());
            s = restart(draft);
            money(&s.draft, family);
        }
        s.draft
    }

    pub(in crate::tests) fn ownership(family: Family) {
        let mut observations = vec![];
        // Drain every financial/public-source-Save profile before ownership checks.
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for order in 0..4 {
                let d = persist(family, injured, order);
                let r = money(&d, family);

                observations.push((injured, order, d, r));
            }
        }
        for (injured, order, d, r) in observations {
            let (used, elective) = match family {
                Family::NoUse => ([0, 0], [3000, 2000]),
                Family::BindingUse => ([4000, 7192], [1000, 37808]),
            };
            let i = usize::from(injured == Owner::Spouse);
            assert_eq!(
                [column(&r, "17", "b"), column(&r, "17", "c")],
                [elective[i], elective[1 - i]],
                "{family:?} {injured:?} order{order}"
            );
            assert_eq!([column(&r, "16", "b"), column(&r, "16", "c")], [used[i], used[1 - i]]);
            let plan = r.injured_spouse.as_ref().unwrap();
            match family {
                Family::NoUse => assert!(plan.required.is_empty()),
                Family::BindingUse => {
                    assert_eq!(plan.required.len(), 1, "no independent17 allocation");
                    let row = &plan.required[0];
                    assert_eq!(row.component, USED);
                    assert_eq!(row.joint, 1119200);
                    let bounds = if i == 0 { (0, 500000) } else { (619200, 1119200) };
                    assert_eq!((row.minimum, row.maximum), bounds);
                    assert_eq!(row.injured, used[i] * 100);
                    assert_eq!(plan.accepted, d.injured_spouse.as_ref().unwrap().allocations);
                }
            }
            let mut s = restart(d);
            let Reply::Edit(page) =
                native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
            else {
                panic!("normal completed native allocation editor")
            };
            accepted(&send(&mut s, page));
            let after = money(&s.draft, family);
            assert_eq!(after.return_lines, r.return_lines);
            assert_eq!(
                [column(&after, "17", "b"), column(&after, "17", "c")],
                [elective[i], elective[1 - i]]
            );
            let draft = roundtrip_native_draft(s.draft.clone());
            let restarted = restart(draft);
            assert_eq!(money(&restarted.draft, family), after);
        }
    }

    pub(in crate::tests) fn reproject(d: &FederalDraft, actual: &FederalResult) -> FederalResult {
        let mut w2_context = d.clone();
        let mut businesses = 0;
        w2_context.income.retain(|entry| {
            if matches!(entry.income, FederalIncome::Business { .. }) {
                assert_eq!(entry.income, cash_business(0));
                businesses += 1;
                false
            } else {
                assert!(matches!(entry.income, FederalIncome::Wage { .. }));
                true
            }
        });
        assert_eq!(businesses, 2);
        for key in ["schedule_c_net_profit", "self_employment_tax", "qbi_deduction"] {
            assert_eq!(
                actual.lines.iter().find(|row| row.id == key).map_or(0, |row| row.amount / 100),
                0
            );
        }
        let mut facts = liability_portfolio_native_facts(&w2_context, Some([0; 2]), Some(actual));
        // The exhaustive source check above and the original accepted credit
        // history admit only45X, no household or business fuel credit contributor.
        facts.income_categories.fuel_credit_sources =
            Some(Box::new(crate::injured_spouse::income_categories::NativeFuelCreditSources {
                joint_cents: 0,
                business_cents: [0; 2],
                home_cents: 0,
            }));
        assert_eq!(
            actual
                .lines
                .iter()
                .find(|row| row.id == "schedule_3_fuel_tax_credit")
                .map_or(0, |row| row.amount / 100),
            0
        );
        let mut projected = actual.clone();
        projected.lines.retain(|row| !row.id.starts_with("form_8379_"));
        projected
            .return_lines
            .retain(|row| row.form_line.as_ref().is_none_or(|line| line.form != TaxForm::Form8379));
        projected.injured_spouse = None;
        crate::injured_spouse::allocate(
            d,
            &mut projected,
            d.injured_spouse.as_ref().unwrap(),
            &facts,
            actual.refund.expect("ordinary pre-election overpayment"),
        );
        projected
    }

    pub(in crate::tests) fn actual_columns(
        r: &FederalResult,
        family: Family,
        injured: Owner,
        corrected: bool,
    ) {
        let (mut used, mut net) = match family {
            Family::NoUse => ([0, 0], [3000, 2000]),
            Family::BindingUse => ([4000, 7192], [1000, 37808]),
        };
        if corrected {
            used.reverse();
            net.reverse();
        }
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!([column(r, "16", "b"), column(r, "16", "c")], [used[i], used[1 - i]]);
        assert_eq!([column(r, "17", "b"), column(r, "17", "c")], [net[i], net[1 - i]]);
        let plan = r.injured_spouse.as_ref().unwrap();
        match family {
            Family::NoUse => assert!(plan.required.is_empty()),
            Family::BindingUse => {
                assert_eq!(plan.required.len(), 1);
                let row = &plan.required[0];
                assert_eq!(row.component, USED);
                assert_eq!(row.joint, 1119200);
                let smaller = (!corrected && i == 0) || (corrected && i == 1);
                assert_eq!(
                    (row.minimum, row.maximum),
                    if smaller { (0, 500000) } else { (619200, 1119200) }
                );
                assert_eq!(row.injured, used[i] * 100);
            }
        }
    }
}

pub(in crate::tests) mod foreign_credit_ordinary_sources {
    use crate::tests::support::assert::injured_credit::foreign_credit_ordinary_sources::{
        columns, money,
    };
    use crate::tests::support::builders::injured_credit::foreign_credit_ordinary_sources::{
        Family, fixture,
    };
    use crate::tests::*;
    pub(in crate::tests) fn profiles(
        families: &[Family],
    ) -> Vec<(Family, Owner, Owner, FederalDraft)> {
        let mut profiles = vec![];
        // Finish all literal financial/source Save/restart axes before allocation.
        for &family in families {
            for owner in [Owner::Taxpayer, Owner::Spouse] {
                for injured in [Owner::Taxpayer, Owner::Spouse] {
                    for reverse in [false, true] {
                        profiles.push((
                            family,
                            owner,
                            injured,
                            saved(family, owner, injured, reverse),
                        ));
                    }
                }
            }
        }
        profiles
    }

    pub(in crate::tests) fn known(families: &[Family]) {
        for (family, owner, injured, d) in profiles(families) {
            let r = money(&d, family);
            let physical = match family {
                Family::Current | Family::CurrentAccrued => {
                    if owner == Owner::Taxpayer {
                        [100000, 0]
                    } else {
                        [0, 100000]
                    }
                }
                Family::PriorWage | Family::PriorPassive => {
                    if owner == Owner::Taxpayer {
                        [20000, 0]
                    } else {
                        [0, 20000]
                    }
                }
                Family::MixedCategories => {
                    if owner == Owner::Taxpayer {
                        [10000, 13200]
                    } else {
                        [13200, 10000]
                    }
                }
                Family::K3 { .. } => {
                    if owner == Owner::Taxpayer {
                        [66600, 0]
                    } else {
                        [0, 66600]
                    }
                }
                Family::EmploymentExcluded => {
                    if owner == Owner::Taxpayer {
                        [767800, 0]
                    } else {
                        [0, 767800]
                    }
                }
                Family::BusinessExcluded => {
                    if owner == Owner::Taxpayer {
                        [0, 773500]
                    } else {
                        [773500, 0]
                    }
                }
                Family::HighTax => {
                    if owner == Owner::Taxpayer {
                        [1317000, 0]
                    } else {
                        [0, 1317000]
                    }
                }
                Family::Historical | Family::K3Pair => panic!(),
            };
            let expected =
                if injured == Owner::Taxpayer { physical } else { [physical[1], physical[0]] };

            assert_eq!(
                columns(&r),
                expected.map(Some),
                "actual current payee/category/native allowance owner"
            );
            let plan = r.injured_spouse.as_ref().unwrap();
            assert!(
                plan.required.is_empty(),
                "known physical native credit owner needs no arbitrary split"
            );
        }
    }

    pub(in crate::tests) fn saved(
        family: Family,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut s = restart(fixture(family, owner, injured, reverse));
        let sources = s.draft.income.clone();
        let credits = s.draft.credits.clone();
        for _ in 0..2 {
            let topics: &[TaxSection] = match family {
                Family::K3 { .. } | Family::K3Pair => &[TaxSection::Wages, TaxSection::PassThrough],
                Family::BusinessExcluded => {
                    &[TaxSection::Wages, TaxSection::WorkAbroad, TaxSection::SelfEmployment]
                }
                Family::MixedCategories | Family::HighTax => {
                    &[TaxSection::Wages, TaxSection::WorkAbroad, TaxSection::InterestDividends]
                }
                _ => &[TaxSection::Wages, TaxSection::WorkAbroad],
            };
            save_native_credit_sources(&mut s, topics);
            money(&s.draft, family);
            assert_eq!(s.draft.income, sources, "actualIDs/owners/K3 links/sources unchanged");
            assert_eq!(s.draft.credits, credits, "actual paid tax/history provenance unchanged");
            let draft = roundtrip_native_draft(s.draft.clone());
            s = restart(draft);
            money(&s.draft, family);
            assert_eq!(s.draft.income, sources);
            assert_eq!(s.draft.credits, credits);
        }
        s.draft
    }
}

pub(in crate::tests) mod fuel_credit_owner_sources {
    use crate::tests::cross::interview::{accepted, from_review};
    use crate::tests::support::assert::injured_credit::fuel_credit_owner_sources::ordinary;
    use crate::tests::*;
    pub(in crate::tests) fn assert_owner_symmetry(
        business: [i64; 2],
        raw_credit: i64,
        annual_credit: i64,
        expected: [i64; 2],
        message: &str,
    ) {
        use crate::tests::support::builders::injured_credit::fuel_credit_owner_sources::fixture;
        let mut observations = Vec::new();
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let result =
                    saved(fixture(business, injured, reverse), business, raw_credit, annual_credit);
                let actual = [column(&result, "17", "b"), column(&result, "17", "c")];
                observations.push((actual, expected));
            }
        }
        assert!(
            observations.iter().all(|(actual, expected)| actual == expected),
            "{message}: {observations:?}"
        );
    }

    pub(in crate::tests) fn saved(
        draft: FederalDraft,
        business: [i64; 2],
        raw_credit: i64,
        annual_credit: i64,
    ) -> FederalResult {
        let mut session = restart(draft);
        for _ in 0..2 {
            save_native_credit_sources(&mut session, &[TaxSection::Wages, TaxSection::PassThrough]);
            let result = supported(&session.draft);
            ordinary(&result, raw_credit, annual_credit);
            for (owner, amount) in [Owner::Taxpayer, Owner::Spouse].into_iter().zip(business) {
                let entry = session
                    .draft
                    .income
                    .iter()
                    .find(|row| {
                        row.owner == owner
                            && matches!(row.income, FederalIncome::Partnership { .. })
                    })
                    .unwrap();
                let FederalIncome::Partnership { data } = &entry.income else { unreachable!() };
                assert_eq!(data.regular.ordinary, 0);
                assert_eq!(
                    data.fuel_tax_credit.as_ref().map_or(0, PartnerFuelTaxCredit::credit_cents),
                    amount
                );
            }
            session = restart(session.draft);
            assert_eq!(estimate(&session.draft), result);
        }
        // Exercise the actual normal Workflow and a lossless complete request;
        // every existing home choice is already explicit, never a copied bound.
        for _ in 0..2 {
            let Reply::Edit(page) =
                native_editor(from_review(&mut session, TaxSection::InjuredSpouse), &[])
            else {
                panic!()
            };
            accepted(&send(&mut session, page));
            session = restart(session.draft);
            ordinary(&supported(&session.draft), raw_credit, annual_credit);
        }
        let result = supported(&session.draft);
        let mut plain = session.draft.clone();
        plain.injured_spouse = None;
        let native = supported(&plain);
        assert_eq!(
            native.return_lines,
            result
                .return_lines
                .iter()
                .filter(|line| line
                    .form_line
                    .as_ref()
                    .is_none_or(|form| form.form != TaxForm::Form8379))
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            (native.tax, native.refund, native.balance, native.payments),
            (result.tax, result.refund, result.balance, result.payments)
        );
        result
    }
}

pub(in crate::tests) mod financial_omission_scope {
    use crate::tests::*;
    pub(in crate::tests) fn persisted(d: FederalDraft) -> FederalDraft {
        let draft = roundtrip_native_draft(d);
        restart(draft).draft
    }
}

pub(in crate::tests) mod r8_category_rounding {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    use crate::tests::*;
    pub(in crate::tests) fn run(sale: bool) {
        let mut a = allocation();
        a.community = InjuredCommunityFacts::Yes {
            states: vec![5],
            debt_facts: InjuredDebtFacts::NotApplicable,
        };
        let mut d = injured_return((30000, 0), (0, 0), 0, a);
        let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
        data.withholding = 49;
        let kind = if sale {
            let mut kind = ordinary_investment_sale(10000, 10000, GainTerm::Long);
            let FederalIncome::Sale { data } = &mut kind else { panic!() };
            data.withholding = 49;
            kind
        } else {
            let mut kind = savings_income(0, 0);
            let FederalIncome::Interest { data } = &mut kind else { panic!() };
            data.taxable = 49;
            data.withholding = 49;
            kind
        };
        let mut e = income(kind);
        e.owner = Owner::Spouse;
        add(&mut d, e);
        for e in &mut d.income {
            e.community = CommunityShare::Community;
        }
        let p = d.payments.as_mut().unwrap();
        p.estimated = Some(10000);
        p.estimated_payments = vec![crate::DatedPayment {
            date: 20260415,
            amount: 10000,
            payer: PaymentPayer::Taxpayer,
            refund_property: Some(RefundSourcePropertyFacts::Community {
                management: RefundManagement::NotApplicable,
            }),
        }];
        let check = |d: &FederalDraft| {
            let r = supported(d);
            assert_eq!(value(&r, "agi"), 30000);
            assert_eq!(value(&r, "taxable_income"), 0);
            assert_eq!(value(&r, "regular_income_tax"), 0);
            for line in ["25a", "25b", "25c", "25d"] {
                assert_eq!(return_line(&r, line), 0);
            }
            assert_eq!(return_line(&r, "26"), 10000);
            assert_eq!(return_line(&r, "27a"), 0);
            assert_eq!((r.tax, r.payments, r.refund), (Some(0), Some(10000), Some(10000)));
            let mut plain = d.clone();
            plain.injured_spouse = None;
            let ordinary = supported(&plain);
            assert_eq!(r.return_lines, ordinary.return_lines);
            assert_eq!(
                (r.tax, r.payments, r.refund),
                (ordinary.tax, ordinary.payments, ordinary.refund)
            );
            r
        };
        check(&d);
        let old = d.income.clone();
        let old_payments = d.payments.clone();
        let mut s = restart(d);
        for topic in [
            TaxSection::Wages,
            if sale { TaxSection::InvestmentSales } else { TaxSection::InterestDividends },
        ] {
            let p = open_topic(&mut s, topic);
            accepted(&send(&mut s, p));
        }
        let Reply::Edit(p) = from_review(&mut s, TaxSection::Payments) else { panic!() };
        accepted(&send(&mut s, p));
        let Reply::Edit(p) = native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[])
        else {
            panic!()
        };
        accepted(&send(&mut s, p));
        assert_eq!(s.draft.income.len(), old.len());
        for e in &old {
            assert_eq!(s.draft.income.iter().find(|r| r.id == e.id), Some(e));
        }
        assert_eq!(s.draft.payments, old_payments);
        let draft = roundtrip_native_draft(s.draft.clone());
        let mut s = restart(draft);
        let Reply::Edit(p) = from_review(&mut s, TaxSection::Payments) else { panic!() };
        accepted(&send(&mut s, p));
        let r = check(&s.draft);
        let estimate =
            r.lines.iter().find(|l| l.id == crate::injured_spouse::ESTIMATE_LINE).map(|l| l.amount);

        assert_eq!(
            estimate,
            Some(5000),
            "all native category money/publicSave/Avro/restart checks passed before the R8 property estimate"
        );
    }
}

pub(in crate::tests) mod r8_credit_relevance {
    use crate::tests::*;
    pub(in crate::tests) fn stored_restart(d: FederalDraft) -> FederalDraft {
        let draft = roundtrip_native_draft(d);
        restart(draft).draft
    }
}

pub(in crate::tests) mod r8_employee_withholding {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    use crate::tests::support::assert::injured_credit::r8_employee_withholding::money;
    use crate::tests::*;
    pub(in crate::tests) fn employee_id(d: &FederalDraft) -> i64 {
        d.income
            .iter()
            .find(|e| matches!(e.income, FederalIncome::EmployeeWages { .. }))
            .unwrap()
            .id
    }

    pub(in crate::tests) fn public_roundtrip(d: FederalDraft, owner: Owner) -> FederalDraft {
        money(&d, owner);
        let old = d.income.clone();
        let mut s = restart(d);
        let p = open_topic(&mut s, TaxSection::Wages);
        accepted(&send(&mut s, p));
        let reply = native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[]);
        let Reply::Edit(p) = reply else { panic!() };
        accepted(&send(&mut s, p));
        assert_eq!(s.draft.income.len(), old.len());
        for e in &old {
            assert_eq!(s.draft.income.iter().find(|r| r.id == e.id), Some(e));
        }
        let draft = roundtrip_native_draft(s.draft.clone());
        let mut s = restart(draft);
        let p = open_topic(&mut s, TaxSection::Wages);
        accepted(&send(&mut s, p));
        money(&s.draft, owner);
        s.draft
    }
}

pub(in crate::tests) mod r8_sale_withholding {
    use crate::tests::cross::interview::{accepted, from_review, open_topic};
    use crate::tests::support::assert::injured_credit::r8_sale_withholding::money;
    use crate::tests::*;
    pub(in crate::tests) fn persist(d: FederalDraft, owner: Owner) -> FederalDraft {
        money(&d, owner);
        let original = d.income.clone();
        let mut s = restart(d);
        for topic in [TaxSection::Wages, TaxSection::InvestmentSales] {
            let p = open_topic(&mut s, topic);
            accepted(&send(&mut s, p));
        }
        let reply = native_editor(from_review(&mut s, TaxSection::InjuredSpouse), &[]);
        let Reply::Edit(p) = reply else { panic!() };
        accepted(&send(&mut s, p));
        for e in &original {
            assert_eq!(s.draft.income.iter().find(|v| v.id == e.id), Some(e));
        }
        assert_eq!(s.draft.income.len(), original.len());
        let draft = roundtrip_native_draft(s.draft.clone());
        let mut s = restart(draft);
        let p = open_topic(&mut s, TaxSection::InvestmentSales);
        accepted(&send(&mut s, p));
        money(&s.draft, owner);
        s.draft
    }
}

pub(in crate::tests) mod original_map_boundaries {
    use crate::tests::cross::interview::from_review;
    use crate::tests::*;
    pub(in crate::tests) fn open_r9(session: &mut Interview) -> P {
        let before = session.draft.clone();
        let Reply::Response(response @ P::BeginInjuredSpouse { .. }) =
            from_review(session, TaxSection::InjuredSpouse)
        else {
            panic!("expected the current injured-spouse WorkflowRequest")
        };
        let bytes = response.to_avro();
        let response = P::from_avro(&bytes).unwrap();
        assert_eq!(response.to_avro(), bytes);
        assert_eq!(session.draft, before);
        let P::BeginInjuredSpouse {
            cookie, revision, page, previous, review, value: initial, ..
        } = response
        else {
            unreachable!()
        };
        assert!(initial.allocations.is_empty() && initial.new_allocations.is_empty());
        let result = supported(&session.draft);
        let value = session.draft.injured_spouse.as_ref().unwrap().page(&session.draft, &result);
        assert!(value.allocations.is_empty() && value.new_allocations.is_empty());
        assert_eq!(value.known, initial.known);
        assert_eq!(value.components, initial.components);
        assert_eq!(value.injured, initial.injured);
        P::SaveInjuredSpouse {
            user: TEST_USER,
            cookie,
            revision,
            page,
            previous,
            review,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            value,
        }
    }
}
