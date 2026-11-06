pub use crate::form_1095a_source::{
    Form1095ACoverageAmounts, Form1095ACoveredIndividual, Form1095APaper, Form1095ARecipient,
};
use alloc::{string::String, vec::Vec};

// The derived page and the owner matcher read the recipient as one printed party: the street
// row, then lines 13 through 15, which print side by side on one row.
impl From<&Form1095ARecipient> for crate::PrintedParty {
    fn from(recipient: &Form1095ARecipient) -> Self {
        let Form1095ARecipient { name, ssn, street_address, city, state, country_zip } = recipient;
        let locality = [city, state, country_zip]
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        let address = [street_address.as_str(), locality.as_str()]
            .into_iter()
            .filter(|row| !row.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Self { name: name.clone(), federal_id: ssn.clone(), address }
    }
}

// Fixture helper: a party named in the derived shape prints its whole address on the street row.
#[cfg(test)]
impl From<&crate::PrintedParty> for Form1095ARecipient {
    fn from(party: &crate::PrintedParty) -> Self {
        Self {
            name: party.name.clone(),
            ssn: party.federal_id.clone(),
            street_address: party.address.clone(),
            ..Default::default()
        }
    }
}

impl From<&Form1095APaper> for crate::MarketplacePolicyRecordPage {
    fn from(paper: &Form1095APaper) -> Self {
        let Form1095APaper {
            calendar_year: _,
            void,
            corrected,
            marketplace_identifier: _,
            policy_number,
            issuer,
            recipient,
            recipient_date_of_birth: _,
            spouse_name: _,
            spouse_ssn: _,
            spouse_date_of_birth: _,
            policy_start_date: _,
            policy_termination_date: _,
            covered_individuals: _,
            january,
            february,
            march,
            april,
            may,
            june,
            july,
            august,
            september,
            october,
            november,
            december,
            annual_totals: _,
        } = paper;
        let months = [
            january, february, march, april, may, june, july, august, september, october, november,
            december,
        ]
        .into_iter()
        .enumerate()
        .filter_map(|(index, row)| {
            row.as_ref().map(|row| {
                let Form1095ACoverageAmounts { premium, benchmark, advance } = row;
                crate::PolicyMonth {
                    month: index as u32 + 1,
                    premium: premium.unwrap_or(0),
                    benchmark: *benchmark,
                    advance: advance.unwrap_or(0),
                }
            })
        })
        .collect();
        Self {
            id: 0,
            recipient: recipient.into(),
            issuer: issuer.clone(),
            policy_number: policy_number.clone(),
            amounts: crate::PolicyAmounts::ByMonth { months },
            status: if *void {
                crate::MarketplaceStatementStatus::Void
            } else if *corrected {
                crate::MarketplaceStatementStatus::Corrected
            } else {
                crate::MarketplaceStatementStatus::Original
            },
        }
    }
}

// Fixture helpers: tests state Marketplace amounts in the derived page shape.
#[cfg(test)]
impl From<crate::MarketplacePolicyRecordPage> for Form1095APaper {
    fn from(page: crate::MarketplacePolicyRecordPage) -> Self {
        let mut paper = Self {
            calendar_year: crate::HEALTH_TAX_YEAR,
            void: page.status == crate::MarketplaceStatementStatus::Void,
            corrected: page.status == crate::MarketplaceStatementStatus::Corrected,
            marketplace_identifier: String::new(),
            policy_number: page.policy_number,
            issuer: page.issuer,
            recipient: (&page.recipient).into(),
            recipient_date_of_birth: None,
            spouse_name: String::new(),
            spouse_ssn: String::new(),
            spouse_date_of_birth: None,
            policy_start_date: None,
            policy_termination_date: None,
            covered_individuals: Vec::new(),
            january: None,
            february: None,
            march: None,
            april: None,
            may: None,
            june: None,
            july: None,
            august: None,
            september: None,
            october: None,
            november: None,
            december: None,
            annual_totals: Form1095ACoverageAmounts {
                premium: None,
                benchmark: None,
                advance: None,
            },
        };
        paper.set_amounts(page.amounts);
        paper
    }
}

#[cfg(test)]
impl Form1095APaper {
    fn slots_mut(&mut self) -> [&mut Option<Form1095ACoverageAmounts>; 12] {
        [
            &mut self.january,
            &mut self.february,
            &mut self.march,
            &mut self.april,
            &mut self.may,
            &mut self.june,
            &mut self.july,
            &mut self.august,
            &mut self.september,
            &mut self.october,
            &mut self.november,
            &mut self.december,
        ]
    }
    /// Page-shaped monthly amounts of this printed form
    pub(crate) fn amounts(&self) -> crate::PolicyAmounts {
        crate::MarketplacePolicyRecordPage::from(self).amounts
    }
    /// Replace the monthly lines from page-shaped amounts
    pub(crate) fn set_amounts(&mut self, amounts: crate::PolicyAmounts) {
        for slot in self.slots_mut() {
            *slot = None;
        }
        for month in 1..=12 {
            if let Some((premium, benchmark, advance)) =
                crate::marketplace_sources::policy_month(&amounts, month)
            {
                *self.month_slot(month) = Some(Form1095ACoverageAmounts {
                    premium: Some(premium),
                    benchmark,
                    advance: Some(advance),
                });
            }
        }
    }
    pub(crate) fn month_slot(&mut self, month: u32) -> &mut Option<Form1095ACoverageAmounts> {
        self.slots_mut().into_iter().nth(month as usize - 1).unwrap()
    }
    /// The printed amounts of one month, created blank when absent
    pub(crate) fn month_mut(&mut self, month: u32) -> &mut Form1095ACoverageAmounts {
        self.month_slot(month).get_or_insert(Form1095ACoverageAmounts {
            premium: None,
            benchmark: None,
            advance: None,
        })
    }
    pub(crate) fn set_month(
        &mut self,
        month: u32,
        premium: i64,
        benchmark: Option<i64>,
        advance: i64,
    ) {
        *self.month_slot(month) = Some(Form1095ACoverageAmounts {
            premium: Some(premium),
            benchmark,
            advance: Some(advance),
        });
    }
}
