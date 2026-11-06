pub(in crate::tests) mod alimony_recapture_sources {
    #[derive(Clone, Copy)]
    pub(in crate::tests) enum Case {
        Stopped,
        CurrentAndPaid,
        Paid49Cents,
        Paid50Cents,
    }
}

pub(in crate::tests) mod bonds_cooperative_sources {
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Route {
        Business,
        Farm,
        FarmRental,
    }
}

pub(in crate::tests) mod estate_estimated_payment_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        beneficiary: Owner,
        injured: Owner,
        reverse: bool,
        dated_cents: i64,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        let mut d = injured_return((30_000, 1_000), (20_000, 1_000), 0, a);
        let mut source = estate_trust_code_c_draft(0, 0).income.remove(1);
        source.id = 3;
        source.owner = crate::DocumentOwner::from(beneficiary);
        // The K-1 prints the beneficiary's name, so it is the beneficiary's document.
        if let FederalIncome::EstateTrustPortfolio { data } = &mut source.income {
            data.form.beneficiary.name.clear();
        }
        source.printed = None;
        let person = if beneficiary == Owner::Spouse { -2 } else { -1 };
        crate::tests::support::builders::prepare_income_fixture(
            &mut source,
            crate::tests::support::builders::fixture_document_names(&d, person).map(|mut names| {
                names.set_assignment(crate::document_names::Assignment::matched(
                    crate::DocumentRecipient::from_key(person).unwrap(),
                ));
                names
            }),
        );
        let FederalIncome::EstateTrustPortfolio { data } = &mut source.income else { panic!() };
        {
            let mut record = edit_estate(data);
            record.allocated_estimated_tax = 100_049;
            record.form_1041t_timely = true;
        }
        d.income.push(source);
        d.next_id = 4;
        if dated_cents != 0 {
            let pay = d.payments.as_mut().unwrap();

            crate::tests::support::builders::dated_payments_mut(pay).push(crate::DatedPayment {
                date: 20260415,
                amount: dated_cents,
                payer: if beneficiary == Owner::Taxpayer {
                    PaymentPayer::Spouse
                } else {
                    PaymentPayer::Taxpayer
                },
                refund_property: None,
            });
        }
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod home_employee_niit_sources {
    use crate::tests::support::builders::home_employee::Source;
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        source: Source,
        owner: Owner,
        injured: Owner,
        reversed: bool,
    ) -> FederalDraft {
        crate::tests::support::builders::home_employee::fixture(
            source, owner, injured, reversed, true,
        )
    }
}

pub(in crate::tests) mod k1_activity_niit_sources {
    use crate::tests::*;
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        Partner,
        Scorp,
        Rental,
    }

    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Activity {
        Active,
        Passive,
        RentalCurrent,
        RentalCarry,
    }

    pub(in crate::tests) fn source(source: Source, activity: Activity) -> FederalIncome {
        let ordinary = matches!(activity, Activity::Active | Activity::Passive);
        let material = matches!(activity, Activity::Active);
        let prior = if matches!(activity, Activity::RentalCarry) { 300_000 } else { 0 };
        let rentals = K1RentalAmounts {
            real_estate: if ordinary { 0 } else { 1_000_000 },
            real_estate_carryover: prior,
            ..Default::default()
        };
        match source {
            Source::Partner => {
                let mut data = ordinary_partner_source(if ordinary { 10_000 } else { 0 }, material);
                data.regular.basis.opening = Some(5_000_000);
                data.regular.risk.start =
                    ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
                data.regular.rentals = rentals;
                data.rental_facts.participation = Some(K1RentalParticipation::NotActive);
                if ordinary {
                    data.qbi = Some(PartnerQbiStatement {
                        income: 1_000_000,
                        wages: 0,
                        property: 0,
                        service: false,
                        section1231_qbi: 0,
                        reit_dividends: 0,
                        ptp_income: 0,
                    });
                }
                // Explicit synthetic statement facts: these regular/AMT current
                // amounts and prior balances are equal; no missing AMT view is
                // filled by a financial consumer or inferred after the fact.
                data.alternative = Some(data.regular.clone());
                FederalIncome::Partnership { data }
            }
            Source::Scorp => {
                let mut data = complete_ordinary_scorp(if ordinary { 10_000 } else { 0 }, material);
                data.opening = Some(5_000_000);
                data.risk.start =
                    ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
                data.rentals = rentals;
                data.rental_facts.participation = Some(K1RentalParticipation::NotActive);
                data.amt = Some(amt_records(&data));
                FederalIncome::Scorp { data: scorp_source(data) }
            }
            Source::Rental => {
                assert!(!ordinary);
                // Reuse the retained residential building fixture:275000 MACRS
                // building basis/27.5 years gives actual annual depreciation10000.
                let mut data = rental(20_000, 0);
                // The native helper includes a zero Other expense row; the normal
                // editor requires positive Other rows, so represent zero by none.
                data.expenses.clear();
                data.history =
                    RentalHistory { regular: prior, amt: prior, active_in_loss_year: true };
                FederalIncome::Rental { data }
            }
        }
    }

    pub(in crate::tests) fn fixture(
        source_kind: Source,
        activity: Activity,
        owner: Owner,
    ) -> (FederalDraft, i64) {
        let mut draft = injured_return((200_000, 100_000), (50_000, 30_000), 0, allocation());
        let mut interest = income(savings_income(5000, 0));
        interest.owner = if owner == Owner::Taxpayer {
            crate::DocumentOwner::Spouses
        } else {
            crate::DocumentOwner::Yours
        };
        add(&mut draft, interest);
        let mut entry = income(source(source_kind, activity));
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut draft, entry);
        let id = draft.income.last().unwrap().id;
        (known_fact_partnership_draft(draft), id)
    }
}

pub(in crate::tests) mod ltc_cent_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(shared: bool) -> FederalDraft {
        let mut draft = injured_return((30_000, 100), (0, 0), 0, allocation());
        let periods = [20260601, 20260602].map(|date| CareBenefitPeriod {
            starts: date,
            ends: date,
            received: 43_049,
            services: 0,
            reimbursements: 0,
            recipient: CareBenefitRecipient::Insured { other_payees: false },
            injured_payees: Some(InjuredLtcPeriodPayees::Payments {
                payments: if shared {
                    vec![
                        InjuredLtcPeriodPayment { source: 4, amount: 21_525 },
                        InjuredLtcPeriodPayment { source: 5, amount: 21_524 },
                    ]
                } else {
                    vec![InjuredLtcPeriodPayment { source: 4, amount: 43_049 }]
                },
            }),
        });
        add(
            &mut draft,
            income(ltc_record(LtcPeriods::Several {
                method: CarePeriodMethod::Contract,
                periods: periods.to_vec(),
            })),
        );
        let mut own = ltc_form(0, Some(3));
        let FederalIncome::LtcBenefits { data } = &mut own else { panic!() };
        data.form.benefits = Some(if shared { 43_050 } else { 86_098 });
        add(&mut draft, income(own));
        if shared {
            let mut spouse = income(ltc_form(0, Some(3)));
            spouse.owner = crate::DocumentOwner::Spouses;
            let FederalIncome::LtcBenefits { data } = &mut spouse.income else { panic!() };
            data.form.benefits = Some(43_048);
            add(&mut draft, spouse);
        }
        draft
    }
}

pub(in crate::tests) mod ltc_cent_actual_clients {
    use crate::tests::*;
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(in crate::tests) enum Family {
        Low,
        High,
        Interior,
        Exclusive,
        Zero,
    }

    impl Family {
        pub(in crate::tests) fn shared(self) -> bool {
            matches!(self, Self::Low | Self::High | Self::Interior)
        }
        pub(in crate::tests) fn answers(self, injured: Owner, corrected: bool) -> [i64; 2] {
            let initial = match self {
                Self::Low => [0, 0],
                Self::High => [49, 49],
                Self::Interior => [24, 25],
                _ => panic!(),
            };
            initial.map(|amount| {
                let taxpayer = if corrected { 49 - amount } else { amount };
                if injured == Owner::Taxpayer { taxpayer } else { 49 - taxpayer }
            })
        }
    }

    pub(in crate::tests) fn fixture(family: Family, injured: Owner, reverse: bool) -> FederalDraft {
        let mut d = crate::tests::support::builders::injured_income::ltc_cent_sources::fixture(
            family != Family::Exclusive,
        );
        // 2026-10-15 actual01: the ordinary LTC source editor supplies blank
        // payer/insured labels. Keep this new bridge fixture consistent with that
        // lossless page mapping, rather than borrowing the native helper's generic
        // test-only "Synthetic document" label. Source IDs and all money stay exact.
        for entry in &mut d.income {
            if matches!(
                entry.income,
                FederalIncome::LtcInsured { .. } | FederalIncome::LtcBenefits { .. }
            ) {
                entry.label.clear();
            }
        }
        d.injured_spouse.as_mut().unwrap().injured = injured;
        if family == Family::Zero {
            // Exact independent native1e58 annual-zero facts; no financial engine
            // or inferred source amount is copied into this lifecycle adapter.
            for e in &mut d.income {
                match &mut e.income {
                    FederalIncome::LtcInsured { data } => {
                        data.periods = LtcPeriods::One {
                            starts: 20260601,
                            ends: 20260601,
                            services: 0,
                            reimbursements: 0,
                            recipient: CareBenefitRecipient::Insured { other_payees: false },
                        }
                    }
                    FederalIncome::LtcBenefits { data } => {
                        data.form.benefits = Some(if e.owner == crate::DocumentOwner::Yours {
                            21_525
                        } else {
                            21_524
                        })
                    }
                    _ => {}
                }
            }
        }
        if reverse {
            d.income.reverse();
        }
        d
    }

    pub(in crate::tests) fn opposite(owner: Owner) -> Owner {
        if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer }
    }

    pub(in crate::tests) fn source_rows(d: &FederalDraft) -> (i64, Vec<(i64, Owner)>) {
        let groups: Vec<_> = d
            .income
            .iter()
            .filter(|e| matches!(e.income, FederalIncome::LtcInsured { .. }))
            .collect();
        assert_eq!(groups.len(), 1);
        let group = groups[0].id;
        assert!(group > 0);
        let mut rows: Vec<_> = d
            .income
            .iter()
            .filter_map(|e| {
                if let FederalIncome::LtcBenefits { data } = &e.income {
                    assert_eq!(data.insured, Some(group));
                    Some((e.id, e.owner.owned()))
                } else {
                    None
                }
            })
            .collect();
        rows.sort_unstable_by_key(|row| row.0);
        assert!(rows.iter().all(|row| row.0 > 0));
        (group, rows)
    }
}

pub(in crate::tests) mod medicare_actual_clients {
    use crate::tests::support::builders::additional_medicare::Source;
    use crate::tests::*;
    pub(in crate::tests) fn is_primary(source: Source, entry: &FederalIncomeEntry) -> bool {
        match (&entry.income, source) {
            (FederalIncome::Partnership { .. }, Source::PartnerSe)
            | (FederalIncome::Business { .. }, Source::SmallBusiness) => true,
            (FederalIncome::Wage { data }, Source::Railroad) => {
                matches!(FederalWage::from(data).payroll, W2Payroll::Railroad { .. })
            }
            _ => false,
        }
    }
}

pub(in crate::tests) mod niit_relief_completeness {
    use crate::tests::*;
    pub(in crate::tests) fn source(case: usize) -> (FederalIncome, i64, TaxSection) {
        match case {
            0 | 1 => (
                FederalIncome::HealthLoanAssistance {
                    data: HealthLoanAssistance {
                        program: HealthLoanProgram::National,
                        amount: 500_000,
                        interest: 0,
                        confirmed: case == 0,
                    },
                },
                if case == 0 { 0 } else { 5000 },
                TaxSection::CanceledDebt,
            ),
            2 | 3 => (
                FederalIncome::WildfireRelief {
                    data: FederalWildfireRelief {
                        source_issuer: Default::default(),
                        source_recipient: Default::default(),
                        paid: 20260310,
                        declared: 20250108,
                        reference: "FEMA-FM-5000".into(),
                        amount: 800_000,
                        withholding: 0,
                        qualified: case == 2,
                    },
                },
                if case == 2 { 0 } else { 8000 },
                TaxSection::OtherIncome,
            ),
            4 | 5 => (
                FederalIncome::DisasterRelief {
                    data: FederalDisasterRelief {
                        source_issuer: Default::default(),
                        source_recipient: Default::default(),
                        received: 20260612,
                        qualification: DisasterQualification::FederalDeclaration {
                            reference: "DR-4800".into(),
                        },
                        purpose: DisasterReliefPurpose::PersonalExpense,
                        amount: 800_000,
                        withholding: 0,
                        qualified: case == 4,
                    },
                },
                if case == 4 { 0 } else { 8000 },
                TaxSection::OtherIncome,
            ),
            6 | 7 => (
                FederalIncome::CasualtyLivingInsurance {
                    data: FederalCasualtyLivingInsurance {
                        event_reference: "Kitchen fire".into(),
                        loss_of_use: FederalHomeLossOfUse::Casualty,
                        lost_use_on: 20260601,
                        regained_use_on: Some(20260701),
                        payments: vec![FederalCasualtyLivingPayment {
                            insurer: "Synthetic Mutual".into(),
                            claim_reference: "CLM-1".into(),
                            payment_reference: "PAY-1".into(),
                            received_on: 20260610,
                            amount: 220_000,
                            withholding: 0,
                        }],
                        actual_living_expenses: 385_000,
                        normal_living_expenses: 190_000,
                        prior_taxable_reported: 0,
                        federal_disaster_area: case == 7,
                        federal_declaration_reference: if case == 7 {
                            "DR-4801".into()
                        } else {
                            alloc::string::String::new()
                        },
                        confirmed: true,
                    },
                },
                if case == 6 { 250 } else { 0 },
                TaxSection::OtherIncome,
            ),
            8 => (
                FederalIncome::TrumpDeath {
                    data: trump_death_source(TrumpDeathAcquisition {
                        form_boxes: Default::default(),
                        box_2a: crate::kernel::ratio::positive((150_000) - (100_000)),
                        box_3: 0,
                        box_6: 0,
                        box_8a: 0,
                        box_8b_percent: None,
                        box_9a_percent: None,
                        box_9b: None,
                        box_10: 0,
                        box7: crate::DistributionCodes::CODE_4.0,
                        ira: false,
                        trump: true,
                        beneficiary_estate: false,
                        value: 150_000,
                        basis: Some(100_000),
                        withholding: 0,
                    }),
                },
                500,
                TaxSection::Retirement,
            ),
            _ => unreachable!(),
        }
    }
}

pub(in crate::tests) mod niit_supporting_sources {
    use crate::tests::*;
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Supporting {
        Mitigation,
        AbleRollover,
        PersonalLoss,
        InvestmentGain,
    }

    impl Supporting {
        pub(in crate::tests) fn topic(self) -> TaxSection {
            match self {
                Self::Mitigation => TaxSection::OtherIncome,
                Self::AbleRollover => TaxSection::Retirement,
                Self::PersonalLoss | Self::InvestmentGain => TaxSection::CanceledDebt,
            }
        }

        pub(in crate::tests) fn income(self) -> FederalIncome {
            match self {
                Self::Mitigation => FederalIncome::DisasterMitigation {
                    data: FederalDisasterMitigation {
                        received: 20260415,
                        award_reference: "Synthetic HMGP-12".into(),
                        program: DisasterMitigationProgram::StaffordAct,
                        property: "Synthetic property".into(),
                        amount: 1_500_000,
                        qualified: true,
                    },
                },
                Self::AbleRollover => FederalIncome::TrumpAbleRollover {
                    data: trump_able_source(TrumpAbleRollover {
                        form_boxes: Default::default(),
                        box_2a: 0,
                        box_3: 0,
                        box_6: 0,
                        box_8a: 0,
                        box_8b_percent: None,
                        box_9a_percent: None,
                        box_9b: None,
                        box_10: 0,
                        box7: crate::DistributionCodes::CODE_G.0,
                        ira: false,
                        trump: true,
                        withholding: 0,
                        amount: 200_000,
                        transferred: 20260801,
                    }),
                },
                Self::PersonalLoss | Self::InvestmentGain => FederalIncome::PropertyAcquisition {
                    data: acquisition_source(PropertyAcquisition {
                        description: alloc::string::String::new(),
                        date: 20260801,
                        balance: if matches!(self, Self::PersonalLoss) {
                            850_000
                        } else {
                            5_000_000
                        },
                        value: if matches!(self, Self::PersonalLoss) { 700_000 } else { 3_000_000 },
                        personally_liable: matches!(self, Self::PersonalLoss),
                        property: Some(if matches!(self, Self::PersonalLoss) {
                            TransferredProperty::PersonalUse {
                                basis: 1_200_000,
                                term: GainTerm::Long,
                                collectible: false,
                            }
                        } else {
                            TransferredProperty::Investment {
                                basis: 4_000_000,
                                term: GainTerm::Long,
                                collectible: false,
                            }
                        }),
                        crop: None,
                    }),
                },
            }
        }
    }

    pub(in crate::tests) fn fixture(
        kind: Supporting,
        source_owner: Owner,
        injured: Owner,
        reversed: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        let mut d = injured_return((200_000, 60_000), (50_000, 10_000), 0, a);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut entry = income(savings_income(10_000, 0));
            entry.owner = crate::DocumentOwner::from(owner);
            add(&mut d, entry);
        }
        if matches!(kind, Supporting::AbleRollover) {
            let people = d.people.as_mut().unwrap();
            let beneficiary = if source_owner == Owner::Taxpayer {
                &mut people.taxpayer
            } else {
                people.spouse.as_mut().unwrap()
            };
            // An actual 2026 calendar-year age17 beneficiary, independently of
            // the joint return's earned income and actual rollover source owner.
            beneficiary.birth_date = 20090704;
        }
        let mut entry = income(kind.income());
        // These ordinary supporting editors do not retain a separate row label.
        // Use their native empty label from the beginning; identity and every
        // source field still round-trip exactly.
        entry.label.clear();
        entry.owner = crate::DocumentOwner::from(source_owner);
        add(&mut d, entry);
        if reversed {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod reit_owner_footing {
    use crate::tests::*;
    #[derive(Clone, Copy, Debug)]
    pub(in crate::tests) enum Source {
        Partner,
        Scorp,
        Investment,
        Estate,
    }

    impl Source {
        pub(in crate::tests) fn topic(self) -> TaxSection {
            match self {
                Self::Partner | Self::Scorp | Self::Estate => TaxSection::PassThrough,
                Self::Investment => TaxSection::InterestDividends,
            }
        }
    }

    pub(in crate::tests) fn source(kind: Source) -> FederalIncomeEntry {
        match kind {
            Source::Partner => {
                let mut data = ordinary_partner_source(0, true);
                let mut statement = partner_qbi_statement(0);
                statement.reit_dividends = 1000000;
                data.qbi = Some(statement);
                data.separate = Some(PartnerSeparateTotals {
                    interest: 0,
                    dividends: 1000000,
                    qualified_dividends: 0,
                    royalties: 0,
                    short: 0,
                    long: 0,
                    collectibles: 0,
                    unrecaptured_partnership: 0,
                    unrecaptured_fund: 0,
                    cash_charity: 0,
                    backup_withholding: 0,
                    exempt_interest: 0,
                    foreign_taxes: 0,
                });
                income(FederalIncome::Partnership { data })
            }
            Source::Scorp => {
                let mut data = complete_ordinary_scorp(0, true);
                data.dividends = 1000000;
                data.qbi_statement.as_mut().unwrap().reit_dividends = 1000000;
                income(FederalIncome::Scorp { data: scorp_source(data) })
            }
            Source::Investment => income(reit_income(10000, 10000, true)),
            Source::Estate => {
                let mut donor = estate_trust_code_c_draft(0, 0);
                let mut e = donor.income.pop().unwrap();
                let FederalIncome::EstateTrustPortfolio { data } = &mut e.income else { panic!() };
                data.form.ordinary_dividends = 1000000;
                data.statements.section199a_dividends =
                    ReitDividends::Reported { amount: 1000000, eligible: Some(true) };
                data.statements.code_i_dividends_only = true;
                e
            }
        }
    }

    pub(in crate::tests) fn fixture(
        kind: Source,
        owner: Owner,
        injured: Owner,
        reverse: bool,
    ) -> FederalDraft {
        let mut a = allocation();
        a.injured = injured;
        let mut d = injured_return((160000, 20000), (20000, 8000), 0, a);
        let mut e = source(kind);
        e.owner = crate::DocumentOwner::from(owner);
        e.label.clear();
        add(&mut d, e);
        if reverse {
            d.income.reverse();
        }
        d
    }
}

pub(in crate::tests) mod estate_carry_sources {
    use crate::tests::*;
    pub(in crate::tests) fn fixture(
        injured: Owner,
        carries: &[(Owner, i64, Option<i64>)],
    ) -> FederalDraft {
        let mut claim = allocation();
        claim.injured = injured;
        // State the independent permitted payroll-tax allocation. The
        // fixture's $400000 Medicare wages produce $1350 joint Form8959 tax;
        // the user assigns that pool to the taxpayer, whose wages are $300000.
        claim.allocations.push(InjuredAmount {
            component: "additional_medicare_tax".into(),
            injured: if injured == Owner::Taxpayer { 135_000 } else { 0 },
        });
        let mut draft = injured_return((300_000, 60_000), (100_000, 20_000), 0, claim);
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut sale = income(ordinary_investment_sale(30_000, 0, GainTerm::Short));
            sale.owner = crate::DocumentOwner::from(owner);
            add(&mut draft, sale);
        }
        for &(owner, carry, difference) in carries {
            let mut data = empty_estate_portfolio();
            data.final_short_carry = carry;
            data.final_k1 = true;
            data.final_year_successor = Some(true);
            // These are the final-year shortcut confirmations written by
            // the ordinary K1EstateTrust editor, even for a zero tax amount.
            data.final_67e_no_niit_allocation = true;
            data.final_taxes_only = true;
            data.final_carry_same_tax_character = true;
            data.final_character = Some(crate::EstateFinalCharacter {
                entity_terminated: true,

                section67e_nii: 0,
                section67e_in_h: false,
                additional: vec![],
                capital: crate::EstateFinalCapital {
                    alternative_short: carry,
                    alternative_long: 0,
                    rate28: 0,
                    unrecaptured: 0,
                    niit_difference: difference,
                    regular_investment: Some(crate::EstateCapitalLossParts {
                        short: carry,
                        long: 0,
                    }),
                    alternative_investment: Some(crate::EstateCapitalLossParts {
                        short: carry,
                        long: 0,
                    }),
                    regular_business: None,
                    alternative_business: None,
                },
            });
            let mut entry =
                income(FederalIncome::EstateTrustPortfolio { data: estate_source(data) });
            entry.owner = crate::DocumentOwner::from(owner);
            add(&mut draft, entry);
        }
        draft
    }

    pub(in crate::tests) fn long_collectible(injured: Owner) -> FederalDraft {
        let mut draft = fixture(injured, &[(Owner::Spouse, 1_000_000, Some(0))]);
        let estate = draft
            .income
            .iter_mut()
            .find_map(|entry| match &mut entry.income {
                FederalIncome::EstateTrustPortfolio { data } => Some(data),
                _ => None,
            })
            .unwrap();
        {
            let mut record = edit_estate(estate);
            record.final_short_carry = 0;
            record.final_long_carry = 1_000_000;
        }
        let character = &mut estate.statements.final_character.as_mut().unwrap().capital;
        character.alternative_short = 0;
        character.alternative_long = 1_000_000;
        character.regular_investment =
            Some(crate::EstateCapitalLossParts { short: 0, long: 1_000_000 });
        character.alternative_investment = character.regular_investment.clone();
        let mut ordinary = income(ordinary_investment_sale(10_000, 0, GainTerm::Long));
        ordinary.owner = crate::DocumentOwner::Yours;
        add(&mut draft, ordinary);
        let mut collectible = income(collectible_sale(15_000, 0, GainTerm::Long, false));
        collectible.owner = crate::DocumentOwner::Spouses;
        add(&mut draft, collectible);
        draft
    }
}
