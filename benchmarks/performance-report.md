# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-10-06 13:11:47

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 15.3x faster
- **Best Performance**: 30.7x faster
- **Architecture**: Rust with concurrent operations, multi-layered caching

## Detailed Results


## Performance Visualizations

The following charts provide visual insights into Lectern's performance compared to Composer:

### Execution Time Comparison

Side-by-side comparison of execution times for each command, showing the absolute time difference between Lectern and Composer.

![Execution Time Comparison](charts/performance_comparison.png)

### Performance Scatter Plot

Scatter plot showing the relationship between Composer and Lectern execution times. Points below the diagonal line indicate faster Lectern performance.

![Performance Scatter Plot](charts/execution_time_scatter.png)

### Performance Improvement Distribution

Distribution analysis of performance improvements, showing both histogram and box plot views of the speed-up factors.

![Performance Improvement Distribution](charts/improvement_distribution.png)

### Category Performance Analysis

Performance breakdown by command category, comparing average improvements and execution times across different types of operations.

![Category Performance Analysis](charts/category_performance.png)



### Performance Summary Table

| Command | Lectern Time | Composer Time | Performance | Status |
|---------|--------------|---------------|-------------|--------|
| Install Dependencies | 0.037s | 0.494s | 13.5x | 🚀 13.5x faster |
| Update Dependencies | 0.042s | 1.022s | 24.2x | 🚀 24.2x faster |
| Search Packages | 0.181s | 0.458s | 2.5x | 🚀 2.5x faster |
| Show Package Info | 0.044s | 0.552s | 12.5x | 🚀 12.5x faster |
| Check Outdated | 0.104s | 0.116s | 1.1x | 🚀 1.1x faster |
| Show Licenses | 0.006s | 0.102s | 16.5x | 🚀 16.5x faster |
| Show Status | 0.008s | 0.101s | 12.7x | 🚀 12.7x faster |
| Require Package | 0.045s | 1.382s | 30.7x | 🚀 30.7x faster |
| Remove Package | 0.053s | 0.610s | 11.5x | 🚀 11.5x faster |
| Status Check (simple-laravel) | 0.006s | 0.142s | 22.0x | 🚀 22.0x faster |
| Outdated Check (simple-laravel) | 0.101s | 0.148s | 1.5x | 🚀 1.5x faster |
| Status Check (symfony-app) | 0.005s | 0.133s | 27.3x | 🚀 27.3x faster |
| Outdated Check (symfony-app) | 0.006s | 0.140s | 23.0x | 🚀 23.0x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
10 commands with exceptional performance gains:
- **Install Dependencies**: 13.5x faster (0.494s → 0.037s)
- **Update Dependencies**: 24.2x faster (1.022s → 0.042s)
- **Show Package Info**: 12.5x faster (0.552s → 0.044s)
- **Show Licenses**: 16.5x faster (0.102s → 0.006s)
- **Show Status**: 12.7x faster (0.101s → 0.008s)
- **Require Package**: 30.7x faster (1.382s → 0.045s)
- **Remove Package**: 11.5x faster (0.610s → 0.053s)
- **Status Check (simple-laravel)**: 22.0x faster (0.142s → 0.006s)
- **Status Check (symfony-app)**: 27.3x faster (0.133s → 0.005s)
- **Outdated Check (symfony-app)**: 23.0x faster (0.140s → 0.006s)

#### 🚀 Fast Commands (2-10x improvement)
1 commands with significant performance gains:
- **Search Packages**: 2.5x faster (0.458s → 0.181s)

#### 🟰 Similar Performance (0.5-2x)
2 commands with comparable performance:
- **Check Outdated**: 1.1x (0.116s vs 0.104s)
- **Outdated Check (simple-laravel)**: 1.5x (0.148s vs 0.101s)

## Technical Architecture

-### Caching System
- **Multi-layered caching**: Filesystem-based persistent cache in `$XDG_CACHE_HOME/lectern` (or `~/.cache/lectern`)
- **Cache structure**: SHA-256 hashed keys for efficient storage
- **Specialized caches**: 
  - Package metadata cache
  - Dependency resolution cache
  - Search results cache
- **Bulk operations**: Efficient batch API calls

### Concurrent Operations
- **Async/await**: Tokio runtime for concurrency
- **Parallel API calls**: Futures-based concurrent processing
- **Timeout handling**: 10-second timeouts per request
- **Error resilience**: Graceful degradation

### Compatibility
- **Full Composer compatibility**: All major commands supported
- **Lock file format**: Compatible with composer.lock
- **Configuration**: Reads composer.json and Lectern.toml
- **Package sources**: Uses Packagist.org API

## Command Analysis

### Core Commands
#### Install Dependencies
- **Performance**: 13.5x faster
- **Lectern**: 0.037s
- **Composer**: 0.494s
- **Status**: 🚀 13.5x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 24.2x faster
- **Lectern**: 0.042s
- **Composer**: 1.022s
- **Status**: 🚀 24.2x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 2.5x faster
- **Lectern**: 0.181s
- **Composer**: 0.458s
- **Status**: 🚀 2.5x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 12.5x faster
- **Lectern**: 0.044s
- **Composer**: 0.552s
- **Status**: 🚀 12.5x faster
- **Notes**: Standard operation



## Environment Information

- **Platform**: Linux
- **Lectern Version**: v0.1.0 (Rust)
- **Test Projects**: complex-app, simple-laravel, simple-test, symfony-app
- **Test Date**: 2025-10-06
- **Build Mode**: Release (optimized)

## Conclusion

Lectern demonstrates significant performance improvements over Composer while maintaining full compatibility. The combination of Rust's performance, concurrent operations, and intelligent caching provides substantial speed gains for PHP package management.

### Key Achievements
- **Average 15.3x performance improvement**
- **Full command compatibility** with Composer
- **Intelligent caching** with persistence
- **Concurrent operations** for parallel processing
- **Robust error handling** and timeouts

### Recommendations
- Use Lectern for large projects with many dependencies
- Leverage caching for repeated operations
- Consider Lectern for CI/CD pipelines requiring fast dependency resolution

---
*Report generated by Lectern Benchmark Suite*
*Lectern v0.1.0 - Rust-powered PHP package management*
