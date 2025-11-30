//! Issue caching for offline viewing.
//!
//! This module provides disk-based caching functionality to store issue data locally
//! for offline access and reduced API calls. Features include:
//! - Configurable TTL (time-to-live)
//! - Per-profile cache separation
//! - Cache size limits with LRU eviction
//! - Search result caching with JQL hash keys

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, trace, warn};

use crate::api::types::{Issue, SearchResult};

/// Default cache TTL in minutes.
pub const DEFAULT_CACHE_TTL_MINUTES: u32 = 30;

/// Default maximum cache size in MB.
pub const DEFAULT_MAX_CACHE_SIZE_MB: u64 = 100;

/// Cache status indicating data freshness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheStatus {
    /// Data was freshly fetched from the API.
    Fresh,
    /// Data was served from cache (still valid).
    FromCache,
    /// Data was served from cache while offline.
    Offline,
}

impl CacheStatus {
    /// Get the display icon for the cache status.
    pub fn icon(&self) -> &'static str {
        match self {
            CacheStatus::Fresh => "●",
            CacheStatus::FromCache => "○",
            CacheStatus::Offline => "✗",
        }
    }

    /// Get the display text for the cache status.
    pub fn text(&self) -> &'static str {
        match self {
            CacheStatus::Fresh => "Live",
            CacheStatus::FromCache => "Cached",
            CacheStatus::Offline => "Offline",
        }
    }

    /// Check if the status indicates cached data.
    pub fn is_cached(&self) -> bool {
        matches!(self, CacheStatus::FromCache | CacheStatus::Offline)
    }
}

/// A cache entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data.
    pub data: T,
    /// When the entry was cached (Unix timestamp).
    pub cached_at: u64,
    /// When the entry expires (Unix timestamp).
    pub expires_at: u64,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with the given TTL.
    pub fn new(data: T, ttl: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Self {
            data,
            cached_at: now,
            expires_at: now + ttl.as_secs(),
        }
    }

    /// Check if the cache entry has expired.
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        now > self.expires_at
    }

    /// Get the age of the cache entry.
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Duration::from_secs(now.saturating_sub(self.cached_at))
    }

    /// Get the time remaining until expiration.
    pub fn time_remaining(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        Duration::from_secs(self.expires_at.saturating_sub(now))
    }
}

/// Cached search result with JQL query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSearchResult {
    /// The JQL query used for this search.
    pub jql: String,
    /// The search results.
    pub results: SearchResult,
}

/// Cache manager for storing and retrieving cached data.
pub struct CacheManager {
    /// Base directory for cache storage.
    base_dir: PathBuf,
    /// Current profile name.
    profile: String,
    /// Cache TTL.
    ttl: Duration,
    /// Maximum cache size in bytes.
    max_size_bytes: u64,
}

impl CacheManager {
    /// Create a new cache manager for the given profile.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile name (used for cache separation)
    /// * `ttl_minutes` - Cache TTL in minutes
    pub fn new(profile: &str, ttl_minutes: u32) -> io::Result<Self> {
        let base_dir = dirs::cache_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No cache directory available"))?
            .join("lazyjira");

        Ok(Self {
            base_dir,
            profile: profile.to_string(),
            ttl: Duration::from_secs(ttl_minutes as u64 * 60),
            max_size_bytes: DEFAULT_MAX_CACHE_SIZE_MB * 1024 * 1024,
        })
    }

    /// Create a new cache manager with custom max size.
    pub fn with_max_size(profile: &str, ttl_minutes: u32, max_size_mb: u64) -> io::Result<Self> {
        let mut manager = Self::new(profile, ttl_minutes)?;
        manager.max_size_bytes = max_size_mb * 1024 * 1024;
        Ok(manager)
    }

    /// Get the profile-specific cache directory.
    fn profile_dir(&self) -> PathBuf {
        self.base_dir.join(&self.profile)
    }

    /// Get the path for an issue cache file.
    fn issue_path(&self, key: &str) -> PathBuf {
        // Sanitize the key to be safe for filesystem
        let safe_key = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.profile_dir()
            .join("issues")
            .join(format!("{}.json", safe_key))
    }

    /// Get the path for a search result cache file.
    fn search_path(&self, jql: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        jql.hash(&mut hasher);
        let hash = hasher.finish();

        self.profile_dir()
            .join("search_results")
            .join(format!("{:016x}.json", hash))
    }

    /// Get an issue from the cache.
    ///
    /// Returns `None` if the issue is not cached or has expired.
    pub fn get_issue(&self, key: &str) -> Option<Issue> {
        let path = self.issue_path(key);
        self.read_cache(&path)
    }

    /// Store an issue in the cache.
    pub fn set_issue(&self, issue: &Issue) -> io::Result<()> {
        let path = self.issue_path(&issue.key);
        self.write_cache(&path, issue)?;
        self.check_cache_size()
    }

    /// Get search results from the cache.
    ///
    /// Returns `None` if the results are not cached or have expired.
    pub fn get_search_results(&self, jql: &str) -> Option<CachedSearchResult> {
        let path = self.search_path(jql);
        self.read_cache(&path)
    }

    /// Store search results in the cache.
    pub fn set_search_results(&self, jql: &str, results: &SearchResult) -> io::Result<()> {
        let path = self.search_path(jql);
        let cached = CachedSearchResult {
            jql: jql.to_string(),
            results: results.clone(),
        };
        self.write_cache(&path, &cached)?;
        self.check_cache_size()
    }

    /// Read a cache entry from disk.
    fn read_cache<T: DeserializeOwned>(&self, path: &Path) -> Option<T> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    debug!("Failed to read cache file {:?}: {}", path, e);
                }
                return None;
            }
        };

        let entry: CacheEntry<T> = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(e) => {
                debug!("Failed to parse cache entry {:?}: {}", path, e);
                // Remove corrupted cache file
                let _ = fs::remove_file(path);
                return None;
            }
        };

        if entry.is_expired() {
            trace!("Cache expired for {:?}", path);
            // Remove expired cache file
            let _ = fs::remove_file(path);
            return None;
        }

        trace!("Cache hit for {:?} (age: {:?})", path, entry.age());
        Some(entry.data)
    }

    /// Write a cache entry to disk.
    fn write_cache<T: Serialize>(&self, path: &Path, data: &T) -> io::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry = CacheEntry::new(data, self.ttl);
        let content = serde_json::to_string(&entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(path, content)?;
        trace!("Cached data to {:?}", path);
        Ok(())
    }

    /// Invalidate a cached issue.
    pub fn invalidate_issue(&self, key: &str) -> io::Result<()> {
        let path = self.issue_path(key);
        if path.exists() {
            fs::remove_file(&path)?;
            debug!("Invalidated cache for issue {}", key);
        }
        Ok(())
    }

    /// Invalidate all cached search results.
    pub fn invalidate_search_results(&self) -> io::Result<()> {
        let search_dir = self.profile_dir().join("search_results");
        if search_dir.exists() {
            fs::remove_dir_all(&search_dir)?;
            debug!("Invalidated all search result caches");
        }
        Ok(())
    }

    /// Clear all cached data for this profile.
    pub fn clear(&self) -> io::Result<()> {
        let dir = self.profile_dir();
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
            debug!("Cleared all cache for profile {}", self.profile);
        }
        Ok(())
    }

    /// Check cache size and evict old entries if necessary.
    fn check_cache_size(&self) -> io::Result<()> {
        let size = self.calculate_size()?;
        if size > self.max_size_bytes {
            debug!(
                "Cache size {} bytes exceeds limit {} bytes, evicting",
                size, self.max_size_bytes
            );
            self.evict_oldest()?;
        }
        Ok(())
    }

    /// Calculate the total size of the cache.
    fn calculate_size(&self) -> io::Result<u64> {
        let profile_dir = self.profile_dir();
        if !profile_dir.exists() {
            return Ok(0);
        }

        let mut total_size: u64 = 0;
        for entry in walkdir::WalkDir::new(&profile_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
        Ok(total_size)
    }

    /// Evict the oldest cache entries until under the size limit.
    fn evict_oldest(&self) -> io::Result<()> {
        let profile_dir = self.profile_dir();
        if !profile_dir.exists() {
            return Ok(());
        }

        // Collect all cache files with their modification times
        let mut entries: Vec<(PathBuf, SystemTime)> = Vec::new();

        for entry in walkdir::WalkDir::new(&profile_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    entries.push((entry.path().to_path_buf(), modified));
                }
            }
        }

        // Sort by modification time (oldest first)
        entries.sort_by_key(|(_, time)| *time);

        // Delete oldest 25% of entries
        let to_delete = (entries.len() / 4).max(1);
        for (path, _) in entries.into_iter().take(to_delete) {
            if let Err(e) = fs::remove_file(&path) {
                warn!("Failed to evict cache file {:?}: {}", path, e);
            } else {
                debug!("Evicted old cache file {:?}", path);
            }
        }

        Ok(())
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let profile_dir = self.profile_dir();
        let mut file_count = 0u64;
        let mut total_size = 0u64;

        if profile_dir.exists() {
            for entry in walkdir::WalkDir::new(&profile_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                file_count += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        CacheStats {
            file_count,
            total_size_bytes: total_size,
            max_size_bytes: self.max_size_bytes,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached files.
    pub file_count: u64,
    /// Total size in bytes.
    pub total_size_bytes: u64,
    /// Maximum size in bytes.
    pub max_size_bytes: u64,
    /// TTL in seconds.
    pub ttl_seconds: u64,
}

impl CacheStats {
    /// Get total size in MB.
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get max size in MB.
    pub fn max_size_mb(&self) -> f64 {
        self.max_size_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get usage percentage.
    pub fn usage_percent(&self) -> f64 {
        if self.max_size_bytes == 0 {
            0.0
        } else {
            (self.total_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{IssueFields, IssueType, Status};
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    fn create_test_issue(key: &str, summary: &str) -> Issue {
        Issue {
            id: "1".to_string(),
            key: key.to_string(),
            self_url: "https://example.com".to_string(),
            fields: IssueFields {
                summary: summary.to_string(),
                description: None,
                status: Status {
                    id: "1".to_string(),
                    name: "Open".to_string(),
                    status_category: None,
                },
                issuetype: IssueType {
                    id: "1".to_string(),
                    name: "Bug".to_string(),
                    subtask: false,
                    description: None,
                    icon_url: None,
                },
                priority: None,
                assignee: None,
                reporter: None,
                project: None,
                labels: vec![],
                components: vec![],
                created: None,
                updated: None,
                duedate: None,
                story_points: None,
            },
        }
    }

    fn create_test_cache_manager() -> CacheManager {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_path_buf();
        // Keep the tempdir alive by leaking it (acceptable in tests)
        std::mem::forget(temp_dir);
        CacheManager {
            base_dir: path,
            profile: "test".to_string(),
            ttl: Duration::from_secs(60),
            max_size_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new("test data", Duration::from_secs(60));
        assert!(!entry.is_expired());
        assert!(entry.age() < Duration::from_secs(1));
    }

    #[test]
    fn test_cache_entry_expired() {
        // Create an entry with very short TTL (1 second)
        let entry = CacheEntry::new("test data", Duration::from_secs(1));
        // Entry should not be expired immediately
        assert!(!entry.is_expired());
        // Sleep to let it expire (2 seconds to be safe due to second boundary)
        thread::sleep(Duration::from_secs(2));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_status_icon() {
        assert_eq!(CacheStatus::Fresh.icon(), "●");
        assert_eq!(CacheStatus::FromCache.icon(), "○");
        assert_eq!(CacheStatus::Offline.icon(), "✗");
    }

    #[test]
    fn test_cache_status_text() {
        assert_eq!(CacheStatus::Fresh.text(), "Live");
        assert_eq!(CacheStatus::FromCache.text(), "Cached");
        assert_eq!(CacheStatus::Offline.text(), "Offline");
    }

    #[test]
    fn test_cache_status_is_cached() {
        assert!(!CacheStatus::Fresh.is_cached());
        assert!(CacheStatus::FromCache.is_cached());
        assert!(CacheStatus::Offline.is_cached());
    }

    #[test]
    fn test_cache_issue_roundtrip() {
        let manager = create_test_cache_manager();
        let issue = create_test_issue("TEST-123", "Test issue");

        // Store the issue
        manager.set_issue(&issue).unwrap();

        // Retrieve the issue
        let cached = manager.get_issue("TEST-123");
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.key, "TEST-123");
        assert_eq!(cached.fields.summary, "Test issue");
    }

    #[test]
    fn test_cache_issue_not_found() {
        let manager = create_test_cache_manager();
        let cached = manager.get_issue("NONEXISTENT-123");
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_issue_invalidate() {
        let manager = create_test_cache_manager();
        let issue = create_test_issue("TEST-123", "Test issue");

        manager.set_issue(&issue).unwrap();
        assert!(manager.get_issue("TEST-123").is_some());

        manager.invalidate_issue("TEST-123").unwrap();
        assert!(manager.get_issue("TEST-123").is_none());
    }

    #[test]
    fn test_cache_search_results_roundtrip() {
        let manager = create_test_cache_manager();
        let results = SearchResult {
            start_at: 0,
            max_results: 50,
            total: 2,
            issues: vec![
                create_test_issue("TEST-1", "First issue"),
                create_test_issue("TEST-2", "Second issue"),
            ],
            next_page_token: None,
            is_last: true,
        };

        let jql = "project = TEST ORDER BY created DESC";

        // Store the results
        manager.set_search_results(jql, &results).unwrap();

        // Retrieve the results
        let cached = manager.get_search_results(jql);
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.jql, jql);
        assert_eq!(cached.results.issues.len(), 2);
    }

    #[test]
    fn test_cache_different_jql_different_cache() {
        let manager = create_test_cache_manager();
        let results1 = SearchResult {
            start_at: 0,
            max_results: 50,
            total: 1,
            issues: vec![create_test_issue("TEST-1", "First")],
            next_page_token: None,
            is_last: true,
        };
        let results2 = SearchResult {
            start_at: 0,
            max_results: 50,
            total: 1,
            issues: vec![create_test_issue("TEST-2", "Second")],
            next_page_token: None,
            is_last: true,
        };

        manager.set_search_results("jql1", &results1).unwrap();
        manager.set_search_results("jql2", &results2).unwrap();

        let cached1 = manager.get_search_results("jql1").unwrap();
        let cached2 = manager.get_search_results("jql2").unwrap();

        assert_eq!(cached1.results.issues[0].key, "TEST-1");
        assert_eq!(cached2.results.issues[0].key, "TEST-2");
    }

    #[test]
    fn test_cache_clear() {
        let manager = create_test_cache_manager();
        let issue = create_test_issue("TEST-123", "Test issue");

        manager.set_issue(&issue).unwrap();
        assert!(manager.get_issue("TEST-123").is_some());

        manager.clear().unwrap();
        assert!(manager.get_issue("TEST-123").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let manager = create_test_cache_manager();

        // Empty cache
        let stats = manager.stats();
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.total_size_bytes, 0);

        // Add some issues
        for i in 0..5 {
            let issue = create_test_issue(&format!("TEST-{}", i), "Test issue");
            manager.set_issue(&issue).unwrap();
        }

        let stats = manager.stats();
        assert_eq!(stats.file_count, 5);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_cache_special_characters_in_key() {
        let manager = create_test_cache_manager();

        // Keys with special characters should be sanitized
        let issue = create_test_issue("TEST/123", "Test issue");
        manager.set_issue(&issue).unwrap();

        // Should be able to retrieve with the original key
        let cached = manager.get_issue("TEST/123");
        assert!(cached.is_some());
    }

    #[test]
    fn test_cache_stats_usage_percent() {
        let stats = CacheStats {
            file_count: 10,
            total_size_bytes: 50 * 1024 * 1024, // 50 MB
            max_size_bytes: 100 * 1024 * 1024,  // 100 MB
            ttl_seconds: 1800,
        };

        assert_eq!(stats.usage_percent(), 50.0);
        assert_eq!(stats.total_size_mb(), 50.0);
        assert_eq!(stats.max_size_mb(), 100.0);
    }

    #[test]
    fn test_cache_entry_time_remaining() {
        let entry = CacheEntry::new("test", Duration::from_secs(60));
        let remaining = entry.time_remaining();
        // Should have close to 60 seconds remaining
        assert!(remaining > Duration::from_secs(55));
        assert!(remaining <= Duration::from_secs(60));
    }
}
