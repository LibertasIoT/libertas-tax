pub(in crate::tests) mod business_credit_owner_sources {
    use crate::tests::support::builders::injured_credit::business_credit_owner_sources::{
        PayeeLimitCase, ResidualFamily,
    };
    use crate::tests::*;
    pub(in crate::tests) fn source(d: &FederalDraft, owner: Owner) -> i64 {
        let history = d.credits.as_ref().unwrap().business_credits.as_ref().unwrap();
        assert!(history.carryforwards.is_empty() && history.carrybacks.is_empty());
        let [credit] = history.credits.as_slice() else {
            panic!("one actual current source credit")
        };
        assert_eq!(credit.credit, BusinessCredit::AdvancedManufacturingProduction);
        assert_eq!(credit.amount, 500000);
        assert_eq!(credit.election, CreditElection::None);
        let CreditSource::Business { business: Some(id), reduced: true } = credit.source else {
            panic!("actual current business identity")
        };
        assert!(id > 0);
        let entry = d.income.iter().find(|entry| entry.id == id).unwrap();
        assert_eq!(entry.owner, owner);
        assert!(matches!(&entry.income, FederalIncome::Business { data }
        if data.material_participation && data.all_at_risk()
            && matches!(data.activity, BusinessActivity::Ordinary { .. })));
        id
    }

    pub(in crate::tests) fn money(d: &FederalDraft) -> FederalResult {
        let r = supported(d);
        // Independent pre-execution hypotheses from the existing190000 fixture:
        // 157800 taxable,24140 regular,12948 tentative minimum; the actual5000
        // credit fits native11192 capacity, leaving19140 tax and19040 balance.
        for (key, expected) in [
            ("agi", 190000),
            ("taxable_income", 157800),
            ("regular_income_tax", 24140),
            ("form_3800_line_14", 12948),
            ("general_business_credit", 5000),
        ] {
            assert_eq!(value(&r, key), expected, "ordinary {key}");
        }
        // 2026-10-15 new-fixture fitting: native Schedule3 emits nonzero rows
        // only (F28699). Preserve the zero elective amount through mandatory1040
        // line31 and the actual refundable subtotal, not an invented zero row.
        assert!(r.lines.iter().all(|row| row.id != "schedule_3_elective_payment"));
        assert_eq!(r.refundable_credits, Some(0));
        assert_eq!(return_line(&r, "31"), 0);
        assert_eq!(return_line(&r, "33"), 10000);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(1914000), Some(10000), Some(0), Some(1904000))
        );
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refundable_credits, r.refund, r.balance),
            (
                ordinary.tax,
                ordinary.payments,
                ordinary.refundable_credits,
                ordinary.refund,
                ordinary.balance
            )
        );
        r
    }

    pub(in crate::tests) fn residual_money(
        d: &FederalDraft,
        family: ResidualFamily,
    ) -> FederalResult {
        let r = supported(d);
        let (agi, taxable, regular, minimum, used, elective, tax, refund, balance) = match family {
            ResidualFamily::ForeignTax => (191000, 158800, 24360, 13108, 5000, 0, 19260, 0, 19160),
            // Tax Table10100..10150 midpoint10125:1013 regular. The5000
            // current credit uses1013 and leaves3987 EPE; PTC4408 is independent.
            ResidualFamily::PremiumTax => (42300, 10100, 1013, 0, 1013, 3987, 0, 8495, 0),
        };
        for (key, expected) in [
            ("agi", agi),
            ("taxable_income", taxable),
            ("regular_income_tax", regular),
            ("form_3800_line_14", minimum),
            ("general_business_credit", used),
        ] {
            assert_eq!(value(&r, key), expected, "{family:?} ordinary {key}");
        }
        let refundable = match family {
            ResidualFamily::ForeignTax => {
                assert_eq!(value(&r, "foreign_tax_credit"), 100);
                assert!(r.lines.iter().all(|l| l.id != "schedule_3_elective_payment"));
                0
            }
            ResidualFamily::PremiumTax => {
                assert_eq!(value(&r, "marketplace_household_income"), 42300);
                assert_eq!(value(&r, "marketplace_annual_contribution"), 2792);
                assert_eq!(value(&r, "premium_tax_credit"), 4408);
                assert_eq!(value(&r, "schedule_3_elective_payment"), elective);
                elective + 4408
            }
        };
        assert_eq!(r.refundable_credits, Some(refundable * 100));
        assert_eq!(return_line(&r, "31"), refundable * 100);
        assert_eq!(return_line(&r, "33"), (refundable + 100) * 100);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(tax * 100), Some(10000), Some(refund * 100), Some(balance * 100))
        );
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refundable_credits, r.refund, r.balance),
            (
                ordinary.tax,
                ordinary.payments,
                ordinary.refundable_credits,
                ordinary.refund,
                ordinary.balance
            )
        );
        r
    }

    pub(in crate::tests) fn payee_limit_money(
        draft: &FederalDraft,
        case: PayeeLimitCase,
    ) -> FederalResult {
        let r = supported(draft);
        let (agi, taxable, regular, minimum, foreign, business, tax, refund, balance) = match case {
            // 2026-10-15 NEW fixture correction: native25D TaxTable rows below3000;
            //1800..1825 midpoint1812.50 gives181, matching IRS2025 i1040.
            // https://www.irs.gov/pub/irs-prior/i1040gi--2025.pdf
            // The actual500D
            // payee taxes fit the600D election but are capped by native tax181D.
            PayeeLimitCase::SoleCap(_) | PayeeLimitCase::MixedCap => {
                (34000, 1800, 181, 0, 181, 0, 0, 100, 0)
            }
            // Same2026 brackets as the original191000 fixture, one more1000D
            // dividend. Actual99/100C tax pool reports1D FTC, also reducing TMT.
            PayeeLimitCase::FullMixed(_) => {
                (192000, 159800, 24580, 13467, 1, 5000, 19579, 0, 19479)
            }
        };
        for (key, expected) in [
            ("agi", agi),
            ("taxable_income", taxable),
            ("regular_income_tax", regular),
            ("form_3800_line_14", minimum),
            ("foreign_tax_credit", foreign),
            ("general_business_credit", business),
        ] {
            assert_eq!(value(&r, key), expected, "{case:?} native {key}");
        }
        assert_eq!(r.refundable_credits, Some(0));
        assert_eq!(return_line(&r, "31"), 0);
        assert_eq!(return_line(&r, "33"), 10000);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(tax * 100), Some(10000), Some(refund * 100), Some(balance * 100))
        );
        let mut ordinary = draft.clone();
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!(r.return_lines, plain.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refundable_credits, r.refund, r.balance),
            (plain.tax, plain.payments, plain.refundable_credits, plain.refund, plain.balance)
        );
        r
    }

    pub(in crate::tests) fn payee_limit_sources(
        draft: &FederalDraft,
        case: PayeeLimitCase,
        business_owner: Owner,
    ) -> Vec<(i64, Owner, i64)> {
        source(draft, business_owner);
        assert_eq!(
            draft.credits.as_ref().unwrap().foreign_tax,
            ForeignTaxClaim::Statements { eligible: Some(true) }
        );
        let mut rows: Vec<_> = draft
            .income
            .iter()
            .filter_map(|entry| match &entry.income {
                FederalIncome::Dividend { data } => {
                    assert!(entry.id > 0);
                    assert_eq!(
                        (data.ordinary, data.qualified, data.capital_distributions),
                        (100000, 0, 0)
                    );
                    Some((entry.id, entry.owner, data.foreign_tax))
                }
                _ => None,
            })
            .collect();
        rows.sort_by_key(|row| row.0);
        let expected = match case {
            PayeeLimitCase::SoleCap(payer) => vec![(payer, 50000)],
            PayeeLimitCase::FullMixed(taxes) => {
                vec![(Owner::Taxpayer, taxes[0]), (Owner::Spouse, taxes[1])]
            }
            PayeeLimitCase::MixedCap => vec![(Owner::Taxpayer, 10000), (Owner::Spouse, 40000)],
        };
        assert_eq!(rows.iter().map(|(_, owner, tax)| (*owner, *tax)).collect::<Vec<_>>(), expected);
        rows
    }

    pub(in crate::tests) fn payee_columns(result: &FederalResult) -> [Option<i64>; 2] {
        ["b", "c"].map(|column| {
            result
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("form_8379_line_16_{column}"))
                .map(|row| row.amount)
        })
    }

    pub(in crate::tests) fn residual_sources(
        d: &FederalDraft,
        family: ResidualFamily,
        owner: Owner,
        payer: Owner,
    ) {
        let history = d.credits.as_ref().unwrap().business_credits.as_ref().unwrap();
        let [credit] = history.credits.as_slice() else { panic!() };
        assert_eq!(credit.amount, 500000);
        assert_eq!(credit.credit, BusinessCredit::AdvancedManufacturingProduction);
        assert_eq!(
            credit.election,
            match family {
                ResidualFamily::ForeignTax => CreditElection::None,
                ResidualFamily::PremiumTax => CreditElection::ElectivePayment { amount: 500000 },
            }
        );
        let CreditSource::Business { business: Some(id), reduced: true } = credit.source else {
            panic!()
        };
        assert!(id > 0);
        let entry = d.income.iter().find(|e| e.id == id).unwrap();
        assert_eq!(entry.owner, owner);
        assert_eq!(entry.income, cash_business(0));
        match family {
            ResidualFamily::ForeignTax => {
                assert_eq!(
                    d.credits.as_ref().unwrap().foreign_tax,
                    ForeignTaxClaim::Statements { eligible: Some(true) }
                );
                let entries: Vec<_> = d
                    .income
                    .iter()
                    .filter_map(|entry| match &entry.income {
                        FederalIncome::Dividend { data } => Some((entry, data)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0.owner, payer);
                assert_eq!(entries[0].1.foreign_tax, 10000);
            }
            ResidualFamily::PremiumTax => {
                let MarketplaceClaim::Yes { facts } = &d.credits.as_ref().unwrap().marketplace
                else {
                    panic!()
                };
                assert_eq!(facts.forms.len(), 1);
                assert!(facts.forms[0].id > 0 && facts.forms[0].id != id);
            }
        }
    }

    pub(in crate::tests) fn foreign_statement_money(draft: &FederalDraft) -> FederalResult {
        residual_money(draft, ResidualFamily::ForeignTax)
    }

    pub(in crate::tests) fn foreign_statement_source_links(
        draft: &FederalDraft,
        owner: Owner,
        payer: Owner,
    ) -> (i64, i64) {
        residual_sources(draft, ResidualFamily::ForeignTax, owner, payer);
        let credit = &draft.credits.as_ref().unwrap().business_credits.as_ref().unwrap().credits[0];
        let CreditSource::Business { business: Some(id), .. } = credit.source else { panic!() };
        let dividend = draft
            .income
            .iter()
            .find(|entry| matches!(entry.income, FederalIncome::Dividend { .. }))
            .unwrap()
            .id;
        assert!(id > 0 && dividend > 0 && id != dividend);
        (id, dividend)
    }
}

pub(in crate::tests) mod elective_credit_mixed_sources {
    use crate::tests::support::builders::injured_credit::elective_credit_mixed_sources::Family;

    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, family: Family) -> FederalResult {
        let r = supported(d);
        // Save observations before assertions so a new-fixture financial hypothesis
        // failure cannot be relabeled as an ownership regression.

        let (agi, taxable, regular, tentative, used, elective, tax, refund) = match family {
            Family::NoUse => (30000, 0, 0, 0, 0, 5000, 0, 5100),
            // Independent hypotheses reviewed before execution:
            // TI190000-32200=157800; regular11600+57000*22%=24140.
            // TMT(190000-140200)*26%=12948. Native GBC capacity is11192;
            // netEPE50000-11192=38808; refund100+38808-12948=25960.
            Family::BindingUse => (190000, 157800, 24140, 12948, 11192, 38808, 12948, 25960),
        };
        for (key, expected) in [
            ("agi", agi),
            ("taxable_income", taxable),
            ("regular_income_tax", regular),
            ("form_3800_line_14", tentative),
            ("general_business_credit", used),
            ("schedule_3_elective_payment", elective),
        ] {
            assert_eq!(value(&r, key), expected, "{family:?} {key}");
        }
        assert_eq!(return_line(&r, "31"), elective * 100);
        assert_eq!(return_line(&r, "33"), (elective + 100) * 100);
        assert_eq!(r.refundable_credits, Some(elective * 100));
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(tax * 100), Some(10000), Some(refund * 100), Some(0)),
            "{family:?}",
        );
        assert_eq!(return_line(&r, "23"), 0);
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refundable_credits, r.refund, r.balance),
            (
                ordinary.tax,
                ordinary.payments,
                ordinary.refundable_credits,
                ordinary.refund,
                ordinary.balance
            ),
        );
        r
    }

    pub(in crate::tests) fn source_links(
        d: &FederalDraft,
        family: Family,
    ) -> Vec<(i64, Owner, i64)> {
        let history = d.credits.as_ref().unwrap().business_credits.as_ref().unwrap();
        assert_eq!(history.credits.len(), 2);
        let mut links = vec![];
        for credit in &history.credits {
            let CreditSource::Business { business: Some(id), reduced: true } = credit.source else {
                panic!("actual accepted C source required")
            };
            assert!(id > 0);
            let e = d.income.iter().find(|e| e.id == id).unwrap();
            assert!(matches!(&e.income, FederalIncome::Business { data }
            if data.material_participation && data.all_at_risk()));
            assert_eq!(credit.credit, BusinessCredit::AdvancedManufacturingProduction);
            let expected = family.credits()[usize::from(e.owner == Owner::Spouse)] * 100;
            assert_eq!(credit.amount, expected);
            assert_eq!(credit.election, CreditElection::ElectivePayment { amount: expected });
            links.push((id, e.owner, expected));
        }
        links.sort_by_key(|row| row.0);
        assert_ne!(links[0].0, links[1].0);
        links
    }
}

pub(in crate::tests) mod elective_credit_owner_sources {
    use crate::tests::*;
    pub(in crate::tests) fn ordinary(d: &FederalDraft) -> FederalResult {
        let r = supported(d);
        assert_eq!(value(&r, "agi"), 30000);
        assert_eq!(value(&r, "taxable_income"), 0);
        assert_eq!(value(&r, "regular_income_tax"), 0);
        assert_eq!(value(&r, "general_business_credit"), 0);
        assert_eq!(value(&r, "schedule_3_elective_payment"), 5000);
        assert_eq!(return_line(&r, "31"), 500000);
        assert_eq!(return_line(&r, "33"), 510000);
        // 2026-10-15: native FederalResult records elective payment among
        // refundable_credits, separately from payments. The initial new bd27
        // hypothesis put all $5,100 in payments; unchanged line33/refund is $5,100.
        assert_eq!(r.refundable_credits, Some(500000));
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(0), Some(10000), Some(510000), Some(0))
        );
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let joint = supported(&plain);
        assert_eq!(r.return_lines, joint.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (joint.tax, joint.payments, joint.refund, joint.balance)
        );
        r
    }
}

pub(in crate::tests) mod foreign_credit_ordinary_sources {
    use crate::tests::support::builders::injured_credit::foreign_credit_ordinary_sources::Family;
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, family: Family) -> FederalResult {
        let r = supported(d);
        let (agi, taxable, regular, foreign, tax, payments, refund) = match family {
            // Native2026 TaxTable67800..67850 midpoint67825:2480+43025*.12.
            Family::Current | Family::CurrentAccrued => {
                (100000, 67800, 7643, 1000, 6643, 20000, 13357)
            }
            Family::PriorWage | Family::PriorPassive | Family::Historical => {
                (100000, 67800, 7643, 200, 7443, 20000, 12557)
            }
            // Independent per-category Form1116 arithmetic: general net8398
            // permits100; passive net840/168800 *26560 rounds132 (tax200).
            Family::MixedCategories => (201000, 168800, 26560, 232, 26328, 40000, 13672),
            // Exact pre-existing crate::tests::foreign::foreign::k3_passive_tax... monetary oracles.
            Family::K3 { .. } => (162000, 129800, 17980, 666, 17314, 40000, 22686),
            // NEW financial-first hypothesis:174000 gross,32200 standard;
            // JA12000 gross has2221 allocated deduction/net9779;
            //20620*9779/141800 rounds1422, under1800 actual current tax.
            Family::K3Pair => (174000, 141800, 20620, 1422, 19198, 40000, 20802),
            // NEW joint hypotheses preserve the original Form2555 source bodies.
            // Employee stacking:tax167800−tax132900=26340−18662.
            Family::EmploymentExcluded => (67100, 34900, 7678, 7678, 0, 0, 0),
            // Existing joint wage/business fixture beforeFTC:14800 total,
            //7065SE; fullyexcluded branchtax adds no eligible credit source.
            Family::BusinessExcluded => (67100, 34900, 7735, 7735, 7065, 0, 0),
            // Existing high-tax source body with joint filing:33560 passive net
            // is kicked into general;83900/167800 is one-half, tax26340.
            Family::HighTax => (200000, 167800, 26340, 13170, 13170, 40000, 26830),
        };
        for (key, expected) in [
            ("agi", agi),
            ("taxable_income", taxable),
            ("regular_income_tax", regular),
            ("foreign_tax_credit", foreign),
            ("alternative_minimum_tax", 0),
            ("net_investment_income_tax", 0),
        ] {
            assert_eq!(value(&r, key), expected, "{family:?} ordinary {key}");
        }
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (
                Some(tax * 100),
                Some(payments * 100),
                Some(refund * 100),
                Some(if matches!(family, Family::BusinessExcluded) { 706500 } else { 0 })
            )
        );
        assert_eq!(r.refundable_credits, Some(0));
        if matches!(family, Family::EmploymentExcluded | Family::BusinessExcluded) {
            assert_eq!(
                value(&r, "foreign_earned_exclusion"),
                if matches!(family, Family::BusinessExcluded) { 182900 } else { 132900 }
            );
            assert_eq!(value(&r, "foreign_general_tax_reduction"), -19935);
            if matches!(family, Family::BusinessExcluded) {
                assert_eq!(value(&r, "self_employment_tax"), 7065);
                assert_eq!(value(&r, "foreign_branch_tax_reduction"), -5000);
                let mut before = d.clone();
                before.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
                assert_eq!(supported(&before).tax, Some(1480000));
            }
        }
        if matches!(family, Family::HighTax) {
            assert_eq!(value(&r, "foreign_high_tax_net_income"), 33560);
            assert_eq!(value(&r, "foreign_high_tax_taxes"), 15000);
            assert_eq!(value(&r, "foreign_general_unused_tax"), 1830);
            assert_eq!(value(&r, "foreign_passive_unused_tax"), 0);
        }
        if matches!(family, Family::Historical) {
            assert_eq!(value(&r, "foreign_general_regular_carry_used"), 200);
        }
        if matches!(family, Family::MixedCategories) {
            assert_eq!(value(&r, "foreign_passive_unused_tax"), 68);
            assert_eq!(value(&r, "foreign_general_unused_tax"), 0);
            assert_eq!(value(&r, "foreign_high_tax_net_income"), 0);
        }
        if matches!(family, Family::K3Pair) {
            assert_eq!(value(&r, "foreign_passive_unused_tax"), 378);
            assert_eq!(value(&r, "foreign_k3_gross_income_correction"), 0);
        }
        if matches!(family, Family::K3 { .. }) {
            assert_eq!(value(&r, "foreign_passive_unused_tax"), 234);
            assert_eq!(value(&r, "foreign_k3_gross_income_correction"), 0);
        }
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (ordinary.tax, ordinary.payments, ordinary.refund, ordinary.balance)
        );
        r
    }

    pub(in crate::tests) fn columns(r: &FederalResult) -> [Option<i64>; 2] {
        ["b", "c"].map(|column| {
            r.lines
                .iter()
                .find(|row| row.id == alloc::format!("form_8379_line_16_{column}"))
                .map(|row| row.amount)
        })
    }
}

pub(in crate::tests) mod fuel_credit_owner_sources {
    use crate::tests::*;
    pub(in crate::tests) fn ordinary(result: &FederalResult, raw_credit: i64, annual_credit: i64) {
        assert_eq!(value(result, "agi"), 30_000);
        assert_eq!(value(result, "taxable_income"), 0);
        assert_eq!(value(result, "regular_income_tax"), 0);
        assert_eq!(result.tax, Some(0));
        assert_eq!(result.payments, Some((100 + annual_credit) * 100));
        assert_eq!(result.refund, Some((100 + annual_credit) * 100));
        assert_eq!(result.balance, Some(0));
        assert_eq!(return_line(result, "31"), annual_credit * 100);
        assert_eq!(value(result, "schedule_3_fuel_tax_credit"), annual_credit);
        assert_eq!(
            result.lines.iter().find(|row| row.id == "fuel_tax_credit").unwrap().amount,
            raw_credit
        );
        assert_eq!(
            result.lines.iter().find(|row| row.id == "home_kerosene_fuel_4a").unwrap().amount,
            50
        );
        assert_eq!(column(result, "17", "a"), annual_credit);
    }
}

pub(in crate::tests) mod financial_omission_scope {
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, agi: i64) -> FederalResult {
        let r = supported(d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "regular_income_tax"), 0);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (Some(0), Some(100000), Some(100000), Some(0))
        );
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let joint = supported(&ordinary);
        assert_eq!(r.return_lines, joint.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund, r.balance),
            (joint.tax, joint.payments, joint.refund, joint.balance)
        );
        r
    }
}

pub(in crate::tests) mod r8_credit_relevance {
    use crate::tests::*;
    pub(in crate::tests) fn estimate(r: &FederalResult) -> Option<i64> {
        r.lines.iter().find(|l| l.id == crate::injured_spouse::ESTIMATE_LINE).map(|l| l.amount)
    }
}

pub(in crate::tests) mod r8_employee_withholding {
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, owner: Owner) -> FederalResult {
        let r = supported(d);
        // Actual7500 employee pay +22500 ordinary W2, below32200 standard.
        // Form8919:465 SocialSecurity +109 Medicare =574; no EIC at30000.
        for (id, v) in [
            ("agi", 30000),
            ("taxable_income", 0),
            ("regular_income_tax", 0),
            ("net_investment_income_tax", 0),
            ("additional_medicare_tax", 0),
        ] {
            assert_eq!(value(&r, id), v);
        }
        assert_eq!(return_line(&r, "1a"), 2250000);
        assert_eq!(return_line(&r, "1g"), 750000);
        assert_eq!(return_line(&r, "25a"), 75000);
        assert_eq!(return_line(&r, "25b"), 25000);
        assert_eq!(return_line(&r, "25d"), 100000);
        assert_eq!(return_line(&r, "27a"), 0);
        assert_eq!(
            value(
                &r,
                &alloc::format!("employee_wages_{}_tax", usize::from(owner == Owner::Spouse))
            ),
            574
        );
        assert_eq!((r.tax, r.payments, r.refund), (Some(57400), Some(100000), Some(42600)));
        // Form8379 PartIII remains each physical source's full amount; the
        // community refund overlay must not halve earnings, payroll or withholding.
        let injured = d.injured_spouse.as_ref().unwrap().injured;
        let owns = owner == injured;
        for (line, expected) in [
            ("13a", if owns { [0, 22500] } else { [22500, 0] }),
            ("13b", if owns { [7500, 0] } else { [0, 7500] }),
            ("18", if owns { [574, 0] } else { [0, 574] }),
            ("19", if owns { [250, 750] } else { [750, 250] }),
        ] {
            assert_eq!([column(&r, line, "b"), column(&r, line, "c")], expected);
        }
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(ordinary.return_lines, r.return_lines);
        assert_eq!(
            (ordinary.tax, ordinary.payments, ordinary.refund),
            (r.tax, r.payments, r.refund)
        );
        r
    }

    pub(in crate::tests) fn estimate_if_known(r: &FederalResult) -> Option<i64> {
        r.lines.iter().find(|l| l.id == crate::injured_spouse::ESTIMATE_LINE).map(|l| l.amount)
    }
}

pub(in crate::tests) mod r8_sale_withholding {
    use crate::tests::*;
    pub(in crate::tests) fn money(d: &FederalDraft, owner: Owner) -> FederalResult {
        let r = supported(d);
        assert_eq!(value(&r, "agi"), 30000);
        assert_eq!(value(&r, "taxable_income"), 0);
        assert_eq!(value(&r, "regular_income_tax"), 0);
        assert_eq!(value(&r, "capital_gain"), 0);
        assert_eq!(value(&r, "net_investment_income_tax"), 0);
        assert_eq!(return_line(&r, "25a"), 75000);
        assert_eq!(return_line(&r, "25b"), 25000);
        assert_eq!(return_line(&r, "25d"), 100000);
        assert_eq!(return_line(&r, "27a"), 0);
        assert_eq!((r.tax, r.payments, r.refund), (Some(0), Some(100000), Some(100000)));
        let injured = d.injured_spouse.as_ref().unwrap().injured;
        assert_eq!(
            [column(&r, "19", "b"), column(&r, "19", "c")],
            if injured == owner { [250, 750] } else { [750, 250] }
        );
        let mut plain = d.clone();
        plain.injured_spouse = None;
        let ordinary = supported(&plain);
        assert_eq!(r.return_lines, ordinary.return_lines);
        assert_eq!(
            (r.tax, r.payments, r.refund),
            (ordinary.tax, ordinary.payments, ordinary.refund)
        );
        r
    }
}

pub(in crate::tests) mod r9_actual_clients {
    use crate::tests::*;
    pub(in crate::tests) fn checked(
        d: &FederalDraft,
        farm_owner: Owner,
        injured: Owner,
    ) -> FederalResult {
        assert_eq!(d.injured_spouse.as_ref().unwrap().injured, injured);
        assert!(d.injured_spouse.as_ref().unwrap().allocations.is_empty());
        let r = supported(d);
        assert_eq!(value(&r, "agi"), 212935);
        assert_eq!(value(&r, "regular_income_tax"), 22297);
        assert_eq!(r.tax, Some(3642700));
        assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
        assert!(r.injured_spouse.as_ref().unwrap().required.is_empty());
        assert_eq!(column(&r, "17", "a"), 6000);
        for id in
            ["form_8379_line_17_b", "form_8379_line_17_c", crate::injured_spouse::ESTIMATE_LINE]
        {
            assert!(!r.lines.iter().any(|row| row.id == id), "{id} stays unavailable");
        }
        assert!(
            r.review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_FARMLAND_ESTIMATE_UNAVAILABLE")
        );
        assert!(
            !r.review_notes
                .iter()
                .any(|note| note.display.key == "WARN_8379_ALLOCATION_CHOICES_NEEDED")
        );
        assert_eq!(
            [column(&r, "13b", "b"), column(&r, "13b", "c")],
            if farm_owner == injured { [140000, 0] } else { [0, 140000] }
        );
        assert_eq!([column(&r, "20", "b"), column(&r, "20", "c")], [0, 0]);
        assert!(
            !r.injured_spouse
                .as_ref()
                .unwrap()
                .known
                .iter()
                .any(|row| Message::decode(&row.label).key == "EXPLAIN_FORM8379_REFUNDABLE_JOINT")
        );
        let mut ordinary = d.clone();
        ordinary.injured_spouse = None;
        let plain = supported(&ordinary);
        assert_eq!(
            (r.tax, r.refund, r.balance, r.payments),
            (plain.tax, plain.refund, plain.balance, plain.payments)
        );
        assert_eq!(r.return_lines, plain.return_lines);
        assert_eq!(r.farmland_installments, plain.farmland_installments);
        r
    }
}
