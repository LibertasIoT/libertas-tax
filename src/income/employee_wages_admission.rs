// Admit the new uncapped source amounts only when their actual native aggregate
// carriers fit. This is an i64 export boundary, not a tax-source amount cap.
pub(crate) fn return_issue(
    d: &crate::FederalDraft,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    let source_claims = crate::related_forms::effective_claimants(d);
    return_issue_with_claims(d, &source_claims)
}

pub(crate) fn return_issue_with_claims(
    d: &crate::FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, crate::validation::ValidationIssue)> {
    // Keep the source of each actual carrier; a zero-value employee record
    // cannot explain another record's wage or payroll overflow.
    type Source = (usize, i64, &'static [&'static str]);
    #[derive(Clone, Copy, Default)]
    struct Carrier {
        amount: i128,
        source: Option<Source>,
    }
    impl Carrier {
        fn add(&mut self, amount: i128, source: Source) {
            self.amount += amount;
            if amount != 0 && self.source.is_none() {
                self.source = Some(source);
            }
        }
        fn combined(self, other: Self) -> Self {
            Self {
                amount: self.amount + other.amount,
                source: self.source.into_iter().chain(other.source).min_by_key(|source| source.0),
            }
        }
    }
    let mut wages = Carrier::default();
    let mut wage_previews = Carrier::default();
    let mut medicare = [Carrier::default(); 2];
    let mut social = [Carrier::default(); 2];
    let mut withheld = Carrier::default();
    let mut railroad = [Carrier::default(); 2];
    let mut payroll_taxes = [Carrier::default(); 2];
    let joint =
        d.setup.as_ref().is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Joint);
    if !d
        .income
        .iter()
        .any(|entry| matches!(entry.income, crate::FederalIncome::EmployeeWages { .. }))
    {
        return None;
    }
    for (index, entry) in d.income.iter().enumerate() {
        let source = |path: &'static [&'static str]| (index, entry.id, path);
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        let income = |amount| i128::from(crate::community::income_amount(d, entry.id, amount));
        let withholding = |amount| {
            i128::from(
                crate::community::withholding_share(d, entry, source_claims)
                    .map_or(amount, |share| share.filer(amount)),
            )
        };
        match &entry.income {
            crate::FederalIncome::Wage { data } => {
                let data = &crate::FederalWage::from(data);
                wages.add(income(data.wages), source(&["wages"]));
                // The Income hub sums whole document previews, before community sharing.
                if let Some(amount) = crate::interview::document_amount(
                    &entry.income,
                    crate::federal_rules::separate_return(d),
                ) {
                    wage_previews.add(
                        i128::from(amount),
                        source(if data.wages != 0 { &["wages"] } else { &[] }),
                    );
                }
                medicare[owner].add(i128::from(data.medicare_wages), source(&["medicare_wages"]));
                social[owner].add(i128::from(data.social_wages), source(&["social_wages"]));
                social[owner].add(i128::from(data.social_tips), source(&["social_tips"]));
                withheld.add(withholding(data.withholding), source(&["withholding"]));
                payroll_taxes[owner].add(i128::from(data.social_tax), source(&["social_tax"]));
                payroll_taxes[owner].add(i128::from(data.medicare_tax), source(&["medicare_tax"]));
                if let crate::W2Payroll::Railroad {
                    compensation,
                    tier_1,
                    medicare: rrta_medicare,
                    additional_medicare,
                    ..
                } = &data.payroll
                {
                    railroad[owner].add(
                        i128::from(*compensation),
                        source(&["more", "payroll", "compensation"]),
                    );
                    social[owner].add(
                        i128::from(
                            (*compensation)
                                .min(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS),
                        ),
                        source(&["more", "payroll", "compensation"]),
                    );
                    if joint || owner == 0 {
                        withheld.add(
                            i128::from(*additional_medicare),
                            source(&["more", "payroll", "additional_medicare"]),
                        );
                    }
                    payroll_taxes[owner]
                        .add(i128::from(*tier_1), source(&["more", "payroll", "tier_1"]));
                    payroll_taxes[owner]
                        .add(i128::from(*rrta_medicare), source(&["more", "payroll", "medicare"]));
                }
            }
            crate::FederalIncome::EmployeeWages { data } => {
                wages.add(income(data.wages), source(&["wages"]));
                if let Some(amount) = crate::interview::document_amount(
                    &entry.income,
                    crate::federal_rules::separate_return(d),
                ) {
                    wage_previews.add(i128::from(amount), source(&["wages"]));
                }
                withheld.add(withholding(data.withholding), source(&["withholding"]));
                if let Some(reported) = data.railroad() {
                    let [compensation, tier_1, _, ordinary_medicare, additional] =
                        reported.known().map(|amount| i128::from(amount.unwrap_or(0)));
                    let compensation_source =
                        source(&["payroll", "reported", "compensation", "amount"]);
                    railroad[owner].add(compensation, compensation_source);
                    social[owner].add(
                        compensation.min(i128::from(
                            crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS,
                        )),
                        compensation_source,
                    );
                    payroll_taxes[owner]
                        .add(tier_1, source(&["payroll", "reported", "tier_1", "amount"]));
                    payroll_taxes[owner].add(
                        ordinary_medicare,
                        source(&["payroll", "reported", "medicare", "amount"]),
                    );
                    if joint || owner == 0 {
                        withheld.add(
                            additional,
                            source(&["payroll", "reported", "additional_medicare", "amount"]),
                        );
                    }
                } else {
                    let coverage = data.coverage();
                    // Include unavailable endpoints too: they must be representable
                    // before the native owners may compare their final results.
                    if coverage[0] != Some(false) {
                        social[owner].add(i128::from(data.wages), source(&["wages"]));
                    }
                    if coverage[1] != Some(false) {
                        medicare[owner].add(i128::from(data.wages), source(&["wages"]));
                    }
                }
            }
            _ => {}
        }
    }

    let physical = [medicare, social, railroad, payroll_taxes].into_iter().flat_map(|amounts| {
        amounts.into_iter().chain(joint.then(|| amounts[0].combined(amounts[1])))
    });
    let failed = [wages, withheld, wage_previews]
        .into_iter()
        .chain(physical)
        .find(|carrier| i64::try_from(carrier.amount).is_err())?;
    let (_, id, path) = failed.source.expect("nonzero aggregate has an actual source");
    let mut issue = crate::validation::ValidationIssue::new("EMPLOYEE_PAYROLL_RANGE").at_record();
    for field in path.iter().rev() {
        issue = issue.in_field(field);
    }
    Some((id, issue))
}
