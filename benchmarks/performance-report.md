# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-09-17 10:56:15

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 13.6x faster
- **Best Performance**: 26.2x faster
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
| Install Dependencies | 0.175s | 2.046s | 11.7x | 🚀 11.7x faster |
| Update Dependencies | 0.133s | 2.951s | 22.2x | 🚀 22.2x faster |
| Search Packages | 0.269s | 1.069s | 4.0x | 🚀 4.0x faster |
| Show Package Info | 0.045s | 1.190s | 26.2x | 🚀 26.2x faster |
| Check Outdated | 0.233s | 0.100s | 0.4x | ⚡ 2.3x slower |
| Show Licenses | 0.006s | 0.099s | 15.3x | 🚀 15.3x faster |
| Show Status | 0.007s | 0.096s | 14.7x | 🚀 14.7x faster |
| Require Package | 0.138s | 3.062s | 22.2x | 🚀 22.2x faster |
| Remove Package | 0.129s | 1.092s | 8.5x | 🚀 8.5x faster |
| Status Check (simple-laravel) | 0.007s | 0.096s | 13.6x | 🚀 13.6x faster |
| Outdated Check (simple-laravel) | 0.200s | 0.099s | 0.5x | ⚡ 2.0x slower |
| Status Check (symfony-app) | 0.005s | 0.093s | 19.3x | 🚀 19.3x faster |
| Outdated Check (symfony-app) | 0.005s | 0.099s | 18.6x | 🚀 18.6x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
9 commands with exceptional performance gains:
- **Install Dependencies**: 11.7x faster (2.046s → 0.175s)
- **Update Dependencies**: 22.2x faster (2.951s → 0.133s)
- **Show Package Info**: 26.2x faster (1.190s → 0.045s)
- **Show Licenses**: 15.3x faster (0.099s → 0.006s)
- **Show Status**: 14.7x faster (0.096s → 0.007s)
- **Require Package**: 22.2x faster (3.062s → 0.138s)
- **Status Check (simple-laravel)**: 13.6x faster (0.096s → 0.007s)
- **Status Check (symfony-app)**: 19.3x faster (0.093s → 0.005s)
- **Outdated Check (symfony-app)**: 18.6x faster (0.099s → 0.005s)

#### 🚀 Fast Commands (2-10x improvement)
2 commands with significant performance gains:
- **Search Packages**: 4.0x faster (1.069s → 0.269s)
- **Remove Package**: 8.5x faster (1.092s → 0.129s)

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
- **Performance**: 11.7x faster
- **Lectern**: 0.175s
- **Composer**: 2.046s
- **Status**: 🚀 11.7x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 22.2x faster
- **Lectern**: 0.133s
- **Composer**: 2.951s
- **Status**: 🚀 22.2x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 4.0x faster
- **Lectern**: 0.269s
- **Composer**: 1.069s
- **Status**: 🚀 4.0x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 26.2x faster
- **Lectern**: 0.045s
- **Composer**: 1.190s
- **Status**: 🚀 26.2x faster
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
- **Average 13.6x performance improvement**
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
