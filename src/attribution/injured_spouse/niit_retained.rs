// Source attribution of the one native Form8960 retained-loss worksheet.
// All operands are its existing annual Dollars, not reconstructed cents.
pub(crate) struct RetainedNiitUse {
    pub(crate) prior_dollars: i64,
    // The signed native disposition adjustment J, including current exclusions.
    pub(crate) used_dollars: i64,
    pub(crate) next_dollars: i64,
    // Every actual current contribution is individually known zero. An
    // aggregate zero, an unavailable source, or opposite offsets do not pass.
    pub(crate) current_components_known_zero: bool,
}

impl RetainedNiitUse {
    // The existing shared-limit row receives cents only at its page boundary.
    // Equal bounds identify the sole possible result without asking a choice.
    pub(crate) fn bounds(&self, prior: [i64; 2], injured: usize) -> Option<(i64, i64)> {
        if !self.current_components_known_zero {
            return None;
        }
        self.bounds_with_current(prior, injured, 0).map(|(_, minimum, maximum)| (minimum, maximum))
    }

    // The caller must supply a confirmed current exclusion from the native
    // component packet, including complete zero evidence for every other
    // current contributor. The native carry is not allocated or recalculated:
    // U = P - N and J = U - E must both foot the existing worksheet.
    pub(crate) fn bounds_with_current(
        &self,
        prior: [i64; 2],
        injured: usize,
        current_excluded: i64,
    ) -> Option<(i64, i64, i64)> {
        if injured > 1
            || prior[0].checked_add(prior[1])? != self.prior_dollars
            || self.next_dollars < 0
            || current_excluded < 0
        {
            return None;
        }
        let used = self.prior_dollars.checked_sub(self.next_dollars)?;
        if used.checked_sub(current_excluded)? != self.used_dollars {
            return None;
        }
        let (minimum, maximum) =
            crate::kernel::attribution::allowed_bounds(used, [prior[injured], prior[1 - injured]])?;
        Some((used, minimum, maximum))
    }
}
