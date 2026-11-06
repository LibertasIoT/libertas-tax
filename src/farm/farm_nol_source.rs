use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// 2026 farming-loss carryback choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum FarmNolElection {
    /// Keep the two-year carryback
    /// The farming portion must first be applied to 2024, then 2025, before determining the remaining carryforward. The refund for 2024 and 2025 is claimed on Form 1045 or Form 1040-X and is not in this estimate; a note says so. The 2026 return shows the whole loss as carried forward, before the amount the earlier years absorb.
    Carryback,
    /// Waive the entire farming-loss carryback period
    /// This applies to the 2026 loss year as a whole. The election is generally irrevocable once validly made. The section 172(b) waiver statement is or will be attached to the original 2026 return by its due date, including extensions; alternatively, a timely original filer qualifies for the six-month amended-return election relief and uses its required statement. The App calculates the carryforward but does not file the election statement.
    Waive,
}
