use crate::tests::cross::interview::{accepted, document, save_topic};
use crate::tests::*;

pub(in crate::tests) fn capital_history() -> PriorCapitalReturn {
    PriorCapitalReturn {
        injured_niit_owners: None,
        injured_owners: None,

        amt: None,
        origins: vec![],
        owners: PriorReturnOwners::Taxpayer,
        status: FederalStatus::Single,
        agi: 1_000_000,
        deductions: 1_200_000,
        senior_deduction: 0,
        short: -500_000,
        long: -1_000_000,
        deducted_loss: 300_000,
        section_1256: 0,
        ordinary_history: true,
        investment_parts: Some(InvestmentCapitalParts {
            short: 400_000,
            long: 1_000_000,
            amt: None,
        }),
        niit_difference: Some(0),
        research_parts: None,
    }
}

// Ordinary dividend components stay in native cents; specialized boxes remain explicit.
pub(in crate::tests) fn native_dividend(
    ordinary: i64,
    qualified: i64,
    capital_distributions: i64,
) -> FederalDividend {
    FederalDividend {
        ordinary,
        qualified,
        capital_distributions,
        special: crate::DividendSpecial::NoneOfThese,
        ..Default::default()
    }
}

pub(in crate::tests) fn reit_income(ordinary: i64, reit: i64, eligible: bool) -> FederalIncome {
    FederalIncome::Dividend {
        data: FederalDividend {
            ordinary: ordinary * 100,
            section199a: ReitDividends::Reported { amount: reit * 100, eligible },
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
    }
}

// Ordinary interest inputs stay in native cents; the dollar convenience owner delegates here.
pub(in crate::tests) fn native_interest(taxable: i64, early_penalty: i64) -> FederalInterest {
    FederalInterest {
        taxable,
        early_penalty,
        special: InterestSpecial::NoneOfThese,
        ..Default::default()
    }
}

pub(in crate::tests) fn native_exempt_interest(
    exempt: i64,
    private_activity: i64,
) -> FederalInterest {
    FederalInterest {
        exempt,
        private_activity,
        special: InterestSpecial::NoneOfThese,
        ..Default::default()
    }
}

pub(in crate::tests) fn native_treasury_interest(us_obligations: i64) -> FederalInterest {
    FederalInterest { us_obligations, special: InterestSpecial::NoneOfThese, ..Default::default() }
}

pub(in crate::tests) fn savings_income(interest: i64, penalty: i64) -> FederalIncome {
    FederalIncome::Interest { data: native_interest(interest * 100, penalty * 100) }
}

pub(in crate::tests) fn hobby_sales(receipts: i64, purchases: i64, closing: i64) -> FederalIncome {
    FederalIncome::HobbySales {
        data: HobbySales {
            receipts: receipts * 100,
            ordinary: true,
            inventory: BusinessInventory::Cost {
                method: crate::InventoryMethod::Cost,
                opening: 0,
                purchases: purchases * 100,
                personal: 0,
                labor: 0,
                materials: 0,
                other: 0,
                closing: closing * 100,
                eligible: true,
            },
        },
    }
}

pub(in crate::tests) fn personal_sale(proceeds: i64, basis: i64, term: GainTerm) -> FederalIncome {
    FederalIncome::Sale {
        data: FederalSale {
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
            amt: crate::SaleAmt::SameAsRegular,
            collectible: false,
            proceeds: proceeds * 100,
            basis: Some(basis * 100),
            withholding: 0,
            term,
            use_kind: SaleUse::Personal,
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: SaleAdjustment::None,
            basis_reporting: BasisReporting::NoForm1099B,
        },
    }
}

pub(in crate::tests) fn collectible_sale(
    proceeds: i64,
    basis: i64,
    term: GainTerm,
    personal: bool,
) -> FederalIncome {
    let mut result = personal_sale(proceeds, basis, term);
    let FederalIncome::Sale { data } = &mut result else { panic!() };
    data.collectible = true;
    if !personal {
        data.use_kind = SaleUse::Investment { wash_sale: 0 };
    }
    result
}

pub(in crate::tests) fn undistributed_gain() -> FederalIncome {
    FederalIncome::UndistributedGain {
        data: UndistributedGain {
            section_1202_exclusion: None,
            gain: 500000,
            unrecaptured: 0,
            small_business: 0,
            collectibles: 300000,
            tax_paid: 105000,
            special: false,
        },
    }
}

pub(in crate::tests) fn mixed_gain_distribution() -> FederalIncome {
    FederalIncome::Dividend {
        data: FederalDividend {
            capital_distributions: 1000000,
            collectible_distributions: 300000,
            unrecaptured_distributions: 500000,
            special: crate::DividendSpecial::NoneOfThese,
            ..Default::default()
        },
    }
}

pub(in crate::tests) fn capital_return(amount: i64, basis: i64, term: GainTerm) -> FederalIncome {
    FederalIncome::ReturnOfCapital {
        kind: CapitalDistributionKind::Nondividend,
        property: vec![],
        amount: amount * 100,
        basis: basis * 100,
        term,
        special: false,
    }
}

pub(in crate::tests) fn bad_debt(principal: i64, recovered: i64) -> FederalIncome {
    FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt {
            source: BadDebtSource::CashLoan {
                advanced: principal * 100,
                recovered: recovered * 100,
            },
            investment: true,
            bona_fide: true,
            political_debtor: false,
            worthless: true,
        },
    }
}
pub(in crate::tests) fn purchased_bad_debt(face: i64, price: i64, fees: i64) -> FederalIncome {
    FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt {
            source: BadDebtSource::PurchasedNonsecurity {
                face: face * 100,
                price_paid: price * 100,
                acquisition_costs: fees * 100,
                principal_collected: 0,
                acquired: 20240412,
                had_value_when_bought: true,
                security: false,
                basis_adjustments: false,
            },
            investment: true,
            bona_fide: true,
            political_debtor: false,
            worthless: true,
        },
    }
}
pub(in crate::tests) fn contractor_bad_debt(deposited: i64, recovered: i64) -> FederalIncome {
    FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt {
            source: BadDebtSource::ContractorDeposit {
                deposited: deposited * 100,
                recovered: recovered * 100,
                contractor_insolvent: true,
                personal_work: true,
            },
            investment: false,
            bona_fide: true,
            political_debtor: false,
            worthless: true,
        },
    }
}
pub(in crate::tests) fn former_home_mortgage_bad_debt(paid: i64, recovered: i64) -> FederalIncome {
    FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt {
            source: BadDebtSource::FormerHomeMortgage {
                paid: paid * 100,
                recovered: recovered * 100,
                buyer_assumed: true,
                secondary_liability: true,
                foreclosure_shortfall: true,
                no_duplicate_basis_or_loss: true,
            },
            investment: false,
            bona_fide: true,
            political_debtor: false,
            worthless: true,
        },
    }
}

pub(in crate::tests) fn section1244_loss(
    basis: i64,
    proceeds: i64,
    acquired: u32,
) -> FederalIncome {
    FederalIncome::SmallBusinessStock {
        data: Section1244Stock {
            basis: StockLossBasis::Cash { cost: basis * 100, added: 0 },
            acquired,
            event: StockLossEvent::Sale { date: 20261231, proceeds: proceeds * 100 },

            ownership: crate::StockLossOwnership::OriginalIndividual { qualified: true },
        },
    }
}

pub(in crate::tests) fn niit_tax_source_return(election: PersonalTaxElection) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(savings_income(100000, 0)));
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.personal_tax = election;
    facts.investment_property_taxes = 5000000;
    d
}

pub(in crate::tests) fn investment_borrowing(
    d: &mut FederalDraft,
    paid: i64,
    carry: i64,
    amt_carry: i64,
) {
    let deductions = d.deductions.as_mut().unwrap();
    let facts = itemized(deductions).clone();
    deductions.investment_interest = Some(InvestmentBorrowing {
        loans: vec![native_investment_loan(
            "Portfolio borrowing".into(),
            paid * 100,
            carry * 100,
            amt_carry * 100,
        )],
        election: InvestmentElection::None,
        income_use: InvestmentInterestUse::Unresolved,
        limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
    });
    deductions.choice = DeductionSelection::Itemized { facts };
}

pub(in crate::tests) fn elected_portfolio() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: FederalDividend {
                ordinary: 2000000,
                qualified: 2000000,
                capital_distributions: 1000000,
                special: crate::DividendSpecial::NoneOfThese,
                ..Default::default()
            },
        }),
    );
    investment_borrowing(&mut d, 15000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 1500000 };
    d
}
pub(in crate::tests) fn ordinary_investment_sale(
    proceeds: i64,
    basis: i64,
    term: GainTerm,
) -> FederalIncome {
    let mut sale = personal_sale(proceeds, basis, term);
    let FederalIncome::Sale { data } = &mut sale else { panic!() };
    data.use_kind = SaleUse::Investment { wash_sale: 0 };
    sale
}

pub(in crate::tests) fn sale_with_amt(
    mut sale: FederalIncome,
    basis: i64,
    wash_sale: i64,
) -> FederalIncome {
    let FederalIncome::Sale { data } = &mut sale else { panic!() };
    data.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: basis * 100, wash_sale: wash_sale * 100 },
    };
    sale
}

// Common synthetic lot: granted January 1, 2020 and exercised January 1, 2026.
// Prices are native cents per share; callers retain their exact share quantities.
pub(in crate::tests) fn iso_exercise(
    label: &str,
    shares: f64,
    exercise_price: i64,
    market_price: i64,
) -> IsoExercise {
    IsoExercise {
        exercised: 20260101,
        recognition: IsoRecognition::Exercise,
        option_cost: 0,
        owner: Owner::Taxpayer,
        label: label.into(),
        exercise_price,
        market_price,
        shares,
        granted: 20200101,
        corporation: None,
        sold: 0.0,
    }
}

pub(in crate::tests) fn qualifying_iso_sale() -> IsoSale {
    IsoSale {
        option_cost: 0,
        selling_fees: 0,
        in_wages: false,
        granted: 20230103,
        exercised: 20240313,
        sold: 20260421,
        shares: 100.0,
        exercise_price: 10000,
        market_price: 100000,
        acquisition_costs: 0,
        proceeds: 6000000,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        confirmed: true,
    }
}

pub(in crate::tests) fn disqualifying_iso_sale() -> IsoSale {
    let mut sale = qualifying_iso_sale();
    sale.granted = 20250101;
    sale.exercised = 20250601;
    sale.sold = 20260421;
    sale
}

pub(in crate::tests) fn ordinary_espp_lot() -> EsppSale {
    EsppSale {
        granted: 20230101,
        exercised: 20250101,
        sold: 20260201,
        shares: 100.0,
        grant_market: 2200,
        grant_price: 2000,
        exercise_price: 2000,
        market_price: 2300,
        acquisition_costs: 0,
        proceeds: 300000,
        selling_fees: 0,
        in_wages: false,
        wash_sale: 0,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        special: false,
    }
}

pub(in crate::tests) fn qsb_source_sale(
    issuer: &str,
    issued: u32,
    gain: i64,
    reporting: BasisReporting,
) -> QsbIssuerSale {
    QsbIssuerSale {
        issuer: issuer.into(),
        prior_eligible_gain: 0,
        special: false,
        issuer_qualified: true,
        active_business: true,
        no_disqualification: true,
        zone_60_exclusion: Some(false),
        complete_history: true,
        lots: vec![QsbStockLot {
            issued,
            sold: 20260501,
            basis: 1_000_000,
            proceeds: 1_000_000 + gain,
            zone_gain_through_2018: None,
        }],
        reporting,
        withholding: 0,
    }
}

pub(in crate::tests) fn qsb_income(
    issuer: &str,
    basis: i64,
    proceeds: i64,
    prior: i64,
) -> FederalIncomeEntry {
    income(FederalIncome::QualifiedSmallBusinessSale {
        data: QsbIssuerSale {
            issuer: issuer.into(),
            prior_eligible_gain: prior,
            special: false,
            issuer_qualified: true,
            active_business: true,
            no_disqualification: true,
            zone_60_exclusion: None,
            complete_history: true,
            lots: vec![QsbStockLot {
                issued: 20150601,
                sold: 20260601,
                basis,
                proceeds,
                zone_gain_through_2018: None,
            }],
            reporting: BasisReporting::NoForm1099B,
            withholding: 0,
        },
    })
}

pub(in crate::tests) fn direct_tax_credit_bond(
    identifier: &str,
    kind: FederalTaxCreditBondKind,
    box1_credit: i64,
) -> FederalTaxCreditBond1097 {
    FederalTaxCreditBond1097 {
        owner: Owner::Taxpayer,
        issuer_name: alloc::format!("Synthetic issuer {identifier}"),
        issuer_tin: "12-3456789".into(),
        unique_identifier: identifier.into(),
        kind,
        box3_code_199: true,
        issued_on: 20100101,
        box1_credit,
        prior_carry_dollars: 0,
        allowed_credit_dollars: None,
        allowed_credit_confirmed: false,
        prior_carry_confirmed: true,

        origin: crate::TaxCreditOrigin::Direct(crate::TaxCreditDirect {
            directly_held_full_year: true,
            direct_issuer_report: true,
            no_direct_pay_election: true,
            deemed_interest_not_elsewhere: true,
            separate_interest_entered: true,
            ordinary_investment_interest: true,
        }),
    }
}

pub(in crate::tests) fn single_bond_with_same_total_credit(draft: &FederalDraft) -> FederalDraft {
    let mut probe = draft.clone();
    let bonds = &mut probe.credits.as_mut().unwrap().tax_credit_bonds;
    let total_current: i64 = bonds.iter().map(|bond| bond.box1_credit).sum();
    let total_prior: i64 = bonds.iter().map(|bond| bond.prior_carry_dollars).sum();
    bonds[0].box1_credit = total_current;
    bonds[0].prior_carry_dollars = total_prior;
    bonds[0].allowed_credit_dollars = None;
    bonds[0].allowed_credit_confirmed = false;
    bonds.truncate(1);
    probe
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tests) enum K1PortfolioNiitSource {
    Partner,
    Scorp,
    Estate,
}

pub(in crate::tests) fn k1_niit_portfolio_entry(
    source: K1PortfolioNiitSource,
    code_a: bool,
    final_deduction: bool,
) -> FederalIncomeEntry {
    match source {
        K1PortfolioNiitSource::Partner => {
            assert!(!final_deduction);
            let mut data = ordinary_partner_source(0, true);
            data.separate = Some(PartnerSeparateItems {
                interest: 1_000_000,
                dividends: 600_000,
                qualified_dividends: 200_000,
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
            if code_a {
                data.income.push(PartnerIncome {
                    code: PartnerIncomeCode::PortfolioIncome,
                    amount: 200_000,
                });
            }
            income(FederalIncome::Partnership { data })
        }
        K1PortfolioNiitSource::Scorp => {
            assert!(!final_deduction);
            let mut data = complete_ordinary_scorp(0, true);
            data.interest = 1_000_000;
            data.dividends = 600_000;
            data.qualified_dividends = 200_000;
            if code_a {
                data.income
                    .push(ScorpIncome { code: ScorpIncomeCode::PortfolioIncome, amount: 200_000 });
            }
            income(FederalIncome::Scorp { data })
        }
        K1PortfolioNiitSource::Estate => {
            assert!(!code_a);
            let mut donor = estate_trust_code_c_draft(0, 0);
            let mut entry = donor.income.pop().unwrap();
            let FederalIncome::EstateTrustPortfolio { data } = &mut entry.income else {
                unreachable!()
            };
            data.interest = 1_000_000;
            data.ordinary_dividends = 600_000;
            data.qualified_dividends = 200_000;
            if final_deduction {
                data.final_67e_deduction = 200_000;
                data.final_year_successor = true;
                data.final_character = Some(crate::EstateFinalCharacter {
                    entity_terminated: true,
                    final_beneficiary_k1: true,
                    successor: true,
                    section67e_nii: 50_000,
                    section67e_in_h: false,
                    additional: vec![],
                    capital: crate::EstateFinalCapital {
                        alternative_short: 0,
                        alternative_long: 0,
                        rate28: 0,
                        unrecaptured: 0,
                        niit_difference: None,
                        regular_investment: None,
                        alternative_investment: None,
                        regular_business: None,
                        alternative_business: None,
                    },
                });
                assert_eq!(crate::estate_trust::source_rule(data), None);
            }
            entry
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::tests) enum NiitDeductionExpense {
    Foreign,
    Estate,
    ManualForeign,
    Property,
    Portfolio,
    PortfolioCarry,
    PartnerInterest,
    ScorpInterest,
}

pub(in crate::tests) fn niit_deduction_fixture(
    kind: NiitDeductionExpense,
    owner: Owner,
    injured: Owner,
    reversed: bool,
) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    // Independent existing R5 answer, not a source-ownership inference.
    claim.allocations.push(InjuredAmount {
        component: "INJURED_COMPONENT_ITEMIZED".into(),
        injured: if owner == injured { 100_000 } else { 0 },
    });
    let mut d = injured_return((200_000, 60_000), (50_000, 10_000), 0, claim);
    d.deductions.as_mut().unwrap().choice =
        DeductionSelection::Itemized { facts: itemized(d.deductions.as_mut().unwrap()).clone() };
    if matches!(kind, NiitDeductionExpense::Estate) {
        let mut estate = estate_trust_code_c_draft(0, 0).income.pop().unwrap();
        estate.owner = owner;
        let FederalIncome::EstateTrustPortfolio { data } = &mut estate.income else {
            unreachable!()
        };
        data.interest = 1_000_000;
        data.box_10_interest_ird_estate_tax = 100_000;
        data.box_10_interest_ird_only = true;
        add(&mut d, estate);
        let mut other = income(savings_income(20_000, 0));
        other.owner = if owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
        add(&mut d, other);
    } else {
        for (payee, amount) in [(Owner::Taxpayer, 20_000), (Owner::Spouse, 10_000)] {
            let mut entry = income(savings_income(amount, 0));
            entry.owner = payee;
            if matches!(kind, NiitDeductionExpense::Foreign) && payee == owner {
                let FederalIncome::Interest { data } = &mut entry.income else { unreachable!() };
                data.foreign_tax = 100_000;
                data.foreign_country = "CA".into();
            }
            add(&mut d, entry);
        }
    }
    match kind {
        NiitDeductionExpense::Foreign => {
            d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction { taxes: vec![] }
        }
        NiitDeductionExpense::ManualForeign => {
            d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Deduction {
                taxes: vec![ForeignTaxDeducted {
                    owners: None,
                    label: "Actual aggregate foreign investment tax".into(),
                    income: ForeignTaxIncome::Investment,
                    tax: 100_000,
                }],
            }
        }
        NiitDeductionExpense::Property => {
            itemized(d.deductions.as_mut().unwrap()).investment_property_taxes = 100_000
        }
        NiitDeductionExpense::Portfolio | NiitDeductionExpense::PortfolioCarry => {
            d.deductions.as_mut().unwrap().investment_interest = Some(InvestmentBorrowing {
                loans: vec![native_investment_loan(
                    "Actual current portfolio borrowing".into(),
                    if matches!(kind, NiitDeductionExpense::PortfolioCarry) { 0 } else { 100_000 },
                    if matches!(kind, NiitDeductionExpense::PortfolioCarry) { 100_000 } else { 0 },
                    0,
                )],
                election: InvestmentElection::None,
                income_use: InvestmentInterestUse::Ordinary,
                limited_allocation: InvestmentLimitedAllocation::SourceDetermined,
            })
        }
        NiitDeductionExpense::PartnerInterest | NiitDeductionExpense::ScorpInterest => {
            let source = if matches!(kind, NiitDeductionExpense::PartnerInterest) {
                let mut data = ordinary_partner_source(0, true);
                data.deductions.push(PartnerDeduction {
                    code: PartnerDeductionCode::InvestmentInterest,
                    amount: 100_000,
                });
                FederalIncome::Partnership { data }
            } else {
                let mut data = complete_ordinary_scorp(0, true);
                data.deductions.push(ScorpDeduction {
                    code: ScorpDeductionCode::InvestmentInterest,
                    amount: 100_000,
                });
                FederalIncome::Scorp { data }
            };
            let mut entry = income(source);
            entry.owner = owner;
            add(&mut d, entry);
        }
        NiitDeductionExpense::Estate => {}
    }
    if reversed {
        d.income.reverse();
    }
    d
}

pub(in crate::tests) fn niit_binding_fixture(injured: Owner, reversed: bool) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    // Explicit independently chosen accepted R5 answer; never a minimum
    // prefill or an inferred payer/source allocation.
    claim.allocations.push(InjuredAmount {
        component: "INJURED_COMPONENT_ITEMIZED".into(),
        injured: if injured == Owner::Taxpayer { 100_000 } else { 400_000 },
    });
    let mut d = injured_return((200_000, 60_000), (50_000, 10_000), 0, claim);
    for (owner, interest, paid) in [(Owner::Taxpayer, 3000, 2000), (Owner::Spouse, 2000, 10000)] {
        let mut entry = income(savings_income(interest, 0));
        entry.owner = owner;
        add(&mut d, entry);
        let mut entry = income(qualified_dividend(10000));
        entry.owner = owner;
        add(&mut d, entry);
        let mut data = ordinary_partner_source(0, true);
        data.deductions.push(PartnerDeduction {
            code: PartnerDeductionCode::InvestmentInterest,
            amount: paid * 100,
        });
        let mut entry = income(FederalIncome::Partnership { data });
        entry.owner = owner;
        add(&mut d, entry);
    }
    let deductions = d.deductions.as_mut().unwrap();
    deductions.choice = DeductionSelection::Itemized { facts: itemized(deductions).clone() };
    if reversed {
        d.income.reverse();
    }
    d
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tests) enum NiitExpenseSource {
    Paid,
    Carry,
    Property,
    InvestmentTax,
    PassiveTax,
    IncomeTax,
}

impl NiitExpenseSource {
    pub(in crate::tests) fn topic(self) -> TaxSection {
        match self {
            Self::InvestmentTax | Self::PassiveTax => TaxSection::MoreCredits,
            Self::IncomeTax => TaxSection::Taxes,
            _ => TaxSection::OtherItemized,
        }
    }
    pub(in crate::tests) fn niit_deduction_fixture(
        self,
        owner: Owner,
        injured: Owner,
        reversed: bool,
    ) -> FederalDraft {
        let kind = match self {
            Self::Paid => NiitDeductionExpense::Portfolio,
            Self::Carry => NiitDeductionExpense::PortfolioCarry,
            Self::Property | Self::IncomeTax => NiitDeductionExpense::Property,
            Self::InvestmentTax | Self::PassiveTax => NiitDeductionExpense::ManualForeign,
        };
        let mut draft = niit_deduction_fixture(kind, owner, injured, reversed);
        if matches!(self, Self::IncomeTax) {
            let facts = itemized(draft.deductions.as_mut().unwrap());
            facts.investment_property_taxes = 0;
            // Native CurrentReturn allocates paid1000 by gross NII30000 /
            // AGI280000, producing107 Dollars. The unrelated Form4952
            // directly-connected-tax record remains None.
            facts.personal_tax = PersonalTaxElection::Income {
                paid: 100_000,
                allocation: TaxIncomeAllocation::CurrentReturn,
            };
            facts.investment_income_tax = InvestmentIncomeTax::None;
        }
        if matches!(self, Self::PassiveTax) {
            let ForeignTaxClaim::Deduction { taxes } =
                &mut draft.credits.as_mut().unwrap().foreign_tax
            else {
                unreachable!()
            };
            taxes[0].income = ForeignTaxIncome::Passive;
        }
        draft
    }
}

pub(in crate::tests) fn deduction_owner_fact(owner: Owner) -> DeductionOwners {
    if owner == Owner::Taxpayer { DeductionOwners::Taxpayer } else { DeductionOwners::Spouse }
}

pub(in crate::tests) fn niit_zero_fixture(
    paid_cents: i64,
    injured: Owner,
    reversed: bool,
) -> FederalDraft {
    // The same existing wages/interest/ordinary-loan fixture; only its
    // independent R5 answer and the actual fractional loan source differ.
    let mut d =
        niit_deduction_fixture(NiitDeductionExpense::Portfolio, Owner::Taxpayer, injured, false);
    let claim = d.injured_spouse.as_mut().unwrap();
    claim.allocations.clear();
    if paid_cents == 50 {
        // This whole-itemized answer never establishes an unidentified payer.
        claim
            .allocations
            .push(InjuredAmount { component: "INJURED_COMPONENT_ITEMIZED".into(), injured: 0 });
    }
    let loan = &mut d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().loans[0];
    loan.label = "Actual portfolio interest with unidentified individual payer".into();
    loan.paid = paid_cents;
    if reversed {
        d.income.reverse();
    }
    d
}

pub(in crate::tests) fn retained_owner_source() -> PriorCapitalReturn {
    PriorCapitalReturn {
        research_parts: None,
        owners: PriorReturnOwners::Joint,
        status: FederalStatus::Joint,
        agi: 10_000_000,
        deductions: 3_000_000,
        senior_deduction: 0,
        short: -2_300_000,
        long: 0,
        deducted_loss: 300_000,
        section_1256: 1_000_000,
        amt: None,
        origins: vec![],
        ordinary_history: true,
        investment_parts: None,
        niit_difference: Some(1_000_000),
        injured_niit_owners: None,
        injured_owners: Some(InjuredCapitalLossOwners::Shares {
            taxpayer_short: 1_000_000,
            spouse_short: 1_000_000,
            taxpayer_long: 0,
            spouse_long: 0,
        }),
    }
}

pub(in crate::tests) fn add_owner_interest(draft: &mut FederalDraft, dollars: i64) {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut entry = income(savings_income(dollars, 0));
        entry.owner = owner;
        add(draft, entry);
    }
}

fn add_retained_niit_source(session: &mut Interview) {
    let mut prior = retained_owner_source();
    prior.section_1256 = 0;
    prior.injured_niit_owners =
        Some(InjuredRetainedOwners::Both { taxpayer: 400_000, spouse: 600_000 });
    let page = save_topic(
        &session.draft,
        TaxSection::InvestmentSales,
        vec![document(0, IncomeDocument::CapitalLossCarryover(prior.into()))],
    );
    accepted(&send(session, page));
}

pub(in crate::tests) fn retained_niit_return(injured: Owner, reversed: bool) -> FederalDraft {
    let mut a = allocation();
    a.injured = injured;
    a.allocations = vec![
        InjuredAmount {
            component: "additional_medicare_tax".into(),
            injured: if injured == Owner::Taxpayer { 90_000 } else { 45_000 },
        },
        InjuredAmount { component: "capital_loss_limit".into(), injured: -150_000 },
    ];
    let mut session = restart(injured_return((300_000, 60_000), (100_000, 20_000), 0, a));
    add_retained_niit_source(&mut session);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut sale = income(ordinary_investment_sale(6000, 0, GainTerm::Short));
        sale.owner = owner;
        add(&mut session.draft, sale);
        let mut interest = income(savings_income(5000, 0));
        interest.owner = owner;
        add(&mut session.draft, interest);
    }
    if reversed {
        session.draft.income.reverse();
    }
    session.draft
}

pub(in crate::tests) fn add_current_niit_property(
    d: &mut FederalDraft,
    owner: Owner,
    gain: i64,
    passive: bool,
) {
    let FederalIncome::Business { data: mut business } = raised_farm(0, 0) else { panic!() };
    business.expenses.clear();
    let BusinessActivity::Farming { data: farm } = &mut business.activity else { panic!() };
    farm.accounting = FarmAccounting::Cash { receipts: vec![] };
    let mut sale = farm_land_sale(20100616, 20260616);
    sale.basis = 1_000_000;
    sale.proceeds = (10_000 + gain) * 100;
    sale.selling = 0;
    sale.conservation = 0;
    set_land_sales(farm, vec![sale], &mut d.next_id);
    let source = if passive {
        let BusinessActivity::Farming { data: mut farm } = business.activity else { panic!() };
        farm.header = None;
        FederalIncome::FarmRental {
            data: FederalFarmRental {
                farm,
                expenses: business.expenses,
                assets: vec![],
                vehicles: vec![],
                active_participation: true,
                at_risk: Some(crate::RentalAtRisk::All),
                history: RentalHistory::NONE,
                qbi_business: RentalBusiness::No,
                qbi_aggregation_group: None,
                payroll: BusinessPayroll::No,
            },
        }
    } else {
        FederalIncome::Business { data: business }
    };
    let mut entry = income(source);
    entry.owner = owner;
    add(d, entry);
}

fn owner_nua_sale(
    owner: Owner,
    deferred_nua: i64,
    proceeds: i64,
    label: &str,
) -> FederalIncomeEntry {
    let mut entry = income(FederalIncome::NuaSale {
        data: NuaSale {
            inheritance: None,
            distributed: 20250101,
            sold: 20260701,
            distribution_value: 10_000_000,
            deferred_nua,
            proceeds,
            wash_sale: 0,
            withholding: 0,
            reported_basis: None,
            reporting: crate::BasisReporting::ReportedToIrs,
            special: false,
        },
    });
    entry.owner = owner;
    entry.label = label.into();
    entry
}

pub(in crate::tests) fn current_nua_return(
    owner: Owner,
    injured: Owner,
    reversed: bool,
) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    let mut d = injured_return((200_000, 60_000), (50_000, 10_000), 0, claim);
    add_owner_interest(&mut d, 10_000);
    // The existing Pub.575/Form8960 source fixture: $25,000 basis,
    // $75,000 deferred NUA and $20,000 subsequent long-term appreciation.
    // No prior-year NIIT difference or other current exclusion is present.
    let entry =
        owner_nua_sale(owner, 7_500_000, 12_000_000, "Synthetic actual employer-stock NUA sale");
    add(&mut d, entry);
    if reversed {
        d.income.reverse();
    }
    d
}

pub(in crate::tests) fn hobby_niit_return(
    owner: Owner,
    injured: Owner,
    reversed: bool,
) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    let mut d = injured_return((200000, 60000), (50000, 10000), 0, claim);
    add_owner_interest(&mut d, 10_000);
    // Existing retirement/relief source oracle: receipts10000 minus purchases
    //8000 plus closing2000 =4000 annualDollars, no SE or QBI deduction.
    let mut e = income(hobby_sales(10000, 8000, 2000));
    e.owner = owner;
    e.label = "Actual inventory-backed hobby income".into();
    add(&mut d, e);
    if reversed {
        d.income.reverse();
    }
    d
}

pub(in crate::tests) fn mixed_niit_return(
    nua_owner: Owner,
    injured: Owner,
    reversed: bool,
) -> FederalDraft {
    let mut claim = allocation();
    claim.injured = injured;
    claim
        .allocations
        .push(InjuredAmount { component: "capital_loss_limit".into(), injured: -150_000 });
    let mut session = restart(injured_return((200_000, 60_000), (50_000, 10_000), 0, claim));
    add_retained_niit_source(&mut session);
    add_owner_interest(&mut session.draft, 10_000);
    let entry = owner_nua_sale(
        nua_owner,
        500_000,
        10_200_000,
        "Actual 7000 gain with 5000 current NUA exclusion",
    );
    add(&mut session.draft, entry);
    if reversed {
        session.draft.income.reverse();
    }
    session.draft
}

pub(in crate::tests) fn noninvestment_election_fixture(
    injured: Owner,
    reverse: bool,
) -> FederalDraft {
    let mut d = unit13_election_fixture(0, 10_000);
    d.injured_spouse.as_mut().unwrap().injured = injured;
    let screen = d.screening.as_mut().unwrap();
    screen.section1231 = Some(Section1231History { regular: vec![], amt: vec![] });
    // Original accepted earlier-property fixture: 5000 payment x 60% =3000
    // gain. Its explicit Business use is not Form4952 investment property.
    screen.installment_payments = vec![EarlierInstallment {
        owner: Owner::Taxpayer,
        sale: InstallmentPayments {
            property: "Earlier noninvestment business parcel".into(),
            sold: 20230601,
            gross_profit: 600_000,
            contract_price: 1_000_000,
            earlier_payments: 100_000,
            payments: 500_000,
            kind: crate::PropertyCharacter::BusinessProperty {
                used: crate::PropertyUse::Business,
                unrecaptured: 0,
            },
            special: false,
            qbi: InstallmentQbi::NotQualified,
        },
    }];
    if reverse {
        d.income.reverse();
    }
    d
}

pub(in crate::tests) fn unit13_election_fixture(qualified: i64, capital: i64) -> FederalDraft {
    let mut d = injured_return((60_000, 20_000), (20_000, 10_000), 0, allocation());
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: FederalDividend {
                ordinary: qualified * 100,
                qualified: qualified * 100,
                capital_distributions: capital * 100,
                ..Default::default()
            },
        }),
    );
    investment_borrowing(&mut d, 1, 0, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Standard;
    ded.investment_interest.as_mut().unwrap().election =
        InvestmentElection::Allocated { capital: capital * 100, dividends: qualified * 100 };
    d
}

// Actual distinct short source periods and the complete succeeding history.
// Used to cross both retired estate history record ceilings with $1 losses.
pub(in crate::tests) fn retained_estate_nol_periods(history: &mut FinalEstateTrustNol) {
    history.terminated_on = Some(20261231);
    history.taxable_periods = (0..1000)
        .map(|day| {
            let date = crate::dates::add_days(20180101, day);
            FinalEstateTrustTaxPeriod {
                period: FinalEstateTrustLossPeriod { began: date, ended: date },
                accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
            }
        })
        .collect();
    let mut began = crate::dates::add_days(20180101, 1000);
    for year in began / 10000..=2026 {
        history.taxable_periods.push(FinalEstateTrustTaxPeriod {
            period: FinalEstateTrustLossPeriod { began, ended: year * 10000 + 1231 },
            accounting: FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        });
        began = (year + 1) * 10000 + 101;
    }
    history.years = history
        .taxable_periods
        .iter()
        .take(101)
        .map(|source| FinalEstateTrustNolYear {
            year: source.period.began / 10000,
            original_period: Some(source.period.clone()),
            remaining_years: None,
            regular_balance: 100,
            alternative_balance: 100,
        })
        .collect();
}

// Raw source cents; dollars-based helpers keep their separate unit APIs.
pub(in crate::tests) fn native_sale(
    term: GainTerm,
    proceeds: i64,
    basis: Option<i64>,
) -> FederalSale {
    FederalSale {
        term,
        proceeds,
        basis,
        digital_asset: false,
        market_discount: 0,
        several_sales: false,
        amt: crate::SaleAmt::SameAsRegular,
        collectible: false,
        withholding: 0,
        use_kind: SaleUse::Investment { wash_sale: 0 },
        special: crate::SaleSpecial::NoneOfThese,
        adjustment: SaleAdjustment::None,
        basis_reporting: BasisReporting::ReportedToIrs,
    }
}

pub(in crate::tests) fn native_investment_loan(
    label: alloc::string::String,
    paid: i64,
    carry: i64,
    amt_carry: i64,
) -> InvestmentLoan {
    InvestmentLoan {
        label,
        paid,
        carry,
        amt_carry,
        source: 0,
        originating_k1: None,
        paid_owners: None,
        carry_owners: None,
        private_activity: 0,
    }
}
