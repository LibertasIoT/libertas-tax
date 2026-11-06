use crate::social_sources::{
    LumpSumElection, PrintedSocialBenefitYearPage, SocialBenefitYear, SocialElection,
    SocialPriorReturnFacts,
};
// Coverage: Social Security lump-sum election, Form 1040 line 6c and
// Publication 915 Worksheets 2–4. Printed earlier-year receipts bind to the
// existing election; retained prior-return facts remain independent. The
// Retirement page shows known receipt amounts read-only and permits manual
// payment records when no printed allocation supplies them. Source corrections
// rederive amounts and remove elections for payments no longer reported.
// Tests: retirement/unasked_pages.rs and federal/printed_form_boxes.rs.
// https://www.irs.gov/publications/p915
use crate::FederalStatus;
use crate::kernel::phase;
use alloc::vec::Vec;

// The Retirement page wraps the Compare branch; absence selects Regular.
impl LumpSumElection {
    pub(crate) fn election_for(&self, draft: &crate::FederalDraft) -> SocialElection {
        let printed = crate::social_documents::printed_lump_sums(draft);
        let mut years: Vec<_> =
            self.years.iter().filter(|row| !printed.contains_key(&row.year)).cloned().collect();
        let mut complete = self.complete;
        for (year, amount) in printed {
            if let Some(prior) = self
                .printed_years
                .iter()
                .find(|row| row.year == year)
                .and_then(|row| row.prior.as_ref())
            {
                years.push(prior.year(year, amount));
            } else if let Some(prior) = self.years.iter().find(|row| row.year == year) {
                // Existing manual records become a printed wrapper on reopen.
                let mut prior = prior.clone();
                prior.received = amount;
                years.push(prior);
            } else {
                complete = false;
            }
        }
        complete &= !years.is_empty();
        SocialElection::Compare { years, complete }
    }
    pub(crate) fn from_draft(draft: &crate::FederalDraft) -> Option<Self> {
        let election = &draft.adjustments.as_ref()?.social_election;
        let mut page = Self::from_election(election)?;
        page.printed_years = printed_year_pages(draft);
        page.years.retain(|row| !page.printed_years.iter().any(|printed| printed.year == row.year));
        Some(page)
    }
}

impl SocialBenefitYear {
    pub(crate) fn additional(&self) -> i64 {
        use crate::federal_rules::social_taxable;
        use crate::kernel::ratio::dollars;
        let benefits = dollars(phase::floor(self.benefits + self.earlier_lumps + self.received, 0));
        let other = dollars(self.agi) - dollars(self.taxable) + dollars(self.addbacks);
        let exempt = dollars(self.exempt);
        let revised = if self.year >= 1994 {
            social_taxable(benefits, other, exempt, self.status, self.apart)
        } else {
            let base = match self.status {
                FederalStatus::Joint => crate::year_constants::SOCIAL_SECURITY_BASE_JOINT_DOLLARS,
                FederalStatus::Separate if !self.apart => 0,
                _ => crate::year_constants::SOCIAL_SECURITY_BASE_OTHER_DOLLARS,
            };
            dollars(phase::cap(
                benefits * 50,
                [crate::kernel::ratio::trunc_i64(
                    phase::floor((other + exempt - base) * 100 + benefits * 50, 0),
                    2,
                )],
            ))
        };
        phase::floor(revised - dollars(self.refigured), 0)
    }
}
impl SocialElection {
    pub(crate) fn bound_to_printed(&self, draft: &crate::FederalDraft) -> Self {
        let printed = crate::social_documents::printed_lump_sums(draft);
        let mut result = self.clone();
        if let Self::Compare { years, .. } = &mut result {
            for year in years {
                if let Some(amount) = printed.get(&year.year) {
                    year.received = *amount;
                }
            }
        }
        result
    }

    pub(crate) fn printed_years_complete(&self, draft: &crate::FederalDraft) -> bool {
        match self {
            Self::Regular | Self::Compare { complete: false, .. } => true,
            Self::Compare { years, .. } => {
                crate::social_documents::unallocated_railroad_years(draft).is_none()
                    && crate::social_documents::printed_lump_sums(draft)
                        .keys()
                        .all(|year| years.iter().any(|facts| facts.year == *year))
            }
        }
    }

    pub(crate) fn taxable(
        &self,
        benefits: i64,
        other: i64,
        exempt: i64,
        status: FederalStatus,
        apart: bool,
    ) -> i64 {
        use crate::federal_rules::social_taxable;
        use crate::kernel::ratio::dollars;
        let regular = social_taxable(benefits, other, exempt, status, apart);
        match self {
            Self::Regular => regular,
            Self::Compare { years, .. } => {
                let received = dollars(years.iter().map(|v| v.received).sum());
                let current = social_taxable(
                    phase::floor(benefits - received, 0),
                    other,
                    exempt,
                    status,
                    apart,
                );
                phase::cap(
                    regular,
                    [current + years.iter().map(SocialBenefitYear::additional).sum::<i64>()],
                )
            }
        }
    }
}

pub(crate) fn printed_year_pages(draft: &crate::FederalDraft) -> Vec<PrintedSocialBenefitYearPage> {
    let years = match draft.adjustments.as_ref().map(|a| &a.social_election) {
        Some(SocialElection::Compare { years, .. }) => years.as_slice(),
        _ => &[],
    };
    crate::social_documents::printed_lump_sums(draft)
        .into_iter()
        .map(|(year, received)| PrintedSocialBenefitYearPage {
            year,
            received,
            prior: years.iter().find(|row| row.year == year).map(SocialPriorReturnFacts::from_year),
        })
        .collect()
}
