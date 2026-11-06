// form_8379 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) struct Form8379BusinessIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) business: i64,
    pub(super) agi: i64,
    pub(super) adjustments: i64,
    pub(super) excess_business_loss: i64,
    pub(super) estate_niit_excess_deduction: i64,
    pub(super) pure_loan_gain: i64,
}

pub(super) struct Form8379BusinessOut {
    pub(super) schedule_c: i64,
    pub(super) schedule_f: i64,
    pub(super) total_income: i64,
    pub(super) business_form_owners: [[i64; 2]; 2],
    pub(super) statutory_form_owners: [[i64; 2]; 2],
    pub(super) statutory_sources_known: bool,
}

pub(super) fn form_8379_business_phase(
    pass: &Pass,
    r: &FederalResult,
    t: &mut Totals,
    input: Form8379BusinessIn<'_>,
) -> Form8379BusinessOut {
    let &Pass { d, context, selected_regular, .. } = pass;
    let direct_participation = context.participation;
    let Form8379BusinessIn {
        rental,
        business,
        agi,
        adjustments,
        excess_business_loss,
        estate_niit_excess_deduction,
        pure_loan_gain,
    } = input;
    // Both forms retain their actual collected sources, including clergy
    // expense restoration and linked farm receipts. Community Share owns
    // the income-tax allocation; the source map keeps operator earnings.
    let mut business_forms = [t.statutory_profit.iter().sum::<i64>(), 0];
    let mut business_form_owners = [t.statutory_profit, [0; 2]];
    let mut statutory_form_owners = [t.statutory_profit, [0; 2]];
    // The admitted statutory route is an actual same-owner expenses-only C.
    // Other current passive C sources require the same completed PAL and
    // selected source operands used by the final collector.
    let mut statutory_sources_known =
        r.issues.is_empty() && excess_business_loss == 0 && t.community_business == 0;
    for entry in &d.income {
        let FederalIncome::Business { data } = &entry.income else {
            continue;
        };
        let profit = t.business_sources[&entry.id][0].income.filer;
        let form = usize::from(matches!(data.activity, crate::BusinessActivity::Farming { .. }));
        business_forms[form] += profit;
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        business_form_owners[form][owner] += profit;
        let statutory = crate::statutory_wages::business(d, entry.id);
        if statutory {
            statutory_form_owners[form][owner] += profit;
        }
        let passive_source_known = !statutory
            && rental.inputs_complete
            && rental.source_issue.is_none()
            && rental.pool_issue.is_none()
            && rental.post_pal.as_ref().is_some_and(|packet| {
                !packet.alternative
                    && packet.sources.iter().any(|source| {
                        source.income == entry.id
                            && source.owner == entry.owner.owned()
                            && source.component == extended_rules::PostPalComponent::DirectBusiness
                            && source.common_pool
                            && source.characters.iter().any(|part| {
                                part.character == extended_rules::PostPalCharacter::ScheduleC
                                    && part.allowed_net_cents == t.business_profits[&entry.id]
                                    && selected_regular
                                        .and_then(|allocation| {
                                            allocation.passive_ordinary.get(&entry.id)
                                        })
                                        .and_then(|source| source.allowed_cents)
                                        == Some(part.allowed_net_cents)
                            })
                    })
            });
        // The accepted statutory association retains wage character even
        // without material participation; its collected net is not a PAL item.
        statutory_sources_known &= (direct_participation.get(entry.id)
            == crate::participation::Participation::Yes
            || ((statutory || passive_source_known)
                && data.current_passive_supported(direct_participation.get(entry.id))))
            && (profit >= 0 || data.current_risk_supported(direct_participation.get(entry.id)));
    }
    // Keep the native annual business amount and existing C-first form
    // rounding. The kernel verifies both signed source groups before it
    // gives the annual rounding remainder to the actual F component.
    let [schedule_c, schedule_f] =
        crate::kernel::ratio::project_rounded_pair(business, business_forms, 0)
            .expect("collected C/F sources must reconcile to native business income");
    let total_income = agi + adjustments;
    // Pass actual native category amounts to both final8379 and the early
    // changed-filer3800 consumer. No synthetic financial rows are emitted.
    t.injured_k1_nii.targets = [
        dollars(t.k1_line_7),
        dollars(t.k1_nii_excluded),
        dollars(t.estate_niit_line_7),
        estate_niit_excess_deduction,
        pure_loan_gain,
        dollars(t.k1_ordinary[0] + rental.k1_cuts.ordinary.get()),
    ];
    t.injured_k1_nii.ordinary_cuts = rental
        .k1_cuts
        .ordinary_by_id
        .iter()
        .try_fold(
            [crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::default(); 2],
            |mut owners, (id, cut)| {
                let entry = d.income.iter().find(|entry| entry.id == *id)?;
                let index = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                owners[index] =
                    crate::kernel::carry::Amount::new(owners[index].get() + cut.get()).ok()?;
                Some(owners)
            },
        )
        .filter(|owners| {
            owners.iter().map(|amount| amount.get()).sum::<i64>() == rental.k1_cuts.ordinary.get()
        });
    for &(id, cut) in &rental.k1_cuts.ordinary_by_id {
        if let Some(entry) = d.income.iter().find(|entry| entry.id == id) {
            t.record_income_component(
                id,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                crate::injured_spouse::income_categories::IncomeCategory::K1OrdinaryGain,
                Some(cut.get()),
                Some(cut.get()),
            );
        }
    }
    Form8379BusinessOut {
        schedule_c,
        schedule_f,
        total_income,
        business_form_owners,
        statutory_form_owners,
        statutory_sources_known,
    }
}

pub(super) struct InjuredCreditRatiosIn<'a> {
    pub(super) nonqualified_annuity: i64,
    pub(super) unemployment: i64,
    pub(super) injured_facts: &'a crate::injured_spouse::JointFacts<'a>,
    pub(super) credit_entries: &'a [crate::general_business_credit::CreditEntry],
    pub(super) account_distribution_taxes: &'a crate::retirement_return::AccountDistributionTaxes,
    pub(super) early_distribution_taxes: &'a crate::retirement_return::EarlyDistributionTaxes,
    pub(super) care_components: &'a crate::forms::form_2441::CreditComponents,
    pub(super) rmd_taxes: &'a crate::rmd::RmdTaxes,
    pub(super) property_niit: &'a Option<crate::section1231::PropertyNiit>,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) income_tax_expense: Option<i64>,
    pub(super) retirement_taxable: [i64; 2],
    pub(super) additional: [i64; 3],
    pub(super) use_itemized: bool,
    pub(super) care_used: i64,
    pub(super) education_used: i64,
    pub(super) qbi_ded: i64,
    pub(super) additional_medicare: i64,
    pub(super) niit: i64,
    pub(super) nii: i64,
    pub(super) jury: i64,
    pub(super) jury_remitted: i64,
    pub(super) olympic_prize: i64,
    pub(super) olympic_taxable: i64,
    pub(super) schedule_c: i64,
    pub(super) schedule_f: i64,
    pub(super) housing_deduction: i64,
    pub(super) personal_rent: i64,
    pub(super) personal_rent_expenses: i64,
    pub(super) alimony_paid: i64,
    pub(super) msa_deduction: i64,
    pub(super) savings_penalty: i64,
    pub(super) educator: i64,
    pub(super) employee_deduction: i64,
    pub(super) estate_67e: i64,
    pub(super) half_se: i64,
    pub(super) hsa: i64,
    pub(super) moving_deduction: i64,
    pub(super) sep_deduction: i64,
    pub(super) loan: i64,
    pub(super) premium_repayment: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) senior_ded: i64,
    pub(super) niit_itemized: i64,
    pub(super) estate_niit_excess_deduction: i64,
    pub(super) w2_wages: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) social: i64,
    pub(super) capital: i64,
    pub(super) total_income: i64,
    pub(super) adjustments: i64,
    pub(super) itemized: i64,
    pub(super) itemized_reduction: i64,
    pub(super) standard: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) cash_standard: i64,
    pub(super) child_nonrefund: i64,
    pub(super) other_nonrefundable_before_business: i64,
    pub(super) additional_taxes: i64,
    pub(super) taxable: i64,
    pub(super) qualified: i64,
}

pub(super) fn injured_credit_ratios_phase(
    pass: &Pass<'_>,
    r: &FederalResult,
    i: InjuredCreditRatiosIn<'_>,
) -> Option<[i64; 2]> {
    let &Pass { d, context, s, a, pension, .. } = pass;
    let InjuredCreditRatiosIn {
        nonqualified_annuity,
        unemployment,
        injured_facts,
        credit_entries,
        account_distribution_taxes,
        early_distribution_taxes,
        care_components,
        rmd_taxes,
        property_niit,
        ira,
        health,
        income_tax_expense,
        retirement_taxable,
        additional,
        use_itemized,
        care_used,
        education_used,
        qbi_ded,
        additional_medicare,
        niit,
        nii,
        jury,
        jury_remitted,
        olympic_prize,
        olympic_taxable,
        schedule_c,
        schedule_f,
        housing_deduction,
        personal_rent,
        personal_rent_expenses,
        alimony_paid,
        msa_deduction,
        savings_penalty,
        educator,
        employee_deduction,
        estate_67e,
        half_se,
        hsa,
        moving_deduction,
        sep_deduction,
        loan,
        premium_repayment,
        clean_vehicle_repayment,
        senior_ded,
        niit_itemized,
        estate_niit_excess_deduction,
        w2_wages,
        interest,
        dividends,
        social,
        capital,
        total_income,
        adjustments,
        itemized,
        itemized_reduction,
        standard,
        qualified_net_casualty,
        cash_standard,
        child_nonrefund,
        other_nonrefundable_before_business,
        additional_taxes,
        taxable,
        qualified,
    } = i;
    if s.filing_status() == FederalStatus::Joint
        && credit_entries.iter().any(|entry| entry.filer != crate::CarryFiler::SameFilers)
    {
        changed_filer_credit_ratios(context.forms(d), r, injured_facts, || {
            let mut calculation_amounts = alloc::collections::BTreeMap::new();
            // Read only actual current source rows. A missing row is not zero;
            // components emitted later come from their owning locals below.
            for key in [
                "account_spouse_tax",
                "account_taxpayer_tax",
                "additional_medicare_tax",
                "archer_contribution_1_deduction",
                "archer_contribution_2_deduction",
                "care_credit",
                "care_credit_before_prior",
                "care_expenses_smallest",
                "care_percent",
                "care_prior_worksheet_a_line10",
                "care_prior_worksheet_a_line12",
                "early_spouse_tax",
                "early_taxpayer_tax",
                "education_credit",
                "form_8960_deductions_and_modifications",
                "form_8960_net_investment_income",
                "jury_pay_remitted",
                "household_0_tax",
                "household_1_tax",
                "net_investment_income_tax",
                "niit_disposition_adjustment",
                "nonqualified_annuity_investment_income",
                "olympic_exclusion",
                "overtime_deduction",
                "prior_year_care_credit",
                "qbi_deduction",
                "rmd_0_tax",
                "rmd_1_tax",
                "schedule_1_alimony_paid",
                "schedule_1_archer_msa_deduction",
                "schedule_1_business_income",
                "schedule_1_early_withdrawal_penalty",
                "schedule_1_educator_expenses",
                "schedule_1_employee_business_expenses",
                "schedule_1_estate_deductions",
                "schedule_1_farm_income",
                "schedule_1_half_self_employment_tax",
                "schedule_1_hsa_deduction",
                "schedule_1_ira_deduction",
                "schedule_1_moving_expenses",
                "schedule_1_pension_contributions",
                "schedule_1_self_employed_health_insurance",
                "schedule_1_self_employed_retirement",
                "schedule_1_student_loan_interest",
                "schedule_2_credit_repayments_total",
                "schedule_2_premium_credit_repayment",
                "senior_deduction",
                "tips_deduction",
                "vehicle_interest_deduction",
            ] {
                if let Some(row) = r.lines.iter().find(|row| row.id == key) {
                    calculation_amounts.insert(
                        key,
                        if matches!(key, "care_percent" | "care_prior_worksheet_a_line12") {
                            row.amount
                        } else {
                            dollars(row.amount)
                        },
                    );
                }
            }
            calculation_amounts.extend([
                ("account_taxpayer_tax", account_distribution_taxes.owners[0]),
                ("account_spouse_tax", account_distribution_taxes.owners[1]),
                ("early_taxpayer_tax", early_distribution_taxes.owners[0]),
                ("early_spouse_tax", early_distribution_taxes.owners[1]),
                ("care_credit", care_used),
                ("care_credit_before_prior", care_components.current),
                ("care_expenses_smallest", care_components.expenses_smallest),
                ("care_percent", care_components.rate),
                ("prior_year_care_credit", care_components.prior),
                ("education_credit", education_used),
                ("qbi_deduction", qbi_ded),
                ("rmd_0_tax", rmd_taxes.owners[0]),
                ("rmd_1_tax", rmd_taxes.owners[1]),
                ("additional_medicare_tax", additional_medicare),
                ("net_investment_income_tax", niit),
                ("nonqualified_annuity_investment_income", dollars(nonqualified_annuity)),
                ("form_8960_net_investment_income", nii),
                ("jury_pay", jury),
                ("schedule_1_unemployment", dollars(unemployment)),
                ("jury_pay_remitted", jury_remitted),
                ("olympic_exclusion", olympic_prize - olympic_taxable),
                ("schedule_1_business_income", schedule_c),
                ("schedule_1_farm_income", schedule_f),
                ("foreign_housing_deduction", housing_deduction),
                ("nonbusiness_personal_rent", personal_rent),
                ("schedule_1_personal_property_rental_expenses", personal_rent_expenses),
                ("schedule_1_alimony_paid", alimony_paid),
                ("schedule_1_archer_msa_deduction", msa_deduction),
                ("schedule_1_early_withdrawal_penalty", savings_penalty),
                ("schedule_1_educator_expenses", educator),
                ("schedule_1_employee_business_expenses", employee_deduction),
                ("schedule_1_estate_deductions", estate_67e),
                ("schedule_1_half_self_employment_tax", half_se),
                ("schedule_1_hsa_deduction", dollars(hsa)),
                ("schedule_1_ira_deduction", ira.deduction),
                ("schedule_1_moving_expenses", moving_deduction),
                ("schedule_1_pension_contributions", pension.deduction),
                ("schedule_1_self_employed_health_insurance", health.deduction),
                ("schedule_1_self_employed_retirement", sep_deduction),
                ("schedule_1_student_loan_interest", loan),
                ("schedule_2_credit_repayments_total", premium_repayment + clean_vehicle_repayment),
                ("schedule_2_premium_credit_repayment", premium_repayment),
                ("senior_deduction", senior_ded),
                ("tips_deduction", additional[0]),
                ("overtime_deduction", additional[1]),
                ("vehicle_interest_deduction", additional[2]),
            ]);
            if let (Some(property), Some(_)) = (property_niit, income_tax_expense) {
                calculation_amounts.insert("niit_disposition_adjustment", property.adjustment);
                calculation_amounts.insert(
                    "form_8960_deductions_and_modifications",
                    niit_itemized + savings_penalty + estate_niit_excess_deduction,
                );
            }
            // The Archer owner calculation explicitly returns no deduction for an
            // absent contributor; retain that known result, never a missing-row default.
            for (owner, key) in [
                (Owner::Taxpayer, "archer_contribution_1_deduction"),
                (Owner::Spouse, "archer_contribution_2_deduction"),
            ] {
                let present = matches!(&a.archer, crate::ArcherContributions::Records { owners, .. }
                    if owners.iter().any(|entry| entry.owner == owner));
                if !present {
                    calculation_amounts.insert(key, 0);
                }
            }
            crate::injured_spouse::owners::LiabilityTotals {
                // The shared projection reads the native1040line1a subtotal.
                // Its retained actual sources determine8379 classification;
                // the native destination alone does not establish that class.
                wages: w2_wages,
                interest,
                dividends,
                retirement: retirement_taxable,
                social,
                capital,
                total_income,
                adjustments,
                standard_or_itemized: if use_itemized {
                    itemized - itemized_reduction
                } else {
                    standard + qualified_net_casualty
                },
                cash_charity: if use_itemized { 0 } else { cash_standard },
                child_credit: child_nonrefund,
                other_nonrefundable: other_nonrefundable_before_business,
                other_taxes: additional_taxes + premium_repayment + clean_vehicle_repayment,
                taxable,
                qualified_dividends: qualified,
                calculation_amounts,
            }
        })
    } else {
        None
    }
}

pub(super) struct InjuredIncomeCategoriesIn<'a> {
    pub(super) injured_boundaries: &'a [(
        crate::injured_spouse::income_categories::IncomeCategory,
        Exact,
        i64,
        Option<[Exact; 2]>,
    )],
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) business_exclusions: &'a [crate::foreign_business::BusinessExclusion; 2],
    pub(super) health: &'a crate::health::HealthResult,
    pub(super) employee: &'a crate::employee_expenses::Calculation,
    pub(super) form_2441: &'a crate::forms::form_2441::Form2441,
    pub(super) property_niit: &'a Option<crate::section1231::PropertyNiit>,
    pub(super) estate_estimated_cents_by_owner: &'a [Exact; 2],
    pub(super) sep: &'a crate::sep::SepResult,
    pub(super) injured_adoption:
        Box<crate::injured_spouse::income_categories::AdoptionIncomeDollars>,
    pub(super) additional_medicare_sources:
        Box<crate::injured_spouse::income_categories::NativeAdditionalMedicare>,
    pub(super) foreign_credit_sources:
        crate::injured_spouse::income_categories::NativeForeignCreditSources,
    pub(super) injured_medicare_withholding:
        Box<crate::injured_spouse::income_categories::NativeMedicareWithholding>,
    pub(super) injured_fuel_credit_sources:
        Option<Box<crate::injured_spouse::income_categories::NativeFuelCreditSources>>,
    pub(super) injured_niit_itemized:
        Box<crate::injured_spouse::niit_deductions::NiitItemizedDeductions>,
    pub(super) injured_property_sources: Option<crate::section1231::PropertyOwnerAmounts>,
    pub(super) saver_eligible_sources: Option<[i64; 2]>,
    pub(super) employee_wage_taxes: [i64; 2],
    pub(super) tip_payroll_owners: [i64; 2],
    pub(super) half: [i64; 2],
    pub(super) late_investment_sources: [[i64; 2]; 2],
    pub(super) statutory_form_owners: [[i64; 2]; 2],
    pub(super) business_form_owners: [[i64; 2]; 2],
    pub(super) investment_sources_known: [bool; 2],
    pub(super) employee_payroll_complete: bool,
    pub(super) self_rental_current_components_zero: bool,
    pub(super) estate_niit_prior_sources_known: bool,
    pub(super) statutory_sources_known: bool,
    pub(super) hobby: i64,
    pub(super) passive_business_niit: i64,
    pub(super) tip_payroll: i64,
    pub(super) uncollected_payroll: i64,
    pub(super) golden_parachute_tax: i64,
    pub(super) saver: i64,
    pub(super) care_benefits: i64,
    pub(super) prior_niit_difference: i64,
    pub(super) personal_rent: i64,
    pub(super) w2_wages: i64,
    pub(super) schedule_c: i64,
    pub(super) schedule_f: i64,
    pub(super) business: i64,
    pub(super) half_se: i64,
    pub(super) sep_deduction: i64,
    pub(super) business_excluded: i64,
    pub(super) property_business_capital: i64,
    pub(super) property_ordinary: i64,
    pub(super) k1_business_ordinary: i64,
    pub(super) excess_business_loss: i64,
    pub(super) excess_business_loss_operand: i64,
}

pub(super) fn injured_income_categories_phase(
    pass: &Pass<'_>,
    r: &FederalResult,
    t: &Totals,
    i: InjuredIncomeCategoriesIn<'_>,
) -> crate::injured_spouse::income_categories::NativeIncomeCategories {
    let &Pass { d, allocated_entries, context, .. } = pass;
    let direct_participation = context.participation;
    use crate::injured_spouse::income_categories::{IncomeCategory as IC, NativeIncomeCategories};
    let InjuredIncomeCategoriesIn {
        injured_boundaries,
        rental,
        business_exclusions,
        health,
        employee,
        form_2441,
        property_niit,
        estate_estimated_cents_by_owner,
        sep,
        injured_adoption,
        additional_medicare_sources,
        foreign_credit_sources,
        injured_medicare_withholding,
        injured_fuel_credit_sources,
        injured_niit_itemized,
        injured_property_sources,
        saver_eligible_sources,
        employee_wage_taxes,
        tip_payroll_owners,
        half,
        late_investment_sources,
        statutory_form_owners,
        business_form_owners,
        investment_sources_known,
        employee_payroll_complete,
        self_rental_current_components_zero,
        estate_niit_prior_sources_known,
        statutory_sources_known,
        hobby,
        passive_business_niit,
        tip_payroll,
        uncollected_payroll,
        golden_parachute_tax,
        saver,
        care_benefits,
        prior_niit_difference,
        personal_rent,
        w2_wages,
        schedule_c,
        schedule_f,
        business,
        half_se,
        sep_deduction,
        business_excluded,
        property_business_capital,
        property_ordinary,
        k1_business_ordinary,
        excess_business_loss,
        excess_business_loss_operand,
    } = i;
    let mut injured_income_categories =
        NativeIncomeCategories::collect(d, &t.injured_income_components, injured_boundaries);
    // Keep the actual final Hobby-goods annual boundary and its already
    // captured native owner Cents. Generic not-for-profit compensation is
    // a separate category and is not added to Form8960 here.
    injured_income_categories.niit_hobby =
        NativeIncomeCategories::source_owners(d, &t.injured_income_components, IC::Hobby, [0; 2])
            .filter(|owners| owners.iter().cloned().sum::<Exact>() == exact::whole(t.hobby))
            .map(|owners| (hobby, owners));
    // The closed native payroll collection supplies both actual owner taxes,
    // including genuine zero; an absent synthetic packet stays unknown.
    let mut passive_niit_owners = [0; 2];
    for (&id, &amount) in &t.passive_business_niit {
        if let Some(entry) = d.income.iter().find(|entry| entry.id == id) {
            passive_niit_owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] +=
                amount;
        }
    }
    injured_income_categories.niit_passive_business =
        Some((passive_business_niit, passive_niit_owners));
    injured_income_categories.employee_wage_tax_dollars =
        employee_payroll_complete.then_some(employee_wage_taxes);
    injured_income_categories.other_payroll_taxes =
        Some(Box::new(crate::injured_spouse::income_categories::NativeOtherPayrollTaxes {
            tips: (tip_payroll, tip_payroll_owners),
            uncollected: (
                uncollected_payroll,
                [0, 1].map(|i| t.uncollected_social[i] + t.uncollected_medicare_owners[i]),
            ),
            golden: (golden_parachute_tax, t.golden_parachute_tax),
        }));
    injured_income_categories.additional_medicare = Some(additional_medicare_sources);
    // The final allowed pool is already settled. A sole eligible contributor
    // establishes its owner without a second credit computation or tax limit.
    injured_income_categories.saver_credit_owners = if saver == 0 {
        Some([0; 2])
    } else {
        match saver_eligible_sources {
            Some([first, 0]) if first > 0 => Some([saver, 0]),
            Some([0, second]) if second > 0 => Some([0, saver]),
            _ => None,
        }
    };
    injured_income_categories.foreign_credit_sources = Some(Box::new(foreign_credit_sources));
    injured_income_categories.estate_estimated_cents_by_owner =
        Some(estate_estimated_cents_by_owner.clone());
    injured_income_categories.medicare_withholding = Some(injured_medicare_withholding);
    injured_income_categories.fuel_credit_sources = injured_fuel_credit_sources;
    injured_income_categories.ltc_dollars = care_benefits;
    injured_income_categories.ltc_sources_complete =
        t.injured_ltc.iter().try_fold(0_i64, |sum, part| {
            (part.taxable >= 0).then(|| sum.checked_add(part.taxable)).flatten()
        }) == Some(t.care_benefits);
    injured_income_categories.niit_itemized = Some(injured_niit_itemized);
    let non_nua_components_known_zero = property_niit.as_ref().is_some_and(|source| {
        source.current_components_known_zero
            && self_rental_current_components_zero
            && estate_niit_prior_sources_known
            && rental.inputs_complete
            && r.issues.is_empty()
    });
    // Each actual NUA entry must have exactly one confirmed collector result.
    // Nonnegative realized exclusions cannot conceal opposite-source offsets;
    // a missing/unconfirmed result remains unknown, never zero.
    let nua_owners = allocated_entries
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::NuaSale { .. }))
        .try_fold([0_i64; 2], |mut owners, entry| {
            let mut values = t.nua_exclusion_sources.iter().filter(|(id, _)| *id == entry.id);
            let (Some((_, Some(amount))), None) = (values.next(), values.next()) else {
                return None;
            };
            if *amount < 0 {
                return None;
            }
            let own = usize::from(entry.owner == crate::DocumentOwner::Spouses);
            owners[own] = owners[own].checked_add(*amount)?;
            Some(owners)
        });
    injured_income_categories.niit_current_nua_cents =
        non_nua_components_known_zero.then_some(nua_owners).flatten();
    injured_income_categories.niit_retained = property_niit.as_ref().map(|source| {
        alloc::boxed::Box::new(crate::injured_spouse::niit_retained::RetainedNiitUse {
            prior_dollars: prior_niit_difference,
            used_dollars: source.adjustment,
            next_dollars: source.next_capital_adjustment,
            current_components_known_zero: non_nua_components_known_zero
                && nua_owners == Some([0; 2]),
        })
    });
    injured_income_categories.personal_rent_dollars = personal_rent;
    if !injured_adoption.source_class_complete || injured_adoption.deemed_excess != 0 {
        // Unknown source class withholds ALL printed income columns,
        // including column(a). It does not erase known financial ownership.
        injured_income_categories.source_class_unavailable = true;
        injured_income_categories.unclassified_dollars +=
            injured_adoption.owners.iter().sum::<i64>()
                - injured_adoption.w2_owners.iter().sum::<i64>()
                - injured_adoption.deemed_excess;
    }
    injured_income_categories.adoption = Some(injured_adoption);
    injured_income_categories.care_taxable_dollars = form_2441.taxable;
    injured_income_categories.care_taxable_from_w2 = form_2441.taxable_from_w2;
    for i in 0..2 {
        if form_2441.taxable[i] > 0 && !form_2441.taxable_from_w2[i] {
            injured_income_categories.source_class_unavailable = true;
            injured_income_categories.unclassified_dollars += form_2441.taxable[i];
        }
    }
    if let Some(other) = employee.non_w2_excess_cents() {
        injured_income_categories.line_1a_other_cents = other;
        // Establish the whole native category, before the approved W-2-first
        // source-class residual. Independent class rounding need not foot it.
        let native_joint = dollars(
            t.wages.iter().sum::<i64>()
                - t.employee_tips.iter().map(|v| v.income).sum::<i64>()
                - t.employee_wages.iter().sum::<i64>()
                - t.employee_wages_not_covered.iter().sum::<i64>(),
        );
        if native_joint != w2_wages {
            injured_income_categories.line_1a_class_unavailable = true;
            injured_income_categories.source_class_unavailable = true;
        }
    } else {
        injured_income_categories.line_1a_class_unavailable = true;
        injured_income_categories.source_class_unavailable = true;
    }
    for (index, (category, native_cents)) in
        [(IC::Interest, t.interest), (IC::OrdinaryDividends, t.dividends)].into_iter().enumerate()
    {
        injured_income_categories.investment_sources[index] = investment_sources_known[index]
            .then(|| {
                NativeIncomeCategories::source_owners(
                    d,
                    &t.injured_income_components,
                    category,
                    late_investment_sources[index],
                )
            })
            .flatten()
            .filter(|owners| owners.iter().cloned().sum::<Exact>() == exact::whole(native_cents));
    }
    if statutory_sources_known {
        let source = crate::injured_spouse::income_categories::NativeBusinessIncome {
            joint_dollars: [schedule_c, schedule_f],
            w2_cents: statutory_form_owners,
            foreign_exclusion_dollars: business_exclusions
                .each_ref()
                .map(|source| source.statutory_net_excluded),
            other_cents: core::array::from_fn(|form| {
                core::array::from_fn(|owner| {
                    business_form_owners[form][owner] - statutory_form_owners[form][owner]
                })
            }),
        };
        if source.reconciles([schedule_c, schedule_f], t.business) {
            injured_income_categories.statutory_business = Some(source);
        }
    }
    if injured_income_categories.statutory_business.is_none()
        && d.income.iter().any(|entry| {
            matches!(&entry.income,
            FederalIncome::Wage { data } if crate::FederalWage::from(data).statutory_employee)
        })
    {
        // Both printed class targets are unknown, not just their owner split.
        // The original whole native financial category remains independent.
        injured_income_categories.source_class_unavailable = true;
        injured_income_categories.unclassified_dollars += business;
    }
    // Retain the exact native461 operand, not a second461 calculation or
    // research-credit attribution. Source Dollars and genuine Cents remain
    // distinct until the existing whole-category owner projection.
    injured_income_categories.excess_business_loss = {
        use crate::injured_spouse::income_categories::{
            Native461IncomeGroup as Group, Native461SourceIssue as Issue, Native461Sources,
        };
        let groups = (|| {
            if !rental.inputs_complete || !r.issues.is_empty() {
                return Err(Issue::IncompleteSources);
            }
            // Direct risk/PAL sources require their final owner feed;
            // never silently reuse this raw net.
            for entry in &d.income {
                if let FederalIncome::Business { data } = &entry.income {
                    let net = t.business_profits.get(&entry.id).ok_or(Issue::IncompleteSources)?;
                    if !(direct_participation.get(entry.id)
                        == crate::participation::Participation::Yes)
                        || (*net < 0
                            && !data.current_risk_supported(direct_participation.get(entry.id)))
                    {
                        return Err(Issue::IncompleteSources);
                    }
                }
            }
            let owner_dollars = |sources: &alloc::collections::BTreeMap<i64, i64>| {
                sources.iter().try_fold([0i64; 2], |mut owners, (id, amount)| {
                    let entry = d.income.iter().find(|entry| entry.id == *id)?;
                    owners[usize::from(entry.owner == crate::DocumentOwner::Spouses)] = owners
                        [usize::from(entry.owner == crate::DocumentOwner::Spouses)]
                    .checked_add(*amount)?;
                    Some(owners)
                })
            };
            let health_owners =
                owner_dollars(&health.by_business).ok_or(Issue::IncompleteSources)?;
            if health_owners.iter().sum::<i64>() != health.deduction {
                return Err(Issue::IncompleteSources);
            }
            let deductions = [0, 1].map(|i| {
                -half[i]
                    - sep.owners[i]
                    - health_owners[i]
                    - business_exclusions[i].business_net
                    - business_exclusions[i].housing_deduction
            });
            if deductions.iter().sum::<i64>()
                != -half_se - sep_deduction - health.deduction - business_excluded
            {
                return Err(Issue::IncompleteSources);
            }
            if rental.activities.len() != rental.qbi_source_ids.len() {
                return Err(Issue::IncompleteSources);
            }
            let qbi_owners = rental
                .activities
                .iter()
                .zip(&rental.qbi_source_ids)
                .try_fold([0i64; 2], |mut owners, (activity, id)| {
                    let entry = d.income.iter().find(|entry| entry.id == *id)?;
                    if activity.owner.is_some_and(|owner| owner != entry.owner.owned()) {
                        return None;
                    }
                    let i = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                    owners[i] = owners[i].checked_add(activity.income)?;
                    Some(owners)
                })
                .ok_or(Issue::IncompleteSources)?;
            let ordinary =
                owner_dollars(&rental.ordinary_business_by_id).ok_or(Issue::IncompleteSources)?;
            let ordinary_joint = rental.ordinary_pass_through_business();
            if ordinary.iter().sum::<i64>() != ordinary_joint
                || qbi_owners.iter().sum::<i64>() != rental.qbi
            {
                return Err(Issue::IncompleteSources);
            }
            let capital_joint = property_business_capital;
            let property = injured_property_sources.ok_or(if capital_joint == 0 {
                Issue::IncompleteSources
            } else {
                Issue::CapitalOwnerUnavailable
            })?;
            if dollars(property.ordinary.iter().sum::<i64>()) != property_ordinary {
                return Err(Issue::IncompleteSources);
            }
            let capital_owners = if capital_joint == 0 {
                [0; 2]
            } else if property.long_term.iter().all(|value| *value >= 0)
                && dollars(property.long_term.iter().sum::<i64>()) == capital_joint
            {
                property.long_term
            } else {
                match property.long_term {
                    [own, 0] if (0..=dollars(own)).contains(&capital_joint) => [
                        crate::extended_rules::annual_dollars_to_working_cents(capital_joint)
                            .map_err(|_| Issue::CapitalOwnerUnavailable)?,
                        0,
                    ],
                    [0, own] if (0..=dollars(own)).contains(&capital_joint) => [
                        0,
                        crate::extended_rules::annual_dollars_to_working_cents(capital_joint)
                            .map_err(|_| Issue::CapitalOwnerUnavailable)?,
                    ],
                    _ => return Err(Issue::CapitalOwnerUnavailable),
                }
            };
            let cuts = rental
                .k1_cuts
                .ordinary_by_id
                .iter()
                .try_fold([0i64; 2], |mut owners, (id, amount)| {
                    let entry = d.income.iter().find(|entry| entry.id == *id)?;
                    let i = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                    owners[i] = owners[i].checked_add(amount.get())?;
                    Some(owners)
                })
                .ok_or(Issue::IncompleteSources)?;
            if cuts.iter().sum::<i64>() != rental.k1_cuts.ordinary.get() {
                return Err(Issue::IncompleteSources);
            }
            let k1_owners = [
                t.injured_k1_nii.ordinary[0]
                    .checked_add(cuts[0])
                    .ok_or(Issue::IncompleteSources)?,
                t.injured_k1_nii.ordinary[1]
                    .checked_add(cuts[1])
                    .ok_or(Issue::IncompleteSources)?,
            ];
            Ok(alloc::vec![
                Group::Cents { joint_dollars: business, owners: t.business },
                Group::Cents { joint_dollars: k1_business_ordinary, owners: k1_owners },
                Group::Dollars { joint_dollars: deductions.iter().sum(), owners: deductions },
                Group::Dollars { joint_dollars: rental.qbi, owners: qbi_owners },
                Group::Dollars { joint_dollars: ordinary_joint, owners: ordinary },
                Group::Cents { joint_dollars: property_ordinary, owners: property.ordinary },
                Group::Cents { joint_dollars: capital_joint, owners: capital_owners },
            ])
        })();
        Native461Sources {
            joint_addback_dollars: excess_business_loss,
            eligible_joint_dollars: -excess_business_loss_operand,
            groups,
        }
    };
    injured_income_categories
}

pub(super) struct InjuredBoundariesIn<'a> {
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) moving: &'a crate::moving::Moving,
    pub(super) employee_recapture: [i64; 2],
    pub(super) alimony_recaptured_owners: [i64; 2],
    pub(super) scholarship_inclusions: [i64; 2],
    pub(super) royalty_interest: [i64; 2],
    pub(super) clergy_housing_excess: [i64; 2],
    pub(super) household_wages: i64,
    pub(super) medicaid_waiver_wages: i64,
    pub(super) strike_benefits: i64,
    pub(super) tax_refund: i64,
    pub(super) alaska: i64,
    pub(super) prizes: i64,
    pub(super) not_for_profit: i64,
    pub(super) olympic_prize: i64,
    pub(super) medicaid_waiver_excluded: i64,
    pub(super) inmate_pay: i64,
    pub(super) deferred_pension: i64,
    pub(super) digital: i64,
    pub(super) other_8z: i64,
    pub(super) tip_income: i64,
    pub(super) employee_wage_income: i64,
    pub(super) employee_wages_not_covered: i64,
    pub(super) disability_pension: i64,
    pub(super) iso_unreported: i64,
    pub(super) alimony_recaptured: i64,
    pub(super) alimony: i64,
    pub(super) gambling: i64,
    pub(super) personal_debt: i64,
    pub(super) hobby: i64,
    pub(super) scholarship: i64,
    pub(super) taxable_grants: i64,
    pub(super) family_leave: i64,
    pub(super) taxed_relief: i64,
    pub(super) health_assistance: i64,
    pub(super) trump_death: i64,
    pub(super) royalties: i64,
    pub(super) pure_loan_gain: i64,
    pub(super) clergy_excess: i64,
    pub(super) moving_excess: i64,
    pub(super) deferral_excess: i64,
}

pub(super) fn injured_boundaries_phase(
    pass: &Pass<'_>,
    t: &Totals,
    i: InjuredBoundariesIn<'_>,
) -> Vec<(crate::injured_spouse::income_categories::IncomeCategory, Exact, i64, Option<[Exact; 2]>)>
{
    use crate::injured_spouse::income_categories::{IncomeCategory as IC, NativeIncomeCategories};
    let d = pass.d;
    let InjuredBoundariesIn {
        rental,
        moving,
        employee_recapture,
        alimony_recaptured_owners,
        scholarship_inclusions,
        royalty_interest,
        clergy_housing_excess,
        household_wages,
        medicaid_waiver_wages,
        strike_benefits,
        tax_refund,
        alaska,
        prizes,
        not_for_profit,
        olympic_prize,
        medicaid_waiver_excluded,
        inmate_pay,
        deferred_pension,
        digital,
        other_8z,
        tip_income,
        employee_wage_income,
        employee_wages_not_covered,
        disability_pension,
        iso_unreported,
        alimony_recaptured,
        alimony,
        gambling,
        personal_debt,
        hobby,
        scholarship,
        taxable_grants,
        family_leave,
        taxed_relief,
        health_assistance,
        trump_death,
        royalties,
        pure_loan_gain,
        clergy_excess,
        moving_excess,
        deferral_excess,
    } = i;
    alloc::vec![
        (
            IC::HouseholdWages,
            exact::whole(t.household_wages.iter().sum()),
            household_wages,
            Some((t.household_wages).map(exact::whole))
        ),
        (
            IC::MedicaidWaiverWages,
            exact::whole(
                t.medicaid_taxable.iter().sum::<i64>() + t.medicaid_excluded.iter().sum::<i64>()
            ),
            medicaid_waiver_wages,
            Some(
                ([0, 1].map(|i| t.medicaid_taxable[i] + t.medicaid_excluded[i])).map(exact::whole)
            )
        ),
        (
            IC::StrikeBenefits,
            exact::whole(t.strike_benefits.iter().sum()),
            strike_benefits,
            Some((t.strike_benefits).map(exact::whole))
        ),
        (IC::TaxableRefunds, exact::whole(t.tax_refund), tax_refund, None),
        (IC::AlaskaDividends, exact::whole(t.alaska), alaska, None),
        (IC::Prizes, exact::whole(t.prizes), prizes, None),
        (IC::NotForProfit, exact::whole(t.not_for_profit), not_for_profit, None),
        (IC::OlympicPrizes, exact::whole(t.olympic), olympic_prize, None),
        (
            IC::MedicaidWaiverExclusion,
            exact::whole(
                -t.medicaid_excluded.iter().sum::<i64>() - t.medicaid_in_box1.iter().sum::<i64>()
            ),
            -medicaid_waiver_excluded,
            Some(
                ([0, 1].map(|i| -t.medicaid_excluded[i] - t.medicaid_in_box1[i])).map(exact::whole)
            )
        ),
        (
            IC::InmatePay,
            exact::whole(t.inmate_pay.iter().sum()),
            inmate_pay,
            Some((t.inmate_pay).map(exact::whole))
        ),
        (
            IC::DeferredPension,
            exact::whole(t.deferred_pension.iter().sum()),
            deferred_pension,
            Some((t.deferred_pension).map(exact::whole))
        ),
        (IC::DigitalAssets, exact::whole(t.digital), digital, None),
        (
            IC::Other8z,
            exact::whole(t.other_8z + t.recovery_8z),
            other_8z,
            NativeIncomeCategories::source_owners(
                d,
                &t.injured_income_components,
                IC::Other8z,
                employee_recapture,
            )
        ),
        (
            IC::EmployeeTips,
            exact::whole(t.employee_tips.iter().map(|v| v.income).sum()),
            tip_income,
            Some(([0, 1].map(|i| t.employee_tips[i].income)).map(exact::whole))
        ),
        (
            IC::EmployeeWages,
            exact::whole(t.employee_wages.iter().sum()),
            employee_wage_income,
            Some((t.employee_wages).map(exact::whole))
        ),
        (
            IC::UncoveredEmployeeWages,
            exact::whole(t.employee_wages_not_covered.iter().sum()),
            employee_wages_not_covered,
            Some((t.employee_wages_not_covered).map(exact::whole))
        ),
        (
            IC::DisabilityPension,
            exact::whole(t.disability_pension.iter().sum()),
            disability_pension,
            Some((t.disability_pension).map(exact::whole))
        ),
        (
            IC::IsoCompensation,
            exact::whole(t.iso_unreported.iter().sum()),
            iso_unreported,
            Some((t.iso_unreported).map(exact::whole))
        ),
        (
            IC::Alimony,
            exact::whole(t.alimony.iter().sum::<i64>() + alimony_recaptured),
            alimony,
            Some(([0, 1].map(|i| t.alimony[i] + alimony_recaptured_owners[i])).map(exact::whole))
        ),
        (IC::Gambling, exact::whole(t.gambling), gambling, None),
        (IC::PersonalDebt, exact::whole(t.personal_debt), personal_debt, None),
        (IC::Hobby, exact::whole(t.hobby), hobby, None),
        (
            IC::Scholarship,
            exact::whole(t.scholarship),
            scholarship,
            NativeIncomeCategories::source_owners(
                d,
                &t.injured_income_components,
                IC::Scholarship,
                scholarship_inclusions,
            )
        ),
        (IC::Grants, exact::whole(t.taxable_grants), taxable_grants, None),
        (IC::FamilyLeave, exact::whole(t.family_leave), family_leave, None),
        (
            IC::CasualtyLivingInsurance,
            exact::whole(t.casualty_living_insurance),
            dollars(t.casualty_living_insurance),
            None
        ),
        (IC::TaxedRelief, exact::whole(t.taxed_relief), taxed_relief, None),
        (IC::HealthLoanAssistance, exact::whole(t.health_assistance), health_assistance, None),
        (IC::TrumpDeath, exact::whole(t.trump_death), trump_death, None),
        (
            IC::Royalties,
            &t.royalties - exact::whole(royalty_interest[0]) * exact::whole(100),
            royalties,
            None
        ),
        (IC::ScorpLoanGain, exact::whole(t.scorp_loan_gain[0]), pure_loan_gain, None),
        (
            IC::K1OrdinaryGain,
            exact::whole(t.k1_ordinary[0] + rental.k1_cuts.ordinary.get()),
            dollars(t.k1_ordinary[0] + rental.k1_cuts.ordinary.get()),
            None
        ),
        (
            IC::ClergyHousing,
            exact::whole(clergy_housing_excess.iter().sum()),
            clergy_excess,
            Some((clergy_housing_excess).map(exact::whole))
        ),
        (
            IC::MovingExcess,
            exact::whole(moving.excess.iter().sum()),
            moving_excess,
            Some((moving.excess).map(exact::whole))
        ),
        (
            IC::DeferralExcess,
            exact::whole(t.deferral_excess.iter().sum()),
            deferral_excess,
            Some((t.deferral_excess).map(exact::whole))
        ),
    ]
}
