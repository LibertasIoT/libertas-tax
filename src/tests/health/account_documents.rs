// Health, long-term care and ABLE account documents (unit B6a, 2026-10-09): the fixtures F1-F13 of
// the map (the Forms 1099-R ones are the B5 unit's) and the boundaries the stored kinds were tested
// with before they became documents. Amounts in the comments are dollars; the records carry cents.
// Unless stated: single filer born 1980-06-15 with $60,000 of wages (the `complete` household), one
// document, and ids from 2 (the W-2 is 1).
use super::*;
use crate::kernel::ratio::exact::{self, whole};
use crate::tests::cross::interview::{
    PAGE_ADJUSTMENTS, accepted, hsa_account, open_hsa_page, open_topic, page_documents, save_topic,
};
use crate::tests::support::client::survey::survey_preview_document;

fn dollars(v: i64) -> i64 {
    v * 100
}
fn born(d: &mut FederalDraft, birth: u32) {
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
}
fn own_payment(gross: i64, paid: u32, medical: i64) -> MedicalSaPayment {
    MedicalSaPayment::Own {
        nonmedical: if gross == medical {
            vec![]
        } else {
            vec![MedicalAccountWithdrawal { date: paid, amount: dollars(gross - medical) }]
        },
        medical: dollars(medical),
        rollover: None,
        disabled_since: None,
    }
}
// A Form 1099-SA in dollars.
fn sa(gross: i64, account: MedicalSaAccount, payment: MedicalSaPayment) -> FederalIncome {
    FederalIncome::MedicalDistribution {
        data: FederalMedicalDistribution {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(gross),
            account,
            payment,
        },
    }
}
fn archer_form(gross: i64, paid: u32, medical: i64) -> FederalIncome {
    sa(gross, MedicalSaAccount::Archer, own_payment(gross, paid, medical))
}
fn ma_account(balance: i64, deductible: i64) -> MedicalSaAccount {
    MedicalSaAccount::MedicareAdvantage {
        prior: MedicareMsaPrior::Existing {
            balance: dollars(balance),
            deductible: dollars(deductible),
        },
    }
}
fn ma_none() -> MedicalSaAccount {
    MedicalSaAccount::MedicareAdvantage { prior: MedicareMsaPrior::None }
}
fn bill(incurred: u32, paid: u32, amount: i64) -> InheritedMedicalPayment {
    InheritedMedicalPayment { incurred, paid, amount: dollars(amount) }
}
fn inherited_payment(
    died: u32,
    value: i64,
    payments: Vec<InheritedMedicalPayment>,
) -> MedicalSaPayment {
    MedicalSaPayment::Inherited {
        died,
        value: dollars(value),
        beneficiary: MedicalBeneficiary::Nonspouse { payments },
    }
}
fn kinds() -> [MedicalSaAccount; 3] {
    [MedicalSaAccount::Hsa, MedicalSaAccount::Archer, ma_none()]
}
fn die(d: &mut FederalDraft, date: u32) {
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.died = death(date);
    p.died.as_mut().unwrap().ordinary = true;
}
fn issue(d: &FederalDraft, key: &str) -> bool {
    let r = estimate(d);
    r.state == ResultState::Unsupported && r.issues.iter().any(|i| i.display.key == key)
}
// The same income as a prize, for the comparison of what the document adds to the return.
fn as_prize(d: &FederalDraft, index: usize, cash_dollars: i64) -> FederalResult {
    let mut reference = d.clone();
    reference.income[index].income =
        prize_income(dollars(cash_dollars), 0, 0, PrizeDisposition::Received);
    supported(&reference)
}

// ---- Form 1099-SA: an Archer MSA (Form 8853 Section A) ----

// F1. Born 1964-03-10 (62). Box 1 3,000, spent on medical costs 1,800. Line 6a 3,000, 7 1,800, line
// 8 = 3,000 - 1,800 = 1,200 (Schedule 1 line 8e); not 65 and not disabled, so line 9b = 20% x 1,200
// = 240 (Schedule 2). F1b: disabled since 2026-03-01, before the 2026-06-15 payment: no 20% tax.
#[test]
fn f1_archer_msa_partly_medical() {
    let mut d = complete(FederalStatus::Single);
    born(&mut d, 19640310);
    add(&mut d, income(archer_form(3000, 20260615, 1800)));
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_2_gross"), 3000);
    assert_eq!(value(&r, "archer_msa_2_medical"), 1800);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 1200);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 240);
    assert_eq!(return_line(&r, "8"), 120_000);
    assert_eq!(return_line(&r, "23"), 24_000);
    assert_eq!(value(&r, "agi"), 61_200);
    let expected = as_prize(&d, 1, 1200);
    assert_eq!(r.tax.unwrap(), expected.tax.unwrap() + 24_000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::MedicalDistribution { data } = &mut d.income[1].income else { panic!() };
    let MedicalSaPayment::Own { disabled_since, .. } = &mut data.payment else { panic!() };
    *disabled_since = Some(20260301);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 1200);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 0);
}

// F2. Born 1961-06-15: the 65th birthday is 2026-06-15. Form A (paid 06-15) and form B (paid 06-16),
// 1,000 each, nothing spent on medical costs: line 8 = 2,000; A is on the birthday and taxed, B after
// it, so line 9b = 20% x 1,000 = 200.
#[test]
fn f2_the_65th_birthday_is_taxed_and_the_next_day_is_not() {
    let mut d = complete(FederalStatus::Single);
    born(&mut d, 19610615);
    add(&mut d, income(archer_form(1000, 20260615, 0)));
    add(&mut d, income(archer_form(1000, 20260616, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 2000);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 200);
    assert_eq!(value(&r, "archer_msa_2_penalty_base"), 1000);
    assert_eq!(value(&r, "archer_msa_3_penalty_base"), 0);
}

// The age and disability dates by the form's own amounts: the 65th birthday and the disability
// onset are taxed on the day and exempt the day after; an onset before birth is refused.
#[test]
fn archer_msa_age_and_disability_exceptions_start_after_the_event_date() {
    let form = |paid, disabled| FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(1000),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Own {
            nonmedical: vec![MedicalAccountWithdrawal { date: paid, amount: dollars(1000) }],
            medical: 0,
            rollover: None,
            disabled_since: disabled,
        },
    };
    let mut p = person();
    p.birth_date = 19610615;
    let base = |paid, p: &FederalPerson| {
        exact::rounded(&form(paid, None).amounts(p).unwrap().penalized).unwrap()
    };
    assert_eq!(base(20260614, &p), dollars(1000));
    assert_eq!(base(20260615, &p), dollars(1000));
    assert_eq!(base(20260616, &p), 0);
    p.birth_date = 19800101;
    assert_eq!(base(20260616, &p), dollars(1000));
    let disabled = |onset, paid| {
        exact::rounded(&form(paid, Some(onset)).amounts(&p).unwrap().penalized).unwrap()
    };
    assert_eq!(disabled(20260615, 20260615), dollars(1000));
    assert_eq!(disabled(20260614, 20260615), 0);
    assert_eq!(disabled(20250101, 20260615), 0);
    assert_eq!(
        form(20260615, Some(19700101)).amounts(&p).unwrap_err().0,
        "ISSUE_ARCHER_MSA_DISABILITY_BEFORE_BIRTH"
    );
}

// Spouses' bases are added before the 20% rate: 3 + 3 = 6, tax 1 (not two rounded 1s).
#[test]
fn archer_msa_spouses_combine_tax_base_before_rounding_rate() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(archer_msa(3)));
    let mut spouse = income(archer_msa(3));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 6);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 1);
    assert_eq!(return_line(&r, "23"), 100);
}

// Medical use and a rollover are neither income nor deducted again: 10,000 paid, 2,000 spent on
// medical costs, 5,000 rolled over within 60 days leave 3,000 taxed, with the 20% tax.
#[test]
fn archer_msa_medical_and_rollover_exclusions_preserve_nonmedical_tax() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(sa(
            10_000,
            MedicalSaAccount::Archer,
            MedicalSaPayment::Own {
                nonmedical: vec![MedicalAccountWithdrawal {
                    date: 20260615,
                    amount: dollars(3000),
                }],
                medical: dollars(2000),
                rollover: Some(archer_rollover()),
                disabled_since: None,
            },
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_2_gross"), 10_000);
    assert_eq!(value(&r, "archer_msa_2_rollover"), 5000);
    assert_eq!(value(&r, "archer_msa_2_medical"), 2000);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 3000);
    assert_eq!(return_line(&r, "8"), 300_000);
    assert_eq!(return_line(&r, "23"), 60_000);
    let expected = as_prize(&d, 1, 3000);
    assert_eq!(r.tax.unwrap(), expected.tax.unwrap() + 60_000);
    for id in ["agi", "deduction", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(restart(d.clone()).draft, d);
    // Itemized medical costs stay as they were: the medical use is not deducted again.
    let mut itemizing = complete(FederalStatus::Single);
    itemized(itemizing.deductions.as_mut().unwrap()).medical = 1_000_000;
    let before = supported(&itemizing);
    let mut data = archer_msa(0);
    if let FederalIncome::MedicalDistribution { data } = &mut data
        && let MedicalSaPayment::Own { rollover, .. } = &mut data.payment
    {
        data.gross += dollars(5000);
        *rollover = Some(archer_rollover());
    }
    add(&mut itemizing, income(data));
    let r = supported(&itemizing);
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "deduction"), value(&before, "deduction"));
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 0);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 0);
}

// The rollover must be deposited within 60 days of the payment and be the only one from the source
// type in 12 months (section 220(f)(5)): an ineligible one is refused, not taxed silently.
#[test]
fn archer_msa_rollover_enforces_sixty_days_and_preceding_year() {
    let form = |rollover: MedicalRollover| FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(5000),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 0,
            rollover: Some(rollover),
            disabled_since: None,
        },
    };
    let mut v = archer_rollover();
    assert!(form(v.clone()).amounts(&person()).is_ok());
    v.receipts[0].deposits[0].deposited = 20260303;
    assert!(form(v.clone()).amounts(&person()).is_err());
    v.receipts[0].deposits[0].deposited = 20260302;
    v.preceding = Some(20250101);
    assert!(form(v.clone()).amounts(&person()).is_ok());
    v.preceding = Some(20250102);
    assert!(form(v.clone()).amounts(&person()).is_err());
    // A December payment may be deposited in the next year: 60 days from 2026-12-31 is 2027-03-01.
    v.receipts[0].received = 20261231;
    v.receipts[0].deposits[0].deposited = 20270301;
    v.preceding = None;
    assert!(form(v.clone()).valid());
    assert!(form(v.clone()).amounts(&person()).is_ok());
    v.receipts[0].deposits[0].deposited = 20270302;
    assert!(form(v.clone()).amounts(&person()).is_err());
    // A deposit before the receipt is not a rollover.
    v.receipts[0].deposits[0].deposited = 20261230;
    assert!(!form(v).valid());
    let mut d = complete(FederalStatus::Single);
    let mut late = archer_rollover();
    late.receipts[0].deposits[0].deposited = 20260303;
    add(
        &mut d,
        income(sa(
            5000,
            MedicalSaAccount::Archer,
            MedicalSaPayment::Own {
                nonmedical: vec![],
                medical: 0,
                rollover: Some(late),
                disabled_since: None,
            },
        )),
    );
    assert!(issue(&d, "ISSUE_ARCHER_MSA_TREATMENT_UNCONFIRMED"));
}

// An Archer MSA lives beside an HSA and the inherited accounts of the same return.
#[test]
fn archer_msa_can_coexist_with_hsa_and_inherited_death_values() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa.push(hsa_entry(
        Owner::Taxpayer,
        HsaCoverage::SelfOnly,
        100_000,
    ));
    add(&mut d, income(archer_msa(5000)));
    add(&mut d, income(medical_inheritance(MedicalSaAccount::Archer)));
    add(&mut d, income(medical_inheritance(MedicalSaAccount::Hsa)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 140_000); // $141,000 of income less the $1,000 HSA deduction.
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 43_000);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 38_000);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 1000);
    assert_eq!(return_line(&r, "8"), 8_100_000);
    assert_eq!(return_line(&r, "23"), 100_000);
}

// A prohibited transaction distributes the January 1 value on January 1: the 20% tax applies unless
// the holder was already past 65 (born 1960-12-31 turned 65 on 2025-12-31; born 1961-01-01 turns 65
// on 2026-01-01, still taxed).
#[test]
fn archer_prohibited_transaction_uses_the_january_value_and_date() {
    let form = FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(12_345),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Prohibited { occurred: 20261231 },
    };
    let mut p = person();
    p.birth_date = 19610615;
    let a = form.amounts(&p).unwrap();
    assert_eq!((a.taxable, a.penalized), (whole(dollars(12_345)), whole(dollars(12_345))));
    p.birth_date = 19601231;
    assert_eq!(form.amounts(&p).unwrap().penalized, whole(0));
    p.birth_date = 19610101;
    assert_eq!(form.amounts(&p).unwrap().penalized, whole(dollars(12_345)));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::MedicalDistribution { data: form }));
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_2_deemed"), 12_345);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 12_345);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 2469);
    assert_eq!(return_line(&r, "23"), 246_900);
    // A transaction after the holder died belongs to the beneficiary's treatment.
    die(&mut d, 20260615);
    assert!(issue(&d, "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY"));
}

// F7. Box 1 1,150 of an Archer excess returned (code 2), box 2 earnings 150: only the earnings are
// income (Schedule 1 line 8z), with no Section A line 8 amount and no 20% tax.
#[test]
fn f7_archer_excess_returned() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(sa(
            1150,
            MedicalSaAccount::Archer,
            MedicalSaPayment::ExcessReturned { earnings: Some(dollars(150)) },
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "archer_msa_2_excess_earnings"), 150);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 0);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 0);
    assert_eq!(return_line(&r, "8"), 15_000);
    assert_eq!(value(&r, "agi"), 60_150);
    // HSA code2 is linked to its corrective contribution facts; printed earnings cannot exceed box1.
    let mut v = FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(100),
        account: MedicalSaAccount::Hsa,
        payment: MedicalSaPayment::ExcessReturned { earnings: Some(dollars(10)) },
    };
    assert_eq!(v.rule(), None);
    v.account = MedicalSaAccount::Archer;
    assert_eq!(v.rule(), None);
    v.payment = MedicalSaPayment::ExcessReturned { earnings: Some(dollars(101)) };
    assert_eq!(v.rule(), Some("SA1099_EARNINGS_ABOVE_GROSS"));
    v.account = ma_none();
    assert_eq!(v.rule(), Some("SA1099_EXCESS_NOT_ARCHER"));
}

// ---- Form 1099-SA: a Medicare Advantage MSA (Form 8853 Section B) ----

// F3. Born 1955-02-01 (71; age gives no exception). Box 1 5,000, spent on medical costs 2,000, the
// December 31, 2025 balance 4,000 and the January 1 deductible 5,000. Lines 10 5,000, 11 2,000, 12
// 3,000 (Schedule 1 line 8e). Worksheet: 3,000 taxed, relief 4,000 - 60% x 5,000 = 1,000, so 13b =
// 50% x (3,000 - 1,000) = 1,000. F3b: no balance at the end of 2025: 13b = 50% x 3,000 = 1,500.
#[test]
fn f3_medicare_advantage_msa_with_a_prior_balance() {
    let mut d = complete(FederalStatus::Single);
    born(&mut d, 19550201);
    add(&mut d, income(sa(5000, ma_account(4000, 5000), own_payment(5000, 20260901, 2000))));
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 3000);
    assert_eq!(value(&r, "medicare_msa_2_balance_relief"), 1000);
    assert_eq!(value(&r, "medicare_msa_2_exception"), 0);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 1000);
    assert_eq!(return_line(&r, "23"), 100_000);
    let mut again = d.clone();
    again.income[1].income = sa(5000, ma_none(), own_payment(5000, 20260901, 2000));
    assert_eq!(value(&supported(&again), "medicare_msa_additional_tax"), 1500);
}

// The balance relief by the account's two figures: none, partial, whole and none again.
#[test]
fn medicare_msa_prior_balance_relief_and_new_account_tax() {
    for (prior, relief, tax) in [
        (MedicareMsaPrior::None, 0, 2500),
        (
            MedicareMsaPrior::Existing { balance: dollars(10_000), deductible: dollars(10_000) },
            4000,
            500,
        ),
        (
            MedicareMsaPrior::Existing { balance: dollars(20_000), deductible: dollars(10_000) },
            14_000,
            0,
        ),
        (
            MedicareMsaPrior::Existing { balance: dollars(2000), deductible: dollars(10_000) },
            0,
            2500,
        ),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(medicare_msa(5000, prior)));
        let r = supported(&d);
        assert_eq!(value(&r, "medicare_msa_2_balance_relief"), relief);
        assert_eq!(value(&r, "medicare_msa_additional_tax"), tax);
        assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 5000);
    }
}

// Only the disability and the death exempt a Medicare Advantage MSA payment, on the date: age 65
// does not (born 1950).
#[test]
fn medicare_msa_disability_and_death_apply_on_date_but_age_does_not() {
    let mut p = person();
    p.birth_date = 19500101;
    let form = |paid, disabled| FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(2000),
        account: ma_none(),
        payment: MedicalSaPayment::Own {
            nonmedical: vec![MedicalAccountWithdrawal { date: paid, amount: dollars(2000) }],
            medical: 0,
            rollover: None,
            disabled_since: disabled,
        },
    };
    let penalized = |p: &FederalPerson, paid, disabled| {
        exact::rounded(&form(paid, disabled).amounts(p).unwrap().penalized).unwrap()
    };
    for paid in [20260614, 20260615, 20260616] {
        assert_eq!(penalized(&p, paid, None), dollars(2000));
    }
    assert_eq!(penalized(&p, 20260614, Some(20260615)), dollars(2000));
    assert_eq!(penalized(&p, 20260615, Some(20260615)), 0);
    assert_eq!(penalized(&p, 20260616, Some(20260615)), 0);
    assert_eq!(penalized(&p, 20260615, Some(20250101)), 0);
    p.died = death(20260615);
    assert_eq!(penalized(&p, 20260614, None), dollars(2000));
    assert_eq!(penalized(&p, 20260615, None), 0);
}

// The 50% tax follows the money to the return: line 8 income, line 23 tax, nothing else changes.
#[test]
fn medicare_msa_tax_flows_to_agi_return_income_and_additional_tax() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(medicare_msa(
            5000,
            MedicareMsaPrior::Existing { balance: dollars(10_000), deductible: dollars(10_000) },
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 65_000);
    assert_eq!(return_line(&r, "8"), 500_000);
    assert_eq!(return_line(&r, "23"), 50_000);
    assert_eq!(value(&r, "medicare_msa_2_balance_relief"), 4000);
    let expected = as_prize(&d, 1, 5000);
    assert_eq!(r.tax.unwrap(), expected.tax.unwrap() + 50_000);
    for id in ["agi", "deduction", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(restart(d.clone()).draft, d);
}

// The balance relief is each spouse's; two forms of one spouse share it once and must agree on it.
#[test]
fn medicare_msa_spouses_do_not_share_balance_relief() {
    let mut d = complete(FederalStatus::Joint);
    add(
        &mut d,
        income(medicare_msa(
            5000,
            MedicareMsaPrior::Existing { balance: dollars(20_000), deductible: dollars(10_000) },
        )),
    );
    let mut spouse = income(medicare_msa(5000, MedicareMsaPrior::None));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 10_000);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 2500);
    assert_eq!(return_line(&r, "23"), 250_000);
    // A second form of the taxpayer with the same account value: the relief is not taken twice.
    let again = d.income[1].clone();
    add(&mut d, again);
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 15_000);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 2500);
    // ... and a different account value on it is refused.
    d.income[3].income = medicare_msa(5000, MedicareMsaPrior::None);
    assert!(issue(&d, "ISSUE_MEDICARE_MSA_PRIOR_DIFFERS"));
}

// Cents are added across the spouse's forms before the whole-dollar lines round.
#[test]
fn medicare_msa_rounds_form_totals_after_aggregating_payments() {
    let cents = |gross, medical, paid| {
        income(FederalIncome::MedicalDistribution {
            data: FederalMedicalDistribution {
                origin: crate::HealthDocumentOrigin::Form1099Sa,
                gross,
                account: ma_none(),
                payment: MedicalSaPayment::Own {
                    nonmedical: vec![MedicalAccountWithdrawal {
                        date: paid,
                        amount: gross - medical,
                    }],
                    medical,
                    rollover: None,
                    disabled_since: None,
                },
            },
        })
    };
    let mut d = complete(FederalStatus::Single);
    add(&mut d, cents(200_098, 200_000, 20260101));
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 1);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 1);
    // Two 49-cent forms: 98 cents round to one dollar together, not to zero each.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, cents(49, 0, 20260101));
    add(&mut d, cents(49, 0, 20260102));
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 1);
    // Spent entirely on medical costs.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, cents(200_098, 200_098, 20260101));
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 0);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 0);
}

// Medical use of a Medicare Advantage MSA is neither income nor an itemized deduction.
#[test]
fn medicare_msa_medical_use_is_neither_income_nor_an_itemized_deduction() {
    let mut d = complete(FederalStatus::Single);
    itemized(d.deductions.as_mut().unwrap()).medical = 1_000_000;
    let before = supported(&d);
    add(&mut d, income(sa(300_000, ma_none(), own_payment(300_000, 20260615, 300_000))));
    let r = supported(&d);
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "deduction"), value(&before, "deduction"));
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 0);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 0);
}

// The payout is not investment income; a final return refuses a payment after the death.
#[test]
fn medicare_msa_is_not_investment_income_and_final_return_rejects_postdeath_payments() {
    let mut d = complete(FederalStatus::Single);
    let disabled = FederalIncome::MedicalDistribution {
        data: FederalMedicalDistribution {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(302_000),
            account: ma_none(),
            payment: MedicalSaPayment::Own {
                nonmedical: vec![MedicalAccountWithdrawal {
                    date: 20260615,
                    amount: dollars(300_000),
                }],
                medical: dollars(2000),
                rollover: None,
                disabled_since: Some(20260101),
            },
        },
    };
    add(&mut d, income(disabled));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 360_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    d.income[1].income = sa(302_000, ma_none(), own_payment(302_000, 20260615, 2000));
    die(&mut d, 20260615);
    assert_eq!(value(&supported(&d), "medicare_msa_additional_tax"), 0);
    d.income[1].income = sa(302_000, ma_none(), own_payment(302_000, 20260616, 2000));
    assert!(issue(&d, "ISSUE_ARCHER_MSA_POST_DEATH_ACTIVITY"));
}

// A Medicare Advantage MSA's prohibited transaction: the January 1 value is distributed on January 1
// whatever the age, with the balance relief once (relief 10,000 - 60% x 10,000 = 4,000; 10,000 - 4,000
// taxed at 50% = 3,000); a disability on or before January 1 or a death before it exempts it.
#[test]
fn medicare_prohibited_transaction_uses_january_first_and_prior_balance_relief() {
    let mut d = complete(FederalStatus::Single);
    born(&mut d, 19400615);
    add(
        &mut d,
        income(sa(
            10_000,
            ma_account(10_000, 10_000),
            MedicalSaPayment::Prohibited { occurred: 20260105 },
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "medicare_msa_2_deemed"), 10_000);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 10_000);
    assert_eq!(value(&r, "medicare_msa_2_balance_relief"), 4000);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 3000);
    // A later death in 2026 does not exempt the January 1 distribution.
    die(&mut d, 20260615);
    assert_eq!(value(&supported(&d), "medicare_msa_additional_tax"), 3000);
}

// ---- Form 1099-SA: an account inherited (Form 8889 lines 14a-16, Form 8853) ----

// F4. A nonspouse beneficiary of an HSA: box 1 20,400, box 4 value 20,000 at the death on 2026-03-20,
// and 1,500 of the holder's bills paid on 2026-05-10. Form 8889 line 14a 20,000, 15 1,500, 16 =
// 18,500 (Schedule 1 line 8f); earnings after the death 20,400 - 20,000 = 400 (line 8z). F4b: the
// bill paid on 2027-03-21, a year and a day after the death, does not count: 16 = 20,000.
#[test]
fn f4_inherited_hsa_with_bills_and_earnings() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(sa(
            20_400,
            MedicalSaAccount::Hsa,
            inherited_payment(20260320, 20_000, vec![bill(20260301, 20260510, 1500)]),
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_medical_2_value"), 20_000);
    assert_eq!(value(&r, "inherited_medical_2_medical"), 1500);
    assert_eq!(value(&r, "inherited_medical_2_taxable"), 18_500);
    assert_eq!(value(&r, "inherited_medical_2_earnings"), 400);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 18_500);
    assert_eq!(value(&r, "inherited_medical_earnings"), 400);
    assert_eq!(return_line(&r, "8"), 1_890_000);
    assert_eq!(value(&r, "agi"), 60_000 + 18_900);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    let mut late = d.clone();
    late.income[1].income = sa(
        20_400,
        MedicalSaAccount::Hsa,
        inherited_payment(20260320, 20_000, vec![bill(20260301, 20270321, 1500)]),
    );
    let r = supported(&late);
    assert_eq!(value(&r, "inherited_medical_2_taxable"), 20_000);
    // A bill paid the last day of the year counts.
    late.income[1].income = sa(
        20_400,
        MedicalSaAccount::Hsa,
        inherited_payment(20260320, 20_000, vec![bill(20260301, 20270320, 1500)]),
    );
    assert_eq!(value(&supported(&late), "inherited_medical_2_taxable"), 18_500);
}

// F5. The estate inherits and this is the holder's final return (died 2026-05-05): the whole box 4
// value 8,000 is income of the final return with no reduction for bills and no 9b tax. F5b: the
// holder alive (People shows no death), or another date, is refused.
#[test]
fn f5_inherited_archer_msa_to_the_estate_on_the_final_return() {
    let mut d = complete(FederalStatus::Single);
    let estate = |died| MedicalSaPayment::Inherited {
        died,
        value: dollars(8000),
        beneficiary: MedicalBeneficiary::Estate,
    };
    add(&mut d, income(sa(8000, MedicalSaAccount::Archer, estate(20260505))));
    assert!(issue(&d, "ISSUE_INHERITED_MEDICAL_ESTATE_DATE_MISMATCH"));
    die(&mut d, 20260505);
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_medical_2_medical"), 0);
    assert_eq!(value(&r, "inherited_medical_2_taxable"), 8000);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 8000);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 0);
    assert_eq!(value(&r, "inherited_medical_earnings"), 0);
    assert_eq!(return_line(&r, "8"), 800_000);
    assert_eq!(return_line(&r, "23"), 0);
    d.income[1].income = sa(8000, MedicalSaAccount::Archer, estate(20260504));
    assert!(issue(&d, "ISSUE_INHERITED_MEDICAL_ESTATE_DATE_MISMATCH"));
    // An individual beneficiary cannot have died before the holder.
    d.income[1].income =
        sa(8000, MedicalSaAccount::Archer, inherited_payment(20260616, 8000, vec![]));
    assert!(issue(&d, "ISSUE_INHERITED_MEDICAL_BENEFICIARY_PREDECEASED"));
}

// F6. Code 6, the holder died 2025-11-30: the value 10,000 was income of the 2025 return (a note),
// and only the earnings after it, box 1 10,300 - 10,000 = 300, are income now (line 8z). F6b: box 1
// 9,700 is a decline with no deduction: nothing is income, with a note.
#[test]
fn f6_inherited_account_of_an_earlier_death_taxes_only_later_earnings() {
    for account in kinds() {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 0);
        add(&mut d, income(inherited_interest(1_000_000)));
        let expected = supported(&d);
        add(
            &mut d,
            income(sa(10_300, account.clone(), inherited_payment(20251130, 10_000, vec![]))),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "inherited_medical_3_taxable"), 0);
        assert_eq!(value(&r, "inherited_medical_3_earnings"), 300);
        assert_eq!(value(&r, "inherited_medical_earnings"), 300);
        assert!(noted(&r, "NOTE_INHERITED_MEDICAL_PRIOR_YEAR"));
        let reference = {
            let mut reference = d.clone();
            reference.income[2].income =
                prize_income(dollars(300), 0, 0, PrizeDisposition::Received);
            supported(&reference)
        };
        assert_eq!(r.tax, reference.tax);
        assert_ne!(r.tax, expected.tax);
        assert_eq!(restart(d.clone()).draft, d);
        d.income[2].income = sa(9700, account, inherited_payment(20251130, 10_000, vec![]));
        let r = supported(&d);
        assert_eq!(value(&r, "inherited_medical_3_earnings"), 0);
        assert!(noted(&r, "NOTE_INHERITED_MEDICAL_DECLINE"));
        assert_eq!(r.tax, expected.tax);
    }
}

// The earlier death's offset stays on the death-year return: the bill paid in 2026 for a 2025 death
// does not change this return.
#[test]
fn earlier_death_medical_offset_stays_on_death_year_return() {
    let mut d = complete(FederalStatus::Single);
    let expected = supported(&d);
    add(
        &mut d,
        income(sa(
            10_000,
            MedicalSaAccount::Archer,
            inherited_payment(20251231, 10_000, vec![bill(20251201, 20260601, 1000)]),
        )),
    );
    let r = supported(&d);
    assert_eq!(r.tax, expected.tax);
    assert!(r.review_notes.iter().any(|n| n.contains("2025 return")));
}

// The three inherited accounts use the same death-year exclusion and no withdrawal penalty: value
// 50,000 less 12,000 of bills paid within a year = 38,000, on its own account's line.
#[test]
fn inherited_medical_accounts_use_same_exclusion_without_withdrawal_penalty() {
    for (account, id) in [
        (MedicalSaAccount::Hsa, "hsa_taxable_distributions"),
        (MedicalSaAccount::Archer, "archer_msa_taxable_distributions"),
        (ma_none(), "medicare_msa_taxable_distributions"),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(medical_inheritance(account)));
        let r = supported(&d);
        assert_eq!(value(&r, id), 38_000);
        assert_eq!(value(&r, "inherited_medical_2_medical"), 12_000);
        assert_eq!(return_line(&r, "8"), 3_800_000);
        assert_eq!(return_line(&r, "23"), 0);
        let expected = as_prize(&d, 1, 38_000);
        assert_eq!(r.tax, expected.tax);
        for id in ["agi", "deduction", "self_employment_tax", "qbi_deduction"] {
            assert_eq!(value(&r, id), value(&expected, id));
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
}

// The bills' dates: paid after the death and within a year after it, incurred by the death.
#[test]
fn inherited_medical_expenses_use_actual_death_window_including_following_year() {
    for (incurred, paid, medical) in [
        (20251231, 20270615, 12_000),
        (20260615, 20260615, 12_000), // the order of the day's events is the filer's statement
        (20251231, 20270616, 0),
        (20260616, 20260617, 0),
        (20260101, 20260614, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(sa(
                50_000,
                MedicalSaAccount::Hsa,
                inherited_payment(20260615, 50_000, vec![bill(incurred, paid, 12_000)]),
            )),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "inherited_medical_2_medical"), medical);
        assert_eq!(value(&r, "inherited_medical_2_taxable"), 50_000 - medical);
    }
}

// The bills are capped at the value and the sum is rounded once: a 49-cent pair is 98 cents.
#[test]
fn inherited_medical_expenses_are_capped_per_account_and_rounded_after_sum() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(sa(
            1000,
            MedicalSaAccount::Archer,
            inherited_payment(20260615, 1000, vec![bill(20260101, 20260616, 2000)]),
        )),
    );
    add(
        &mut d,
        income(sa(1000, MedicalSaAccount::Archer, inherited_payment(20260615, 1000, vec![]))),
    );
    assert_eq!(value(&supported(&d), "archer_msa_taxable_distributions"), 1000);
    let pair = FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(1000),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Inherited {
            died: 20260615,
            value: dollars(1000),
            beneficiary: MedicalBeneficiary::Nonspouse {
                payments: [49, 49]
                    .map(|amount| InheritedMedicalPayment {
                        incurred: 20260101,
                        paid: 20260616,
                        amount,
                    })
                    .into(),
            },
        },
    };
    let a = pair.amounts(&person()).unwrap();
    assert_eq!((a.inherited_medical, a.inherited), (whole(1), whole(999)));
}

// The bills reduce the inherited income but are no itemized medical deduction.
#[test]
fn inherited_medical_offset_does_not_create_an_itemized_medical_deduction() {
    let mut d = complete(FederalStatus::Single);
    itemized(d.deductions.as_mut().unwrap()).medical = 1_000_000;
    let before = supported(&d);
    let mut form = medical_inheritance(ma_none());
    if let FederalIncome::MedicalDistribution { data } = &mut form {
        data.gross = dollars(12_000);
        if let MedicalSaPayment::Inherited { value, .. } = &mut data.payment {
            *value = dollars(12_000);
        }
    }
    add(&mut d, income(form));
    let r = supported(&d);
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "deduction"), value(&before, "deduction"));
}

// Several inherited accounts live beside the person's own Medicare Advantage MSA.
#[test]
fn multiple_medical_inheritances_coexist_with_own_medicare_account() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(medicare_msa(5000, MedicareMsaPrior::None)));
    for account in [MedicalSaAccount::Hsa, MedicalSaAccount::Archer, ma_none(), ma_none()] {
        let mut e = income(medical_inheritance(account));
        e.owner = Owner::Spouse;
        add(&mut d, e);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 38_000);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 38_000);
    assert_eq!(value(&r, "medicare_msa_taxable_distributions"), 81_000);
    assert_eq!(value(&r, "medicare_msa_additional_tax"), 2500);
    assert_eq!(return_line(&r, "8"), 15_700_000);
    assert_eq!(return_line(&r, "23"), 250_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
}

// An estate inheritance belongs to the holder's final return only: People must show the death date.
#[test]
fn inherited_medical_estate_beneficiary_requires_original_holders_final_return() {
    for account in kinds() {
        let mut d = complete(FederalStatus::Single);
        let mut form = medical_inheritance(account);
        if let FederalIncome::MedicalDistribution { data } = &mut form
            && let MedicalSaPayment::Inherited { beneficiary, .. } = &mut data.payment
        {
            *beneficiary = MedicalBeneficiary::Estate;
        }
        add(&mut d, income(form));
        unsupported(&d, "Income");
        die(&mut d, 20260615);
        let r = supported(&d);
        assert_eq!(value(&r, "inherited_medical_2_medical"), 0);
        assert_eq!(value(&r, "inherited_medical_2_taxable"), 50_000);
        assert_eq!(return_line(&r, "8"), 5_000_000);
        assert_eq!(return_line(&r, "23"), 0);
        d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20260614;
        unsupported(&d, "Income");
    }
}

// The inherited value joins the prior-year loss absorption after the bills.
#[test]
fn prior_nol_uses_inherited_health_value_after_medical_expense_exclusion() {
    for account in kinds() {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(medical_inheritance(account)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "inherited_medical_2_medical"), 12_000);
        assert_eq!(value(&r, "prior_nol_deduction"), 65_520);
        assert_eq!(value(&r, "agi"), 32_480);
        assert_eq!(value(&r, "nol_2025_carryforward"), 34_480);
        assert_eq!(return_line(&r, "23"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_preserves_archer_withdrawal_tax_and_hsa_contribution_deduction() {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa.push(hsa_entry(
        Owner::Taxpayer,
        HsaCoverage::SelfOnly,
        100_000,
    ));
    add(&mut d, income(archer_msa(5000)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 38_320);
    assert_eq!(value(&r, "agi"), 25_680);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 5000);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 1000);
    assert_eq!(value(&r, "nol_2025_carryforward"), 61_680);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_does_not_replace_medicare_msa_balance_relief() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(medicare_msa(
            5000,
            MedicareMsaPrior::Existing { balance: dollars(10_000), deductible: dollars(10_000) },
        )),
    );
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100_000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -35_000);
    assert_eq!(value(&r, "medicare_msa_2_balance_relief"), 4000);
    assert_eq!(return_line(&r, "23"), 50_000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 51_100);
    assert_eq!(restart(d.clone()).draft, d);
}

// The page rules the client checks, checked again on save with the same keys.
#[test]
fn sa1099_rules_are_checked_again_on_the_server() {
    let own =
        |medical, rollover: Option<MedicalRollover>, disabled_since| FederalMedicalDistribution {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(1000),
            account: MedicalSaAccount::Archer,
            payment: MedicalSaPayment::Own {
                nonmedical: vec![MedicalAccountWithdrawal {
                    date: 20260615,
                    amount: (dollars(1000 - medical)
                        - rollover.as_ref().map_or(0, |r| exact::rounded(&r.amount()).unwrap()))
                    .max(0),
                }],
                medical: dollars(medical),
                rollover,
                disabled_since,
            },
        };
    assert_eq!(own(1000, None, None).rule(), None);
    assert!(own(1000, None, None).valid());
    assert!(!own(1001, None, None).valid());
    assert_eq!(own(600, Some(archer_rollover()), None).rule(), Some("SA1099_MEDICAL_ABOVE_GROSS"));
    // 600 spent and 5,000 rolled over out of 1,000.
    let mut ma = own(0, Some(archer_rollover()), None);
    ma.gross = dollars(10_000);
    ma.account = ma_none();
    assert_eq!(ma.rule(), Some("SA1099_ROLLOVER_NOT_ARCHER"));
    let mut early = archer_rollover();
    early.receipts[0].deposits[0].deposited = 20251231;
    let mut early_form = own(0, Some(early), None);
    early_form.gross = dollars(10_000);
    assert_eq!(early_form.rule(), Some("SA1099_ROLLOVER_BEFORE_RECEIPT"));
    // A later onset is valid source context; it cannot exempt an earlier payment.
    assert_eq!(own(0, None, Some(20260616)).rule(), None);
    assert_eq!(
        own(0, None, Some(20260616)).amounts(&person()).unwrap().penalized,
        whole(dollars(1000))
    );
    assert_eq!(own(0, None, Some(20260615)).rule(), None);
    let mut spouse = own(0, None, None);
    spouse.account = MedicalSaAccount::MedicareFromSpouse;
    spouse.payment = inherited_payment(20260101, 100, vec![]);
    assert_eq!(spouse.rule(), Some("SA1099_SPOUSE_ACCOUNT_IS_OWN"));
    // Invalid dates and amounts are not storable.
    let mut bad = own(0, None, None);
    for paid in [20251231, 20260229, 20270101] {
        bad.payment = own_payment(1000, paid, 0);
        assert!(!bad.valid(), "{paid}");
    }
    bad.payment = inherited_payment(20260229, 100, vec![]);
    assert!(!bad.valid());
    bad.payment = inherited_payment(20260615, 100, vec![bill(20260102, 20260101, 1)]);
    assert!(!bad.valid());
    bad.payment = inherited_payment(19961231, 100, vec![]);
    assert!(!bad.valid());
    bad.payment = MedicalSaPayment::Prohibited { occurred: 20251231 };
    assert!(!bad.valid());
    // The save refuses what the rules refuse.
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    let item = item(
        Owner::Taxpayer,
        0,
        IncomeDocument::Sa1099(Sa1099 {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(100),
            account: ma_none(),
            payment: MedicalSaPayment::ExcessReturned { earnings: Some(10) },
        }),
    );
    let Reply::Response(P::Problem { message, .. }) =
        send(&mut s, save_topic(&d, TaxSection::OtherIncome, vec![item]))
    else {
        panic!("expected a refused save")
    };
    assert_eq!(message, libertas::libertas_formatted_text("SA1099_EXCESS_NOT_ARCHER", &[]));
}

// ---- Form 1099-LTC and Form 8853 Section C ----

// One 1099-LTC (a qualified contract) and the periods record it names (empty: the only one).
fn ltc_return(per_diem: i64, periods: LtcPeriods) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(ltc_record(periods)));
    add(&mut d, income(ltc_form(per_diem, None)));
    d
}
fn one_period(starts: u32, ends: u32, services: i64, reimbursements: i64) -> LtcPeriods {
    LtcPeriods::One {
        starts,
        ends,
        services: dollars(services),
        reimbursements: dollars(reimbursements),
        recipient: CareBenefitRecipient::Insured { other_payees: false },
    }
}

// F8 (Instructions for Form 8853, Example 1). The taxpayer is the insured and the policyholder: box 1
// 24,000 per diem for a chronically ill insured; the whole year, care costs 54,750, reimbursements
// 27,375. Line 20 24,000; 21 = 430 x 365 = 156,950; 22 54,750; 23 = the larger = 156,950; 24 27,375;
// 25 = 129,575; line 26 = positive(24,000 - 24,000 limit used) = 0.
#[test]
fn f8_ltc_one_period_the_whole_year() {
    let d = ltc_return(24_000, one_period(20260101, 20261231, 54_750, 27_375));
    let r = supported(&d);
    assert_eq!(value(&r, "ltc_benefit_2_1_limit"), 129_575);
    assert_eq!(value(&r, "ltc_benefit_2_1_excluded"), 24_000);
    assert_eq!(value(&r, "ltc_benefit_2_1_taxable"), 0);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(restart(d.clone()).draft, d);
}

// F9. 30 days of June, box 1 15,000, care costs 9,000, nothing reimbursed: line 21 = 430 x 30 =
// 12,900; 23 = max(12,900, 9,000) = 12,900; 25 = 12,900; line 26 = 15,000 - 12,900 = 2,100 (Schedule 1
// line 8e). F9b: care costs 16,000: 23 = 16,000, 26 = 0. F9c: no periods record: all 15,000 is
// taxed, with a note.
#[test]
fn f9_ltc_over_the_per_diem_limit() {
    let d = ltc_return(15_000, one_period(20260601, 20260630, 9000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "ltc_benefit_2_1_limit"), 12_900);
    assert_eq!(value(&r, "ltc_benefit_2_1_taxable"), 2100);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 2100);
    assert_eq!(return_line(&r, "8"), 210_000);
    assert_eq!(value(&r, "agi"), 62_100);
    let big = ltc_return(15_000, one_period(20260601, 20260630, 16_000, 0));
    let r = supported(&big);
    assert_eq!(value(&r, "ltc_benefit_2_1_limit"), 16_000);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 0);
    let mut missing = complete(FederalStatus::Single);
    add(&mut missing, income(ltc_form(15_000, None)));
    let r = supported(&missing);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 15_000);
    assert!(noted(&r, "WARN_LTC_PERIODS_MISSING"));
}

// F10. Box 2 50,000 accelerated death benefits of a terminally ill insured, neither box 3 checked:
// excluded without Section C (section 101(g)(1)), no periods record needed. F10b: a policy that is
// not a qualified contract retains unresolved mandatory income; it is not an optional benefit omission.
#[test]
fn f10_ltc_terminal_illness_and_a_nonqualified_contract() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::LtcBenefits {
            data: FederalLtcPayment {
                interest: 0,
                benefit_origin: Some(LtcPaymentOrigin::Insurance),
                accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                    business_insurable_interest: false,
                }),
                benefits: 0,
                accelerated: dollars(50_000),
                paid: LtcPaidBasis::NotChecked,
                contract: LtcContract::Qualified {
                    qualification: CareContractQualification::Qualified,
                },
                illness: LtcIllness::Terminal,
                insured: None,
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&r, "ltc_benefit_2_exempt"), 50_000);
    assert_eq!(value(&r, "agi"), 60_000);
    assert!(!noted(&r, "WARN_LTC_PERIODS_MISSING"));
    assert_eq!(restart(d.clone()).draft, d);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::LtcBenefits {
            data: FederalLtcPayment {
                interest: 0,
                benefit_origin: Some(LtcPaymentOrigin::Insurance),
                accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                    business_insurable_interest: false,
                }),
                benefits: dollars(6000),
                accelerated: 0,
                paid: LtcPaidBasis::PerDiem,
                contract: LtcContract::Nonqualified {
                    treatment: NonqualifiedCareTreatment::Unclassified,
                },
                illness: LtcIllness::Chronic,
                insured: None,
            },
        }),
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(
        r.issues.iter().any(|i| i.display.key == "ISSUE_LTC_NONQUALIFIED_CHARACTER_UNRESOLVED")
    );
    assert!(!noted(&r, "WARN_LTC_PERIODS_MISSING"));
    let mut notes = vec![];
    let retained = crate::unknowns::supported(&d, &mut notes);
    assert_eq!(retained.as_ref().unwrap_or(&d).income, d.income);
    assert!(notes.iter().all(|n| n.display.key != "WARN_LTC_NONQUALIFIED_LEFT_OUT"));
    let mut s = restart(d.clone());
    assert_eq!(estimate(&s.draft), r);
    let request = open_topic(&mut s, TaxSection::OtherIncome);
    assert!(
        matches!(&page_documents(&request)[0].document, IncomeDocument::Ltc1099(f) if f.contract == (LtcContractPage::Nonqualified { treatment:NonqualifiedCareTreatmentPage::Unclassified }))
    );
}

// A reimbursed amount of a qualified contract is excluded outright, a chronically ill insured's
// accelerated death benefit paid per diem joins the per diem payments.
#[test]
fn ltc_reimbursed_and_accelerated_benefits_by_their_boxes() {
    let form = |benefits, accelerated, paid, illness| FederalLtcPayment {
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
            business_insurable_interest: false,
        }),
        benefits: dollars(benefits),
        accelerated: dollars(accelerated),
        paid,
        contract: LtcContract::Qualified { qualification: CareContractQualification::Qualified },
        illness,
        insured: None,
    };
    let reimbursed = form(1000, 0, LtcPaidBasis::Reimbursed, LtcIllness::Chronic);
    assert_eq!((reimbursed.periodic(), reimbursed.excluded()), (0, dollars(1000)));
    assert_eq!(reimbursed.preview(), Some(0));
    let chronic = form(1000, 500, LtcPaidBasis::PerDiem, LtcIllness::Chronic);
    assert_eq!((chronic.periodic(), chronic.excluded()), (dollars(1500), 0));
    assert_eq!(chronic.preview(), None);
    let terminal = form(1000, 500, LtcPaidBasis::PerDiem, LtcIllness::Terminal);
    assert_eq!((terminal.periodic(), terminal.excluded()), (dollars(1000), dollars(500)));
    let unchecked = form(0, 500, LtcPaidBasis::NotChecked, LtcIllness::Chronic);
    assert_eq!(unchecked.periodic(), dollars(500));
    assert_eq!(unchecked.rule(), None);
    assert_eq!(
        form(0, 0, LtcPaidBasis::PerDiem, LtcIllness::Chronic).rule(),
        Some("LTC1099_NO_AMOUNT")
    );
    // A form that is not a qualified contract shows no preview.
    let mut nonqualified = form(1000, 0, LtcPaidBasis::PerDiem, LtcIllness::Chronic);
    nonqualified.contract =
        LtcContract::Nonqualified { treatment: NonqualifiedCareTreatment::Unclassified };
    assert_eq!(nonqualified.preview(), None);
}

// Several periods are figured one by one: a period's unused limit cannot offset another's excess.
#[test]
fn ltc_unused_period_limit_cannot_offset_another_period() {
    let mut first = ltc_period(500, CareBenefitRecipient::Insured { other_payees: false });
    first.ends = first.starts;
    first.services = 0;
    first.reimbursements = 0;
    let mut second = first.clone();
    second.starts = 20260102;
    second.ends = 20260102;
    second.received = 20_000;
    let periods = LtcPeriods::Several {
        method: CarePeriodMethod::EqualRate,
        periods: vec![first.clone(), second.clone()],
    };
    // The record is entry 2 and the form entry 3: 500 + 200 = 700 of per diem payments.
    let d = ltc_return(700, periods);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 70);
    assert_eq!(value(&r, "ltc_benefit_2_1_taxable"), 70);
    assert_eq!(value(&r, "ltc_benefit_2_2_taxable"), 0);
    // The periods' payments must total the forms'.
    let mut off = d.clone();
    if let FederalIncome::LtcBenefits { data } = &mut off.income[2].income {
        data.benefits += dollars(10);
    }
    assert!(issue(&off, "ISSUE_LTC_PERIOD_PAYMENTS_DIFFER"));
    // Overlapping periods are not storable.
    let mut overlap = d.clone();
    if let FederalIncome::LtcInsured { data } = &mut overlap.income[1].income {
        data.periods = LtcPeriods::Several {
            method: CarePeriodMethod::Contract,
            periods: vec![first.clone(), first],
        };
    }
    assert!(!overlap.valid_stored());
}

// The multiple-payee allocation (the insured's filing unit first) flows to the return like any
// other unearned income.
#[test]
fn ltc_taxable_benefits_flow_like_other_unearned_noninvestment_income() {
    let other = LtcPeriods::One {
        starts: 20260101,
        ends: 20260131,
        services: 1_000_000,
        reimbursements: 200_000,
        recipient: CareBenefitRecipient::Other {
            insured_received: 1_000_000,
            others_received: 400_000,
        },
    };
    let d = ltc_return(6000, other);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 5202);
    assert_eq!(value(&r, "ltc_benefit_2_1_excluded"), 798);
    assert_eq!(value(&r, "agi"), 65_202);
    assert_eq!(return_line(&r, "8"), 520_200);
    let expected = as_prize(&d, 2, 5202);
    assert_eq!(r.tax, expected.tax);
    for id in ["agi", "deduction", "self_employment_tax", "qbi_deduction", "earned_income_credit"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_uses_only_long_term_care_benefits_above_period_exclusion() {
    let mut d = ltc_return(15_000, ltc_one_period(20260101, 20260131));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    let r = supported(&d);
    // January's $13,330 per-diem limit less $2,000 reimbursements leaves $11,330 excluded and
    // $3,670 taxable from the $15,000 benefit.
    assert_eq!(value(&r, "prior_nol_deduction"), 38_056);
    assert_eq!(value(&r, "agi"), 25_614);
    assert_eq!(value(&r, "nol_2025_carryforward"), 61_944);
    assert_eq!(return_line(&r, "23"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

// The record's dates and payments are checked where they are stored.
#[test]
fn ltc_period_source_rejects_outside_year_invalid_dates_and_negative_payments() {
    for (starts, ends) in
        [(20251231, 20260101), (20261231, 20270101), (20260229, 20260301), (20260102, 20260101)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(ltc_record(ltc_one_period(starts, ends))));
        assert!(!d.valid_stored(), "{starts}-{ends}");
    }
    let mut d = complete(FederalStatus::Single);
    let mut p =
        ltc_period(1000, CareBenefitRecipient::Other { insured_received: -1, others_received: 0 });
    let several =
        |periods| ltc_record(LtcPeriods::Several { method: CarePeriodMethod::EqualRate, periods });
    let mut second = p.clone();
    second.starts = 20260201;
    second.ends = 20260228;
    add(&mut d, income(several(vec![p.clone(), second.clone()])));
    assert!(!d.valid_stored());
    p.recipient = CareBenefitRecipient::Insured { other_payees: false };
    p.received = -1;
    d.income[1].income = several(vec![p, second]);
    assert!(!d.valid_stored());
    // The rule the page checks, checked again on save.
    let record = FederalLtcInsured {
        insured: FederalLtcInsuredPerson::Other { name: "A".into() },
        periods: ltc_one_period(20260110, 20260109),
    };
    assert_eq!(record.rule(), Some("LTC_PERIOD_ENDS_BEFORE_START"));
}

// The 1099-LTC names its record by identity; empty uses the only one (either filer's on a joint
// return); a record of the other filer on a separate return is refused.
#[test]
fn ltc_link_resolves_by_identity_the_only_record_or_none() {
    let periods = || one_period(20260601, 20260630, 9000, 0);
    // Two records and the form names the second: its own limit applies.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(ltc_record(one_period(20260101, 20260131, 0, 0))));
    add(&mut d, income(ltc_record(periods())));
    add(&mut d, income(ltc_form(15_000, Some(3))));
    let r = supported(&d);
    assert_eq!(value(&r, "ltc_benefit_3_1_taxable"), 2100);
    assert_eq!(value(&r, "ltc_benefit_2_1_taxable"), 0);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 2100);
    assert!(!noted(&r, "WARN_LTC_PERIODS_MISSING"));
    // None chosen among two records: taxed in full, with a note.
    d.income[3].income = ltc_form(15_000, None);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 15_000);
    assert!(noted(&r, "WARN_LTC_PERIODS_MISSING"));
    // A stale identity (the record is gone) is the same.
    d.income[3].income = ltc_form(15_000, Some(99));
    assert!(noted(&supported(&d), "WARN_LTC_PERIODS_MISSING"));
    // Joint: the spouse's form uses the taxpayer's record.
    let mut joint = complete(FederalStatus::Joint);
    add(&mut joint, income(ltc_record(periods())));
    let mut form = income(ltc_form(15_000, None));
    form.owner = Owner::Spouse;
    add(&mut joint, form);
    let r = supported(&joint);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 2100);
    // Separate returns: another filer's record is refused.
    let mut separate = complete(FederalStatus::Separate);
    let mut record = income(ltc_record(periods()));
    record.owner = Owner::Spouse;
    add(&mut separate, record);
    add(&mut separate, income(ltc_form(15_000, Some(2))));
    assert!(estimate(&separate).state == ResultState::Unsupported);
}

// ---- Form 1099-QA and Form 5329 Part II ----

fn qa(gross: i64, earnings: i64, treatment: AbleTreatment) -> FederalIncome {
    FederalIncome::Able {
        data: AbleDistribution { gross: dollars(gross), earnings: dollars(earnings), treatment },
    }
}
fn annual(expenses: i64) -> AbleTreatment {
    AbleTreatment::Annual {
        rollover: AbleRollover::None,
        expenses: dollars(expenses),
        following_expenses: 0,
    }
}

// F11. Box 1 10,000, box 2 earnings 2,000, box 3 basis 8,000; 7,500 of qualified disability expenses.
// Excluded earnings = 2,000 x 7,500 / 10,000 = 1,500, so 500 is taxable (Schedule 1 line 8q); Form
// 5329 lines 5 500, 6 0, 7 500, line 8 = 10% x 500 = 50 (Schedule 2 line 5). F11b: 4,000 put back into
// an ABLE account within 60 days with 800 of earnings: 6,000 and 1,200 remain, the expenses cover it.
#[test]
fn f11_able_distribution_partly_qualified() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(qa(10_000, 2000, annual(7500))));
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 500);
    assert_eq!(value(&r, "able_2_annual_taxable"), 500);
    assert_eq!(value(&r, "account_additional_tax"), 50);
    assert_eq!(return_line(&r, "8"), 50_000);
    assert_eq!(return_line(&r, "23"), 5000);
    assert_eq!(restart(d.clone()).draft, d);
    let mut rolled = d.clone();
    rolled.income[1].income = qa(
        10_000,
        2000,
        AbleTreatment::Annual {
            rollover: AbleRollover::Qualified { amount: dollars(4000), earnings: dollars(800) },
            expenses: dollars(7500),
            following_expenses: 0,
        },
    );
    let r = supported(&rolled);
    assert_eq!(value(&r, "able_2_rollover"), 4000);
    assert_eq!(value(&r, "able_2_rollover_earnings"), 800);
    assert_eq!(value(&r, "able_taxable_distributions"), 0);
}

// F12. A payment after the beneficiary's death (box 1 4,000, box 2 600, box 3 3,400) is not taxed.
#[test]
fn f12_able_payment_after_the_death_is_not_taxed() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(qa(4000, 600, AbleTreatment::Death)));
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 0);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
}

// F13. An excess contribution returned (box 6): box 1 1,150, earnings 150 are the contributor's
// income (line 8q), with no 10% tax. F13b: a contribution for 2025 belongs on the 2025 return: 0 and a
// note. A year outside 2015-2026 is not storable.
#[test]
fn f13_able_returned_excess_contribution() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(qa(1150, 150, AbleTreatment::Corrective { year: 2026 })));
    assert_eq!(crate::interview::document_amount(&d.income[1].income, false), Some(15_000));
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 150);
    assert_eq!(value(&r, "able_corrective_earnings"), 150);
    assert_eq!(value(&r, "account_additional_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income = qa(1150, 150, AbleTreatment::Corrective { year: 2025 });
    let r = supported(&d);
    assert_eq!(value(&r, "able_taxable_distributions"), 0);
    let note = r
        .review_notes
        .iter()
        .find(|n| n.display.key == "NOTE_ABLE_CORRECTION_PRIOR_RETURN")
        .expect("the prior-year note");
    assert_eq!(note.display.args[1], MessageArg::Number(2025));
    d.income[1].income = qa(1150, 150, AbleTreatment::Corrective { year: 2027 });
    assert!(!d.valid_stored());
}

// Box 3 is not stored: the page shows gross less earnings, and the rules keep the boxes consistent.
#[test]
fn qa1099_boxes_and_rollover_rules_are_checked_on_the_server() {
    let page = |gross, earnings, basis, treatment| Qa1099 {
        gross: dollars(gross),
        earnings: dollars(earnings),
        basis: dollars(basis),
        treatment,
    };
    assert_eq!(page(100, 20, 80, annual(0)).rule(), None);
    assert_eq!(page(100, 20, 70, annual(0)).rule(), Some("QA1099_BOXES_DO_NOT_ADD_UP"));
    let rolled = |amount, earnings| AbleTreatment::Annual {
        rollover: AbleRollover::Qualified { amount: dollars(amount), earnings: dollars(earnings) },
        expenses: 0,
        following_expenses: 0,
    };
    assert_eq!(page(100, 20, 80, rolled(101, 0)).rule(), Some("QA1099_ROLLOVER_ABOVE_GROSS"));
    assert_eq!(
        page(100, 20, 80, rolled(50, 51)).rule(),
        Some("QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT")
    );
    assert_eq!(page(100, 20, 80, rolled(50, 20)).rule(), None);
    let record = page(100, 20, 80, annual(0)).record();
    assert_eq!(record.page(), page(100, 20, 80, annual(0)));
    // A loss (box 2 negative) is not deductible: it taxes nothing.
    let mut loss = qa(100, -20, annual(0));
    let FederalIncome::Able { data } = &mut loss else { panic!() };
    assert!(data.valid());
    assert_eq!(data.taxable(), 0);
    // A rollover must leave the remaining earnings within the remaining gross.
    data.earnings = dollars(90);
    data.treatment = rolled(80, 10);
    assert!(!data.valid());
}

// Two forms are two accounts: each has its own earnings ratio and its own expenses.
#[test]
fn two_able_accounts_use_separate_earnings_ratios_and_expense_allocations() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(qa(1000, 900, annual(1000))));
    add(&mut d, income(qa(1000, 100, annual(0))));
    let r = supported(&d);
    assert_eq!(value(&r, "able_2_annual_taxable"), 0);
    assert_eq!(value(&r, "able_3_annual_taxable"), 100);
    assert_eq!(value(&r, "able_taxable_distributions"), 100);
    assert_eq!(value(&r, "account_additional_tax"), 10);
    assert_eq!(value(&r, "agi"), 60_100);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

// The elected first-60-days-of-2027 expenses count with the expenses of the year.
#[test]
fn able_partial_rollover_combines_with_the_expense_election() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(mixed_able(4000, 800, 3000)));
    assert_eq!(value(&supported(&d), "able_taxable_distributions"), 1100);
    d.income.pop();
    let mut entry = income(mixed_able(4000, 800, 3000));
    if let FederalIncome::Able { data } = &mut entry.income
        && let AbleTreatment::Annual { following_expenses, .. } = &mut data.treatment
    {
        *following_expenses = 300_000;
    }
    add(&mut d, entry);
    assert_eq!(value(&supported(&d), "able_taxable_distributions"), 0);
}

// ---- The documents on the Other income page ----

fn other_income_page(d: &FederalDraft) -> P {
    let mut s = Interview::new(TEST_USER, d.clone());
    open_topic(&mut s, TaxSection::OtherIncome)
}
pub(in crate::tests) fn save_items(d: &FederalDraft, items: Vec<IncomeDocumentItem>) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    accepted(&send(&mut s, save_topic(d, TaxSection::OtherIncome, items)));
    s
}
pub(in crate::tests) use crate::tests::support::builders::survey::survey_uncatalogued_document as item;

// The four documents are offered by the Other income page, save onto their stored records and
// reopen as they were entered; the records show on no other page.
#[test]
fn the_four_documents_save_and_reopen_on_the_other_income_page() {
    let d = complete(FederalStatus::Single);
    let documents = vec![
        IncomeDocument::Sa1099(Sa1099 {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(3000),
            account: MedicalSaAccount::Archer,
            payment: own_payment(3000, 20260615, 1800),
        }),
        IncomeDocument::Ltc1099(Ltc1099 {
            interest: 0,
            benefit_origin: Some(LtcPaymentOrigin::Insurance),
            accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                business_insurable_interest: false,
            }),
            benefits: dollars(15_000),
            accelerated: 0,
            paid: LtcPaidBasis::PerDiem,
            contract: LtcContractPage::Qualified {
                qualification: CareContractQualification::Qualified,
            },
            illness: LtcIllness::Chronic,
            insured: None,
        }),
        IncomeDocument::LtcInsured(LtcInsured {
            insured: LtcInsuredPerson::OnReturn { person: -1 },
            periods: one_period(20260601, 20260630, 9000, 0).into(),
        }),
        IncomeDocument::Qa1099(Qa1099 {
            gross: dollars(10_000),
            earnings: dollars(2000),
            basis: dollars(8000),
            treatment: annual(7500),
        }),
    ];
    let items: Vec<_> =
        documents.iter().cloned().map(|document| item(Owner::Taxpayer, 0, document)).collect();
    let s = save_items(&d, items);
    assert!(matches!(s.draft.income[1].income, FederalIncome::MedicalDistribution { .. }));
    assert!(matches!(s.draft.income[2].income, FederalIncome::LtcBenefits { .. }));
    assert!(matches!(s.draft.income[3].income, FederalIncome::LtcInsured { .. }));
    assert!(matches!(s.draft.income[4].income, FederalIncome::Able { .. }));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 1200);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 2100);
    assert_eq!(value(&r, "able_taxable_distributions"), 500);
    // Reopened: the page returns the documents, each with its amount on the return.
    let mut again = restart(s.draft.clone());
    let page = open_topic(&mut again, TaxSection::OtherIncome);
    let shown = page_documents(&page);
    assert_eq!(shown.iter().map(|i| i.document.clone()).collect::<Vec<_>>(), documents);
    assert_eq!(shown[0].preview, Some(dollars(1200)));
    assert_eq!(shown[1].preview, None);
    assert_eq!(shown[2].preview, None);
    assert_eq!(shown[3].preview, Some(dollars(500)));
    let shown = shown.to_vec();
    let draft = again.draft.clone();
    accepted(&send(&mut again, save_topic(&draft, TaxSection::OtherIncome, shown)));
    assert_eq!(again.draft, s.draft);
    // Other pages do not offer them.
    let retirement = open_topic(&mut again, TaxSection::Retirement);
    assert!(page_documents(&retirement).is_empty());
}

// The catalog round trip: a 1099-LTC names a record by its income identity, keeps the choice across
// a reopen, and each record keeps its own limit; the catalog labels the record by its insured person.
#[test]
fn ltc_insured_catalog_round_trips_through_the_documents_page() {
    let mut d = complete(FederalStatus::Single);
    d.dependents.clear();
    let form = |insured| {
        item(
            Owner::Taxpayer,
            0,
            IncomeDocument::Ltc1099(Ltc1099 {
                interest: 0,
                benefit_origin: Some(LtcPaymentOrigin::Insurance),
                accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                    business_insurable_interest: false,
                }),
                benefits: dollars(15_000),
                accelerated: 0,
                paid: LtcPaidBasis::PerDiem,
                contract: LtcContractPage::Qualified {
                    qualification: CareContractQualification::Qualified,
                },
                illness: LtcIllness::Chronic,
                insured,
            }),
        )
    };
    let record = |id, insured, periods: LtcPeriods| {
        item(
            Owner::Taxpayer,
            id,
            IncomeDocument::LtcInsured(LtcInsured { insured, periods: periods.into() }),
        )
    };
    let you = LtcInsuredPerson::OnReturn { person: -1 };
    let parent = LtcInsuredPerson::Other { name: "Pat Sample".into() };
    let first = save_items(
        &d,
        vec![
            record(0, you.clone(), one_period(20260601, 20260630, 9000, 0)),
            record(0, parent.clone(), one_period(20260101, 20260131, 0, 0)),
        ],
    );
    let page = other_income_page(&first.draft);
    let P::SaveIncomes { ltc_insured, .. } = &page else { panic!("the income page") };
    assert_eq!(ltc_insured.iter().map(|c| c.id).collect::<Vec<_>>(), vec![2, 3]);
    assert!(status_text(&ltc_insured[1].label).contains("Pat Sample"));
    let items = vec![
        record(2, you, one_period(20260601, 20260630, 9000, 0)),
        record(3, parent, one_period(20260101, 20260131, 0, 0)),
        form(Some(2)),
        form(Some(3)),
    ];
    let s = save_items(&first.draft, items.clone());
    let r = supported(&s.draft);
    // Record 2: 12,900 limit of 15,000 = 2,100 taxed. Record 3: January's 13,330 limit: nothing.
    assert_eq!(value(&r, "ltc_benefit_2_1_taxable"), 2100);
    assert_eq!(value(&r, "ltc_benefit_3_1_taxable"), 1670);
    assert_eq!(value(&r, "taxable_ltc_benefits"), 3770);
    let mut again = restart(s.draft.clone());
    let page = open_topic(&mut again, TaxSection::OtherIncome);
    let mut shown: Vec<_> = page_documents(&page).iter().map(|i| i.document.clone()).collect();
    let mut want: Vec<_> = items.iter().map(|i| i.document.clone()).collect();
    let order = |d: &IncomeDocument| alloc::format!("{d:?}");
    shown.sort_by_key(order);
    want.sort_by_key(order);
    assert_eq!(shown, want);
    // A removed record clears the links that name it (F3, 2026-10-09): the form
    // that chose it has no choice any more and takes the only record left, not
    // a stale identity taxed in full.
    let kept: Vec<_> = items
        .into_iter()
        .filter(|i| !matches!(&i.document, IncomeDocument::LtcInsured(v) if matches!(v.insured, LtcInsuredPerson::Other { .. })))
        .collect();
    let s = save_items(&s.draft, kept);
    let links: Vec<_> = s
        .draft
        .income
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::LtcBenefits { data } => Some(data.insured),
            _ => None,
        })
        .collect();
    assert_eq!(links, vec![Some(2), None]);
    let r = supported(&s.draft);
    assert!(!noted(&r, "WARN_LTC_PERIODS_MISSING"));
}

// A periods record for a person who left the return goes with the person; one for a person still on
// the return, or for someone else, stays.
#[test]
fn a_periods_record_for_a_person_who_left_the_return_is_removed() {
    let mut d = complete(FederalStatus::Single);
    add_child(&mut d);
    let child = d.dependents[0].id;
    let record = |insured| {
        item(
            Owner::Taxpayer,
            0,
            IncomeDocument::LtcInsured(LtcInsured {
                insured,
                periods: one_period(20260601, 20260630, 9000, 0).into(),
            }),
        )
    };
    let s = save_items(
        &d,
        vec![
            record(LtcInsuredPerson::OnReturn { person: child }),
            record(LtcInsuredPerson::Other { name: "Pat 🧾".repeat(50) }),
            record(LtcInsuredPerson::OnReturn { person: -1 }),
        ],
    );
    let stored = |d: &FederalDraft| {
        d.income.iter().filter(|e| matches!(e.income, FederalIncome::LtcInsured { .. })).count()
    };
    assert_eq!(stored(&s.draft), 3);
    assert!(s.draft.income.iter().any(|entry| matches!(
        &entry.income,
        FederalIncome::LtcInsured { data }
            if matches!(&data.insured, FederalLtcInsuredPerson::Other { name }
                if name == &"Pat 🧾".repeat(50))
    )));
    let mut kept = s.draft.clone();
    let result = crate::interview::prune(&mut kept);
    assert!(result.removed.iter().all(|row| row.key() != "SAVING_KIND_LTC_INSURED"), "{result:?}");
    assert_eq!(stored(&kept), 3);
    let mut d = s.draft.clone();
    d.dependents.clear();
    let result = crate::interview::prune(&mut d);
    assert_eq!(stored(&d), 2);
    assert!(result.removed.iter().any(|row| row.key() == "SAVING_KIND_LTC_INSURED"), "{result:?}");
}

// ---- Import mapping ----

// The import fields marked on a document's type, in source order.
fn imported(source: &str, name: &str, until: &str) -> Vec<alloc::string::String> {
    let start = source.find(&alloc::format!("pub struct {name} {{")).unwrap();
    let rest = &source[start..];
    let rest = &rest[..rest.find(until).unwrap_or(rest.len())];
    rest.lines()
        .scan(false, |marked, line| {
            let text = line.trim();
            if text.starts_with("#[libertas_llm_doc_field") {
                *marked = true;
                return Some(None);
            }
            if *marked && !text.starts_with("#[") && !text.starts_with("//") {
                *marked = false;
                return Some(
                    text.trim_start_matches("pub ")
                        .split_once(':')
                        .map(|(n, _)| alloc::string::String::from(n)),
                );
            }
            Some(None)
        })
        .flatten()
        .collect()
}
fn instruction<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source.find(&alloc::format!("pub struct {name} {{")).unwrap();
    let head = &source[..start];
    &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..]
}

// Form 1099-SA import: boxes 1, 3 (the code), 4 and 5 (and box 2 for code 2); the payment dates,
// medical use, rollover, disability, the death date, the beneficiary and the prior-year value stay
// the filer's.
#[test]
fn sa1099_import_reads_the_printed_boxes_and_leaves_personal_records() {
    let source = include_str!("../../health/medical_account.rs");
    let action = instruction(source, "Sa1099");
    assert!(action.contains("Form 1099-SA"));
    assert!(action.contains("For box 5 return Hsa, Archer or MedicareAdvantage"));
    assert!(action.contains(
        "Own for code 1 or 3, ExcessReturned with the printed box 2 amount for code 2 (null when genuinely blank; never infer the signed custodian net-income adjustment), Inherited with the box 4 amount as value for code 4 or 6, and Prohibited for code 5"
    ));
    assert!(action.contains("validation = \"$.document_facts\""));
    assert_eq!(
        imported(source, "Sa1099", "/// Who inherited it"),
        vec!["gross", "account", "payment", "earnings", "value"]
    );
}

// Form 1099-LTC import: boxes 1 to 5; the insured's periods record is the filer's.
#[test]
fn ltc1099_import_reads_boxes_1_to_5_and_never_the_periods_record() {
    let source = include_str!("../../health/care_benefits.rs");
    let action = instruction(source, "Ltc1099");
    assert!(action.contains("Form 1099-LTC"));
    assert!(action.contains("PerDiem when Per diem is checked, Reimbursed when Reimbursed amount"));
    assert!(
        action.contains("box 4 return Qualified with qualification Qualified when it is checked")
    );
    assert!(action.contains("Chronic when Chronically ill is checked and Terminal"));
    assert!(action.contains("do not choose the insured person's periods record"));
    assert_eq!(
        imported(source, "Ltc1099", "/// How it was paid (box 3)\n#[derive"),
        vec!["benefits", "accelerated", "paid", "contract", "illness"]
    );
    // The periods record has no import at all.
    let record = &source[source.find("pub struct LtcInsured {").unwrap()..];
    let head = &source[..source.find("pub struct LtcInsured {").unwrap()];
    assert!(!head[head.rfind("/// Long-term care periods").unwrap()..].contains("llm_doc_trans"));
    assert!(!record[..record.find("\n}").unwrap()].contains("libertas_llm_doc_field"));
}

// Form 1099-QA import: boxes 1, 2, 3 and 6 only; the rollover, the expenses and the year are the
// filer's.
#[test]
fn qa1099_import_reads_boxes_1_2_3_and_6_only() {
    let source = include_str!("../../family/able.rs");
    let action = instruction(source, "Qa1099");
    assert!(action.contains("Form 1099-QA"));
    assert!(
        action
            .contains("For treatment return Corrective when box 6 is checked and Annual otherwise")
    );
    assert!(action.contains("Do not read boxes 4 and 5"));
    assert_eq!(
        imported(source, "Qa1099", "/// What the payment was\n#[derive"),
        vec!["gross", "earnings", "basis", "treatment"]
    );
}

// ---- The stored fields no page wrote: the HSA page's ten fields, the Archer MSA contributions ----

// A stored return opened on the HSA page and saved unchanged: the stored accounts and Archer
// records come back as they were (the page shows every field it stores).
fn hsa_page_round_trips(d: FederalDraft) -> Interview {
    let mut s = Interview::new(TEST_USER, d.clone());
    let page = open_hsa_page(&mut s);
    let mut before_result = crate::federal_rules::estimate(&d);
    accepted(&send(&mut s, page));
    let (before, after) = (d.adjustments.unwrap(), s.draft.adjustments.clone().unwrap());
    assert_eq!(after.hsa, before.hsa);
    assert_eq!(after.archer, before.archer);
    assert_eq!(after.hsa_family_share, before.hsa_family_share);
    before_result.revision = s.draft.revision;
    assert_eq!(crate::federal_rules::estimate(&s.draft), before_result);
    s
}
fn with_hsa(h: FederalHsa) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.adjustments.as_mut().unwrap().hsa = vec![h];
    d
}

// Timely corrections, a late withdrawal of an old excess and a rollover on one account. Personal 5,000, 600 returned in time (50 of earnings): 4,400 deducted; the 1,000 late
// withdrawal is income with the 20% tax (200); the 5,000 rolled over within 60 days leaves the
// contribution limit alone.
#[test]
fn hsa_corrections_late_excess_and_rollover_save_and_reopen() {
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, dollars(5000));
    h.corrections = vec![hsa_correction(2026, 20260930, dollars(600), dollars(50))];
    h.late_excess = vec![HsaLateExcessWithdrawal {
        contribution_year: 2024,
        deadline: 20250415,
        date: 20260301,
        amount: dollars(1000),
        disabled: false,
    }];
    let mut d = with_hsa(h);
    add_hsa_distribution(&mut d, Owner::Taxpayer, 0, vec![], None, Some(hsa_rollover()));
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 4400);
    assert_eq!(value(&r, "hsa_1_corrected_contributions"), 600);
    assert_eq!(value(&r, "hsa_correction_earnings"), 50);
    assert_eq!(value(&r, "hsa_1_rollover"), 5000);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    let s = hsa_page_round_trips(d.clone());
    assert_eq!(supported(&s.draft).tax, r.tax);
}

// An uncorrected employer excess of 2025 deposited in 2026 with the retained 2025 Form 5329 figures.
#[test]
fn hsa_prior_employer_excess_and_its_accounting_save_and_reopen() {
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::SelfOnly, 440_000);
    h.prior_employer_excess.push(HsaPriorEmployerExcessDeposit {
        contributed_on: 20260315,
        deadline: 20260415,
        amount: 60_000,
        in_wages: false,
    });
    h.excess_accounting = Some(hsa_excess_accounting(
        500_000,
        Some(HsaPriorExcess { line_48: 60_000, line_49: 3600 }),
        0,
        0,
        0,
    ));
    let d = with_hsa(h);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_1_prior_designated_employer_excess_income"), 600);
    hsa_page_round_trips(d);
}

// A prohibited transaction and the 2025 transfer whose testing period ends in 2026.
#[test]
fn hsa_deemed_distribution_and_earlier_funding_save_and_reopen() {
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.coverage = HsaEligibility::Monthly { months: vec![HsaCoverage::None; 12] };
    h.funding_history = vec![hsa_funding(20250617, 20260616, dollars(4000))];
    let mut d = with_hsa(h);
    add_hsa_deemed(
        &mut d,
        Owner::Taxpayer,
        MedicalDeemed::Terminated { occurred: 20261001, january_value: dollars(1000) },
        None,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 4000);
    assert_eq!(value(&r, "hsa_testing_period_tax"), 400);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    hsa_page_round_trips(d);
}

// The last-month rule: eligible only in December with the first loss in December, and the same rule
// used in 2025.
#[test]
fn hsa_last_month_rule_facts_save_and_reopen() {
    let mut h = december_hsa(HsaCoverage::SelfOnly, 440_000);
    h.last_month_loss = Some(HsaEligibilityLoss { date: 20261215, reason: HsaLossReason::Other });
    let d = with_hsa(h);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_last_month_recapture"), 4033);
    hsa_page_round_trips(d);
    let mut h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    h.coverage = HsaEligibility::Monthly { months: vec![HsaCoverage::None; 12] };
    h.last_month_history = Some(last_month_history(6, 430_000));
    let d = with_hsa(h);
    assert_eq!(value(&supported(&d), "hsa_last_month_recapture"), 2150);
    hsa_page_round_trips(d);
}

// Qualified IRA-to-HSA transfers are listed once, each naming its owner and its Form 1099-R by
// identity; the page's catalog offers the owner's IRA distributions.
#[test]
fn hsa_ira_transfers_name_their_1099_r_through_the_catalog() {
    let mut d = complete(FederalStatus::Single);
    current_hsa_funding(&mut d, 4000);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_funding_recapture"), 0);
    let mut s = Interview::new(TEST_USER, d.clone());
    let page = open_hsa_page(&mut s);
    let P::SaveHsa { ira_sources, archer_income, value: page_value, .. } = &page else {
        panic!("the HSA page")
    };
    // The 1099-R of the transfer (entry 2) is the IRA distribution on offer; the W-2 (entry 1) is
    // the compensation an Archer MSA can cover.
    assert_eq!(ira_sources.iter().map(|c| c.id).collect::<Vec<_>>(), vec![2]);
    assert_eq!(archer_income.iter().map(|c| c.id).collect::<Vec<_>>(), vec![1]);
    assert!(status_text(&ira_sources[0].label).contains("You"));
    assert_eq!(page_value.funding.len(), 1);
    assert_eq!(page_value.funding[0].source, 2);
    hsa_page_round_trips(d.clone());
    // A transfer that loses its eligibility within the year is recaptured: the first loss and the
    // rule on its date.
    let mut lost = d.clone();
    lost.adjustments.as_mut().unwrap().hsa[0].funding[0].loss =
        Some(HsaEligibilityLoss { date: 20260901, reason: HsaLossReason::Other });
    let r = supported(&lost);
    assert_eq!(value(&r, "hsa_funding_recapture"), 4000);
    hsa_page_round_trips(lost);
}

fn hsa_save(d: &FederalDraft, mut value: HsaPage) -> P {
    if value.employer_sources.is_empty() {
        value.employer_sources =
            crate::health_employer::page_rows(d, crate::health_employer::bindings(d)).unwrap();
        // Existing source fixtures describe current-year contributions.
        for source in &mut value.employer_sources {
            source.allocation = HealthEmployerYear::CurrentYear;
        }
    }
    P::SaveHsa {
        user: TEST_USER,
        hsa_failed_deposits: vec![],
        archer_failed_deposits: vec![],
        hsa_return_documents: vec![],
        archer_return_documents: vec![],
        cookie: d.cookie.clone(),
        revision: d.revision,
        page: PAGE_ADJUSTMENTS,
        previous: PAGE_ADJUSTMENTS,
        review: false,
        estimate: vec![],
        warnings: vec![],
        progress: vec![],
        other_coverage_allowed: false,
        allowed_owners: vec![],
        allowed_family_split: vec![],
        ira_sources: vec![],
        archer_income: vec![],
        value,
    }
}
fn key_of(reply: Reply) -> alloc::string::String {
    let Reply::Response(P::Problem { message, .. }) = reply else {
        panic!("expected a refused save, got {reply:?}")
    };
    status_message(&message).key
}

// The rules the page declares on its types are checked again on save with the same keys.
#[test]
fn hsa_page_rules_are_checked_again_on_the_server() {
    let d = complete(FederalStatus::Single);
    let account = || hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, dollars(5000));
    let page = |account: HsaAccount| {
        health_account_page(vec![account], vec![], ArcherContributionsPage::None)
    };
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut refused = |mutate: &dyn Fn(&mut HsaAccount), key: &str| {
        let mut a = account();
        mutate(&mut a);
        let request = hsa_save(&d, page(a));
        let reply = if key == "HSA_RECORD_INVALID" {
            current_source_refusal(
                "health-context-00",
                &mut s,
                request,
                "HSA_RECORD_INVALID",
                Some("[value/accounts/0/late_excess/0/date]"),
                SourceScope::Health,
            )
        } else {
            send(&mut s, request)
        };
        assert_eq!(key_of(reply), key, "{key}");
        assert_eq!(s.draft, d);
    };
    refused(
        &|a| {
            let mut c = hsa_correction(2026, 20260930, dollars(600), dollars(50));
            c.returned = 20260101 - 1;
            a.corrections = vec![c.page()];
        },
        "HSA_CORRECTION_RETURNED_BEFORE_DEPOSIT",
    );
    refused(
        &|a| {
            let mut c = hsa_correction(2026, 20260930, dollars(600), dollars(50));
            c.deadline = 20251231;
            a.corrections = vec![c.page()];
        },
        "HSA_CORRECTION_DEADLINE_BEFORE_DEPOSIT",
    );
    refused(
        &|a| {
            a.corrections =
                vec![hsa_correction(2026, 20260930, dollars(600), -dollars(601)).page()];
        },
        "HSA_CORRECTION_LOSS_ABOVE_PRINCIPAL",
    );
    refused(
        &|a| {
            a.late_excess = vec![HsaLateExcessWithdrawal {
                contribution_year: 2024,
                deadline: 20260401,
                date: 20260301,
                amount: dollars(1000),
                disabled: false,
            }];
        },
        "HSA_LATE_EXCESS_NOT_LATE",
    );
    refused(
        &|a| {
            a.prior_employer_excess = vec![HsaPriorEmployerExcessDeposit {
                contributed_on: 20260416,
                deadline: 20260415,
                amount: 60_000,
                in_wages: false,
            }];
        },
        "HSA_PRIOR_EXCESS_DEPOSIT_AFTER_DEADLINE",
    );
    refused(
        &|a| {
            a.excess_accounting = Some(hsa_excess_accounting(0, None, 100, 101, 0));
        },
        "HSA_LATER_EMPLOYER_DEPOSITS_ABOVE_LATER",
    );
    refused(
        &|a| {
            let mut h = hsa_funding(20250617, 20250616, dollars(4000));
            h.loss.as_mut().unwrap().date = 20250616;
            a.funding_history = vec![h];
        },
        "HSA_FUNDING_LOSS_BEFORE_FUNDING",
    );
    // The sums across the account's records, each with the key of its client rule: more returned
    // than the payer contributed (the employer's total is the W-2 box 12 code W, none here), later
    // deposits above what the corrections left, two transfers in one month and a last-month
    // history that is not eligible in December.
    refused(
        &|a| {
            a.corrections = vec![hsa_correction(2026, 20260930, dollars(5001), 0).page()];
        },
        "HSA_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
    );
    refused(
        &|a| {
            let mut c = hsa_correction(2026, 20260930, dollars(100), 0);
            c.payer = HsaCorrectionPayer::Employer { in_wages: true };
            a.corrections = vec![c.page()];
        },
        "HSA_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
    );
    let later = |total: i64, employer: i64| hsa_excess_accounting(0, None, total, employer, 0);
    refused(
        &|a| a.excess_accounting = Some(later(dollars(200), dollars(150))),
        "HSA_LATER_EMPLOYER_ABOVE_NET",
    );
    refused(
        &|a| {
            a.corrections = vec![hsa_correction(2026, 20260930, dollars(600), 0).page()];
            a.excess_accounting = Some(later(dollars(4401), 0));
        },
        "HSA_LATER_PERSONAL_ABOVE_NET",
    );
    refused(
        &|a| {
            a.funding_history = vec![
                hsa_funding(20250617, 20250801, dollars(4000)),
                hsa_funding(20250620, 20250801, dollars(1000)),
            ];
        },
        "HSA_FUNDING_HISTORY_SAME_MONTH",
    );
    refused(
        &|a| {
            let mut history = last_month_history(12, dollars(3000));
            history.months[11] = HsaCoverage::None;
            a.last_month_history = Some(history);
        },
        "HSA_LAST_MONTH_DECEMBER_NEEDED",
    );
    // What no page rule states stays the generic record problem: a withdrawal dated outside 2026.
    refused(
        &|a| {
            a.late_excess = vec![HsaLateExcessWithdrawal {
                contribution_year: 2024,
                deadline: 20250415,
                date: 20270101,
                amount: 100,
                disabled: false,
            }];
        },
        "HSA_RECORD_INVALID",
    );
    // The transfers: a row's rules, and an owner with no account.
    let transfer = |loss: Option<HsaEligibilityLoss>, coverage| HsaFundingRow {
        ongoing_sep_simple: false,
        owner: Owner::Taxpayer,
        source: 2,
        funded: 20260115,
        amount: dollars(6000),
        basis: HsaFundingBasis::Pretax { distributed: 20260115 },
        coverage,
        loss,
    };
    let with_transfer = |row: HsaFundingRow| {
        health_account_page(vec![account()], vec![row], ArcherContributionsPage::None)
    };
    let early = Some(HsaEligibilityLoss { date: 20260114, reason: HsaLossReason::Other });
    assert_eq!(
        key_of(send(&mut s, hsa_save(&d, with_transfer(transfer(early, HsaCoverage::SelfOnly))))),
        "HSA_FUNDING_LOSS_BEFORE_FUNDING"
    );
    assert_eq!(
        key_of(send(&mut s, hsa_save(&d, with_transfer(transfer(None, HsaCoverage::None))))),
        "HSA_FUNDING_NEEDS_COVERAGE"
    );
    let mut nobody = with_transfer(transfer(None, HsaCoverage::SelfOnly));
    nobody.funding[0].owner = Owner::Spouse;
    nobody.accounts[0].owner = Owner::Taxpayer;
    assert_eq!(key_of(send(&mut s, hsa_save(&d, nobody))), "HSA_FUNDING_NEEDS_AN_ACCOUNT");
    // A page with none of these saves, and the stored account carries no leftover.
    accepted(&send(&mut s, hsa_save(&d, page(account()))));
    let stored = &s.draft.adjustments.as_ref().unwrap().hsa;
    assert_eq!(stored.len(), 1);
    assert!(stored[0].funding.is_empty() && stored[0].corrections.is_empty());
}

fn archer_page(owners: Vec<ArcherContributorRow>) -> ArcherContributionsPage {
    ArcherContributionsPage::Records { owners, sharing: ArcherSharing::Equal }
}

// The Archer MSA contributions (Form 8853 Part I): one row for each owner with the compensation the
// plan covers chosen by identity; they save, reopen and calculate.
//   Self-only plan with a $3,000 deductible, personal deposits 1,000 into an MSA of a W-2 holder:
//   the deduction is 1,000 (65% of the deductible is $1,950).
#[test]
fn archer_contributions_save_through_the_hsa_page_and_reopen() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    let row = archer_contributor_row(
        Owner::Taxpayer,
        dollars(1000),
        0,
        ArcherEligibility::FullYear { coverage: archer_self(3000) },
        vec![1],
    );
    let request = hsa_save(&d, health_account_page(vec![], vec![], archer_page(vec![row.clone()])));
    accepted(&send(&mut s, request));
    let stored = &s.draft.adjustments.as_ref().unwrap().archer;
    assert_eq!(
        stored,
        &ArcherContributions::Records {
            owners: vec![archer_contributor(Owner::Taxpayer, archer_self(3000), 1000)],
            sharing: ArcherSharing::Equal,
        }
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "archer_msa_deduction"), 1000);
    assert_eq!(value(&r, "agi"), 59_000);
    let reopened = hsa_page_round_trips(s.draft.clone());
    assert_eq!(reopened.draft.adjustments, s.draft.adjustments);
    // Two rows for one owner are refused with the page's key, and a correction that returns before
    // it was deposited with the correction's.
    let twice = hsa_save(
        &s.draft,
        health_account_page(vec![], vec![], archer_page(vec![row.clone(), row.clone()])),
    );
    assert_eq!(key_of(send(&mut s, twice)), "ARCHER_OWNER_TWICE");
    let mut early = row.clone();
    early.corrections = vec![
        ArcherCorrection {
            failed_deposit: None,
            document: None,
            year: 2026,
            contributed_on: 20260315,
            principal: dollars(100),
            earnings: 0,
            returned: 20260314,
            deadline: 20270415,
            payer: ArcherCorrectionPayer::Personal,
        }
        .page(),
    ];
    let refused = hsa_save(&s.draft, health_account_page(vec![], vec![], archer_page(vec![early])));
    assert_eq!(key_of(send(&mut s, refused)), "ARCHER_CORRECTION_RETURNED_BEFORE_DEPOSIT");
    // Taking the rows away is "None": the stored record goes.
    let none =
        hsa_save(&s.draft, health_account_page(vec![], vec![], ArcherContributionsPage::None));
    accepted(&send(&mut s, none));
    assert_eq!(s.draft.adjustments.as_ref().unwrap().archer, ArcherContributions::None);
}

// The sums across an owner's records and the compensation List host, each refused on save with its
// own key: more returned than the payer contributed, later deposits above what the corrections left,
// and a compensation element that names no W-2, Form 8919 or business of the return (a stale one)
// or names one twice. Nothing is dropped or remapped.
#[test]
fn archer_sums_and_the_compensation_host_are_checked_again_on_the_server() {
    let d = complete(FederalStatus::Single);
    let row = archer_contributor_row(
        Owner::Taxpayer,
        dollars(1000),
        dollars(500),
        ArcherEligibility::FullYear { coverage: archer_self(3000) },
        vec![1],
    );
    let correction = |payer, principal| {
        ArcherCorrection {
            failed_deposit: None,
            document: None,
            year: 2026,
            contributed_on: 20260315,
            principal,
            earnings: 0,
            returned: 20260801,
            deadline: 20270415,
            payer,
        }
        .page()
    };
    let employer = ArcherCorrectionPayer::Employer { in_wages: true };
    let later =
        |total: i64, employer: i64| Some(archer_excess_accounting(0, None, total, employer, 0));
    let mut s = Interview::new(TEST_USER, d.clone());
    let mut refused = |mutate: &dyn Fn(&mut ArcherContributorRow), key: &str| {
        let mut r = row.clone();
        mutate(&mut r);
        let request = hsa_save(&d, health_account_page(vec![], vec![], archer_page(vec![r])));
        assert_eq!(key_of(send(&mut s, request)), key, "{key}");
        assert_eq!(s.draft, d);
    };
    refused(
        &|r| r.corrections = vec![correction(ArcherCorrectionPayer::Personal, dollars(1001))],
        "ARCHER_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED",
    );
    refused(
        &|r| r.corrections = vec![correction(employer.clone(), dollars(501))],
        "ARCHER_CORRECTED_EMPLOYER_ABOVE_CONTRIBUTED",
    );
    refused(
        &|r| r.excess_accounting = later(dollars(600), dollars(501)),
        "ARCHER_LATER_EMPLOYER_ABOVE_NET",
    );
    refused(
        &|r| {
            r.corrections = vec![correction(ArcherCorrectionPayer::Personal, dollars(200))];
            r.excess_accounting = later(dollars(801), 0);
        },
        "ARCHER_LATER_PERSONAL_ABOVE_NET",
    );
    // A stale identity (no such entry in the return) and a repeat.
    refused(&|r| r.income = vec![99], "ARCHER_INCOME_NOT_OFFERED");
    refused(&|r| r.income = vec![1, 99], "ARCHER_INCOME_NOT_OFFERED");
    refused(&|r| r.income = vec![1, 1], "ARCHER_INCOME_TWICE");
    // The empty host is a complete answer (a coverage-only row), and the offered entry is kept as chosen.
    let mut coverage_only = row.clone();
    coverage_only.income = vec![];
    let request =
        hsa_save(&d, health_account_page(vec![], vec![], archer_page(vec![coverage_only])));
    accepted(&send(&mut s, request));
}

// The Archer corrections, prior employer excess and excess accounting round trip.
#[test]
fn archer_corrections_and_excess_accounting_save_and_reopen() {
    let mut d = complete(FederalStatus::Single);
    let mut owner = archer_contributor(Owner::Taxpayer, archer_self(4000), 3000);
    owner.excess_accounting = Some(archer_excess_accounting(30_000, None, 10_000, 0, 0));
    owner.corrections = vec![ArcherCorrection {
        failed_deposit: None,
        document: None,
        year: 2026,
        contributed_on: 20260315,
        principal: 100_000,
        earnings: 5000,
        returned: 20260801,
        deadline: 20270415,
        payer: ArcherCorrectionPayer::Personal,
    }];
    owner.prior_employer_excess = vec![ArcherPriorEmployerExcessDeposit {
        contributed_on: 20260301,
        deadline: 20260415,
        amount: 20_000,
        in_wages: false,
    }];
    set_archer(&mut d, vec![owner], ArcherSharing::Equal);
    hsa_page_round_trips(d);
}

// ---- ABLE contributions (Form 5329 Part VIII) ----

fn able_item(owner: Owner, own: i64, other: i64, year_end: i64) -> IncomeDocumentItem {
    item(
        owner,
        0,
        IncomeDocument::AbleContributions(AbleContributionsPage {
            own: dollars(own),
            other: dollars(other),
            work: AbleWork::No,
            year_end: dollars(year_end),
            additional_open: false,
        }),
    )
}

// Own 8,000 and others 20,000 are 28,000 against the $20,000 limit: excess 8,000, 6% tax capped by
// the 30,000 year-end value = 480 (Schedule 2 line 8, Form 5329 Part VIII).
#[test]
fn able_contributions_save_through_the_other_income_page() {
    let d = complete(FederalStatus::Single);
    let s = save_items(&d, vec![able_item(Owner::Taxpayer, 8000, 20_000, 30_000)]);
    assert_eq!(
        s.draft.screening.as_ref().unwrap().able_contributions,
        vec![AbleContributions {
            owner: Owner::Taxpayer,
            own: dollars(8000),
            other: dollars(20_000),
            work: AbleWork::No,
            year_end: dollars(30_000),
            additional_open: false,
        }]
    );
    let r = supported(&s.draft);
    assert_eq!(value(&r, "able_taxpayer_contribution_limit"), 20_000);
    assert_eq!(value(&r, "able_taxpayer_excess_contributions"), 8000);
    assert_eq!(value(&r, "able_taxpayer_excess_tax"), 480);
    // It reopens as entered and saves unchanged.
    let mut again = restart(s.draft.clone());
    let page = open_topic(&mut again, TaxSection::OtherIncome);
    let shown = page_documents(&page).to_vec();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].owner, Owner::Taxpayer);
    let draft = again.draft.clone();
    accepted(&send(&mut again, save_topic(&draft, TaxSection::OtherIncome, shown)));
    assert_eq!(again.draft.screening, s.draft.screening);
    // Unticking the topic erases it with the other income kinds.
    let mut erased = s.draft.clone();
    erased.checklist.income &= !IncomeTopics::OTHER_INCOME.0;
    crate::interview::prune(&mut erased);
    assert!(erased.screening.unwrap().able_contributions.is_empty());
}

// One record for each beneficiary; your spouse's only on a joint return.
#[test]
fn able_contributions_one_for_each_beneficiary_and_a_spouse_only_when_joint() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d.clone());
    let twice = save_topic(
        &d,
        TaxSection::OtherIncome,
        vec![able_item(Owner::Taxpayer, 1000, 0, 1000), able_item(Owner::Taxpayer, 1000, 0, 1000)],
    );
    assert_eq!(key_of(send(&mut s, twice)), "ABLE_CONTRIBUTIONS_TWICE");
    let spouse =
        save_topic(&d, TaxSection::OtherIncome, vec![able_item(Owner::Spouse, 1000, 0, 1000)]);
    assert!(matches!(
        current_income_refusal(
            "income-source-1",
            &mut s,
            spouse,
            "MESSAGE_ABLE_CONTRIBUTIONS_YOUR_SPOUSE_S_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTLY",
            "[documents/documents/0/owner]"
        ),
        Reply::Response(P::Problem { .. })
    ));
    let joint = complete(FederalStatus::Joint);
    let both = save_items(
        &joint,
        vec![able_item(Owner::Taxpayer, 1000, 0, 1000), able_item(Owner::Spouse, 21_000, 0, 5000)],
    );
    let r = supported(&both.draft);
    // The spouse's 21,000 is 1,000 over the limit: 6% of the lesser of 1,000 and 5,000.
    assert_eq!(value(&r, "able_spouse_excess_contributions"), 1000);
    assert_eq!(value(&r, "able_spouse_excess_tax"), 60);
    assert_eq!(value(&r, "able_taxpayer_excess_tax"), 0);
    // A working beneficiary's compensation allowance: the page's work choice keeps its facts.
    let mut working = able_item(Owner::Taxpayer, 8000, 0, 30_000);
    if let IncomeDocument::AbleContributions(v) = &mut working.document {
        v.work = AbleWork::Working {
            compensation: dollars(10_000),
            residence: MarketplaceResidence::Contiguous,
            no_plan: true,
        };
    }
    let kept = save_items(&d, vec![working.clone()]);
    let mut again = restart(kept.draft.clone());
    let page = open_topic(&mut again, TaxSection::OtherIncome);
    assert_eq!(page_documents(&page)[0].document, working.document);
}

// Derived dated-source case from the 2026 draft i8853, printed Sep21,2026,
// printed pp6–7, Lines9a/9b Example3 and Lines13a/13b exceptions:
// https://www.irs.gov/pub/irs-dft/i8853--dft.pdf . The source describes the
// birthday split, without these dollar amounts. Synthetic completion: the
// ordinary60000 wage household, holder born1961-06-15, one real statement,
// medical500 and dated nonmedical1000/2000/3000 on June14/15/16.
#[test]
fn sa1099_one_statement_dates_each_age_or_disability_exception() {
    for medicare in [false, true] {
        let mut d = complete(FederalStatus::Single);
        born(&mut d, 19610615);
        let document = IncomeDocument::Sa1099(Sa1099 {
            origin: crate::HealthDocumentOrigin::Form1099Sa,
            gross: dollars(6500),
            account: if medicare { ma_none() } else { MedicalSaAccount::Archer },
            payment: MedicalSaPayment::Own {
                nonmedical: vec![
                    MedicalAccountWithdrawal { date: 20260614, amount: dollars(1000) },
                    MedicalAccountWithdrawal { date: 20260615, amount: dollars(2000) },
                    MedicalAccountWithdrawal { date: 20260616, amount: dollars(3000) },
                ],
                medical: dollars(500),
                rollover: None,
                disabled_since: medicare.then_some(20260615),
            },
        });
        let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, document.clone())]);
        let r = supported(&s.draft);
        // Archer birthday itself remains taxed:20% of1000+2000=600.
        // MA disability begins on its date:50% of1000=500; age is no exception.
        let kind = if medicare { "medicare" } else { "archer" };
        assert_eq!(value(&r, &alloc::format!("{kind}_msa_taxable_distributions")), 6000);
        assert_eq!(
            value(&r, &alloc::format!("{kind}_msa_additional_tax")),
            if medicare { 500 } else { 600 }
        );
        assert_eq!(s.draft.income.len(), 2); // One wage plus one actual statement.
        let id = s.draft.income[1].id;
        let page = open_topic(&mut s, TaxSection::OtherIncome);
        assert_eq!(page_documents(&page)[0].document, document);
        assert_eq!(page_documents(&page)[0].preview, Some(dollars(6000)));
        accepted(&send(&mut s, page));
        assert_eq!(s.draft.income[1].id, id);
        let mut restarted = restart(s.draft.clone());
        let page = open_topic(&mut restarted, TaxSection::OtherIncome);
        assert_eq!(page_documents(&page)[0].document, document);
        assert_eq!(supported(&restarted.draft), supported(&s.draft));
    }
}

#[test]
fn sa1099_dated_uses_must_reconcile_without_consuming_identity() {
    let d = complete(FederalStatus::Single);
    let data = FederalMedicalDistribution {
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: dollars(1000),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Own {
            medical: dollars(500),
            nonmedical: vec![MedicalAccountWithdrawal { date: 20260615, amount: dollars(400) }],
            rollover: None,
            disabled_since: None,
        },
    };
    assert_eq!(data.rule(), Some("SA1099_USES_NOT_GROSS"));
    assert_eq!(data.preview(), None);
    let mut s = Interview::new(TEST_USER, d.clone());
    let items = vec![item(Owner::Taxpayer, 0, IncomeDocument::Sa1099(data.page()))];
    let reply = send(&mut s, save_topic(&d, TaxSection::OtherIncome, items));
    assert_eq!(key_of(reply), "SA1099_USES_NOT_GROSS");
    assert_eq!(s.draft, d);
}

// IRC220(e)(2),223(e)(2), incorporating408(e)(4); Pub9692025 final,
// Other distributions. Derived source fixture: two actual collateral pledges,
// June14 1000 and June16 2000, with the owner's65th birthday June15.
// Neutral completion is the standard60000-wage single return. No Form1099-SA
// boxes are invented for the unreported event.
#[test]
fn archer_unreported_pledges_use_actual_statement_and_event_dates() {
    let mut d = complete(FederalStatus::Single);
    born(&mut d, 19610615);
    let mut data = FederalMedicalDistribution {
        origin: HealthDocumentOrigin::TrusteeStatement,
        gross: dollars(3000),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::Unreported {
            event: MedicalDeemed::Pledged {
                assets: vec![
                    MedicalAccountWithdrawal { date: 20260614, amount: dollars(1000) },
                    MedicalAccountWithdrawal { date: 20260616, amount: dollars(2000) },
                ],
            },
            disabled_since: None,
        },
    };
    let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, IncomeDocument::Sa1099(data.page()))]);
    let saved_id = s.draft.income.last().unwrap().id;
    let r = supported(&s.draft);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 3000);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 200);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
    let page = open_topic(&mut s, TaxSection::OtherIncome);
    let reopened = &page_documents(&page)[0];
    assert_eq!(reopened.id, saved_id);
    assert_eq!(reopened.document, IncomeDocument::Sa1099(data.page()));
    data.origin = HealthDocumentOrigin::Form1099Sa;
    assert_eq!(data.rule(), Some("SA1099_UNREPORTED_SOURCE_INVALID"));
    assert!(data.amounts(&d.people.as_ref().unwrap().taxpayer).is_err());
    let before = s.draft.clone();
    let reply = send(
        &mut s,
        save_topic(
            &before,
            TaxSection::OtherIncome,
            vec![item(Owner::Taxpayer, saved_id, IncomeDocument::Sa1099(data.page()))],
        ),
    );
    assert_eq!(key_of(reply), "SA1099_UNREPORTED_SOURCE_INVALID");
    assert_eq!(s.draft, before);
    data.origin = HealthDocumentOrigin::TrusteeStatement;
    data.gross += 100;
    assert_eq!(data.rule(), Some("SA1099_EVENT_VALUE_NOT_GROSS"));
    add(&mut d, income(FederalIncome::MedicalDistribution { data }));
    assert!(issue(&d, "SA1099_EVENT_VALUE_NOT_GROSS"));
}

// Instructions for Form 8889, 2026 draft, printed July 21, 2026,
// printed pages 4–5, 8: https://www.irs.gov/pub/irs-dft/i8889--dft.pdf.
// Instructions for Form 1099-SA/5498-SA, revision December 2026,
// boxes 1–2/code 2: https://www.irs.gov/instructions/i1099sa.
// Derived custodian fixtures: $1,000 original principal plus $100 earnings
// returns $1,100 cash; a $100 loss returns $900. Neutral completion: a
// single owner born in 1980, $60,000 wages and self-only HSA eligibility
// all year. The statement and corrective detail describe one source.
#[test]
fn own_hsa_statement_saves_once_without_a_contribution_account() {
    let d = complete(FederalStatus::Single);
    let document = IncomeDocument::Sa1099(Sa1099 {
        origin: HealthDocumentOrigin::Form1099Sa,
        gross: dollars(5000),
        account: MedicalSaAccount::Hsa,
        payment: own_payment(5000, 20260615, 4000),
    });
    let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, document.clone())]);
    assert!(s.draft.adjustments.as_ref().unwrap().hsa.is_empty());
    assert_eq!(s.draft.income.len(), 2);
    let id = s.draft.income[1].id;
    let r = supported(&s.draft);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 0);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(return_line(&r, "23"), 20000);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
    let page = open_topic(&mut s, TaxSection::OtherIncome);
    let rows = page_documents(&page);
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].document, document);
    let before = s.draft.clone();
    accepted(&send(&mut s, save_topic(&before, TaxSection::OtherIncome, rows.to_vec())));
    assert_eq!(s.draft.income.len(), 2);
    assert_eq!(s.draft.income[1].id, id);
    assert_eq!(value(&supported(&s.draft), "hsa_taxable_distributions"), 1000);
    let mut claimed = s.draft.clone();
    claimed.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    assert_eq!(value(&supported(&claimed), "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&supported(&claimed), "hsa_additional_tax"), 200);
}
// Instructions8889 2026 draft (printedJuly21,2026), printedpp4–5,8,
// https://www.irs.gov/pub/irs-dft/i8889--dft.pdf; Instructions1099-SA/5498-SA,
// revisionDecember2026, Boxes1–2 and code2,
// https://www.irs.gov/instructions/i1099sa. Derived custodian fixtures:
// 1000 original principal +100 earnings=1100cash, or -100loss=900cash.
// Neutral completion: single1980born taxpayer,60000wages, self-only HSA
// eligibility all year. The statement and corrective detail are one source.
#[test]
fn hsa_code2_actual_statement_links_signed_correction_once() {
    for (earnings, printed) in [(100, Some(dollars(100))), (-100, None)] {
        let d = complete(FederalStatus::Single);
        let source = IncomeDocument::Sa1099(Sa1099 {
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: dollars(1000 + earnings),
            account: MedicalSaAccount::Hsa,
            payment: MedicalSaPayment::ExcessReturned { earnings: printed },
        });
        let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, source.clone())]);
        let id = s.draft.income[1].id;
        assert!(issue(&s.draft, "HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED"));
        let mut request = open_hsa_page(&mut s);
        let P::SaveHsa { value: page, hsa_return_documents, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(hsa_return_documents.len(), 1);
        assert_eq!(hsa_return_documents[0].id, id);
        assert_eq!(hsa_return_documents[0].gross, dollars(1000 + earnings));
        assert_eq!(hsa_return_documents[0].earnings, printed);
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, dollars(1000));
        let mut correction =
            hsa_correction(2026, 20260930, dollars(1000), dollars(earnings)).page();
        correction.document = Some(id);
        account.corrections.push(correction);
        page.accounts = vec![account];
        // Echoed catalogs are never authority for the current source.
        hsa_return_documents.clear();
        accepted(&send(&mut s, request));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "hsa_1_distribution_gross"), 1000 + earnings);
        assert_eq!(value(&r, "hsa_1_distribution_excluded"), 1000 + earnings);
        assert_eq!(value(&r, "hsa_1_distribution_net"), 0);
        assert_eq!(value(&r, "hsa_1_corrected_contributions"), 1000);
        assert_eq!(value(&r, "adjustments"), 0);
        assert_eq!(value(&r, "hsa_correction_earnings"), earnings.max(0));
        assert_eq!(value(&r, "hsa_taxable_distributions"), 0);
        assert_eq!(value(&r, "hsa_additional_tax"), 0);
        assert_eq!(value(&r, "agi"), 60000 + earnings.max(0));
        assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
        let page = open_topic(&mut s, TaxSection::OtherIncome);
        assert_eq!(page_documents(&page)[0].id, id);
        assert_eq!(page_documents(&page)[0].document, source);
        let request = open_hsa_page(&mut s);
        let before = s.draft.clone();
        accepted(&send(&mut s, request));
        assert_eq!(s.draft, before);
    }
}

#[test]
fn hsa_code2_correction_source_guards_owner_amounts_and_stale_identity() {
    let d = complete(FederalStatus::Single);
    let data = Sa1099 {
        origin: HealthDocumentOrigin::Form1099Sa,
        gross: dollars(1100),
        account: MedicalSaAccount::Hsa,
        payment: MedicalSaPayment::ExcessReturned { earnings: Some(dollars(100)) },
    };
    let mut s =
        save_items(&d, vec![item(Owner::Taxpayer, 0, IncomeDocument::Sa1099(data.clone()))]);
    let id = s.draft.income[1].id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: page, .. } = &mut request else { unreachable!() };
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, dollars(1000));
    let mut correction = hsa_correction(2026, 20260930, dollars(1000), dollars(100)).page();
    correction.document = Some(id);
    account.corrections = vec![correction];
    page.accounts = vec![account];
    let good = request.clone();
    for (mutation, key) in [
        (0, "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE"),
        (1, "HEALTH_CORRECTION_DOCUMENT_GROSS"),
        (2, "HEALTH_CORRECTION_DOCUMENT_EARNINGS"),
    ] {
        let mut request = good.clone();
        let P::SaveHsa { value, .. } = &mut request else { unreachable!() };
        let c = &mut value.accounts[0].corrections[0];
        match mutation {
            0 => c.document = Some(id + 1000),
            1 => {
                c.earnings += 100;
            }
            _ => {
                c.principal -= 100;
                c.earnings += 100;
            }
        }
        let before = s.draft.clone();
        assert_eq!(key_of(send(&mut s, request)), key);
        assert_eq!(s.draft, before);
    }
    accepted(&send(&mut s, good));
    let before = s.draft.clone();
    let removal = save_topic(&before, TaxSection::OtherIncome, vec![]);
    assert_eq!(key_of(send(&mut s, removal)), "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE");
    assert_eq!(s.draft, before);
    let mut changed = data;
    changed.gross += 100;
    accepted(&send(
        &mut s,
        save_topic(
            &before,
            TaxSection::OtherIncome,
            vec![item(Owner::Taxpayer, id, IncomeDocument::Sa1099(changed))],
        ),
    ));
    assert!(issue(&s.draft, "HEALTH_CORRECTION_DOCUMENT_GROSS"));
    // Amount changes preserve identity and reopen review rather than trapping
    // the user between two source editors.
    assert_eq!(s.draft.income[1].id, id);
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: page, .. } = &mut request else { unreachable!() };
    page.accounts[0].personal += 100;
    page.accounts[0].corrections[0].principal += 100;
    accepted(&send(&mut s, request));
    assert_eq!(value(&supported(&s.draft), "hsa_correction_earnings"), 100);
    let before = s.draft.clone();
    let mut checklist = before.checklist;
    checklist.income &= !IncomeTopics::OTHER_INCOME.0;
    assert_eq!(
        key_of(send(
            &mut s,
            crate::tests::cross::interview::save_checklist(&before, PAGE_ADJUSTMENTS, checklist)
        )),
        "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE"
    );
    assert_eq!(s.draft, before);
    checklist.adjustments &= !AdjustmentTopics::HSA.0;
    accepted(&send(
        &mut s,
        crate::tests::cross::interview::save_checklist(&before, PAGE_ADJUSTMENTS, checklist),
    ));
    assert!(
        s.draft
            .income
            .iter()
            .all(|e| !matches!(e.income, FederalIncome::MedicalDistribution { .. }))
    );
    assert!(s.draft.adjustments.as_ref().unwrap().hsa.is_empty());
}

#[test]
fn hsa_code2_one_statement_can_detail_multiple_contribution_years_and_payers() {
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeDocument::Sa1099(Sa1099 {
                origin: HealthDocumentOrigin::Form1099Sa,
                gross: dollars(1650),
                account: MedicalSaAccount::Hsa,
                payment: MedicalSaPayment::ExcessReturned { earnings: Some(dollars(150)) },
            }),
        )],
    );
    let id = s.draft.income[1].id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: page, .. } = &mut request else { unreachable!() };
    let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, dollars(1000));
    let mut current = hsa_correction(2026, 20260930, dollars(1000), dollars(100)).page();
    current.document = Some(id);
    let mut prior = hsa_correction(2025, 20260330, dollars(500), dollars(50)).page();
    prior.document = Some(id);
    // Prior-year employer principal already in its actual W-2 wages creates
    // no fabricated current wages or compensation.
    prior.payer = HsaCorrectionPayer::Employer { in_wages: true };
    account.corrections = vec![current, prior];
    page.accounts = vec![account];
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "hsa_correction_earnings"), 150);
    assert_eq!(value(&r, "agi"), 60150);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
}
// Instructions for Form 8853, 2026 draft, printed September 21, 2026,
// printed pages 4–6: https://www.irs.gov/pub/irs-dft/i8853--dft.pdf.
// Instructions for Form 1099-SA, revision December 2026, boxes 1–2/code 2:
// https://www.irs.gov/instructions/i1099sa. Derived fixture: a self-only
// $4,000 deductible gives $2,600 capacity; $3,000 original contributions
// less $400 timely returned principal leaves $2,600. Actual NIA is +$20
// with $420 cash, or −$20 with $380 cash. Neutral completion: joint filers
// born in 1980, taxpayer $60,000 wages from accepted source 1, spouse
// $40,000 wages, and no other deposits.
#[test]
fn archer_code2_source_and_corrective_detail_count_income_once() {
    for earnings in [20, -20] {
        let d = complete(FederalStatus::Joint);
        let printed = (earnings > 0).then_some(dollars(earnings));
        let source = IncomeDocument::Sa1099(Sa1099 {
            origin: HealthDocumentOrigin::Form1099Sa,
            gross: dollars(400 + earnings),
            account: MedicalSaAccount::Archer,
            payment: MedicalSaPayment::ExcessReturned { earnings: printed },
        });
        let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, source.clone())]);
        let id = s.draft.income.last().unwrap().id;
        if earnings < 0 {
            assert!(issue(&s.draft, "HEALTH_CORRECTION_DOCUMENT_DETAILS_REQUIRED"));
        } else {
            // Existing standalone printed-income treatment remains available.
            assert_eq!(value(&supported(&s.draft), "agi"), 100020);
        }
        let mut request = open_hsa_page(&mut s);
        let P::SaveHsa { value: page, archer_return_documents, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(archer_return_documents.len(), 1);
        assert_eq!(archer_return_documents[0].id, id);
        let row = ArcherContributorRow {
            corrections: vec![ArcherCorrectionPage {
                failed_deposit: None,
                document: Some(id),
                year: 2026,
                contributed_on: 20260601,
                principal: dollars(400),
                earnings: dollars(earnings),
                returned: 20260930,
                deadline: 20270415,
                payer: ArcherCorrectionPayer::Personal,
            }],
            ..archer_contributor_row(
                Owner::Taxpayer,
                dollars(3000),
                0,
                ArcherEligibility::FullYear { coverage: archer_self(4000) },
                vec![1],
            )
        };
        page.archer = archer_page(vec![row]);
        archer_return_documents.clear();
        accepted(&send(&mut s, request));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "archer_msa_deduction"), 2600);
        assert_eq!(value(&r, "archer_1_corrected_contributions"), 400);
        assert_eq!(value(&r, "archer_correction_earnings"), earnings.max(0));
        assert_eq!(value(&r, &alloc::format!("archer_msa_{id}_excess_earnings")), 0);
        assert_eq!(value(&r, "agi"), 97400 + earnings.max(0));
        assert_eq!(value(&r, "archer_msa_additional_tax"), 0);
        assert_eq!(estimate(&restart(s.draft.clone()).draft), r);
        let request = open_hsa_page(&mut s);
        let before = s.draft.clone();
        accepted(&send(&mut s, request));
        assert_eq!(s.draft, before);
        assert_eq!(
            page_documents(&open_topic(&mut s, TaxSection::OtherIncome))[0].document,
            source
        );
        let changed_owner = save_topic(
            &before,
            TaxSection::OtherIncome,
            vec![item(Owner::Spouse, id, source.clone())],
        );
        assert_eq!(key_of(send(&mut s, changed_owner)), "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE");
        assert_eq!(s.draft, before);
        let request = save_topic(&before, TaxSection::OtherIncome, vec![]);
        assert_eq!(key_of(send(&mut s, request)), "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE");
        assert_eq!(s.draft, before);
        let mut bad = s.draft.clone();
        let ArcherContributions::Records { owners, .. } =
            &mut bad.adjustments.as_mut().unwrap().archer
        else {
            unreachable!()
        };
        owners[0].corrections[0].document = Some(id + 999);
        assert!(issue(&bad, "HEALTH_CORRECTION_DOCUMENT_UNAVAILABLE"));
    }
}

// Reg1.105-1(d)(1), https://www.ecfr.gov/current/title-26/section-1.105-1,
// current text reopened2026-10-11: published annual premium24 TOTAL,16employer,
//8employee =>2/3employer. Derived900benefit,100eligible reimbursement =>500
//income;25separately paid interest =>AGI60525. Neutral completion: actual
//complete single1980born/60000W2 return, no prior deduction recovery or wage-
//continuation payment. The source is actual accident/health LTC insurance.
#[test]
fn ltc_nonqualified_actual_funding_and_interest_round_trip() {
    let d = complete(FederalStatus::Single);
    let make = |funding, medical_reimbursement, permanent_injury| Ltc1099 {
        benefits: dollars(900),
        accelerated: 0,
        interest: dollars(25),
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        paid: LtcPaidBasis::PerDiem,
        contract: LtcContractPage::Nonqualified {
            treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
                funding,
                medical_reimbursement,
                permanent_injury,
                recovery: None,
            },
        },
        illness: LtcIllness::Chronic,
        insured: None,
    };
    let individual = CareBenefitFunding::Individual {
        premiums: CareFundingAmounts {
            employee_premium: dollars(8),
            employer_premium: dollars(16),
        },
    };
    let form = make(individual.clone(), dollars(100), None);
    let mut s =
        save_items(&d, vec![item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(form.clone()))]);
    let r = supported(&s.draft);
    let id = s.draft.income.last().unwrap().id;
    assert_eq!(value(&r, "taxable_ltc_benefits"), 500);
    assert_eq!(value(&r, &alloc::format!("ltc_benefit_{id}_nonqualified_excluded")), 400);
    assert_eq!(return_line(&r, "2b"), dollars(25));
    assert_eq!(return_line(&r, "8"), dollars(500));
    assert_eq!(value(&r, "agi"), 60_525);
    assert!(!noted(&r, "WARN_LTC_PERIODS_MISSING"));
    assert_eq!(
        page_documents(&open_topic(&mut s, TaxSection::OtherIncome))[0].document,
        IncomeDocument::Ltc1099(form)
    );
    assert_eq!(supported(&restart(s.draft.clone()).draft), r);
    let before = s.draft.clone();
    let unchanged = open_topic(&mut s, TaxSection::OtherIncome);
    accepted(&send(&mut s, unchanged));
    assert_eq!(s.draft, before);
    for (funding, medical, injury, expected) in [
        (CareBenefitFunding::AfterTax, 0, None, 0),
        (CareBenefitFunding::EmployerOnly, dollars(100), None, 800),
        (
            individual.clone(),
            dollars(100),
            Some(CarePermanentInjury {
                amount: dollars(200),
                kind: CareInjuryKind::PermanentMemberLoss,
                computed_by_injury: true,
            }),
            300,
        ),
        (
            individual,
            dollars(100),
            Some(CarePermanentInjury {
                amount: dollars(200),
                kind: CareInjuryKind::PermanentMemberLoss,
                computed_by_injury: false,
            }),
            500,
        ),
    ] {
        let s = save_items(
            &d,
            vec![item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(make(funding, medical, injury)))],
        );
        assert_eq!(value(&supported(&s.draft), "taxable_ltc_benefits"), expected);
    }
    let bad = make(CareBenefitFunding::AfterTax, dollars(1), None);
    assert_eq!(bad.record().rule(), Some("LTC1099_EXCLUDED_PORTIONS_ABOVE_EMPLOYER"));
    let kept = s.draft.clone();
    assert!(matches!(
        send(
            &mut s,
            save_topic(
                &kept,
                TaxSection::OtherIncome,
                vec![item(Owner::Taxpayer, id, IncomeDocument::Ltc1099(bad))]
            )
        ),
        Reply::Edit(P::SaveIncomes { .. }) | Reply::Response(P::Problem { .. })
    ));
    assert_eq!(s.draft, kept);
}

// Reg1.105-1(d)(2)'s published group premiums8000/9000/7000 and employer
//3000/3500/1500 give8000/24000=1/3. Original1955timeline uniformly shifted
//to2026 (latest known years2024/2023/2022), preserving the missing newest
//policy-year adjustment. Derived900benefit =>300income, not the average of
//three individual annual ratios. First-year cases are source-derived rule
//tests, not published dollar examples. Same neutral household as above.
#[test]
fn ltc_group_latest_known_years_and_first_year_methods() {
    let histories = [
        (
            CareBenefitFunding::Group {
                years: vec![
                    CareFundingYear {
                        year: 2024,
                        premiums: CareFundingAmounts {
                            employee_premium: dollars(5000),
                            employer_premium: dollars(3000),
                        },
                    },
                    CareFundingYear {
                        year: 2023,
                        premiums: CareFundingAmounts {
                            employee_premium: dollars(5500),
                            employer_premium: dollars(3500),
                        },
                    },
                    CareFundingYear {
                        year: 2022,
                        premiums: CareFundingAmounts {
                            employee_premium: dollars(5500),
                            employer_premium: dollars(1500),
                        },
                    },
                ],
                first_year: None,
            },
            300,
        ),
        (
            CareBenefitFunding::Group {
                years: vec![],
                first_year: Some(CareFirstYearFunding::Actual {
                    premiums: CareFundingAmounts {
                        employee_premium: dollars(80),
                        employer_premium: dollars(20),
                    },
                }),
            },
            180,
        ),
        (
            CareBenefitFunding::Group {
                years: vec![],
                first_year: Some(CareFirstYearFunding::Estimated {
                    premiums: CareFundingAmounts {
                        employee_premium: dollars(10),
                        employer_premium: dollars(30),
                    },
                }),
            },
            675,
        ),
    ];
    for (funding, expected) in histories {
        let form = Ltc1099 {
            benefits: dollars(900),
            accelerated: 0,
            interest: 0,
            benefit_origin: Some(LtcPaymentOrigin::Insurance),
            accelerated_origin: None,
            contract: LtcContractPage::Nonqualified {
                treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
                    funding,
                    medical_reimbursement: 0,
                    permanent_injury: None,
                    recovery: None,
                },
            },
            paid: LtcPaidBasis::PerDiem,
            illness: LtcIllness::Chronic,
            insured: None,
        };
        let s = save_items(
            &complete(FederalStatus::Single),
            vec![item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(form))],
        );
        assert_eq!(value(&supported(&s.draft), "taxable_ltc_benefits"), expected);
    }
    let history = CareBenefitFunding::Group { years: vec![], first_year: None };
    assert!(!history.valid());
    let repeated = CareFundingYear {
        year: 2025,
        premiums: CareFundingAmounts { employee_premium: 100, employer_premium: 100 },
    };
    assert!(
        !CareBenefitFunding::Group { years: vec![repeated.clone(), repeated], first_year: None }
            .valid()
    );
    assert!(
        !CareBenefitFunding::Individual {
            premiums: CareFundingAmounts { employee_premium: 0, employer_premium: 0 }
        }
        .valid()
    );
}

// IRC101(g)(1),(2),(5), https://www.irs.gov/publications/p525 Accelerated
//Death Benefits;2026draftf8853 printedp2, https://www.irs.gov/pub/irs-dft/f8853--dft.pdf.
// Derived900terminal viatical proceeds exclude only with actual provider
//qualification/no business insurable interest. A failed qualification leaves
//mandatory life-sale character unresolved, not fabricated900basis-free tax.
//Mixed box1900after-tax nonqualified insurance+box2500qualified chronic
//life advance uses only500in the one-day430SectionC limit =>70income.
#[test]
fn ltc_viatical_and_life_streams_do_not_borrow_contract_qualification() {
    let d = complete(FederalStatus::Single);
    let origin = LtcPaymentOrigin::Viatical {
        regular_business: true,
        regulation: ViaticalRegulation::StateLicense { licensed: true },
        business_insurable_interest: false,
    };
    let form = Ltc1099 {
        benefits: 0,
        accelerated: dollars(900),
        interest: 0,
        benefit_origin: None,
        accelerated_origin: Some(origin),
        paid: LtcPaidBasis::NotChecked,
        contract: LtcContractPage::Unknown,
        illness: LtcIllness::Terminal,
        insured: None,
    };
    let mut s =
        save_items(&d, vec![item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(form.clone()))]);
    assert_eq!(value(&supported(&s.draft), "taxable_ltc_benefits"), 0);
    let mut bad = form;
    if let Some(LtcPaymentOrigin::Viatical { business_insurable_interest, .. }) =
        &mut bad.accelerated_origin
    {
        *business_insurable_interest = true;
    }
    let id = s.draft.income.last().unwrap().id;
    let kept = s.draft.clone();
    accepted(&send(
        &mut s,
        save_topic(
            &kept,
            TaxSection::OtherIncome,
            vec![item(Owner::Taxpayer, id, IncomeDocument::Ltc1099(bad))],
        ),
    ));
    assert!(issue(&s.draft, "ISSUE_LTC_LIFE_CHARACTER_UNRESOLVED"));
    let mixed = Ltc1099 {
        benefits: dollars(900),
        accelerated: dollars(500),
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
            business_insurable_interest: false,
        }),
        paid: LtcPaidBasis::PerDiem,
        contract: LtcContractPage::Nonqualified {
            treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
                funding: CareBenefitFunding::AfterTax,
                medical_reimbursement: 0,
                permanent_injury: None,
                recovery: None,
            },
        },
        illness: LtcIllness::Chronic,
        insured: None,
    };
    let s = save_items(
        &d,
        vec![
            item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(mixed)),
            item(
                Owner::Taxpayer,
                0,
                IncomeDocument::LtcInsured(LtcInsured {
                    insured: LtcInsuredPerson::OnReturn { person: -1 },
                    periods: one_period(20260601, 20260601, 0, 0).into(),
                }),
            ),
        ],
    );
    assert_eq!(value(&supported(&s.draft), "taxable_ltc_benefits"), 70);
}

//2026draftf8853 printedp2 nonqualified caution;2025finalPub525 Recoveries
//Worksheet2 andPub575payer-determined taxable annuity, https://www.irs.gov/publications/p525
//and/p575. Derived actual-page900annuity/400payer-taxable payment named by
//the same LTC receipt remains400income once; a linked300prior medical
//recovery, prior deduction200 and1000itemizing excess, gives200income once.
//Neutral single1980born/60000W2 household, no separate unverified exclusion.
#[test]
fn ltc_actual_annuity_and_recovery_sources_keep_income_once_and_protect_ids() {
    let d = complete(FederalStatus::Single);
    let mut s = Interview::new(TEST_USER, d);
    let mut annuity =
        crate::tests::cross::interview::r1099(dollars(900), DistributionCode::Nonqualified, false);
    annuity.taxable = dollars(400);
    let kept = s.draft.clone();
    accepted(&send(
        &mut s,
        save_topic(
            &kept,
            TaxSection::Retirement,
            vec![item(Owner::Taxpayer, 0, IncomeDocument::R1099(annuity))],
        ),
    ));
    let source = s.draft.income.last().unwrap().id;
    let form = Ltc1099 {
        benefits: dollars(900),
        accelerated: 0,
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        paid: LtcPaidBasis::NotChecked,
        contract: LtcContractPage::Nonqualified {
            treatment: NonqualifiedCareTreatmentPage::LifeAnnuity { source },
        },
        illness: LtcIllness::Chronic,
        insured: None,
    };
    let page = open_topic(&mut s, TaxSection::OtherIncome);
    let P::SaveIncomes { annuity_sources, .. } = &page else { panic!() };
    assert!(
        annuity_sources
            .iter()
            .any(|v| v.id == source && v.owner == Owner::Taxpayer && v.gross == dollars(900))
    );
    let kept = s.draft.clone();
    accepted(&send(
        &mut s,
        save_topic(
            &kept,
            TaxSection::OtherIncome,
            vec![item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(form.clone()))],
        ),
    ));
    let result = supported(&s.draft);
    assert_eq!(value(&result, "agi"), 60_400);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
    assert!(noted(&result, "WARN_LTC_ANNUITY_EXCLUSION_UNAVAILABLE"));
    let kept = s.draft.clone();
    assert_eq!(
        key_of(send(&mut s, save_topic(&kept, TaxSection::Retirement, vec![]))),
        "LTC1099_SOURCE_UNAVAILABLE"
    );
    assert_eq!(s.draft, kept);
    assert_eq!(supported(&restart(kept.clone()).draft), result);
    let mut forged = kept;
    let FederalIncome::LtcBenefits { data } = &mut forged.income.last_mut().unwrap().income else {
        panic!()
    };
    data.contract = LtcContract::Nonqualified {
        treatment: NonqualifiedCareTreatment::LifeAnnuity { source: 1 },
    };
    assert!(issue(&forged, "LTC1099_SOURCE_UNAVAILABLE"));

    let d = complete(FederalStatus::Single);
    let recovery = IncomeDocument::OtherIncome(OtherIncomeItem {
        amount: dollars(300),
        withholding: 0,
        kind: OtherIncomeKind::Recovery {
            worksheet: PriorYearRecovery {
                year: 2025,
                deducted: dollars(200),
                itemized: dollars(20000),
                standard: dollars(19000),
                taxable_income: dollars(40000),
                ordinary: true,
            },
        },
    });
    let mut s = save_items(&d, vec![item(Owner::Taxpayer, 0, recovery)]);
    let page = open_topic(&mut s, TaxSection::OtherIncome);
    let recovery = page_documents(&page)[0].clone();
    let P::SaveIncomes { recovery_sources, .. } = &page else { panic!() };
    assert!(recovery_sources.iter().any(|v| v.id == recovery.id));
    let form = Ltc1099 {
        benefits: dollars(900),
        accelerated: 0,
        interest: 0,
        benefit_origin: Some(LtcPaymentOrigin::Insurance),
        accelerated_origin: None,
        paid: LtcPaidBasis::PerDiem,
        contract: LtcContractPage::Nonqualified {
            treatment: NonqualifiedCareTreatmentPage::AccidentHealth {
                funding: CareBenefitFunding::AfterTax,
                medical_reimbursement: 0,
                permanent_injury: None,
                recovery: Some(recovery.id),
            },
        },
        illness: LtcIllness::Chronic,
        insured: None,
    };
    let kept = s.draft.clone();
    accepted(&send(
        &mut s,
        save_topic(
            &kept,
            TaxSection::OtherIncome,
            vec![recovery, item(Owner::Taxpayer, 0, IncomeDocument::Ltc1099(form))],
        ),
    ));
    let result = supported(&s.draft);
    assert_eq!(value(&result, "recovery_2025_taxable"), 200);
    assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
    assert_eq!(value(&result, "agi"), 60_200);
    let page = open_topic(&mut s, TaxSection::OtherIncome);
    let ltc = page_documents(&page)
        .iter()
        .find(|v| matches!(v.document, IncomeDocument::Ltc1099(_)))
        .unwrap()
        .clone();
    let kept = s.draft.clone();
    assert_eq!(
        key_of(send(&mut s, save_topic(&kept, TaxSection::OtherIncome, vec![ltc]))),
        "LTC1099_SOURCE_UNAVAILABLE"
    );
    assert_eq!(s.draft, kept);
    accepted(&send(&mut s, save_topic(&kept, TaxSection::OtherIncome, vec![])));
    assert_eq!(value(&supported(&s.draft), "agi"), 60_000);
}

// Derived endpoints from IRC223(f)(5),4973(g) and draft2026 i8889 pp3–4,8;
// https://www.irs.gov/pub/irs-dft/i8889--dft.pdf (printed July21,2026).
// Ordinary failed-cash deposit classification is the contribution/excess
// definitions' inference, not a published IRS worked example. Neutral60000
// wages/1980 birth complete the actual public source and HSA pages.
pub(in crate::tests) fn attempted_hsa_rollover(
    received: u32,
    deposits: Vec<MedicalRolloverDeposit>,
) -> IncomeDocument {
    IncomeDocument::Sa1099(Sa1099 {
        origin: HealthDocumentOrigin::Form1099Sa,
        gross: deposits.iter().map(|p| p.amount).sum(),
        account: MedicalSaAccount::Hsa,
        payment: MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 0,
            disabled_since: None,
            rollover: Some(MedicalRollover {
                preceding: None,
                receipts: vec![MedicalRolloverReceipt { received, deposits }],
            }),
        },
    })
}
pub(in crate::tests) fn ordinary_hsa_deposit(
    deposited: u32,
    amount: i64,
    year: u32,
) -> MedicalRolloverDeposit {
    MedicalRolloverDeposit {
        deposited,
        amount: dollars(amount),
        destination: MedicalRolloverDestination::Hsa,
        character: MedicalDepositCharacter::Cash { year, deadline: (year + 1) * 10000 + 415 },
    }
}
#[test]
fn hsa_failed_cash_rollover_uses_actual_designated_year_and_public_account_once() {
    for year in [2026, 2027] {
        let d = complete(FederalStatus::Single);
        // Dec1→Jan31 is61days; the2026 source is taxable in both years.
        let mut s = save_items(
            &d,
            vec![item(
                Owner::Taxpayer,
                0,
                attempted_hsa_rollover(20261201, vec![ordinary_hsa_deposit(20270131, 1000, year)]),
            )],
        );
        if year == 2026 {
            assert!(issue(&s.draft, "MEDICAL_FAILED_DEPOSIT_ACCOUNT_REQUIRED"));
        }
        let mut page = open_hsa_page(&mut s);
        let P::SaveHsa { value: input, .. } = &mut page else { panic!() };
        input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
        accepted(&send(&mut s, page));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
        assert_eq!(value(&r, "hsa_additional_tax"), 200);
        assert_eq!(value(&r, "adjustments"), if year == 2026 { 1000 } else { 0 });
        assert_eq!(s.draft.adjustments.as_ref().unwrap().hsa[0].personal, 0);
        let mut again = restart(s.draft.clone());
        let unchanged = open_hsa_page(&mut again);
        let P::SaveHsa { value: input, .. } = &unchanged else { panic!() };
        assert_eq!(input.accounts[0].personal, 0);
        accepted(&send(&mut again, unchanged));
        assert_eq!(again.draft.adjustments, s.draft.adjustments);
        assert_eq!(supported(&again.draft).tax, r.tax);
    }
}
#[test]
fn hsa_rollover_split_day_sixty_and_sixty_one_flows_to_existing_excess_owner() {
    // Jan1→Mar2 is60days; Mar3 is61. Only the actual late500 becomes
    // source income and a current ordinary cash deposit. No automatic full excess.
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(
                20260101,
                vec![
                    ordinary_hsa_deposit(20260302, 500, 2026),
                    ordinary_hsa_deposit(20260303, 500, 2026),
                ],
            ),
        )],
    );
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut page else { panic!() };
    input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "hsa_1_distribution_excluded"), 500);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 500);
    assert_eq!(value(&r, "hsa_additional_tax"), 100);
    assert_eq!(value(&r, "adjustments"), 500);
    // Same actual source, no ordinary eligibility: existing owner determines
    //500 excess and30excise from actual account value; source tax remains100.
    let mut page = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut page else { panic!() };
    input.accounts[0].coverage = HsaCoverageYear::Changed { months: vec![HsaCoverage::None; 12] };
    input.accounts[0].excess_accounting = Some(hsa_excess_accounting(dollars(500), None, 0, 0, 0));
    accepted(&send(&mut s, page));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 500);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 30);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 500);
    assert_eq!(s.draft.adjustments.as_ref().unwrap().hsa[0].personal, 0);
}
#[test]
fn hsa_rollover_history_counts_only_successful_actual_receipts_not_source_order() {
    for earlier_succeeded in [false, true] {
        let d = complete(FederalStatus::Single);
        let earlier_deposit = if earlier_succeeded { 20260601 } else { 20260701 };
        // May1→July1 is61days and does not consume annual rollover relief.
        let mut s = save_items(
            &d,
            vec![
                item(
                    Owner::Taxpayer,
                    0,
                    attempted_hsa_rollover(
                        20261201,
                        vec![ordinary_hsa_deposit(20261215, 1000, 2026)],
                    ),
                ),
                item(
                    Owner::Taxpayer,
                    0,
                    attempted_hsa_rollover(
                        20260501,
                        vec![ordinary_hsa_deposit(earlier_deposit, 1000, 2026)],
                    ),
                ),
            ],
        );
        let projection = crate::medical_rollover::classify(&s.draft).unwrap();
        assert_eq!(projection.failed.len(), 1);
        assert_eq!(
            projection.failed[0].received,
            if earlier_succeeded { 20261201 } else { 20260501 }
        );
        let mut page = open_hsa_page(&mut s);
        let P::SaveHsa { value: input, .. } = &mut page else { panic!() };
        input.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
        accepted(&send(&mut s, page));
        let r = supported(&s.draft);
        assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
        assert_eq!(value(&r, "adjustments"), 1000);
        let mut reversed = s.draft.clone();
        reversed.income.reverse();
        assert_eq!(supported(&reversed), r);
    }
}

// Derived actual-source correction fixture, not an IRS worked example. Failed
// cash uses the existing ordinary contribution owner (4973(g) inference). Actual
// January2027 contribution designated2026 and custodian-returned principal are
// distinct source facts; neutral wages/birth are supplied by complete().
#[test]
fn hsa_failed_deposit_correction_preserves_manual_totals_and_net_later_once() {
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(20261201, vec![ordinary_hsa_deposit(20270131, 1000, 2026)]),
        )],
    );
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, hsa_failed_deposits, .. } = &mut request else { panic!() };
    assert_eq!(hsa_failed_deposits.len(), 1);
    assert_eq!(input.linked_deposits[0].id, source);
    assert_eq!(input.linked_deposits[0].deposits[0].amount, dollars(1000));
    let mut account = hsa_account(
        Owner::Taxpayer,
        HsaCoverageYear::Changed { months: vec![HsaCoverage::None; 12] },
        0,
    );
    let mut correction = hsa_correction(2026, 20270301, dollars(400), -dollars(20));
    correction.contributed_on = 20270131;
    correction.deadline = 20270415;
    correction.failed_deposit = Some(source);
    account.corrections = vec![correction.page()];
    account.excess_accounting = Some(hsa_excess_accounting(0, None, 0, 0, 0));
    input.accounts = vec![account];
    // A forged display cannot supply amounts or replace the accepted source.
    input.linked_deposits[0].deposits[0].amount = dollars(99999);
    hsa_failed_deposits[0].deposits[0].amount = dollars(99999);
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "hsa_1_return_year_uncorrected_excess"), 600);
    assert_eq!(value(&r, "hsa_return_year_excess_contribution_tax"), 36);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1000);
    assert_eq!(value(&r, "hsa_additional_tax"), 200);
    let prepared = crate::medical_rollover::prepare(&s.draft).unwrap();
    let working = prepared.draft.unwrap();
    let effective = &working.adjustments.as_ref().unwrap().hsa[0];
    assert_eq!(effective.personal, 0);
    assert_eq!(
        prepared.contributions.hsa[0].personal,
        crate::kernel::ratio::exact::whole(dollars(1000))
    );
    assert_eq!(
        prepared.contributions.hsa[0].later,
        crate::kernel::ratio::exact::whole(dollars(600))
    );
    assert_eq!(effective.corrections[0].failed_deposit, Some(source));
    assert!(crate::medical_rollover::prepare(&working).is_err());
    let stored = &s.draft.adjustments.as_ref().unwrap().hsa[0];
    assert_eq!(stored.personal, 0);
    assert_eq!(stored.excess_accounting.as_ref().unwrap().later_return_year_deposits, 0);
    assert_eq!(stored.corrections[0].failed_deposit, Some(source));
    let mut again = restart(s.draft.clone());
    let unchanged = open_hsa_page(&mut again);
    let P::SaveHsa { value: input, .. } = &unchanged else { panic!() };
    assert_eq!(input.accounts[0].personal, 0);
    assert_eq!(input.linked_deposits[0].deposits[0].amount, dollars(1000));
    accepted(&send(&mut again, unchanged));
    assert_eq!(again.draft, s.draft);
    assert_eq!(supported(&again.draft).tax, r.tax);
}

#[test]
fn hsa_failed_deposit_links_never_borrow_other_contribution_room() {
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(20260101, vec![ordinary_hsa_deposit(20260303, 1000, 2026)]),
        )],
    );
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    let before = s.draft.clone();
    for (manual, principal, linked, key) in [
        (100, 150, false, "HSA_CORRECTED_PERSONAL_ABOVE_CONTRIBUTED"),
        (1000, 1100, true, "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT"),
    ] {
        let mut request = open_hsa_page(&mut s);
        let P::SaveHsa { value: input, hsa_failed_deposits, .. } = &mut request else { panic!() };
        let mut account = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, dollars(manual));
        let mut c = hsa_correction(2026, 20260401, dollars(principal), 0);
        c.contributed_on = 20260303;
        c.failed_deposit = linked.then_some(source);
        account.corrections = vec![c.page()];
        input.accounts = vec![account];
        hsa_failed_deposits[0].deposits[0].amount = dollars(9000);
        assert_eq!(key_of(send(&mut s, request)), key);
        assert_eq!(s.draft, before);
    }
    // Different date and unknown source IDs are rejected from actual accepted facts.
    for (id, date) in [(source, 20260304), (source + 999, 20260303)] {
        let mut request = open_hsa_page(&mut s);
        let P::SaveHsa { value: input, .. } = &mut request else { panic!() };
        let mut a = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
        let mut c = hsa_correction(2026, 20260401, dollars(100), 0);
        c.contributed_on = date;
        c.failed_deposit = Some(id);
        a.corrections = vec![c.page()];
        input.accounts = vec![a];
        assert_eq!(key_of(send(&mut s, request)), "HEALTH_FAILED_DEPOSIT_UNAVAILABLE");
        assert_eq!(s.draft, before);
    }
}

#[test]
fn hsa_failed_deposit_source_deletion_requires_removing_actual_correction() {
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(20260101, vec![ordinary_hsa_deposit(20260303, 1000, 2026)]),
        )],
    );
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut request else { panic!() };
    let mut a = hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0);
    let mut c = hsa_correction(2026, 20260401, dollars(400), 0);
    c.contributed_on = 20260303;
    c.failed_deposit = Some(source);
    a.corrections = vec![c.page()];
    input.accounts = vec![a];
    accepted(&send(&mut s, request));
    let before = s.draft.clone();
    let deletion = save_topic(&before, TaxSection::OtherIncome, vec![]);
    assert_eq!(key_of(send(&mut s, deletion)), "HEALTH_FAILED_DEPOSIT_UNAVAILABLE");
    assert_eq!(s.draft, before);
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut request else { panic!() };
    input.accounts[0].corrections.clear();
    accepted(&send(&mut s, request));
    let before = s.draft.clone();
    accepted(&send(&mut s, save_topic(&before, TaxSection::OtherIncome, vec![])));
    assert_eq!(value(&supported(&s.draft), "hsa_taxable_distributions"), 0);
}

// Same ordinary-contribution inference, but the actual source and destination
// are Archer MSAs. Self-only deductible3000 gives1950 capacity; actual2350
// gross less400 timely excess return leaves1950 deduction. The original and
// returned-excess statements remain separate IDs.
#[test]
fn archer_failed_deposit_correction_uses_source_and_return_statement_once() {
    let d = complete(FederalStatus::Single);
    let mut original =
        attempted_hsa_rollover(20260101, vec![ordinary_hsa_deposit(20260303, 2350, 2026)]);
    let IncomeDocument::Sa1099(data) = &mut original else { panic!() };
    data.account = MedicalSaAccount::Archer;
    let MedicalSaPayment::Own { rollover: Some(rollover), .. } = &mut data.payment else {
        panic!()
    };
    rollover.receipts[0].deposits[0].destination = MedicalRolloverDestination::Archer;
    let returned = IncomeDocument::Sa1099(Sa1099 {
        origin: HealthDocumentOrigin::Form1099Sa,
        gross: dollars(420),
        account: MedicalSaAccount::Archer,
        payment: MedicalSaPayment::ExcessReturned { earnings: Some(dollars(20)) },
    });
    let mut s = save_items(
        &d,
        vec![item(Owner::Taxpayer, 0, original), item(Owner::Taxpayer, 0, returned)],
    );
    let original_id=s.draft.income.iter().find(|e|matches!(&e.income,FederalIncome::MedicalDistribution{data} if matches!(data.payment,MedicalSaPayment::Own{..}))).unwrap().id;
    let returned_id=s.draft.income.iter().find(|e|matches!(&e.income,FederalIncome::MedicalDistribution{data} if matches!(data.payment,MedicalSaPayment::ExcessReturned{..}))).unwrap().id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, archer_failed_deposits, hsa_failed_deposits, .. } = &mut request
    else {
        panic!()
    };
    assert!(hsa_failed_deposits.is_empty());
    assert_eq!(archer_failed_deposits[0].id, original_id);
    input.archer = archer_page(vec![ArcherContributorRow {
        corrections: vec![
            ArcherCorrection {
                document: Some(returned_id),
                failed_deposit: Some(original_id),
                year: 2026,
                contributed_on: 20260303,
                principal: dollars(400),
                earnings: dollars(20),
                returned: 20260401,
                deadline: 20270415,
                payer: ArcherCorrectionPayer::Personal,
            }
            .page(),
        ],
        ..archer_contributor_row(
            Owner::Taxpayer,
            0,
            0,
            ArcherEligibility::FullYear { coverage: archer_self(3000) },
            vec![1],
        )
    }]);
    accepted(&send(&mut s, request));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "adjustments"), 1950);
    assert_eq!(value(&r, "archer_msa_taxable_distributions"), 2350);
    assert_eq!(value(&r, "archer_msa_additional_tax"), 470);
    assert_eq!(value(&r, "archer_correction_earnings"), 20);
    assert_eq!(value(&r, "agi"), 60420);
    let prepared = crate::medical_rollover::prepare(&s.draft).unwrap();
    let working = prepared.draft.unwrap();
    let ArcherContributions::Records { owners, .. } = &working.adjustments.as_ref().unwrap().archer
    else {
        panic!()
    };
    assert_eq!(owners[0].personal, 0);
    assert_eq!(
        prepared.contributions.archer[0].personal,
        crate::kernel::ratio::exact::whole(dollars(2350))
    );
    assert_eq!(owners[0].corrections[0].document, Some(returned_id));
    assert_eq!(owners[0].corrections[0].failed_deposit, Some(original_id));
    let mut again = restart(s.draft.clone());
    let unchanged = open_hsa_page(&mut again);
    let P::SaveHsa { value: input, .. } = &unchanged else { panic!() };
    let ArcherContributionsPage::Records { owners, .. } = &input.archer else { panic!() };
    assert_eq!(owners[0].personal, 0);
    assert_eq!(owners[0].corrections[0].failed_deposit, Some(original_id));
    accepted(&send(&mut again, unchanged));
    assert_eq!(supported(&again.draft).tax, r.tax);
}

#[test]
fn hsa_failed_deposit_estimate_and_source_edits_keep_correction_graph_authoritative() {
    let d = complete(FederalStatus::Single);
    let mut s = save_items(
        &d,
        vec![item(
            Owner::Taxpayer,
            0,
            attempted_hsa_rollover(20260101, vec![ordinary_hsa_deposit(20260303, 1000, 2026)]),
        )],
    );
    assert!(
        crate::interview::derived_topics(&s.draft, TaxSection::Adjustments)
            .contains(&TaxSection::Hsa)
    );
    let source = s
        .draft
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::MedicalDistribution { .. }))
        .unwrap()
        .id;
    let mut request = open_hsa_page(&mut s);
    let P::SaveHsa { value: input, .. } = &mut request else { panic!() };
    let mut a = hsa_account(
        Owner::Taxpayer,
        HsaCoverageYear::Changed { months: vec![HsaCoverage::None; 12] },
        0,
    );
    let mut c = hsa_correction(2026, 20260401, dollars(400), 0);
    c.contributed_on = 20260303;
    c.failed_deposit = Some(source);
    a.corrections = vec![c.page()];
    a.excess_accounting = Some(hsa_excess_accounting(dollars(600), None, 0, 0, 0));
    input.accounts = vec![a];
    let before = s.draft.clone();
    let P::SaveHsa { value, hsa_failed_deposits, archer_failed_deposits, .. } = request.clone()
    else {
        panic!()
    };
    let reply = send(
        &mut s,
        P::EstimateHsa {
            user: TEST_USER,
            allowed_owners: vec![],
            allowed_family_split: vec![],
            other_coverage_allowed: false,
            hsa_failed_deposits,
            archer_failed_deposits,
            hsa_return_documents: vec![],
            archer_return_documents: vec![],
            ira_sources: vec![],
            archer_income: vec![],
            value,
        },
    );
    assert!(matches!(reply, Reply::Response(P::RunningEstimate { .. })));
    assert_eq!(s.draft, before);
    accepted(&send(&mut s, request));
    assert!(
        !crate::interview::derived_topics(&s.draft, TaxSection::Adjustments)
            .contains(&TaxSection::Hsa)
    );
    let before = s.draft.clone();
    for (received, amount, key) in [
        (20260102, 1000, "HEALTH_FAILED_DEPOSIT_UNAVAILABLE"),
        (20260101, 399, "HEALTH_FAILED_DEPOSIT_RETURN_ABOVE_DEPOSIT"),
    ] {
        // Moving receipt one day makes March3 a successful day60 deposit.
        let changed = item(
            Owner::Taxpayer,
            source,
            attempted_hsa_rollover(received, vec![ordinary_hsa_deposit(20260303, amount, 2026)]),
        );
        assert_eq!(
            key_of(send(&mut s, save_topic(&before, TaxSection::OtherIncome, vec![changed]))),
            key
        );
        assert_eq!(s.draft, before);
    }
    let mut broken = before.clone();
    broken.adjustments.as_mut().unwrap().hsa[0].corrections[0].failed_deposit = Some(source + 100);
    assert!(!broken.valid_stored());
    // Normal checklist pruning can remove both actual source and corrections
    // in one candidate; removing the source alone was rejected above.
    let mut ticks = before.checklist;
    ticks.income = 0;
    ticks.adjustments = 0;
    let request = crate::tests::cross::interview::save_checklist(
        &before,
        crate::tests::cross::interview::PAGE_CHILDREN,
        ticks,
    );
    accepted(&send(&mut s, request));
    assert!(s.draft.adjustments.as_ref().unwrap().hsa.is_empty());
    assert!(!s.draft.income.iter().any(|e| e.id == source));
}

#[test]
fn hsa_failed_deposit_preview_uses_actual_complete_history_without_mutation() {
    for earlier_succeeded in [false, true] {
        let d = complete(FederalStatus::Single);
        let mut s = save_items(
            &d,
            vec![item(
                Owner::Taxpayer,
                0,
                attempted_hsa_rollover(
                    20260501,
                    vec![ordinary_hsa_deposit(
                        if earlier_succeeded { 20260601 } else { 20260701 },
                        1000,
                        2026,
                    )],
                ),
            )],
        );
        let before = s.draft.clone();
        let request = survey_preview_document(attempted_hsa_rollover(
            20261201,
            vec![ordinary_hsa_deposit(20261215, 1000, 2026)],
        ));
        let Reply::Response(P::IncomePreview { preview: amount, .. }) = send(&mut s, request)
        else {
            panic!()
        };
        assert_eq!(amount, Some(dollars(if earlier_succeeded { 1000 } else { 0 })));
        assert_eq!(s.draft, before);
    }
}

#[test]
fn unit13_approved_ltc_period_source_facts_save_reopen_and_preserve_joint_tax() {
    let mut d = complete(FederalStatus::Joint);
    d.checklist.whole_return |= WholeReturnTopics::INJURED_SPOUSE.0;
    d.injured_spouse = Some(InjuredSpouseAllocation {
        injured: Owner::Taxpayer,
        debt: OffsetDebt::ChildSupport,
        obligated: false,
        community: InjuredCommunityFacts::No,
        student_loan_borrower: None,
        estimated: EstimatedSplit::IrsFormula,
        allocations: vec![],
        claimants: vec![],
    });
    let form = |owner| {
        item(
            owner,
            0,
            IncomeDocument::Ltc1099(Ltc1099 {
                interest: 0,
                benefit_origin: Some(LtcPaymentOrigin::Insurance),
                accelerated_origin: Some(LtcPaymentOrigin::AcceleratedLife {
                    business_insurable_interest: false,
                }),
                benefits: dollars(15_000),
                accelerated: 0,
                paid: LtcPaidBasis::PerDiem,
                contract: LtcContractPage::Qualified {
                    qualification: CareContractQualification::Qualified,
                },
                illness: LtcIllness::Chronic,
                insured: None,
            }),
        )
    };
    let periods = vec![(20260601, 20260630), (20260701, 20260731)]
        .into_iter()
        .map(|(starts, ends)| CareBenefitPeriodInput {
            starts,
            ends,
            received: dollars(15_000),
            services: 0,
            reimbursements: 0,
            recipient: CareBenefitRecipient::Insured { other_payees: false },
            injured_payees: None,
        })
        .collect();
    let first = save_items(
        &d,
        vec![
            item(
                Owner::Taxpayer,
                0,
                IncomeDocument::LtcInsured(LtcInsured {
                    insured: LtcInsuredPerson::OnReturn { person: -1 },
                    periods: LtcPeriodsInput::Several {
                        method: CarePeriodMethod::EqualRate,
                        periods,
                    },
                }),
            ),
            form(Owner::Taxpayer),
            form(Owner::Spouse),
        ],
    );
    let before = supported(&first.draft);
    assert_eq!(value(&before, "taxable_ltc_benefits"), 3770);
    let sources: Vec<_> = first
        .draft
        .income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::LtcBenefits { .. }))
        .map(|e| e.id)
        .collect();
    let page = other_income_page(&first.draft);
    let mut rows = page_documents(&page).to_vec();
    let row =
        rows.iter_mut()
            .find_map(|e| {
                if let IncomeDocument::LtcInsured(v) = &mut e.document { Some(v) } else { None }
            })
            .unwrap();
    let LtcPeriodsInput::Several { periods, .. } = &mut row.periods else { panic!() };
    for (period, source) in periods.iter_mut().zip(&sources) {
        period.injured_payees = Some(InjuredLtcPeriodPayeesInput::Payments {
            payments: vec![InjuredLtcPeriodPaymentInput {
                source: *source,
                amount: dollars(15_000),
            }],
        });
    }
    let saved = save_items(&first.draft, rows);
    let after = supported(&saved.draft);
    assert_eq!(value(&after, "taxable_ltc_benefits"), 3770);
    assert_eq!(value(&before, "agi"), value(&after, "agi"));
    let mut reopened = restart(saved.draft.clone());
    let page = open_topic(&mut reopened, TaxSection::OtherIncome);
    let shown = page_documents(&page).to_vec();
    let stored = saved
        .draft
        .income
        .iter()
        .find_map(|e| {
            if let FederalIncome::LtcInsured { data } = &e.income {
                Some(data.page())
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        shown.iter().any(|e| matches!(&e.document,IncomeDocument::LtcInsured(v) if v==&stored))
    );
    let mut invalid = shown;
    let row =
        invalid
            .iter_mut()
            .find_map(|e| {
                if let IncomeDocument::LtcInsured(v) = &mut e.document { Some(v) } else { None }
            })
            .unwrap();
    let LtcPeriodsInput::Several { periods, .. } = &mut row.periods else { panic!() };
    let Some(InjuredLtcPeriodPayeesInput::Payments { payments }) = &mut periods[0].injured_payees
    else {
        panic!()
    };
    payments[0].source = 999999;
    let request = save_topic(&reopened.draft, TaxSection::OtherIncome, invalid);
    crate::tests::cross::interview::rejected_with(
        current_source_refusal(
            "health-context-01",
            &mut reopened,
            request,
            "INJURED_LTC_SOURCE_UNAVAILABLE",
            Some(
                "[documents/documents/0/document/periods/periods/0/injured_payees/payments/0/source]",
            ),
            SourceScope::CommitOnly,
        ),
        "INJURED_LTC_SOURCE_UNAVAILABLE",
    );
    assert_eq!(reopened.draft, saved.draft);
}
