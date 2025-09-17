# Lectern
Lectern is an async and concurrent rewrite of PHP's Composer package manager in Rust. Built with lots of help from ChatGPT-5 and Claude Sonnet 4.

## Todo
Lectern is not a complete replacement for Composer just yet!
Some things I haven't been able to test (as AI wrote that code),
or aren't of high priority.
- Composer compatible plugin system
- Testing of private packages and git repositories
- Implement the rest of Composer's commands
- Speed up Outdated checks, as they are slower than Composer currently

## Benchmarks
This image is the performance comparison for Lectern and Composer cache hits on various different commands (lower is better):
![Performance Comparison](/benchmarks/charts/performance_comparison.png)

Lectern is in orange, and clearly shows that Lectern is much faster than Composer! However, some commands are slower than Composer, but these commands are likely not used often.

This chart shows the average performance based on command category, and the execution time between Lectern and Composer for each category.
![Speedup Chart](/benchmarks/charts/category_performance.png)

Check out the full [Lectern v. Composer Benchmark Report](/benchmarks/performance-report.md) for detailed performance comparisons and insights.

## Cache location
- Lectern uses a global user cache directory by default. It will use `$XDG_CACHE_HOME/lectern` when the XDG environment is set, otherwise `~/.cache/lectern`.
- To clear the cache manually, remove that directory (for example `rm -rf ~/.cache/lectern`).

## Publishing (CI)
- A GitHub Actions workflow has been added to publish the crate to crates.io when a GitHub release is published: `.github/workflows/publish.yml`.
- You must add a repository secret named `CRATES_IO_TOKEN` (your crates.io API token) for publishing to work. The workflow uses this secret to run `cargo publish`.

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
