use std::time::Duration;

/// Maximum number of retry attempts for failed API requests
pub const MAX_RETRIES: u32 = 3;

/// Calculate exponential backoff duration for a given retry attempt
///
/// Returns: Duration = 2^retry_count seconds
/// - Retry 1: 2 seconds
/// - Retry 2: 4 seconds
/// - Retry 3: 8 seconds
pub fn calculate_backoff(retry_count: u32) -> Duration {
    Duration::from_secs(2_u64.pow(retry_count))
}

/// Check if another retry should be attempted
pub fn should_retry(current_retry: u32) -> bool {
    current_retry < MAX_RETRIES
}
