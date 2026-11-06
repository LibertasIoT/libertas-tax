use super::*;
use crate::document_center::{
    K1ActivityComponentRecords, K1ActivityGraphRecords, K1ActivityQbiAllocation,
    K1ActivityQbiRecords, K1PartnerActivityPersonal, K1PartnerActivityRecords,
    K1ScorpActivityRecords,
};
use crate::{DocumentAnswer as A, K1Document as K};

fn changed() -> ValidationIssue {
    ValidationIssue::new("DOCUMENT_QUESTIONS_CHANGED").at_field("value")
}
fn label(name: &str) -> Vec<u8> {
    crate::Message::new(
        "LABEL_K1_ACTIVITY_SOURCE",
        alloc::vec![crate::MessageArg::Text(name.into())],
    )
    .encode()
}
fn graph(v: &crate::K1ComponentDetailsPage) -> Option<K1ActivityGraphRecords> {
    let crate::K1ComponentStatementPage::Entered { source } = &v.statement else { return None };
    Some(K1ActivityGraphRecords {
        choices: v.choices.clone(),
        components: source
            .components
            .iter()
            .map(|v| {
                let (se_history, se_gross) = match &v.self_employment {
                    crate::K1StatementSe::Partnership { source } => {
                        (source.history.clone(), source.gross.clone())
                    }
                    _ => (Default::default(), None),
                };
                K1ActivityComponentRecords {
                    id: v.id,
                    description: label(&v.name),
                    personal_services: v.personal_services.clone(),
                    se_history,
                    se_gross,
                    regular: v.regular.clone(),
                    alternative: v.alternative.clone(),
                    risk_group: v.risk_group,
                    passive_group: v.passive_group,
                }
            })
            .collect(),
        risk_groups: source.risk_groups.clone(),
        passive_groups: source.passive_groups.clone(),
        qbi_blocks: source
            .qbi_blocks
            .iter()
            .map(|v| K1ActivityQbiRecords {
                id: v.id,
                description: label(&v.name),
                qbi_group: v.qbi_group.clone(),
                allocations: v.allocations.as_ref().map(|rows| {
                    rows.iter()
                        .map(|v| K1ActivityQbiAllocation {
                            component: v.component,
                            qualified: v.qualified.clone(),
                        })
                        .collect()
                }),
            })
            .collect(),
        section179_sale_sources: source.section179_sale_sources.as_ref().map(|rows| {
            rows.iter()
                .map(|v| crate::document_center::K1ActivitySaleAssociation {
                    sale: v.sale,
                    component: v.component,
                })
                .collect()
        }),
        investment_property_sources: source.investment_property_sources.as_ref().map(|rows| {
            rows.iter()
                .map(|v| crate::document_center::K1ActivityPropertyAssociation {
                    property: v.property,
                    component: v.component,
                })
                .collect()
        }),
        entity_only: source.entity_only.clone(),
        alternative_entity_only: source.alternative_entity_only.clone(),
    })
}
impl From<&crate::PartnerPersonSource> for K1PartnerActivityPersonal {
    fn from(v: &crate::PartnerPersonSource) -> Self {
        Self {
            limited_for_se: v.limited_for_se,
            se_history: v.se_history.clone(),
            se_gross: v.se_gross.clone(),
            prior_fuel_credit: v.prior_fuel_credit.clone(),
        }
    }
}
impl From<&crate::K1PartnershipActivitiesPage> for K1PartnerActivityRecords {
    fn from(v: &crate::K1PartnershipActivitiesPage) -> Self {
        Self {
            regular: v.entity.regular_record(),
            alternative: v.entity.alternative.clone(),
            amt_section179_income_carry: v.entity.amt_section179_income_carry.clone(),
            interest_term: v.entity.interest_term,
            alternative_current: v.alternative_current.clone(),
            activities: graph(&v.activities),
            person: v.person.as_ref().map(Into::into),
            special: v.special.clone(),
        }
    }
}
impl From<&crate::K1ScorpActivitiesPage> for K1ScorpActivityRecords {
    fn from(v: &crate::K1ScorpActivitiesPage) -> Self {
        Self {
            regular: v.entity.regular_record(),
            alternative: v.entity.alternative.clone(),
            loans: v.entity.loans.clone(),
            amt_section179_income_carry: v.entity.amt_section179_income_carry.clone(),
            stock_term: v.entity.stock_term,
            alternative_current: v.alternative_current.clone(),
            activities: graph(&v.activities),
            special: v.special.clone(),
        }
    }
}
fn same_members(a: impl Iterator<Item = i64>, b: impl Iterator<Item = i64>) -> bool {
    let a: Vec<_> = a.collect();
    let b: Vec<_> = b.collect();
    a.len() == b.len()
        && a.iter().collect::<alloc::collections::BTreeSet<_>>().len() == a.len()
        && a.iter().all(|id| b.contains(id))
}
fn patch_graph(
    v: &mut crate::K1ComponentDetailsPage,
    answer: Option<K1ActivityGraphRecords>,
) -> Result<(), ValidationIssue> {
    let crate::K1ComponentStatementPage::Entered { source } = &mut v.statement else {
        return if answer.is_none() { Ok(()) } else { Err(changed()) };
    };
    let answer = answer.ok_or_else(changed)?;
    if !same_members(answer.components.iter().map(|v| v.id), source.components.iter().map(|v| v.id))
        || !same_members(
            answer.qbi_blocks.iter().map(|v| v.id),
            source.qbi_blocks.iter().map(|v| v.id),
        )
    {
        return Err(changed());
    }
    for a in answer.components {
        let v = source.components.iter_mut().find(|v| v.id == a.id).unwrap();
        v.personal_services = a.personal_services;
        if let crate::K1StatementSe::Partnership { source } = &mut v.self_employment {
            source.history = a.se_history;
            source.gross = a.se_gross;
        } else if a.se_history != crate::K1RetainedSeHistory::default() || a.se_gross.is_some() {
            return Err(changed());
        }
        v.regular = a.regular;
        v.alternative = a.alternative;
        v.risk_group = a.risk_group;
        v.passive_group = a.passive_group;
    }
    for a in answer.qbi_blocks {
        let v = source.qbi_blocks.iter_mut().find(|v| v.id == a.id).unwrap();
        v.qbi_group = a.qbi_group;
        v.allocations = a.allocations.map(|rows| {
            rows.into_iter()
                .map(|v| crate::K1QbiAllocationPage {
                    component: v.component,
                    qualified: v.qualified,
                })
                .collect()
        });
    }
    source.risk_groups = answer.risk_groups;
    source.passive_groups = answer.passive_groups;
    source.section179_sale_sources = answer.section179_sale_sources.map(|rows| {
        rows.into_iter()
            .map(|v| crate::K1Section179SaleSourcePage { sale: v.sale, component: v.component })
            .collect()
    });
    source.investment_property_sources = answer.investment_property_sources.map(|rows| {
        rows.into_iter()
            .map(|v| crate::K1InvestmentPropertySourcePage {
                property: v.property,
                component: v.component,
            })
            .collect()
    });
    source.entity_only = answer.entity_only;
    source.alternative_entity_only = answer.alternative_entity_only;
    Ok(())
}
fn patch_person(
    v: &mut Option<crate::PartnerPersonSource>,
    answer: Option<K1PartnerActivityPersonal>,
) {
    if let Some(a) = answer {
        let v = v.get_or_insert_with(Default::default);
        v.limited_for_se = a.limited_for_se;
        v.se_history = a.se_history;
        v.se_gross = a.se_gross;
        v.prior_fuel_credit = a.prior_fuel_credit;
    } else if let Some(v) = v {
        v.limited_for_se = None;
        v.se_history = Default::default();
        v.se_gross = None;
        v.prior_fuel_credit = None;
    }
}
pub(super) fn apply(source: &mut K, answer: A) -> Result<(), ValidationIssue> {
    match (source, answer) {
        (K::K1PartnershipActivities(v), A::K1PartnerActivityRecords { value: a }) => {
            v.entity.regular = Some(a.regular);
            v.entity.alternative = a.alternative;
            v.entity.amt_section179_income_carry = a.amt_section179_income_carry;
            v.entity.interest_term = a.interest_term;
            v.alternative_current = a.alternative_current;
            v.special = a.special;
            patch_person(&mut v.person, a.person);
            patch_graph(&mut v.activities, a.activities)
        }
        (K::K1ScorpActivities(v), A::K1ScorpActivityRecords { value: a }) => {
            v.entity.regular = Some(a.regular);
            v.entity.alternative = a.alternative;
            v.entity.loans = a.loans;
            v.entity.amt_section179_income_carry = a.amt_section179_income_carry;
            v.entity.stock_term = a.stock_term;
            v.alternative_current = a.alternative_current;
            v.special = a.special;
            patch_graph(&mut v.activities, a.activities)
        }
        _ => Err(changed()),
    }
}
fn retain_graph(
    v: &mut crate::K1ComponentDetailsPage,
    old: Option<&crate::K1ComponentDetailsPage>,
) {
    let crate::K1ComponentStatementPage::Entered { source } = &mut v.statement else { return };
    let old = old.and_then(|v| match &v.statement {
        crate::K1ComponentStatementPage::Entered { source } => Some(source),
        _ => None,
    });
    source.risk_groups = old.map_or_else(Vec::new, |v| v.risk_groups.clone());
    source.passive_groups = old.map_or_else(Vec::new, |v| v.passive_groups.clone());
    source.section179_sale_sources = old.and_then(|v| v.section179_sale_sources.clone());
    source.investment_property_sources = old.and_then(|v| v.investment_property_sources.clone());
    source.alternative_entity_only =
        old.map_or(crate::K1EntityOnlyAmt::Unavailable, |v| v.alternative_entity_only.clone());
    for v in &mut source.components {
        let old = old.and_then(|old| old.components.iter().find(|old| v.id > 0 && old.id == v.id));
        v.personal_services = old
            .map_or(crate::K1ActivityPersonalSource::Unavailable, |v| v.personal_services.clone());
        v.risk_group = old.and_then(|v| v.risk_group);
        v.passive_group = old.and_then(|v| v.passive_group);
        v.alternative = old.map_or(crate::K1StatementAmt::Unavailable, |v| v.alternative.clone());
        v.regular = old.and_then(|v| v.regular.clone());
        if let crate::K1StatementSe::Partnership { source } = &mut v.self_employment {
            let old = old.and_then(|v| match &v.self_employment {
                crate::K1StatementSe::Partnership { source } => Some(source),
                _ => None,
            });
            source.history = old.map_or_else(Default::default, |v| v.history.clone());
            source.gross = old.and_then(|v| v.gross.clone());
        }
    }
    for v in &mut source.qbi_blocks {
        let old = old.and_then(|old| old.qbi_blocks.iter().find(|old| v.id > 0 && old.id == v.id));
        v.allocations = old.and_then(|v| v.allocations.clone());
        v.qbi_group = old.map_or_else(String::new, |v| v.qbi_group.clone());
    }
    source.entity_only = old.and_then(|v| v.entity_only.clone());
}
pub(super) fn retain(source: &mut K, old: Option<&K>) {
    match source {
        K::K1PartnershipActivities(v) => {
            let old = match old {
                Some(K::K1PartnershipActivities(v)) => Some(v),
                _ => None,
            };
            v.entity.regular = old.and_then(|v| v.entity.regular.clone());
            v.entity.alternative = old.and_then(|v| v.entity.alternative.clone());
            v.entity.amt_section179_income_carry =
                old.and_then(|v| v.entity.amt_section179_income_carry.clone());
            v.entity.interest_term = old.and_then(|v| v.entity.interest_term);
            v.alternative_current = old.and_then(|v| v.alternative_current.clone());
            v.special = old.map_or_else(Vec::new, |v| v.special.clone());
            patch_person(&mut v.person, old.and_then(|v| v.person.as_ref()).map(Into::into));
            retain_graph(&mut v.activities, old.map(|v| &v.activities));
            v.activities.prune_child_references(&v.entity.investment_property, &v.section179_sales);
        }
        K::K1ScorpActivities(v) => {
            let old = match old {
                Some(K::K1ScorpActivities(v)) => Some(v),
                _ => None,
            };
            v.entity.regular = old.and_then(|v| v.entity.regular.clone());
            v.entity.alternative = old.and_then(|v| v.entity.alternative.clone());
            v.entity.loans = old.map_or_else(Vec::new, |v| v.entity.loans.clone());
            v.entity.amt_section179_income_carry =
                old.and_then(|v| v.entity.amt_section179_income_carry.clone());
            v.entity.stock_term = old.and_then(|v| v.entity.stock_term);
            v.alternative_current = old.and_then(|v| v.alternative_current.clone());
            v.special = old.map_or_else(Vec::new, |v| v.special.clone());
            retain_graph(&mut v.activities, old.map(|v| &v.activities));
            v.activities.prune_child_references(&v.entity.investment_property, &v.section179_sales);
        }
        _ => {}
    }
}
pub(super) fn bind_issue(
    source: &K,
    issue: ValidationIssue,
    submitted: &[(i64, String, usize)],
    id: i64,
) -> Option<SaveFailure> {
    let index = submitted
        .iter()
        .find(|(found, key, _)| {
            *found == id && key == &alloc::format!("income/{id}/activity_records")
        })?
        .2;
    if matches!(
        issue.key,
        "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
            | "K1_QBI_CURRENT_ABOVE_COMPONENT_SOURCE"
            | crate::k1_activity::ordinary::RANGE_KEY
    ) {
        return Some(
            issue
                .at_record()
                .in_field("activities")
                .in_field("value")
                .in_field("answer")
                .in_list("followups", index)
                .into(),
        );
    }
    let graph = match source {
        K::K1PartnershipActivities(v) => &v.activities,
        K::K1ScorpActivities(v) => &v.activities,
        _ => return None,
    };
    let (components, blocks) = match &graph.statement {
        crate::K1ComponentStatementPage::Entered { source } => {
            (source.components.len(), source.qbi_blocks.len())
        }
        _ => (0, 0),
    };
    let fields: &[(&[&str], &[&str])] = &[
        (&["entity", "regular"], &["regular"]),
        (&["entity", "alternative"], &["alternative"]),
        (&["entity", "loans"], &["loans"]),
        (&["entity", "amt_section179_income_carry"], &["amt_section179_income_carry"]),
        (&["entity", "interest_term"], &["interest_term"]),
        (&["entity", "stock_term"], &["stock_term"]),
        (&["alternative_current"], &["alternative_current"]),
        (&["special"], &["special"]),
        (&["person"], &["person"]),
        (&["activities", "statement", "source"], &["activities"]),
    ];
    let (from, to) = fields.iter().find(|(from, _)| issue.has_field_prefix(from))?;
    let key = issue.key;
    let failure = SaveFailure::from(issue.map_field_prefix(from, to))
        .map_list_item_field_prefix(
            &["activities", "components"],
            &["self_employment", "source", "history"],
            &["se_history"],
            &alloc::vec![true;components],
        )
        .map_list_item_field_prefix(
            &["activities", "components"],
            &["self_employment", "source", "gross"],
            &["se_gross"],
            &alloc::vec![true;components],
        )
        .map_list_item_field_prefix(
            &["activities", "qbi_blocks"],
            &["statement"],
            &["allocations"],
            &alloc::vec![true;blocks],
        );
    Some(
        ValidationIssue { key, field: failure.field }
            .in_field("value")
            .in_field("answer")
            .in_list("followups", index)
            .into(),
    )
}
