# Lectern
Lectern is an async and concurrent rewrite of PHP's Composer package manager in Rust. Built with lots of help from ChatGPT-5 and Claude Sonnet 4[.5].

## Recent Improvements
- ⚡ **Optimized outdated command** with better caching and increased concurrency (50x parallelism)
- 🚀 **HTTP connection pooling** with persistent connections and HTTP/2 multiplexing
- 📦 **New commands added**: create-project, dump-autoload, run-script, diagnose, clear-cache, depends, prohibits, browse, suggests, fund
- 🔧 **Modular codebase** with better organized command structure
- 💾 **Improved caching** with multi-layered in-memory and disk caching

## Todo
Lectern is not a complete replacement for Composer just yet!
Some things I haven't been able to test (as AI wrote that code),
or aren't of high priority.
- Composer compatible plugin system
- Testing of private packages and git repositories
- Implement remaining Composer commands (config, global, archive)
- Further optimize dependency resolver with constraint caching

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

### `lectern create-project <package> [directory]`
Creates a new project from a package (similar to `composer create-project`).

### `lectern dump-autoload`
Regenerates the autoloader files.

### `lectern run-script <script>`
Runs a script defined in `composer.json`.

### `lectern diagnose`
Diagnoses the system to identify common problems.

### `lectern clear-cache [type]`
Clears Lectern's cache (types: all, repo, files).

### `lectern depends <package>`
Shows which packages depend on a given package (similar to `composer why`).

### `lectern prohibits <package>`
Shows which packages prevent installing a given package (similar to `composer why-not`).

### `lectern browse <package>`
Opens the package repository URL in your browser.

### `lectern suggests`
Shows all suggested packages from installed dependencies.

### `lectern fund`
Shows funding information for installed packages.
