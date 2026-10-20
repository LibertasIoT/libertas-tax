// IRC 220(e)(2), 223(e)(2), and incorporated 408(e)(2)/(4).
use crate::MedicalAccountWithdrawal;
use crate::federal_rules::valid_date;
use crate::kernel::ratio::exact::{Exact, whole};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Medical savings account deemed distribution
/// Choose one treatment per affected account. A prohibited transaction terminates the whole account from January 1, so do not also enter its pledges or later cash withdrawals as medical-account distributions. Other accounts remain separate.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MedicalDeemed {
    /// Account terminated by a prohibited transaction
    Terminated {
        /// Date of the prohibited transaction in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        occurred: u32,
        /// Fair market value of all account assets on January 1, 2026
        /// Use January 1 value, not the later transaction value. The deemed distribution occurs on January 1 for age and disability exceptions.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        january_value: i64,
    },
    /// Assets pledged as loan security
    Pledged {
        /// Pledge dates and fair market values
        /// Enter the date each portion was first used as collateral and its fair market value then, not the loan balance. Do not duplicate the same pledged assets. Confirm this account had no transaction terminating it from January 1.
        #[libertas_size(min = 1)]
        assets: Vec<MedicalAccountWithdrawal>,
    },
}
impl MedicalDeemed {
    pub(crate) fn valid(&self) -> bool {
        let date = |d| valid_date(d) && d / 10000 == 2026;
        let amount = |v| v >= 0;
        match self {
            Self::Terminated { occurred, january_value } => {
                date(*occurred) && amount(*january_value)
            }
            Self::Pledged { assets } => {
                !assets.is_empty() && assets.iter().all(|v| date(v.date) && amount(v.amount))
            }
        }
    }
    pub(crate) fn amount(&self) -> Exact {
        match self {
            Self::Terminated { january_value, .. } => whole(*january_value),
            Self::Pledged { assets } => assets.iter().map(|v| whole(v.amount)).sum(),
        }
    }
    pub(crate) fn last_event(&self) -> u32 {
        match self {
            Self::Terminated { occurred, .. } => *occurred,
            Self::Pledged { assets } => assets.iter().map(|v| v.date).max().unwrap_or(0),
        }
    }
    pub(crate) fn penalized(&self, subject: impl Fn(u32) -> bool) -> Exact {
        match self {
            Self::Terminated { january_value, .. } => {
                if subject(20260101) {
                    whole(*january_value)
                } else {
                    whole(0)
                }
            }
            Self::Pledged { assets } => {
                assets.iter().filter(|v| subject(v.date)).map(|v| whole(v.amount)).sum()
            }
        }
    }
}
