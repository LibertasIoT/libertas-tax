// w2 items of src/tests/support/builders/native_income.rs, split out by tools/split-hubs.
use super::super::paper_documents::remember;
#[allow(unused_imports)]
use super::*;
use alloc::string::String;

pub(in crate::tests) fn native_w2(wages: i64) -> W2 {
    W2 {
        employer: Default::default(),
        employee: Default::default(),
        state_details: Default::default(),
        wages,
        withholding: wages / 10,
        social_wages: wages,
        social_tax: wages * 62 / 1000,
        medicare_wages: wages,
        medicare_tax: wages * 145 / 10000,
        more: W2More::default(),
    }
}

// The source inputs of the W-2 refusal table remain typed for oracle recapture.
pub(in crate::tests) fn rejected_w2_cases() -> [(W2, &'static str); 3] {
    let mut duplicate = native_w2(1_000_000);
    duplicate.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        social_tips: 0,
        allocated_tips: 0,
        care_benefits: 0,
        box12: vec![
            Box12Entry { code: W2Code::D, amount: 10_000 },
            Box12Entry { code: W2Code::D, amount: 20_000 },
        ],
        statutory: false,
        retirement_plan: false,
    };
    // One cent above the 2026 $184,500 Social Security wage base.
    let mut above_base = native_w2(18_450_000);
    above_base.more = W2More {
        payroll: W2Payroll::SocialSecurity,
        social_tips: 1,
        allocated_tips: 0,
        care_benefits: 0,
        box12: vec![],
        statutory: false,
        retirement_plan: false,
    };
    let mut withholding = native_w2(1_000_000);
    withholding.withholding = 1_000_001;
    [
        (duplicate, "W2_BOX12_CODE_TWICE"),
        (above_base, "W2_SOCIAL_WAGES_AND_TIPS_ABOVE_BASE"),
        (withholding, "W2_WITHHOLDING_ABOVE_WAGES"),
    ]
}

pub(in crate::tests) fn center_wage_row(
    wages: i64,
    allowed_community: Vec<i32>,
) -> crate::WageDocumentRow {
    crate::WageDocumentRow {
        id: 0,
        owner: crate::DocumentOwner::Yours,
        document: crate::WageDocument::W2(wage_document(native_w2(wages))),
        community: CommunityShare::NotNeeded,
        refund_management: None,
        allowed_community,
        investment_caps: None,
        preview: None,
        refund_management_needed: None,
    }
}

pub(in crate::tests) fn cash_tips_without_w2(
    months: Vec<CashTipMonth>,
    coverage: TipCoverage,
    qualified_occupation: bool,
) -> IncomeSource {
    IncomeSource::Record {
        value: IncomeRecord::CashTipsWithoutW2(CashTipsWithoutW2 {
            employer: "Synthetic tip employer".into(),
            months,
            coverage,
            qualified_occupation,
        }),
    }
}

pub(in crate::tests) fn wage_source(data: FederalWage) -> WageSource {
    let FederalWage {
        tips_without_w2,
        wages,
        disability,
        abroad,
        deferred_pension,
        inmate_pay,
        withholding,
        social_wages,
        social_tips,
        employee_tips,
        work_deductions,
        social_tax,
        medicare_wages,
        medicare_tax,
        uncollected_tax,
        medicaid_waiver,
        statutory_business,
        paying_scorporation,
        payroll,
        coverage,
        care_benefits,
        statutory_employee,
        retirement_plan,
        other_box12,
    } = data;
    if tips_without_w2 {
        let tips = employee_tips.expect("cash-tip fixture records");
        let CashTipRecords::Actual { months } = tips.cash else { panic!("cash-tip records") };
        return WageSource::CashTipsWithoutW2(CashTipsWithoutW2 {
            employer: "Synthetic tip employer".into(),
            months,
            coverage: tips.coverage,
            qualified_occupation: matches!(
                work_deductions.tips_claim,
                TipsClaim::Yes { tips_eligible: Some(true), .. }
            ),
        });
    }
    let mut answers = W2Answers {
        disability: Some(disability),
        abroad: W2Amount::of(abroad),
        deferred_pension: W2Amount::of(deferred_pension),
        inmate_pay,
        medicaid_waiver,
        statutory_business: statutory_business.map(|source| StatutoryBusiness::Business { source }),
        paying_scorporation: paying_scorporation
            .map(crate::business_credit_page::WageScorpEmployer::from_id),
        coverage: railroad_answer(&payroll).or(Some(coverage)),
        ..Default::default()
    };
    let mut allocated_tips = 0;
    if let Some(tips) = employee_tips {
        answers.noncash_tips = W2Amount::of(tips.noncash);
        match tips.cash {
            CashTipRecords::Actual { months } => {
                answers.tip_records = Some(DailyTipRecord::Months { months })
            }
            CashTipRecords::Allocated { amount } => allocated_tips = amount,
        }
    }
    let mut box12: Vec<_> = other_box12
        .into_iter()
        .map(|row| Box12Entry { code: row.code, amount: row.amount })
        .collect();
    if let Some(tax) = uncollected_tax {
        for (code, amount) in [
            (W2Code::A, tax.tip_social),
            (W2Code::B, tax.tip_medicare),
            (W2Code::M, tax.life_social),
            (W2Code::N, tax.life_medicare),
        ] {
            if amount != 0 {
                box12.push(Box12Entry { code, amount });
            }
        }
        // A present all-zero record prints code A with a zero amount, so the
        // paper still carries the uncollected-tax source.
        if [tax.tip_social, tax.tip_medicare, tax.life_social, tax.life_medicare] == [0; 4] {
            box12.push(Box12Entry { code: W2Code::A, amount: 0 });
        }
    }
    let eligible = |yes| {
        if yes { WageDeductionEligibility::Eligible } else { WageDeductionEligibility::Ineligible }
    };
    if let TipsClaim::Yes { w2_tips, tips_eligible, .. } = work_deductions.tips_claim {
        answers.tips_listed_occupation = tips_eligible.map(eligible);
        if w2_tips != 0 {
            box12.push(Box12Entry { code: W2Code::TP, amount: w2_tips });
        }
    }
    if let OvertimeClaim::Yes { overtime, overtime_eligible } = work_deductions.overtime_claim {
        answers.overtime_flsa_required = overtime_eligible.map(eligible);
        box12.push(Box12Entry { code: W2Code::TT, amount: overtime });
    }
    WageSource::W2(W2Source {
        form: FormW2Paper {
            calendar_year: Default::default(),
            employer: Default::default(),
            employee: Default::default(),
            wages: Some(wages),
            withholding: Some(withholding),
            social_wages: Some(social_wages),
            social_tax: Some(social_tax),
            medicare_wages: Some(medicare_wages),
            medicare_tax: Some(medicare_tax),
            social_tips: Some(social_tips),
            allocated_tips: Some(allocated_tips),
            care_benefits: Some(care_benefits),
            box12: box12.iter().map(Into::into).collect(),
            statutory: statutory_employee,
            retirement_plan,
            control_number: String::new(),
            nonqualified_plans: Some(0),
            third_party_sick_pay: false,
            box14a: Vec::new(),
            tipped_occupation_codes: Vec::new(),
            states: Vec::new(),
            localities: Vec::new(),
        },
        answers,
    })
}

pub(in crate::tests) fn wage_document(source: impl WagePaperFixture) -> FormW2Paper {
    if let Some(form) = source.existing_paper() {
        return form;
    }
    let source = source.wage_source();
    let form = source.form;
    remember(
        IncomeSource::Wage { value: crate::WageDocument::W2(form.clone()) },
        vec![crate::DocumentAnswer::Wage { answers: (&source.answers).into() }],
    );
    form
}

pub(in crate::tests) trait WagePaperFixture {
    fn existing_paper(&self) -> Option<FormW2Paper> {
        None
    }
    fn wage_source(self) -> W2Source;
}

impl WagePaperFixture for W2Source {
    fn wage_source(self) -> W2Source {
        self
    }
}

impl WagePaperFixture for W2 {
    fn wage_source(self) -> W2Source {
        let W2 {
            employer,
            employee,
            wages,
            withholding,
            social_wages,
            social_tax,
            medicare_wages,
            medicare_tax,
            more,
            state_details,
        } = self;
        let W2More {
            social_tips,
            allocated_tips,
            care_benefits,
            box12,
            statutory,
            retirement_plan,
            payroll,
        } = more;
        W2Source {
            form: FormW2Paper {
                calendar_year: Default::default(),
                employer,
                employee,
                wages: Some(wages),
                withholding: Some(withholding),
                social_wages: Some(social_wages),
                social_tax: Some(social_tax),
                medicare_wages: Some(medicare_wages),
                medicare_tax: Some(medicare_tax),
                social_tips: Some(social_tips),
                allocated_tips: Some(allocated_tips),
                care_benefits: Some(care_benefits),
                box12: box12.iter().map(Into::into).collect(),
                statutory,
                retirement_plan,
                states: state_details.states,
                localities: state_details.localities,
                control_number: state_details.control_number,
                nonqualified_plans: Some(state_details.nonqualified_plans),
                third_party_sick_pay: state_details.third_party_sick_pay,
                box14a: state_details.box14a,
                tipped_occupation_codes: state_details.tipped_occupation_codes,
            },
            answers: crate::W2Answers { coverage: railroad_answer(&payroll), ..Default::default() },
        }
    }
}

impl WagePaperFixture for FormW2Paper {
    fn existing_paper(&self) -> Option<FormW2Paper> {
        Some(self.clone())
    }
    fn wage_source(self) -> W2Source {
        unreachable!("paper passes through unchanged")
    }
}

/// The W-2 answer that carries a railroad payroll (none for Social Security payroll).
pub(in crate::tests) fn railroad_answer(payroll: &crate::W2Payroll) -> Option<crate::W2Coverage> {
    match payroll {
        crate::W2Payroll::SocialSecurity => None,
        crate::W2Payroll::Railroad {
            compensation,
            tier_1,
            tier_2,
            medicare,
            additional_medicare,
        } => Some(crate::W2Coverage::Railroad {
            compensation: *compensation,
            tier_1: Some(*tier_1),
            tier_2: Some(*tier_2),
            medicare: Some(*medicare),
            additional_medicare: Some(*additional_medicare),
        }),
    }
}
