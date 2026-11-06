use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Role
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EducatorRole {
    /// Teacher, instructor, counselor, principal or aide
    /// Early childhood (a school, preschool or child care facility providing educational or childcare services for more than two non-resident children under 6, operating at public expense or receiving a fee, payment or grant) or kindergarten through grade 12.
    Educator {
        /// Unreimbursed classroom expenses
        /// Books, supplies, computer equipment and software, other equipment and supplementary materials used in the classroom, and professional development courses for what or whom you teach. Up to $350 is deducted whether or not you itemize (Schedule 1 line 11); the rest when you itemize (Schedule A line 17k).
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        classroom: i64,
        /// Other unreimbursed instructional expenses
        /// Nonathletic supplies for health or physical education courses, and items used as part of instructional activity outside the classroom, such as coaching. Deducted only when you itemize (Schedule A line 17k).
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        instructional: i64,
    },
    /// Interscholastic sports administrator or coach only
    /// Not also a teacher, instructor, counselor, principal or aide. Deducted only when you itemize (Schedule A line 17k).
    Coach {
        /// Unreimbursed expenses
        /// Books, supplies, athletic and other equipment and materials used as part of instructional activity, and professional development courses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses: i64,
    },
}

/// Educator expenses to evaluate
/// Empty when there are none.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EducatorClaim {
    /// Your role and expenses
    #[libertas_default($null)]
    pub taxpayer: Option<EducatorRole>,
    /// Spouse's role and expenses
    /// Only on a joint return.
    #[libertas_default($null)]
    pub spouse: Option<EducatorRole>,
}
impl EducatorRole {
    pub(crate) fn classroom(&self) -> i64 {
        match self {
            Self::Educator { classroom, .. } => *classroom,
            Self::Coach { .. } => 0,
        }
    }
    pub(crate) fn itemized_only(&self) -> i64 {
        match self {
            Self::Educator { instructional, .. } => *instructional,
            Self::Coach { expenses } => *expenses,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        let money = |v: i64| (0..=100000000000).contains(&v);
        match self {
            Self::Educator { classroom, instructional } => {
                money(*classroom) && money(*instructional)
            }
            Self::Coach { expenses } => money(*expenses),
        }
    }
}
impl EducatorClaim {
    /// No educator expenses.
    pub(crate) const NONE: Self = Self { taxpayer: None, spouse: None };
    /// Whether any educator expense is entered.
    pub(crate) fn any(&self) -> bool {
        *self != Self::NONE
    }
}
