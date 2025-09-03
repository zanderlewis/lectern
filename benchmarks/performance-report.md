# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-09-03 16:08:46

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 147.1x faster
- **Best Performance**: 969.9x faster
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
| Install Dependencies | 0.119s | 1.991s | 16.7x | 🚀 16.7x faster |
| Update Dependencies | 0.105s | 2.033s | 19.4x | 🚀 19.4x faster |
| Search Packages | 0.006s | 1.779s | 322.6x | 🚀 322.6x faster |
| Show Package Info | 0.030s | 0.698s | 23.4x | 🚀 23.4x faster |
| Check Outdated | 0.252s | 9.418s | 37.4x | 🚀 37.4x faster |
| Show Licenses | 0.231s | 0.715s | 3.1x | 🚀 3.1x faster |
| Show Status | 0.005s | 4.617s | 969.9x | 🚀 969.9x faster |
| Require Package | 0.109s | 3.269s | 30.0x | 🚀 30.0x faster |
| Remove Package | 0.104s | 2.795s | 26.8x | 🚀 26.8x faster |
| Status Check (simple-laravel) | 0.005s | 0.726s | 146.0x | 🚀 146.0x faster |
| Outdated Check (simple-laravel) | 0.253s | 0.784s | 3.1x | 🚀 3.1x faster |
| Status Check (symfony-app) | 0.005s | 0.736s | 160.6x | 🚀 160.6x faster |
| Outdated Check (symfony-app) | 0.005s | 0.707s | 153.7x | 🚀 153.7x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
11 commands with exceptional performance gains:
- **Install Dependencies**: 16.7x faster (1.991s → 0.119s)
- **Update Dependencies**: 19.4x faster (2.033s → 0.105s)
- **Search Packages**: 322.6x faster (1.779s → 0.006s)
- **Show Package Info**: 23.4x faster (0.698s → 0.030s)
- **Check Outdated**: 37.4x faster (9.418s → 0.252s)
- **Show Status**: 969.9x faster (4.617s → 0.005s)
- **Require Package**: 30.0x faster (3.269s → 0.109s)
- **Remove Package**: 26.8x faster (2.795s → 0.104s)
- **Status Check (simple-laravel)**: 146.0x faster (0.726s → 0.005s)
- **Status Check (symfony-app)**: 160.6x faster (0.736s → 0.005s)
- **Outdated Check (symfony-app)**: 153.7x faster (0.707s → 0.005s)

#### 🚀 Fast Commands (2-10x improvement)
2 commands with significant performance gains:
- **Show Licenses**: 3.1x faster (0.715s → 0.231s)
- **Outdated Check (simple-laravel)**: 3.1x faster (0.784s → 0.253s)

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
- **Performance**: 16.7x faster
- **Lectern**: 0.119s
- **Composer**: 1.991s
- **Status**: 🚀 16.7x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 19.4x faster
- **Lectern**: 0.105s
- **Composer**: 2.033s
- **Status**: 🚀 19.4x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 322.6x faster
- **Lectern**: 0.006s
- **Composer**: 1.779s
- **Status**: 🚀 322.6x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 23.4x faster
- **Lectern**: 0.030s
- **Composer**: 0.698s
- **Status**: 🚀 23.4x faster
- **Notes**: Standard operation



## Environment Information

- **Platform**: Linux
- **Lectern Version**: v0.1.0 (Rust)
- **Test Projects**: complex-app, simple-laravel, simple-test, symfony-app
- **Test Date**: 2025-09-03
- **Build Mode**: Release (optimized)

## Conclusion

Lectern demonstrates significant performance improvements over Composer while maintaining full compatibility. The combination of Rust's performance, concurrent operations, and intelligent caching provides substantial speed gains for PHP package management.

### Key Achievements
- **Average 147.1x performance improvement**
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
