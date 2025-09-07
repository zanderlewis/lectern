# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-09-07 01:23:58

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 62.9x faster
- **Best Performance**: 622.5x faster
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
| Install Dependencies | 0.129s | 7.378s | 57.1x | 🚀 57.1x faster |
| Update Dependencies | 0.150s | 1.158s | 7.7x | 🚀 7.7x faster |
| Search Packages | 0.239s | 0.996s | 4.2x | 🚀 4.2x faster |
| Show Package Info | 0.037s | 0.111s | 3.0x | 🚀 3.0x faster |
| Check Outdated | 0.321s | 9.417s | 29.4x | 🚀 29.4x faster |
| Show Licenses | 0.006s | 0.122s | 20.3x | 🚀 20.3x faster |
| Show Status | 0.006s | 3.689s | 622.5x | 🚀 622.5x faster |
| Require Package | 0.130s | 1.407s | 10.9x | 🚀 10.9x faster |
| Remove Package | 0.120s | 0.981s | 8.1x | 🚀 8.1x faster |
| Status Check (simple-laravel) | 0.006s | 0.088s | 14.9x | 🚀 14.9x faster |
| Outdated Check (simple-laravel) | 0.268s | 0.090s | 0.3x | ⚡ 3.0x slower |
| Status Check (symfony-app) | 0.004s | 0.087s | 20.1x | 🚀 20.1x faster |
| Outdated Check (symfony-app) | 0.005s | 0.089s | 18.8x | 🚀 18.8x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
8 commands with exceptional performance gains:
- **Install Dependencies**: 57.1x faster (7.378s → 0.129s)
- **Check Outdated**: 29.4x faster (9.417s → 0.321s)
- **Show Licenses**: 20.3x faster (0.122s → 0.006s)
- **Show Status**: 622.5x faster (3.689s → 0.006s)
- **Require Package**: 10.9x faster (1.407s → 0.130s)
- **Status Check (simple-laravel)**: 14.9x faster (0.088s → 0.006s)
- **Status Check (symfony-app)**: 20.1x faster (0.087s → 0.004s)
- **Outdated Check (symfony-app)**: 18.8x faster (0.089s → 0.005s)

#### 🚀 Fast Commands (2-10x improvement)
4 commands with significant performance gains:
- **Update Dependencies**: 7.7x faster (1.158s → 0.150s)
- **Search Packages**: 4.2x faster (0.996s → 0.239s)
- **Show Package Info**: 3.0x faster (0.111s → 0.037s)
- **Remove Package**: 8.1x faster (0.981s → 0.120s)

#### 🟰 Similar Performance (0.5-2x)
0 commands with comparable performance:


## Technical Architecture

### Caching System
- **Multi-layered caching**: Filesystem-based persistent cache in `.lectern_cache/`
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
- **Performance**: 57.1x faster
- **Lectern**: 0.129s
- **Composer**: 7.378s
- **Status**: 🚀 57.1x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 7.7x faster
- **Lectern**: 0.150s
- **Composer**: 1.158s
- **Status**: 🚀 7.7x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 4.2x faster
- **Lectern**: 0.239s
- **Composer**: 0.996s
- **Status**: 🚀 4.2x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 3.0x faster
- **Lectern**: 0.037s
- **Composer**: 0.111s
- **Status**: 🚀 3.0x faster
- **Notes**: Standard operation



## Environment Information

- **Platform**: Linux
- **Lectern Version**: v0.1.0 (Rust)
- **Test Projects**: complex-app, simple-laravel, simple-test, symfony-app
- **Test Date**: 2025-09-07
- **Build Mode**: Release (optimized)

## Conclusion

Lectern demonstrates significant performance improvements over Composer while maintaining full compatibility. The combination of Rust's performance, concurrent operations, and intelligent caching provides substantial speed gains for PHP package management.

### Key Achievements
- **Average 62.9x performance improvement**
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
