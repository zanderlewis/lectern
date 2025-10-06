# Lectern vs Composer Comprehensive Benchmark Report

Generated: 2025-10-06 11:32:33

## Summary

Lectern is a high-performance Rust-based package manager with full Composer compatibility. This comprehensive benchmark tests all major commands across multiple project types.

Please note that performance is tested when both Composer and Lectern have warm caches.

### Overall Performance
- **Tests Completed**: 13
- **Successful Comparisons**: 13
- **Average Performance Improvement**: 15.6x faster
- **Best Performance**: 33.4x faster
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
| Install Dependencies | 0.035s | 0.493s | 14.1x | 🚀 14.1x faster |
| Update Dependencies | 0.058s | 1.207s | 20.8x | 🚀 20.8x faster |
| Search Packages | 0.188s | 0.337s | 1.8x | 🚀 1.8x faster |
| Show Package Info | 0.045s | 0.618s | 13.8x | 🚀 13.8x faster |
| Check Outdated | 0.236s | 0.144s | 0.6x | ⚡ 1.6x slower |
| Show Licenses | 0.007s | 0.107s | 15.5x | 🚀 15.5x faster |
| Show Status | 0.005s | 0.100s | 20.8x | 🚀 20.8x faster |
| Require Package | 0.040s | 1.345s | 33.4x | 🚀 33.4x faster |
| Remove Package | 0.061s | 0.573s | 9.4x | 🚀 9.4x faster |
| Status Check (simple-laravel) | 0.007s | 0.152s | 21.1x | 🚀 21.1x faster |
| Outdated Check (simple-laravel) | 0.209s | 0.157s | 0.8x | ⚡ 1.3x slower |
| Status Check (symfony-app) | 0.006s | 0.143s | 24.0x | 🚀 24.0x faster |
| Outdated Check (symfony-app) | 0.006s | 0.152s | 26.9x | 🚀 26.9x faster |

### Performance Categories

#### ⚡ Ultra-Fast Commands (10x+ improvement)
9 commands with exceptional performance gains:
- **Install Dependencies**: 14.1x faster (0.493s → 0.035s)
- **Update Dependencies**: 20.8x faster (1.207s → 0.058s)
- **Show Package Info**: 13.8x faster (0.618s → 0.045s)
- **Show Licenses**: 15.5x faster (0.107s → 0.007s)
- **Show Status**: 20.8x faster (0.100s → 0.005s)
- **Require Package**: 33.4x faster (1.345s → 0.040s)
- **Status Check (simple-laravel)**: 21.1x faster (0.152s → 0.007s)
- **Status Check (symfony-app)**: 24.0x faster (0.143s → 0.006s)
- **Outdated Check (symfony-app)**: 26.9x faster (0.152s → 0.006s)

#### 🚀 Fast Commands (2-10x improvement)
1 commands with significant performance gains:
- **Remove Package**: 9.4x faster (0.573s → 0.061s)

#### 🟰 Similar Performance (0.5-2x)
3 commands with comparable performance:
- **Search Packages**: 1.8x (0.337s vs 0.188s)
- **Check Outdated**: 0.6x (0.144s vs 0.236s)
- **Outdated Check (simple-laravel)**: 0.8x (0.157s vs 0.209s)

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
- **Performance**: 14.1x faster
- **Lectern**: 0.035s
- **Composer**: 0.493s
- **Status**: 🚀 14.1x faster
- **Notes**: Real installation with backup/restore

#### Update Dependencies
- **Performance**: 20.8x faster
- **Lectern**: 0.058s
- **Composer**: 1.207s
- **Status**: 🚀 20.8x faster
- **Notes**: Real update with backup/restore

#### Search Packages
- **Performance**: 1.8x faster
- **Lectern**: 0.188s
- **Composer**: 0.337s
- **Status**: 🚀 1.8x faster
- **Notes**: Standard operation

#### Show Package Info
- **Performance**: 13.8x faster
- **Lectern**: 0.045s
- **Composer**: 0.618s
- **Status**: 🚀 13.8x faster
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
- **Average 15.6x performance improvement**
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
