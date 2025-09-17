# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-09-17 11:50:35

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 14.6x faster
- **Best Performance**: 31.6x faster
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
| Install Dependencies | 0.195s | 2.065s | 10.6x | 🚀 10.6x faster |
| Update Dependencies | 0.142s | 3.004s | 21.2x | 🚀 21.2x faster |
| Search Packages | 0.342s | 1.139s | 3.3x | 🚀 3.3x faster |
| Show Package Info | 0.045s | 1.435s | 31.6x | 🚀 31.6x faster |
| Check Outdated | 0.354s | 0.128s | 0.4x | ⚡ 2.8x slower |
| Show Licenses | 0.008s | 0.202s | 24.6x | 🚀 24.6x faster |
| Show Status | 0.008s | 0.108s | 14.0x | 🚀 14.0x faster |
| Require Package | 0.164s | 3.377s | 20.5x | 🚀 20.5x faster |
| Remove Package | 0.149s | 1.164s | 7.8x | 🚀 7.8x faster |
| Status Check (simple-laravel) | 0.006s | 0.098s | 15.5x | 🚀 15.5x faster |
| Outdated Check (simple-laravel) | 0.219s | 0.110s | 0.5x | ⚡ 2.0x slower |
| Status Check (symfony-app) | 0.005s | 0.095s | 19.6x | 🚀 19.6x faster |
| Outdated Check (symfony-app) | 0.005s | 0.100s | 20.1x | 🚀 20.1x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
9 commands with exceptional performance gains:
- **Install Dependencies**: 10.6x faster (2.065s → 0.195s)
- **Update Dependencies**: 21.2x faster (3.004s → 0.142s)
- **Show Package Info**: 31.6x faster (1.435s → 0.045s)
- **Show Licenses**: 24.6x faster (0.202s → 0.008s)
- **Show Status**: 14.0x faster (0.108s → 0.008s)
- **Require Package**: 20.5x faster (3.377s → 0.164s)
- **Status Check (simple-laravel)**: 15.5x faster (0.098s → 0.006s)
- **Status Check (symfony-app)**: 19.6x faster (0.095s → 0.005s)
- **Outdated Check (symfony-app)**: 20.1x faster (0.100s → 0.005s)

#### 🚀 Fast Commands (2-10x improvement)
2 commands with significant performance gains:
- **Search Packages**: 3.3x faster (1.139s → 0.342s)
- **Remove Package**: 7.8x faster (1.164s → 0.149s)

#### 🟰 Similar Performance (0.5-2x)
1 commands with comparable performance:
- **Outdated Check (simple-laravel)**: 0.5x (0.110s vs 0.219s)

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
- **Performance**: 10.6x faster
- **Lectern**: 0.195s
- **Composer**: 2.065s
- **Status**: 🚀 10.6x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 21.2x faster
- **Lectern**: 0.142s
- **Composer**: 3.004s
- **Status**: 🚀 21.2x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 3.3x faster
- **Lectern**: 0.342s
- **Composer**: 1.139s
- **Status**: 🚀 3.3x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 31.6x faster
- **Lectern**: 0.045s
- **Composer**: 1.435s
- **Status**: 🚀 31.6x faster
- **Notes**: Standard operation



## Environment Information

- **Platform**: Linux
- **Lectern Version**: v0.1.0 (Rust)
- **Test Projects**: complex-app, simple-laravel, simple-test, symfony-app
- **Test Date**: 2025-09-17
- **Build Mode**: Release (optimized)

## Conclusion

Lectern demonstrates significant performance improvements over Composer while maintaining full compatibility. The combination of Rust's performance, concurrent operations, and intelligent caching provides substantial speed gains for PHP package management.

### Key Achievements
- **Average 14.6x performance improvement**
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
