# Task 2.7: Issue Caching for Offline Viewing

**Documentation:** [PRD] LazyJira TUI Application.md
**Task Number:** 2.7
**Area:** Infrastructure/Cache
**Estimated Effort:** M (6-8 hours)

## Description

Implement a caching layer that stores fetched issues locally for offline viewing and reduced API calls. The cache should have configurable TTL and support cache invalidation.

## Acceptance Criteria

- [ ] Issues cached to disk after fetching
- [ ] Cached issues served when offline
- [ ] Configurable cache TTL (default 30 minutes)
- [ ] Cache invalidation on issue update
- [ ] Cache size limits to prevent disk bloat
- [ ] Visual indicator when viewing cached data
- [ ] Manual cache refresh option ('r' key)
- [ ] Per-profile cache separation
- [ ] Memory usage under 50MB (per NFR)

## Implementation Details

### Approach

1. Design cache file format (JSON or binary)
2. Implement cache read/write operations
3. Add TTL checking logic
4. Integrate with API client
5. Add cache status to UI
6. Implement manual refresh

### Files to Modify/Create

- `src/cache/mod.rs`: Cache implementation
- `src/api/client.rs`: Integrate caching
- `src/ui/views/list.rs`: Cache status indicator
- `src/config/settings.rs`: Cache TTL setting

### Technical Specifications

**Cache Directory Structure:**
```
~/.cache/lazyjira/
├── work/                    # Profile name
│   ├── issues/
│   │   ├── PROJ-123.json   # Individual issue cache
│   │   └── PROJ-124.json
│   ├── search_results/
│   │   └── abc123.json     # JQL query hash
│   └── metadata.json       # Last sync times
└── personal/
    └── ...
```

**Cache Entry:**
```rust
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub cached_at: SystemTime,
    pub expires_at: SystemTime,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            data,
            cached_at: now,
            expires_at: now + ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.cached_at)
            .unwrap_or(Duration::ZERO)
    }
}
```

**Cache Manager:**
```rust
pub struct CacheManager {
    base_dir: PathBuf,
    profile: String,
    ttl: Duration,
    max_size_mb: u64,
}

impl CacheManager {
    pub fn new(profile: &str, ttl_minutes: u32) -> Result<Self> {
        let base_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("No cache directory"))?
            .join("lazyjira");

        Ok(Self {
            base_dir,
            profile: profile.to_string(),
            ttl: Duration::from_secs(ttl_minutes as u64 * 60),
            max_size_mb: 100, // 100 MB max cache size
        })
    }

    fn profile_dir(&self) -> PathBuf {
        self.base_dir.join(&self.profile)
    }

    fn issue_path(&self, key: &str) -> PathBuf {
        self.profile_dir().join("issues").join(format!("{}.json", key))
    }

    fn search_path(&self, jql: &str) -> PathBuf {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        jql.hash(&mut hasher);
        let hash = hasher.finish();

        self.profile_dir()
            .join("search_results")
            .join(format!("{:x}.json", hash))
    }

    pub fn get_issue(&self, key: &str) -> Option<Issue> {
        let path = self.issue_path(key);
        self.read_cache(&path)
    }

    pub fn set_issue(&self, issue: &Issue) -> Result<()> {
        let path = self.issue_path(&issue.key);
        self.write_cache(&path, issue)
    }

    pub fn get_search_results(&self, jql: &str) -> Option<CachedSearchResult> {
        let path = self.search_path(jql);
        self.read_cache(&path)
    }

    pub fn set_search_results(&self, jql: &str, results: &SearchResult) -> Result<()> {
        let path = self.search_path(jql);
        let cached = CachedSearchResult {
            jql: jql.to_string(),
            results: results.clone(),
        };
        self.write_cache(&path, &cached)
    }

    fn read_cache<T: DeserializeOwned>(&self, path: &Path) -> Option<T> {
        let content = std::fs::read_to_string(path).ok()?;
        let entry: CacheEntry<T> = serde_json::from_str(&content).ok()?;

        if entry.is_expired() {
            tracing::debug!("Cache expired for {:?}", path);
            return None;
        }

        Some(entry.data)
    }

    fn write_cache<T: Serialize>(&self, path: &Path, data: &T) -> Result<()> {
        std::fs::create_dir_all(path.parent().unwrap())?;

        let entry = CacheEntry::new(data, self.ttl);
        let content = serde_json::to_string(&entry)?;
        std::fs::write(path, content)?;

        self.check_cache_size()?;
        Ok(())
    }

    pub fn invalidate_issue(&self, key: &str) -> Result<()> {
        let path = self.issue_path(key);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let dir = self.profile_dir();
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }

    fn check_cache_size(&self) -> Result<()> {
        let size = self.calculate_size()?;
        if size > self.max_size_mb * 1024 * 1024 {
            self.evict_oldest()?;
        }
        Ok(())
    }

    fn evict_oldest(&self) -> Result<()> {
        // Find and delete oldest cache entries until under limit
        let mut entries: Vec<(PathBuf, SystemTime)> = Vec::new();

        for entry in walkdir::WalkDir::new(self.profile_dir())
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

        entries.sort_by_key(|(_, time)| *time);

        // Delete oldest 25%
        let to_delete = entries.len() / 4;
        for (path, _) in entries.into_iter().take(to_delete) {
            std::fs::remove_file(path)?;
        }

        Ok(())
    }
}
```

**API Client Integration:**
```rust
impl JiraClient {
    pub async fn search_issues_cached(
        &self,
        cache: &CacheManager,
        jql: &str,
        start: u32,
        max: u32,
    ) -> Result<(SearchResult, CacheStatus)> {
        // Try cache first
        if let Some(cached) = cache.get_search_results(jql) {
            return Ok((cached.results, CacheStatus::FromCache));
        }

        // Fetch from API
        let results = self.search_issues(jql, start, max).await?;

        // Cache the results
        cache.set_search_results(jql, &results)?;

        // Cache individual issues
        for issue in &results.issues {
            cache.set_issue(issue)?;
        }

        Ok((results, CacheStatus::Fresh))
    }

    pub async fn get_issue_cached(
        &self,
        cache: &CacheManager,
        key: &str,
    ) -> Result<(Issue, CacheStatus)> {
        // Try cache first
        if let Some(cached) = cache.get_issue(key) {
            return Ok((cached, CacheStatus::FromCache));
        }

        // Fetch from API
        let issue = self.get_issue(key).await?;
        cache.set_issue(&issue)?;

        Ok((issue, CacheStatus::Fresh))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CacheStatus {
    Fresh,
    FromCache,
    Offline,
}
```

**UI Cache Indicator:**
```rust
fn render_cache_status(&self, frame: &mut Frame, area: Rect) {
    let (icon, text, style) = match self.cache_status {
        CacheStatus::Fresh => ("●", "Live", Style::default().fg(Color::Green)),
        CacheStatus::FromCache => ("○", "Cached", Style::default().fg(Color::Yellow)),
        CacheStatus::Offline => ("✗", "Offline", Style::default().fg(Color::Red)),
    };

    let widget = Paragraph::new(format!("{} {}", icon, text))
        .style(style);

    frame.render_widget(widget, area);
}
```

## Testing Requirements

- [ ] Issues cached after fetch
- [ ] Cached issues returned when fresh
- [ ] Expired cache triggers new fetch
- [ ] Offline mode serves cached data
- [ ] Cache invalidation works
- [ ] Cache size stays under limit
- [ ] 'r' forces refresh from API
- [ ] Different profiles have separate caches

## Dependencies

- **Prerequisite Tasks:** Task 1.3, Task 1.4
- **Blocks Tasks:** None
- **External:** walkdir (for cache size calculation)

## Definition of Done

- [x] All acceptance criteria met
- [x] Cache respects TTL setting
- [x] Disk usage stays reasonable
- [x] Offline experience is smooth
- [x] Cache status visible to user

---

## Implementation Notes (Completed: 2025-11-30)

### Summary

Implemented a complete disk-based caching system for JIRA issues with:
- Configurable TTL (default 30 minutes)
- Configurable max cache size (default 100 MB)
- Per-profile cache separation
- LRU eviction when cache exceeds size limit
- Visual cache status indicator in the status bar (Live/Cached/Offline)
- Automatic background refresh when showing cached data

### Files Modified

1. **`Cargo.toml`** - Added `walkdir` and `tempfile` dependencies
2. **`src/cache/mod.rs`** - Complete rewrite with:
   - `CacheStatus` enum (Fresh, FromCache, Offline)
   - `CacheEntry<T>` struct with TTL support
   - `CachedSearchResult` struct for JQL query caching
   - `CacheManager` with disk-based storage
   - `CacheStats` for cache statistics
   - Comprehensive unit tests (17 tests)
3. **`src/config/settings.rs`** - Added `cache_max_size_mb` setting
4. **`src/config/mod.rs`** - Updated test for new setting field
5. **`src/ui/views/list.rs`** - Added cache status field and status bar indicator
6. **`src/main.rs`** - Integrated cache with API calls:
   - Cache manager initialization per profile
   - Cache-first loading with background refresh
   - Cache manager recreation on profile switch

### Key Decisions

1. **Stale-While-Revalidate Strategy**: When cached data exists, it's shown immediately while a background fetch updates the data. This provides instant responsiveness.

2. **Per-Profile Cache Isolation**: Each profile has its own cache directory under `~/.cache/lazyjira/{profile_name}/` to prevent data mixing.

3. **JQL Hash-Based Search Caching**: Search results are cached using a hash of the JQL query, allowing efficient lookup without storing the full query in the filename.

4. **LRU Eviction**: When cache exceeds the size limit, the oldest 25% of files are automatically deleted.

### Test Coverage

- 429 tests total, all passing
- 17 new cache-specific tests covering:
  - Cache entry creation and expiration
  - Issue caching roundtrip
  - Search result caching
  - Cache invalidation
  - Cache statistics
  - Special characters in issue keys
