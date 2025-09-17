# Lectern
Lectern is an async and concurrent rewrite of PHP's Composer package manager in Rust. Built with lots of help from ChatGPT-5 and Claude Sonnet 4.

## Todo
Lectern is not a complete replacement for Composer just yet! Some things I haven't been able to test (as AI wrote that code), or aren't of high priority.
- Composer compatible plugin system
- Testing of private packages and git repositories

## Benchmarks
This image is the performance comparison for Lectern and Composer cache hits on various different commands (lower is better):
![Performance Comparison](https://github.com/zanderlewis/lectern/tree/main/benchmarks/charts/performance_comparison.png)

Lectern is in orange, and clearly shows that Lectern is much faster than Composer! However, some commands are slower than Composer, but these commands are likely not used often.

This chart shows the average performance based on command category, and the execution time between Lectern and Composer for each category.
![Speedup Chart](https://github.com/zanderlewis/lectern/tree/main/benchmarks/charts/category_performance.png)

Check out the full [Lectern v. Composer Benchmark Report](https://github.com/zanderlewis/lectern/tree/main/benchmarks/performance-report.md) for detailed performance comparisons and insights.

## Commands

### `lectern install`
Installs the dependencies listed in the `composer.json` file. Equivalent to `composer install`.

### `lectern update`
Updates the dependencies to the latest versions allowed by the `composer.json` file. Equivalent to `composer update`.

### `lectern check-outdated`
Checks for outdated dependencies and displays the current and latest versions.

### `lectern search <package>`
Searches for a package on Packagist and displays relevant results.

### `lectern require <package>`
Adds a new dependency to the `composer.json` file and installs it.

### `lectern remove <package>`
Removes a dependency from the `composer.json` file and uninstalls it.

### `lectern show <package>`
Displays detailed information about a specific package.

### `lectern autoload`
Shows the autoloader setup.

### `lectern init`
Initializes a new project with a `composer.json` file.

### `lectern status`
Lists installed packages and their statuses.

### `lectern licenses`
Displays the licenses of installed dependencies.

### `lectern validate`
Validates the `composer.json` file for correctness.
