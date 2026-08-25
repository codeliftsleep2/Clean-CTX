// src/tests/config/resource_limits.rs
//
// Tests for ResourceLimits validation (A-13).
// Verifies that file size, workspace file count, and memory usage
// checks are properly integrated and enforced.

#[cfg(test)]
mod resource_limits_tests {
    use crate::config::ResourceLimits;
    use crate::config::CleanCtxConfig;

    /// Test that default ResourceLimits are sensible
    #[test]
    fn test_default_resource_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_file_size_bytes, 10 * 1024 * 1024, "Default max file size should be 10 MB");
        assert_eq!(limits.max_workspace_files, 10_000, "Default max workspace files should be 10,000");
        assert_eq!(limits.max_memory_bytes, 512 * 1024 * 1024, "Default max memory should be 512 MB");
    }

    /// Test that check_file_size accepts files within limit
    #[test]
    fn test_check_file_size_accepts_valid_size() {
        let limits = ResourceLimits::default();
        assert!(limits.check_file_size(5 * 1024 * 1024).is_ok(), "5 MB should be accepted");
        assert!(limits.check_file_size(10 * 1024 * 1024).is_ok(), "10 MB should be accepted (at limit)");
    }

    /// Test that check_file_size rejects files exceeding limit
    #[test]
    fn test_check_file_size_rejects_oversized() {
        let limits = ResourceLimits::default();
        let result = limits.check_file_size(11 * 1024 * 1024);
        assert!(result.is_err(), "11 MB should be rejected");
        assert!(result.unwrap_err().contains("exceeds"), "Error should mention 'exceeds'");
    }

    /// Test that check_workspace_file_count accepts valid counts
    #[test]
    fn test_check_workspace_file_count_accepts_valid() {
        let limits = ResourceLimits::default();
        assert!(limits.check_workspace_file_count(100).is_ok(), "100 files should be accepted");
        assert!(limits.check_workspace_file_count(10_000).is_ok(), "10,000 files should be accepted (at limit)");
    }

    /// Test that check_workspace_file_count rejects excessive counts
    #[test]
    fn test_check_workspace_file_count_rejects_excessive() {
        let limits = ResourceLimits::default();
        let result = limits.check_workspace_file_count(10_001);
        assert!(result.is_err(), "10,001 files should be rejected");
        assert!(result.unwrap_err().contains("exceeds"), "Error should mention 'exceeds'");
    }

    /// Test that check_memory_usage accepts valid estimates
    #[test]
    fn test_check_memory_usage_accepts_valid() {
        let limits = ResourceLimits::default();
        assert!(limits.check_memory_usage(100 * 1024 * 1024).is_ok(), "100 MB should be accepted");
        assert!(limits.check_memory_usage(512 * 1024 * 1024).is_ok(), "512 MB should be accepted (at limit)");
    }

    /// Test that check_memory_usage rejects excessive estimates
    #[test]
    fn test_check_memory_usage_rejects_excessive() {
        let limits = ResourceLimits::default();
        let result = limits.check_memory_usage(513 * 1024 * 1024);
        assert!(result.is_err(), "513 MB should be rejected");
        assert!(result.unwrap_err().contains("exceeds"), "Error should mention 'exceeds'");
    }

    /// Test that ResourceLimits can be customized via CleanCtxConfig
    #[test]
    fn test_custom_resource_limits() {
        let mut config = CleanCtxConfig::default();
        config.resource_limits.max_file_size_bytes = 5 * 1024 * 1024; // 5 MB
        config.resource_limits.max_workspace_files = 5_000;
        config.resource_limits.max_memory_bytes = 256 * 1024 * 1024; // 256 MB

        assert!(config.resource_limits.check_file_size(4 * 1024 * 1024).is_ok(), "4 MB should be accepted with custom 5 MB limit");
        assert!(config.resource_limits.check_file_size(6 * 1024 * 1024).is_err(), "6 MB should be rejected with custom 5 MB limit");
        assert!(config.resource_limits.check_workspace_file_count(4_000).is_ok(), "4,000 files should be accepted with custom 5,000 limit");
        assert!(config.resource_limits.check_workspace_file_count(6_000).is_err(), "6,000 files should be rejected with custom 5,000 limit");
        assert!(config.resource_limits.check_memory_usage(200 * 1024 * 1024).is_ok(), "200 MB should be accepted with custom 256 MB limit");
        assert!(config.resource_limits.check_memory_usage(300 * 1024 * 1024).is_err(), "300 MB should be rejected with custom 256 MB limit");
    }

    /// Test that error messages are user-friendly and actionable
    #[test]
    fn test_error_messages_are_actionable() {
        let limits = ResourceLimits::default();
        
        let file_error = limits.check_file_size(20 * 1024 * 1024).unwrap_err();
        assert!(file_error.contains("File size"), "Error should mention 'File size'");
        assert!(file_error.contains("MB"), "Error should show size in MB");
        
        let count_error = limits.check_workspace_file_count(20_000).unwrap_err();
        assert!(count_error.contains("workspace file count"), "Error should mention 'workspace file count'");
        
        let mem_error = limits.check_memory_usage(1024 * 1024 * 1024).unwrap_err();
        assert!(mem_error.contains("memory"), "Error should mention 'memory'");
    }
}