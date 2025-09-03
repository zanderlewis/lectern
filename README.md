# Lectern
Lectern is an async and concurrent rewrite of PHP's Composer package manager in Rust. Built with lots of help from ChatGPT-5 and Claude Sonnet 4.

## Todo
Lectern is not a complete replacement for Composer just yet! Some things I haven't been able to test (as AI wrote that code), or aren't of high priority.
- Composer compatible plugin system
- Testing of private packages and git repositories

## Benchmarks
This image is the performance comparison for Lectern and Composer cache hits on various different commands (lower is better):
![Performance Comparison](benchmarks/charts/performance_comparison.png)

Lectern is in orange, and clearly shows that (especially for outdated checks on larger projects) Lectern is much faster than Composer!

This chart shows the average performance based on command category, and the execution time between Lectern and Composer for each category.
![Speedup Chart](benchmarks/charts/category_performance.png)

Check out the full [Lectern v. Composer Benchmark Report](benchmarks/performance-report.md) for detailed performance comparisons and insights.
