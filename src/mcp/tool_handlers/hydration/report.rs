use super::HydrationReport;

impl HydrationReport {
    pub(in crate::mcp::tool_handlers) fn is_complete(&self) -> bool {
        self.discovery_status == "completed"
    }

    pub(in crate::mcp::tool_handlers) fn merge(&mut self, other: Self) {
        self.hydration_attempted |= other.hydration_attempted;
        self.discovery_provider =
            merged_provider(self.discovery_provider, other.discovery_provider);
        self.discovery_status =
            if self.discovery_status == "completed" && other.discovery_status == "completed" {
                "completed"
            } else if self.hydration_attempted {
                "partial"
            } else {
                "failed"
            };
        if self.fallback_reason.is_none() {
            self.fallback_reason = other.fallback_reason;
        }
        self.candidates_discovered = self
            .candidates_discovered
            .saturating_add(other.candidates_discovered);
        self.candidates_compiled = self
            .candidates_compiled
            .saturating_add(other.candidates_compiled);
        self.project_coverage.extend(other.project_coverage);
    }
}

fn merged_provider(left: &'static str, right: &'static str) -> &'static str {
    match (left, right) {
        ("", provider) | ("none", provider) => provider,
        (provider, "") | (provider, "none") => provider,
        (left, right) if left == right => left,
        ("cbm_and_filesystem", _) | (_, "cbm_and_filesystem") => "cbm_and_filesystem",
        ("cbm", "filesystem") | ("filesystem", "cbm") => "cbm_and_filesystem",
        _ => "none",
    }
}
