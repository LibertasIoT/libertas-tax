//! Return-stage binding of the native selected-system suppliers. The caller
//! supplies its already supported/routed, same-pass care projection; this module
//! never routes receipts or reconstructs source income from an allowed result.
use crate::{FederalDraft, FederalIncome, Section179IncomeSystem};
use alloc::{collections::BTreeMap, vec, vec::Vec};

#[derive(Debug)]
pub(crate) enum Issue {
    Prepared(crate::Section179PreparedIssue),
    Activity(i64, &'static str, &'static str),
    Ptp(i64, crate::Ptp179SourceIssue),
    Passive(i64, crate::ptp_passive::PtpPalIssue),
}
impl Issue {
    pub fn report(self, result: &mut crate::FederalResult) {
        use crate::{K1Section179BasisIssue as B, Section179PreparedIssue as P};
        let (id, key) = match self {
            Self::Prepared(P::NativeAmountOverflow) => {
                (None, crate::k1_activity::ordinary::RANGE_KEY)
            }
            Self::Activity(id, key, english) => {
                let _ = english;
                (Some(id), key)
            }
            Self::Ptp(_, crate::Ptp179SourceIssue::OwnerIssues(issues))
            | Self::Prepared(P::OwnerIssues(issues)) => {
                result.issues.extend(issues);
                return;
            }
            Self::Prepared(P::InvalidSource(id, key) | P::SourceBoundaryUnavailable(id, key))
            | Self::Prepared(P::K1Availability(
                id,
                B::InvalidSource(key) | B::SourceLedgerUnavailable(key),
            )) => (Some(id), key),
            Self::Prepared(P::RiskUnavailable(id))
            | Self::Ptp(id, crate::Ptp179SourceIssue::RiskUnavailable) => {
                (Some(id), "ISSUE_SECTION179_RISK_HISTORY")
            }
            Self::Prepared(P::RiskOrderingCycle(id)) => {
                (Some(id), "ISSUE_SECTION179_RISK_CARRYFORWARD")
            }
            Self::Prepared(P::K1Availability(
                id,
                B::AlternativeHistoryUnavailable | B::AlternativeIncomeCarryUnavailable,
            ))
            | Self::Prepared(P::AlternativeAssetCarryUnavailable(id, _)) => {
                (Some(id), "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE")
            }
            Self::Prepared(
                P::K1Availability(id, _)
                | P::SourceIncomeUnavailable(id)
                | P::UnknownSource(id)
                | P::DuplicateSource(id)
                | P::MissingSource(id)
                | P::WrongSourceKind(id)
                | P::WrongSystem(id)
                | P::AssetIdentityUnavailable(id)
                | P::NegativePriorLoss(id)
                | P::AmountOverflow(id),
            ) => (Some(id), "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"),
            Self::Prepared(
                P::MissingReturnContext
                | P::Association(_)
                | P::DuplicateAsset(_)
                | P::AggregateAmountOverflow
                | P::WageIncomeUnavailable
                | P::OtherDeductionUnavailable
                | P::OtherDeductionWrongSystem
                | P::NegativeOtherDeduction,
            ) => (None, "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"),
            Self::Ptp(id, _) => (Some(id), "ISSUE_SECTION179_PTP_SOURCE_UNAVAILABLE"),
            Self::Passive(id, issue) => {
                let _ = issue;
                (Some(id), "ISSUE_SECTION179_PTP_SOURCE_UNAVAILABLE")
            }
        };
        let context = if key == crate::k1_activity::ordinary::RANGE_KEY {
            crate::IssueContext::Section(crate::TaxSection::PassThrough)
        } else if key.starts_with("HOME_") {
            crate::IssueContext::Section(crate::TaxSection::HomeOffices)
        } else {
            id.filter(|id| *id > 0).map_or(crate::IssueContext::Business, |id| {
                crate::IssueContext::income_id(id).manual()
            })
        };
        let args = if matches!(
            key,
            "ISSUE_SECTION179_RISK_HISTORY" | "ISSUE_SECTION179_RISK_CARRYFORWARD"
        ) {
            alloc::vec![crate::MessageArg::Text("Schedule K-1".into())]
        } else {
            alloc::vec![]
        };
        crate::federal_rules::issue_message(
            result,
            &context,
            #[cfg(test)]
            "Section 179 source",
            #[cfg(test)]
            "The selected section 179 source is unavailable; resolve its actual source records before calculating the return.",
            crate::Message::new(key, args),
        );
    }
}

impl From<crate::Section179PreparedIssue> for Issue {
    fn from(value: crate::Section179PreparedIssue) -> Self {
        Self::Prepared(value)
    }
}

// Mechanical forwarding only: Some reads the caller's bound allocation;
// None preserves the existing lazy direct-business owner path.
pub(crate) fn operating(
    d: &FederalDraft,
    id: i64,
    business: &crate::FederalBusiness,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    if let Some(selected) = selected {
        return selected.operating(id);
    }
    if matches!(&business.activity, crate::BusinessActivity::Farming { data } if data.supplies.is_some())
    {
        let supplies = crate::farm_supplies::selected_amounts_with_home(d, false, home).map_err(
            |(id, key)| crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key),
        )?;
        let amount = supplies
            .get(&id)
            .ok_or(crate::Section179PreparedIssue::SourceIncomeUnavailable(id))?[2];
        return crate::cooperative_basis::operating_cost(
            d,
            id,
            business,
            crate::depreciation::mid_quarter(d),
            false,
        )?
        .checked_add(allowed(d, id, None, home)?)
        .and_then(|value| {
            value.checked_add(crate::farm_conservation::expense_adjustment(d, id, business))
        })
        .and_then(|value| value.checked_add(amount))
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id));
    }
    let mq = crate::depreciation::mid_quarter(d);
    let other = crate::cooperative_basis::operating_cost(d, id, business, mq, false)?
        .checked_add(allowed(d, id, None, home)?)
        .and_then(|value| {
            value.checked_add(crate::farm_conservation::expense_adjustment(d, id, business))
        })
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
    Ok(other
        + match &business.activity {
            crate::BusinessActivity::Farming { data } => {
                data.supplies.as_ref().map_or(0, |supplies| supplies.amounts(other, business)[2])
            }
            _ => 0,
        })
}

pub(crate) fn expenses(
    d: &FederalDraft,
    id: i64,
    business: &crate::FederalBusiness,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    if let Some(selected) = selected {
        return selected.expenses(id);
    }
    let operating = operating(d, id, business, None, home)?;
    Ok(business.expense_with_office(operating, office(d, id, business, None, home)?))
}

pub(crate) fn allowed(
    d: &FederalDraft,
    id: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    if let Some(selected) = selected {
        return selected.allowed(id);
    }
    if let Some(home) = home {
        return Ok(crate::section179::calculate_with_home(d, home)?
            .businesses
            .get(&id)
            .copied()
            .unwrap_or(0));
    }
    if !d.home_sites.is_empty() {
        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            id,
            "HOME_CONTEXT_REQUIRED",
        ));
    }
    Ok(crate::section179::allowed(d, id))
}

pub(crate) fn office(
    d: &FederalDraft,
    id: i64,
    _business: &crate::FederalBusiness,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<i64, crate::Section179PreparedIssue> {
    if let Some(selected) = selected {
        return selected.office(id);
    }
    if d.home_sites.is_empty() {
        return Ok(0);
    }
    let home = home.ok_or(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
        id,
        "HOME_CONTEXT_REQUIRED",
    ))?;
    let profits = d
        .income
        .iter()
        .filter_map(|entry| {
            let FederalIncome::Business { data } = &entry.income else {
                return None;
            };
            Some(operating(d, entry.id, data, None, Some(home)).and_then(|operating_cents| {
                crate::home_office::HomeIncome::from_source(d, entry.id, data, operating_cents)
                    .map(|income| (entry.id, income))
            }))
        })
        .collect::<Result<_, _>>()?;
    crate::home_office::deduction_with_context(d, id, &profits, home)
}

// Keep the existing AMT owners' adjustment stream. Only the bound179 and
// home-office differences are new; using full AMT expenses here would repeat
// depreciation and the farm adjustment owners already present in `existing`.
pub(crate) fn amt_adjustment(
    id: i64,
    existing: i64,
    selected: Option<&[Selected; 2]>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let Some([regular, alternative]) = selected else {
        return Ok(existing);
    };
    let allowance = regular
        .allocation
        .allowed(id)?
        .checked_sub(alternative.allocation.allowed(id)?)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
    let office = regular
        .allocation
        .office(id)?
        .checked_sub(alternative.allocation.office(id)?)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))?;
    crate::kernel::ratio::checked_sum([existing, allowance, office])
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(id))
}

pub(crate) struct Selected {
    pub allocation: crate::Section179PreparedAllocation,
    pub ptp: BTreeMap<i64, crate::ptp_passive::PtpPalResult>,
}
impl Selected {
    pub fn report(&self, result: &mut crate::FederalResult) -> Result<(), Issue> {
        // Only actual successful PAL consumers establish omission of their
        // routing notices. These are references from this exact invocation.
        let consumed = self
            .ptp
            .keys()
            .map(|&id| {
                self.allocation.ptp_section179_pal_input(id).map_err(|issue| Issue::Ptp(id, issue))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.allocation
            .ptp_report_context(&consumed)
            .map_err(|issue| Issue::Ptp(0, issue))?
            .report(result);
        let alternative =
            self.allocation.income.system == crate::Section179IncomeSystem::Alternative;
        for (&id, source) in &self.allocation.final_ordinary {
            crate::extended_rules::report_risk_projection(
                id,
                alternative,
                "income.business.at_risk",
                source.projection.clone(),
                result,
            )
            .map_err(|issue| {
                Issue::Activity(id, issue.key(), "Resolve the business at-risk source.")
            })?;
        }
        Ok(())
    }
}

pub(crate) fn needed(d: &FederalDraft) -> Result<bool, Issue> {
    if d.income.iter().any(|entry| match &entry.income {
        FederalIncome::PartnershipActivities { data } => {
            data.current.section179 != 0
                || data.alternative_current.as_ref().is_some_and(|source| source.section179 != 0)
        }
        FederalIncome::ScorpActivities { data } => {
            data.current.section179 != 0
                || data.alternative_current.as_ref().is_some_and(|source| source.section179 != 0)
        }
        _ => false,
    }) {
        return Ok(true);
    }
    if d.income.iter().any(|entry| {
        matches!(&entry.income,
        FederalIncome::Business { data } if data.current_risk_supported() && !data.all_at_risk())
    }) {
        return Ok(true);
    }
    for entry in &d.income {
        let FederalIncome::Partnership { data } = &entry.income else {
            continue;
        };
        if !data.publicly_traded {
            continue;
        }
        // Preserve the lazy legacy path for unrelated parents. A gross carry
        // is not itself an available deduction: the native source owner removes
        // its sold-property part and certifies the current basis allowance.
        if data.section179 != 0 || data.section179_carryover != 0 {
            let source =
                crate::k1_section179_availability_source(d, entry.id, false).map_err(|issue| {
                    Issue::Prepared(crate::Section179PreparedIssue::K1Availability(entry.id, issue))
                })?;
            if source.current.basis_allowed_cents != 0
                || crate::kernel::carry::native_balance(
                    source.gross_income_carry_cents,
                    source.sold_income_carry_cents,
                ) != 0
            {
                return Ok(true);
            }
        }
        // An actually supplied AMT carry can differ from the regular source.
        // Do not demand an AMT record merely because a regular gross carry was
        // wholly restored to sold-property basis.
        if data
            .amt_section179_income_carry
            .as_ref()
            .is_some_and(|carry| data.section179 != 0 || carry.gross != 0)
        {
            let source =
                crate::k1_section179_availability_source(d, entry.id, true).map_err(|issue| {
                    Issue::Prepared(crate::Section179PreparedIssue::K1Availability(entry.id, issue))
                })?;
            if source.current.basis_allowed_cents != 0
                || crate::kernel::carry::native_balance(
                    source.gross_income_carry_cents,
                    source.sold_income_carry_cents,
                ) != 0
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
pub(crate) fn prepare(d: &FederalDraft, system: Section179IncomeSystem) -> Result<Selected, Issue> {
    if !d.home_sites.is_empty() {
        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            0,
            "HOME_CONTEXT_REQUIRED",
        )
        .into());
    }
    prepare_with_home(d, system, &crate::home_office::HomeContext::default())
}

pub(crate) fn prepare_with_home(
    d: &FederalDraft,
    system: Section179IncomeSystem,
    home: &crate::home_office::HomeContext,
) -> Result<Selected, Issue> {
    let alternative = system == Section179IncomeSystem::Alternative;
    let native_context = if !d.income.iter().any(|entry| {
        matches!(
            &entry.income,
            FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. }
        )
    }) && d.income.iter().any(|entry| {
        matches!(&entry.income,
        FederalIncome::Business { data } if data.current_risk_supported() && !data.all_at_risk())
    }) {
        crate::section179::no_section179_context(d, system, home)?
    } else {
        None
    };
    let scoped = native_context.as_ref().map(|_| {
        d.income
            .iter()
            .filter_map(|entry| {
                matches!(&entry.income, FederalIncome::Business { data }
            if data.current_risk_supported() && !data.all_at_risk())
                .then_some(entry.id)
            })
            .collect::<alloc::collections::BTreeSet<_>>()
    });
    let direct =
        crate::section179::direct_income_sources_with_home(d, system, Some(home), scoped.as_ref())?;
    let mut ptp_activities = BTreeMap::new();
    let allocation = if let Some(native) = native_context {
        Ok(native)
    } else {
        let businesses = direct
            .iter()
            .map(crate::Section179DirectIncomeSource::prepared_input)
            .collect::<Result<Vec<_>, _>>()?;
        let mut k1s = Vec::new();
        let mut native_amounts = Vec::new();
        for entry in crate::community::entries(d) {
            let native = match &entry.income {
                FederalIncome::PartnershipActivities { data } => {
                    Some(data.before_section179(alternative))
                }
                FederalIncome::ScorpActivities { data } => {
                    Some(data.before_section179(alternative))
                }
                _ => None,
            };
            if let Some(output) = native {
                let output = output
                    .map_err(|(issue, message)| Issue::Activity(entry.id, issue.key, message))?;
                for component in &output.components {
                    native_amounts.push(crate::section179::Section179PassThrough {
                        activity: component.id,
                        amount_cents: component.section179,
                    });
                }
                let income = crate::kernel::ratio::checked_sum(
                    output.groups.iter().map(|group| group.nonpassive),
                )
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(entry.id))?;
                k1s.push(crate::Section179BusinessIncomeSource {
                    source: entry.id,
                    system,
                    ordinary: crate::Section179IncomeClass {
                        current_net_cents: income,
                        prior_allowed_loss_cents: 0,
                    },
                    section1231: crate::Section179IncomeClass {
                        current_net_cents: 0,
                        prior_allowed_loss_cents: 0,
                    },
                });
                continue;
            }
            let (current, property, prior) = match &entry.income {
                FederalIncome::Partnership { data } => {
                    let source = if alternative {
                        data.alternative.as_ref().ok_or(Issue::Prepared(
                            crate::Section179PreparedIssue::K1Availability(
                                entry.id,
                                crate::K1Section179BasisIssue::AlternativeHistoryUnavailable,
                            ),
                        ))?
                    } else {
                        &data.regular
                    };
                    let activity = crate::community::partner_pool(d, entry.id, data, source)
                        .map_err(|(key, english)| Issue::Activity(entry.id, key, english))?;
                    if data.publicly_traded {
                        let items = data
                            .items_for_record(source)
                            .map_err(|(key, english)| Issue::Activity(entry.id, key, english))?;
                        // Classification and prior release belong to the activity
                        // owner. Its current field is genuinely before PAL/179;
                        // neither a net allowance nor a released loss is inverted.
                        let nonpassive = data.material_for_losses()
                            == crate::participation::Participation::Yes
                            || items.ended();
                        ptp_activities.insert(
                            entry.id,
                            crate::ptp_passive::PtpPalActivity {
                                k1: entry.id,
                                source: entry.id,
                                components: vec![crate::ptp_passive::PtpPalComponent {
                                    character: crate::ptp_passive::PtpPalCharacter::Ordinary,
                                    passive_current_cents: if nonpassive {
                                        0
                                    } else {
                                        activity.current_ordinary_with_items
                                    },
                                    prior_passive_loss_cents: activity.combined.prior_pool,
                                    nonpassive_cents: activity.combined.nonpassive,
                                }],
                            },
                        );
                    }
                    (
                        activity.current_ordinary_with_items,
                        activity.current_section1231,
                        activity
                            .combined
                            .characters
                            .map_or([activity.combined.prior_released, 0], |parts| {
                                parts.prior_released
                            }),
                    )
                }
                FederalIncome::Scorp { data } => {
                    let selected =
                        if alternative {
                            data.amt
                        .as_ref()
                        .ok_or(Issue::Prepared(crate::Section179PreparedIssue::K1Availability(
                            entry.id,
                            crate::K1Section179BasisIssue::AlternativeHistoryUnavailable,
                        )))?
                        .source(data)
                        .map_err(|(key, english)| Issue::Activity(entry.id, key, english))?
                        } else {
                            data.into()
                        };
                    let activity = crate::community::scorp_pool(d, entry.id, selected, alternative)
                        .map_err(|(key, english)| Issue::Activity(entry.id, key, english))?;
                    (
                        activity.current_ordinary_with_items,
                        activity.current_section1231,
                        activity
                            .combined
                            .characters
                            .map_or([activity.combined.prior_released, 0], |parts| {
                                parts.prior_released
                            }),
                    )
                }
                _ => continue,
            };
            // None is the owner's ordinary-only class, not an unavailable property
            // source. A characterized activity retains both actual prior releases.
            k1s.push(crate::Section179BusinessIncomeSource {
                source: entry.id,
                system,
                ordinary: crate::Section179IncomeClass {
                    current_net_cents: current,
                    prior_allowed_loss_cents: prior[0],
                },
                section1231: crate::Section179IncomeClass {
                    current_net_cents: property,
                    prior_allowed_loss_cents: prior[1],
                },
            });
        }
        let mut callback_issue: Option<crate::Section179PreparedIssue> = None;
        let allocation = crate::section179::section179_with_pass_through_context(
            d,
            system,
            &businesses,
            &k1s,
            Some(home),
            &native_amounts,
            |view, hypothetical, operands| {
                // Use this invocation's actual office stage, which excludes routed
                // receipts and the other Form4562 income-source adjustments.
                let source = (|| {
                    let profits = direct
                        .iter()
                        .map(|source| {
                            let deduction = hypothetical.get(&source.source).copied().unwrap_or(0);
                            operands
                                .home_income(source.source, deduction)
                                .map(|income| (source.source, income))
                        })
                        .collect::<Result<BTreeMap<_, _>, _>>()?;
                    let amounts = direct
                        .iter()
                        .map(|source| {
                            crate::home_office::deduction_with_context(
                                view,
                                source.source,
                                &profits,
                                home,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(crate::Section179OtherLimitedDeduction {
                        system,
                        amount_cents: crate::kernel::ratio::checked_sum(amounts)
                            .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?,
                    })
                })();
                match source {
                    Ok(value) => Some(value),
                    Err(issue) => {
                        callback_issue = Some(issue);
                        None
                    }
                }
            },
        );
        if let Some(issue) = callback_issue {
            return Err(issue.into());
        }
        allocation
    };
    let mut allocation = allocation?;
    allocation.home_context = home.clone();
    // Bind the one current ordinary source after its actual selected179/home
    // stage. Every later consumer reads this map; expenses retain their source
    // meaning and never absorb the suspended loss.
    for source in &direct {
        let entry = d
            .income
            .iter()
            .find(|entry| entry.id == source.source)
            .ok_or(crate::Section179PreparedIssue::UnknownSource(source.source))?;
        let FederalIncome::Business { data } = &entry.income else { continue };
        if data.current_risk_supported() && !data.all_at_risk() {
            if d.setup
                .as_ref()
                .and_then(|setup| {
                    crate::community::share(setup, entry.owner, entry.community, &entry.income)
                })
                .is_some_and(|share| !share.whole())
            {
                return Err(Issue::Activity(
                    entry.id,
                    "BUSINESS_RISK_DIVIDED_SOURCE_REQUIRED",
                    "The divided business needs each person's actual risk and counterpart self-employment source.",
                ));
            }
            if source.recognized_property_gain_dollars != 0 {
                return Err(Issue::Activity(
                    entry.id,
                    "BUSINESS_RISK_PROPERTY_SOURCE_REQUIRED",
                    "Resolve the business property's original loss character before applying at-risk limits.",
                ));
            }
            let office = allocation.office(entry.id)?;
            let operating = source
                .operating_net_cents
                .checked_sub(allocation.allowed(entry.id)?)
                .and_then(|value| value.checked_sub(office))
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(entry.id))?;
            let final_source = crate::business_risk::project(data, operating, alternative)
                .map_err(|issue| {
                    Issue::Activity(
                        entry.id,
                        issue.key(),
                        "Resolve the current business's actual at-risk source facts.",
                    )
                })?;
            if final_source.projection.components.suspended.iter().any(|held| *held != 0)
                && allocation.available_section179() != 0
            {
                return Err(Issue::Activity(
                    entry.id,
                    "BUSINESS_RISK_SECTION179_SOURCE_REQUIRED",
                    "The at-risk limit needs the coordinated before-section-179 source.",
                ));
            }
            allocation.final_ordinary.insert(entry.id, final_source);
        } else if !data.current_risk_supported()
            && crate::federal_rules::farm_receipts(d, entry.id, data)?
                < allocation.expenses(entry.id)?
        {
            return Err(Issue::Activity(
                entry.id,
                "ISSUE_SECTION179_DIRECT_AT_RISK",
                "Resolve the direct business at-risk source for this selected allocation.",
            ));
        }
    }
    let mut ptp = BTreeMap::new();
    for (id, mut activity) in ptp_activities {
        // Other parents can be wholly sold-only even when this return has an
        // active PTP deduction. Only the allocator's actually available source
        // components enter PAL; zero availability does not need a risk getter.
        let available = allocation
            .income
            .k1
            .get(&id)
            .ok_or(Issue::Prepared(crate::Section179PreparedIssue::MissingSource(id)))?;
        if !available.components.iter().any(|component| component.available_cents != 0) {
            continue;
        }
        let source =
            allocation.ptp_section179_pal_input(id).map_err(|issue| Issue::Ptp(id, issue))?;
        let deduction = source.allowed_cents;
        activity.components.push(crate::ptp_passive::PtpPalComponent {
            character: crate::ptp_passive::PtpPalCharacter::Section179,
            passive_current_cents: if source.active { 0 } else { -deduction },
            prior_passive_loss_cents: 0,
            nonpassive_cents: if source.active { -deduction } else { 0 },
        });
        let result = crate::ptp_passive::ptp_passive_source(id, alternative, &[activity], source)
            .map_err(|issue| Issue::Passive(id, issue))?;
        ptp.insert(id, result);
    }
    Ok(Selected { allocation, ptp })
}
