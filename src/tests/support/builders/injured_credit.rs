pub(in crate::tests) mod business_credit_owner_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(owner: Owner, injured: Owner, reverse: bool) -> FederalDraft {
        let mut claim = allocation();
        claim.injured = injured;
        // Reuse the admitted elective-credit fixture's actual current activity and
        // native190000 wage/tax-capacity operands. This credit elects neither sale
        // nor elective payment; no new source fact or monetary oracle is inferred.
        let mut d = injured_return((190000, 100), (0, 0), 0, claim);
        let mut source = income(cash_business(0));
        source.owner = owner;
        source.label = alloc::format!("{owner:?} current production activity");
        add(&mut d, source);
        let id = d.income.last().unwrap().id;
        d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
            credits: vec![EnteredBusinessCredit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                credit: BusinessCredit::AdvancedManufacturingProduction,
                amount: 500000,
                source: CreditSource::Business { business: Some(id), reduced: true },
                election: CreditElection::None,
            }],
            carryforwards: vec![],
            carrybacks: vec![],
            spouse_without_credits: false,
        });
        if reverse {
            d.income.reverse();
        }
        d
    }

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum ResidualFamily {
        ForeignTax,
        PremiumTax,
    }

    impl ResidualFamily {
        pub(in crate::tests) fn key(self) -> &'static str {
            match self {
                Self::ForeignTax => "nonrefundable_credit_remainder",
                Self::PremiumTax => "other_refundable_credit_remainder",
            }
        }

        pub(in crate::tests) fn old_key(self) -> &'static str {
            match self {
                Self::ForeignTax => "INJURED_COMPONENT_NONREFUNDABLE",
                Self::PremiumTax => "INJURED_COMPONENT_REFUNDABLE",
            }
        }

        pub(in crate::tests) fn answer(self) -> i64 {
            match self {
                Self::ForeignTax => 5000,
                Self::PremiumTax => 220400,
            }
        }

        pub(in crate::tests) fn remainder(self) -> i64 {
            match self {
                Self::ForeignTax => 10000,
                Self::PremiumTax => 440800,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum PayeeLimitCase {
        SoleCap(Owner),
        FullMixed([i64; 2]),
        MixedCap,
    }

    pub(in crate::tests) fn residual_fixture(
        family: ResidualFamily,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut d = fixture(owner, injured, false);
        match family {
            ResidualFamily::ForeignTax => {
                // Existing foreign.rs6380-6414 payee-statement election source,
                // without a K1. IRS i1116 election: passive statement income and
                // qualified foreign taxes <=600 on a joint return.
                // https://www.irs.gov/instructions/i1116
                let mut dividend = crate::tests::foreign::foreign::fund_dividends(1000, 0, 0);
                let FederalIncome::Dividend { data } = &mut dividend else { panic!() };
                data.foreign_tax = 10000;
                let mut entry = income(dividend);
                entry.owner = owner;
                entry.label = "Synthetic foreign dividend statement".into();
                add(&mut d, entry);
                d.credits.as_mut().unwrap().foreign_tax =
                    ForeignTaxClaim::Statements { eligible: Some(true) };
            }
            ResidualFamily::PremiumTax => {
                let wage_entry = d
                    .income
                    .iter_mut()
                    .find(|entry| {
                        entry.owner == Owner::Taxpayer
                            && matches!(entry.income, FederalIncome::Wage { .. })
                    })
                    .unwrap();
                *wage_entry = wage(wage_entry.id, Owner::Taxpayer, 4230000, 10000);
                d.credits.as_mut().unwrap().business_credits.as_mut().unwrap().credits[0]
                    .election = CreditElection::ElectivePayment { amount: 500000 };
                // Reuse the settled health_accounts.rs732 source construction.
                // Joint family2 FPL21150, exact200%; RP2025-25 gives6.60%.
                // Annual contribution42300*.066=2791.80 ->2792, credit7200-2792.
                // https://www.irs.gov/pub/irs-drop/rp-25-25.pdf
                let claim = marketplace_claim(
                    &mut d,
                    (1..=12).map(|month| marketplace_month(month, 500, 600, 0)).collect(),
                );
                d.credits.as_mut().unwrap().marketplace = claim;
            }
        }
        d.injured_spouse.as_mut().unwrap().allocations.push(InjuredAmount {
            component: family.old_key().into(),
            // Deliberately inside the NEW residual bounds, so a range-only
            // compatibility check cannot distinguish these different pools.
            injured: family.answer(),
        });
        if reverse {
            d.income.reverse();
        }
        d
    }

    pub(in crate::tests) fn foreign_statement_fixture(
        owner: Owner,
        payer: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut draft = residual_fixture(ResidualFamily::ForeignTax, owner, injured, reverse);
        draft
            .income
            .iter_mut()
            .find(|entry| matches!(entry.income, FederalIncome::Dividend { .. }))
            .unwrap()
            .owner = payer;
        draft.injured_spouse.as_mut().unwrap().allocations.clear();
        draft
    }

    pub(in crate::tests) fn payee_limit_fixture(
        case: PayeeLimitCase,
        business_owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let first_payer = match case {
            PayeeLimitCase::SoleCap(owner) => owner,
            _ => Owner::Taxpayer,
        };
        let mut draft = foreign_statement_fixture(business_owner, first_payer, injured, false);
        let (wages, taxes) = match case {
            PayeeLimitCase::SoleCap(_) => (33000, [50000, 0]),
            PayeeLimitCase::FullMixed(taxes) => (190000, taxes),
            // Two actual1000D dividends plus32000D W2 preserve34000D AGI.
            PayeeLimitCase::MixedCap => (32000, [10000, 40000]),
        };
        let entry = draft
            .income
            .iter_mut()
            .find(|entry| {
                entry.owner == Owner::Taxpayer && matches!(entry.income, FederalIncome::Wage { .. })
            })
            .unwrap();
        *entry = wage(entry.id, Owner::Taxpayer, wages * 100, 10000);
        let first = draft
            .income
            .iter_mut()
            .find(|entry| matches!(entry.income, FederalIncome::Dividend { .. }))
            .unwrap();
        let FederalIncome::Dividend { data } = &mut first.income else { panic!() };
        data.foreign_tax = taxes[0];
        if !matches!(case, PayeeLimitCase::SoleCap(_)) {
            let mut second = crate::tests::foreign::foreign::fund_dividends(1000, 0, 0);
            let FederalIncome::Dividend { data } = &mut second else { panic!() };
            data.foreign_tax = taxes[1];
            let mut entry = income(second);
            entry.owner = Owner::Spouse;
            entry.label = "Synthetic second foreign dividend statement".into();
            add(&mut draft, entry);
        }
        if reverse {
            draft.income.reverse();
        }
        draft
    }
}

pub(in crate::tests) mod elective_credit_mixed_sources {
    use crate::tests::*;
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Family {
        NoUse,
        BindingUse,
    }

    impl Family {
        pub(in crate::tests) fn credits(self) -> [i64; 2] {
            match self {
                Self::NoUse => [3000, 2000],
                Self::BindingUse => [5000, 45000],
            }
        }
    }

    pub(in crate::tests) fn fixture(family: Family, injured: Owner, order: u8) -> FederalDraft {
        let mut claim = allocation();
        claim.injured = injured;
        if matches!(family, Family::BindingUse) {
            // Explicit independent choice for this same used-credit pool; not a
            // minimum prefill, a gross-credit ratio or a second refundable choice.
            claim.allocations.push(InjuredAmount {
                component: USED.into(),
                injured: if injured == Owner::Taxpayer { 400000 } else { 719200 },
            });
        }
        let wage = match family {
            Family::NoUse => 30000,
            Family::BindingUse => 190000,
        };
        let mut d = injured_return((wage, 100), (0, 0), 0, claim);
        let mut credits = vec![];
        for (owner, amount) in [Owner::Taxpayer, Owner::Spouse].into_iter().zip(family.credits()) {
            let mut e = income(cash_business(0));
            e.owner = owner;
            e.label = alloc::format!("{owner:?} production activity");
            let FederalIncome::Business { data } = &e.income else { panic!() };
            assert!(data.material_participation && data.all_at_risk());
            assert!(matches!(data.activity, BusinessActivity::Ordinary { .. }));
            add(&mut d, e);
            credits.push(EnteredBusinessCredit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                credit: BusinessCredit::AdvancedManufacturingProduction,
                amount: amount * 100,
                source: CreditSource::Business {
                    business: Some(d.income.last().unwrap().id),
                    reduced: true,
                },
                election: CreditElection::ElectivePayment { amount: amount * 100 },
            });
        }
        if order & 1 != 0 {
            d.income.reverse();
        }
        if order & 2 != 0 {
            credits.reverse();
        }
        d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
            credits,
            carryforwards: vec![],
            carrybacks: vec![],
            spouse_without_credits: false,
        });
        d
    }

    pub(in crate::tests) const USED: &str = "INJURED_COMPONENT_NONREFUNDABLE";
}

pub(in crate::tests) mod elective_credit_owner_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(owner: Owner, injured: Owner, reverse: bool) -> FederalDraft {
        let mut claim = allocation();
        claim.injured = injured;
        let mut d = injured_return((30000, 100), (0, 0), 0, claim);
        let mut source = income(cash_business(0));
        source.owner = owner;
        add(&mut d, source);
        let id = d.income.last().unwrap().id;
        d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
            credits: vec![EnteredBusinessCredit {
                carbon_oxide_origin: None,
                renewable_facility_origin: None,
                credit: BusinessCredit::AdvancedManufacturingProduction,
                amount: 500000,
                source: CreditSource::Business { business: Some(id), reduced: true },
                election: CreditElection::ElectivePayment { amount: 500000 },
            }],
            carryforwards: vec![],
            carrybacks: vec![],
            spouse_without_credits: false,
        });
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod foreign_credit_ordinary_sources {
    use crate::tests::foreign::foreign::{fund_claim, k3_answer, k3_blank, k3_partner, k3_row};

    use crate::tests::*;
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Family {
        Current,
        PriorWage,
        PriorPassive,
        Historical,
        MixedCategories,
        K3 { released: bool },
        K3Pair,
        CurrentAccrued,
        EmploymentExcluded,
        BusinessExcluded,
        HighTax,
    }

    pub(in crate::tests) fn row(
        owner: Owner,
        category: ForeignTaxCategory,
        income: i64,
        tax: i64,
    ) -> ForeignOrdinaryTax {
        ForeignOrdinaryTax {
            label: alloc::format!("Synthetic actual {owner:?} foreign tax"),
            country: "ZZ".into(),
            owner,
            category,
            income: income * 100,
            tax: tax * 100,
        }
    }

    pub(in crate::tests) fn fixture(
        family: Family,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        if matches!(family, Family::CurrentAccrued) {
            let mut d = fixture(Family::Current, owner, injured, reverse);
            let ForeignTaxClaim::Ordinary { basis, .. } =
                &mut d.credits.as_mut().unwrap().foreign_tax
            else {
                panic!()
            };
            *basis = ForeignTaxBasis::Accrued;
            return d;
        }
        if matches!(family, Family::EmploymentExcluded | Family::BusinessExcluded | Family::HighTax)
        {
            let mut selection = allocation();
            selection.injured = injured;
            let mut d = injured_return(
                if matches!(family, Family::HighTax) { (100000, 40000) } else { (0, 0) },
                (0, 0),
                0,
                selection,
            );
            let employee =
                if matches!(family, Family::EmploymentExcluded) { owner } else { other(owner) };
            let wages = if matches!(family, Family::HighTax) { 60000 } else { 200000 };
            let mut pay = income(foreign_pay(wages));
            pay.owner = employee;
            add(&mut d, pay);
            if !matches!(family, Family::HighTax) {
                set_claim(&mut d, foreign_claim(employee));
            }
            let mut claim = fund_claim(0, 0, 0, ForeignRateAdjustment::Exception);
            let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim else { panic!() };
            sources.clear();
            sources.push(row(
                employee,
                ForeignTaxCategory::Employment,
                wages,
                if matches!(family, Family::HighTax) { 0 } else { 30000 },
            ));
            if matches!(family, Family::BusinessExcluded) {
                let mut business = income(foreign_service_business(50000));
                business.owner = owner;
                add(&mut d, business);
                let business = d.income.last().unwrap().id;
                set_claim(&mut d, foreign_claim(owner));
                sources.push(row(
                    owner,
                    ForeignTaxCategory::Business { business, separate_books: true },
                    0,
                    5000,
                ));
            } else if matches!(family, Family::HighTax) {
                let mut interest = income(savings_income(40000, 0));
                interest.owner = owner;
                add(&mut d, interest);
                sources.push(row(
                    owner,
                    ForeignTaxCategory::Passive {
                        sales: vec![],
                        withholding: ForeignWithholding::AtLeast15,
                        qualified: 0,
                    },
                    40000,
                    15000,
                ));
            }
            d.credits.as_mut().unwrap().foreign_tax = claim;
            if reverse {
                d.income.reverse();
            }
            return d;
        }
        if matches!(family, Family::K3Pair) {
            let mut d = fixture(Family::K3 { released: false }, owner, injured, false);
            let mut second = d
                .income
                .iter()
                .find(|entry| matches!(entry.income, FederalIncome::Partnership { .. }))
                .unwrap()
                .clone();
            second.owner = other(owner);
            second.label = "Synthetic second Global Fund LP same JA category".into();
            add(&mut d, second);
            if reverse {
                d.income.reverse();
            }
            return d;
        }
        let mut selection = allocation();
        selection.injured = injured;
        let (wages, withheld) = match family {
            Family::K3 { .. } | Family::K3Pair => (150000, 40000),
            Family::MixedCategories => (190000, 40000),
            _ => (60000, 20000),
        };
        let mut d = injured_return((wages, withheld), (0, 0), 0, selection);
        let mut claim = fund_claim(0, 0, 0, ForeignRateAdjustment::Exception);
        let ForeignTaxClaim::Ordinary { sources, history, .. } = &mut claim else { panic!() };
        sources.clear();
        match family {
            Family::K3 { released } => {
                // The existing foreign::k3_fixture_a monetary/source body: one
                // actual parent, box5 2000,6a10000,21 900; JA passive6000/tax900.
                let mut k3 = k3_blank();
                k3.countries = vec![k3_row("JA", K3Category::Passive, 6000, 0, 900)];
                k3.withholding = vec![k3_answer("JA", ForeignWithholding::AtLeast15)];
                k3.us_gross = 600000;
                let mut data = k3_partner(2000, 10000, 900, Some(k3));
                if released {
                    // Same accepted release as foreign::k3_box_21_above...:
                    // the earlier1D released class is absent from current K3 tax.
                    data.regular.carried.foreign_taxes = 100;
                    data.regular.basis.opening += 100;
                    data.alternative = Some(data.regular.clone());
                }
                let mut entry = income(FederalIncome::Partnership { data });
                entry.owner = owner;
                entry.label = "Synthetic Global Fund LP current K3".into();
                add(&mut d, entry);
            }
            Family::MixedCategories => {
                let mut pay = income(foreign_pay(10000));
                pay.owner = owner;
                add(&mut d, pay);
                let mut interest = income(savings_income(1000, 0));
                interest.owner = other(owner);
                add(&mut d, interest);
                sources.push(row(owner, ForeignTaxCategory::Employment, 10000, 100));
                sources.push(row(
                    other(owner),
                    ForeignTaxCategory::Passive {
                        sales: vec![],
                        withholding: ForeignWithholding::Below15,
                        qualified: 0,
                    },
                    1000,
                    200,
                ));
            }
            _ => {
                let earner = if matches!(family, Family::PriorWage | Family::PriorPassive) {
                    other(owner)
                } else {
                    owner
                };
                let mut pay = income(foreign_pay(40000));
                pay.owner = earner;
                add(&mut d, pay);
                sources.push(row(
                    earner,
                    ForeignTaxCategory::Employment,
                    40000,
                    if matches!(family, Family::Current) { 1000 } else { 0 },
                ));
                match family {
                    Family::PriorWage | Family::PriorPassive => {
                        let category = if matches!(family, Family::PriorWage) {
                            ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 }
                        } else {
                            ForeignTaxCategory::PriorPassiveTax { income_year: 2025 }
                        };
                        // A tax first paid in2026, not ownerless ScheduleB history.
                        sources.push(row(owner, category, 0, 200));
                    }
                    Family::Historical => {
                        *history = foreign_history_of(&[foreign_balance(
                            2025,
                            ForeignCreditCategory::General,
                            200,
                            0,
                        )]);
                    }
                    _ => {}
                }
            }
        }
        d.credits.as_mut().unwrap().foreign_tax = claim;
        if reverse {
            d.income.reverse();
        }
        d
    }

    pub(in crate::tests) fn other(owner: Owner) -> Owner {
        if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer }
    }
}

pub(in crate::tests) mod fuel_credit_owner_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        business: [i64; 2],
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut claim = allocation();
        claim.injured = injured;
        // 2026-10-15: the source is actual home50c, not the rounded home$1.
        // The preserved94 baseline used100c and separate rounded-category
        // hypotheses. One native fuel category retains this real choice in Cents.
        claim.allocations.push(InjuredAmount {
            component: "home_fuel_tax_credit".into(),
            injured: if injured == Owner::Taxpayer { 50 } else { 0 },
        });
        let mut draft = injured_return((30_000, 100), (0, 0), 0, claim);
        for (owner, cents) in [Owner::Taxpayer, Owner::Spouse].into_iter().zip(business) {
            let mut data = ordinary_partner_source(0, true);
            data.fuel_tax_credit = (cents != 0).then(|| PartnerFuelTaxCredit {
                fuels: fuel(cents),
                statement_complete: true,
                farm_activity: false,
            });
            let mut entry = income(FederalIncome::Partnership { data });
            entry.owner = owner;
            add(&mut draft, entry);
        }
        draft.payments.as_mut().unwrap().home_kerosene = Some(FederalHomeKerosene {
            thousandths_of_gallon: 2058,
            actual_cost: 1000,
            qualified: true,
            not_claimed_elsewhere: true,
            records_complete: true,
        });
        if reverse {
            draft.income.reverse();
        }
        draft
    }

    pub(in crate::tests) fn fuel(cents: i64) -> FederalFuelLines {
        let thousandths_of_gallon = match cents {
            1 => 42,
            24 => 988,
            49 => 2016,
            50 => 2058,
            150 => 6173,
            _ => panic!("fixture needs a literal supported native quantity"),
        };
        let fuels = FederalFuelLines {
            kerosene_off_highway: Some(FederalFuelUse {
                thousandths_of_gallon,
                actual_cost: 1000,
                qualified: true,
                not_claimed_elsewhere: true,
            }),
            ..Default::default()
        };
        assert_eq!(fuels.credit_cents(), cents);
        fuels
    }
}

pub(in crate::tests) mod financial_omission_scope {
    use crate::tests::*;
    pub(in crate::tests) fn positive_refund() -> FederalDraft {
        injured_return((30000, 1000), (0, 0), 0, allocation())
    }
}

pub(in crate::tests) mod pre_gbc_credit_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(owner: Owner, reverse: bool) -> FederalDraft {
        let mut d = complete(FederalStatus::Joint);
        d.income = vec![
            wage(1, Owner::Taxpayer, 4_000_000, 1_000_000),
            wage(2, Owner::Spouse, 2_000_000, 1_000_000),
        ];
        d.next_id = 3;
        d.credits.as_mut().unwrap().saver = SaverChoice::Yes {
            facts: match owner {
                Owner::Taxpayer => saver(200_000, 0),
                Owner::Spouse => saver(0, 200_000),
            },
        };
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod r8_employee_withholding {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        owner: Owner,
        injured: Owner,
        reversed: bool,
        texas: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        a.community = InjuredCommunityFacts::Yes {
            states: vec![if texas { 7 } else { 5 }],
            debt_facts: if texas {
                InjuredDebtFacts::BeforeMarriage
            } else {
                InjuredDebtFacts::NotApplicable
            },
        };
        if texas {
            a.debt = OffsetDebt::FederalTax;
        }
        let mut d = if owner == Owner::Taxpayer {
            injured_return((0, 0), (22500, 750), 0, a)
        } else {
            injured_return((22500, 750), (0, 0), 0, a)
        };
        let mut e = income(FederalIncome::EmployeeWages {
            data: EmployeeWages {
                wages: 750000,
                withholding: 25000,
                form1099: true,
                reason: EmployeeWageReason::Ss8Determination { date: 20260501 },
                complete: true,
            },
        });
        e.owner = owner;
        e.label.clear();
        add(&mut d, e);
        for e in &mut d.income {
            e.community = CommunityShare::Community;
            if texas {
                e.refund_management = Some(if e.owner == Owner::Taxpayer {
                    RefundManagement::Taxpayer
                } else {
                    RefundManagement::Spouse
                });
            }
        }
        if reversed {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod r8_sale_withholding {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(owner: Owner, injured: Owner, reversed: bool) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        a.community = InjuredCommunityFacts::Yes {
            states: vec![5],
            debt_facts: InjuredDebtFacts::NotApplicable,
        };
        let mut d = if owner == Owner::Taxpayer {
            injured_return((0, 0), (30000, 750), 0, a)
        } else {
            injured_return((30000, 750), (0, 0), 0, a)
        };
        let mut sale = ordinary_investment_sale(10000, 10000, GainTerm::Long);
        let FederalIncome::Sale { data } = &mut sale else { panic!() };
        data.withholding = 25000;
        let mut sale = income(sale);
        sale.owner = owner;
        add(&mut d, sale);
        for entry in &mut d.income {
            entry.community = CommunityShare::Community;
        }
        if reversed {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod r9_actual_clients {
    use crate::tests::support::assert::injured_credit::r9_actual_clients::checked;
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        farm_owner: Owner,
        injured: Owner,
        reversed: bool,
    ) -> FederalDraft {
        let original = farmland_installment_return();
        let baseline = supported(&original);
        assert_eq!(value(&baseline, "agi"), 192935);
        assert_eq!(value(&baseline, "regular_income_tax"), 26978);
        assert_eq!(baseline.tax, Some(4110800));
        let mut draft = original;
        let joint = complete(FederalStatus::Joint);
        draft.people = joint.people;
        draft.setup = joint.setup;
        let FederalIncome::Wage { data } = &mut draft.income[0].income else { panic!() };
        data.withholding = 5_000_000;
        draft.income[1].owner = farm_owner;
        add(&mut draft, wage(0, Owner::Spouse, 2_000_000, 100_000));
        if reversed {
            draft.income.reverse();
        }
        let mut claim = allocation();
        claim.injured = injured;
        draft.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
        draft.injured_spouse = Some(claim);
        checked(&draft, farm_owner, injured);
        draft
    }
}
