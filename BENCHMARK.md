# Lectern vs Composer Performance Benchmark

> Generated on: 2025-10-06 14:55:17

This benchmark compares the performance of Lectern against Composer across various common operations.

## System Information

- **Benchmark Tool**: hyperfine with 2 warmup runs and 5 test runs (3 for require/remove)
- **Test Setup**: Warm cache conditions for both tools

## Results Summary

| Command | Lectern (ms) | Composer (ms) | Speedup |
|---------|--------------|---------------|---------|
| install | 53.4 | 1212.7 | 22.7x |
| update | 44.5 | 775.3 | 17.4x |
| search | 4.8 | 581.6 | 121.2x |
| show | 6.9 | 129.3 | 18.7x |
| outdated | 10.8 | 2374.7 | 219.9x |
| licenses | 5.8 | 146.9 | 25.3x |
| status | 3.9 | 109.1 | 28.0x |
| require | 53.8 | 816.8 | 15.2x |
| remove | 43.4 | 664.9 | 15.3x |

## Detailed Results

Each benchmark includes mean execution time ± standard deviation, with min and max values.

### Install Command

Fresh installation of all dependencies.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern install` | 53.4 ± 5.5 | 44.5 | 58.9 | 1.00 |
| `composer install` | 1212.7 ± 74.2 | 1141.3 | 1326.4 | 22.72 ± 2.73 |

### Update Command

Update all dependencies to their latest allowed versions.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern update` | 44.5 ± 2.7 | 39.9 | 47.0 | 1.00 |
| `composer update` | 775.3 ± 135.3 | 666.5 | 1006.5 | 17.41 ± 3.22 |

### Search Command

Search for packages on Packagist.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern search` | 4.8 ± 1.1 | 3.5 | 6.4 | 1.00 |
| `composer search` | 581.6 ± 30.6 | 546.0 | 619.0 | 121.61 ± 27.95 |

### Show Command

Display detailed information about a specific package.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern show` | 6.9 ± 1.0 | 5.9 | 8.2 | 1.00 |
| `composer show` | 129.3 ± 24.8 | 104.2 | 164.6 | 18.86 ± 4.58 |

### Outdated Command

Check for outdated packages.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern outdated` | 10.8 ± 1.0 | 9.3 | 11.8 | 1.00 |
| `composer outdated` | 2374.7 ± 158.9 | 2233.7 | 2626.8 | 220.64 ± 25.66 |

### Licenses Command

Display licenses of installed packages.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern licenses` | 5.8 ± 1.2 | 4.2 | 7.3 | 1.00 |
| `composer licenses` | 146.9 ± 32.3 | 107.8 | 174.1 | 25.13 ± 7.50 |

### Status Command

Show status of installed packages.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern status` | 3.9 ± 1.3 | 2.3 | 5.1 | 1.00 |
| `composer show` | 109.1 ± 13.4 | 96.5 | 126.2 | 27.71 ± 9.84 |

### Require Command

Add a new package to the project.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern require` | 53.8 ± 4.8 | 49.4 | 58.9 | 1.00 |
| `composer require` | 816.8 ± 126.6 | 736.1 | 962.8 | 15.18 ± 2.72 |

### Remove Command

Remove a package from the project.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `lectern remove` | 43.4 ± 1.5 | 42.0 | 45.0 | 1.00 |
| `composer remove` | 664.9 ± 45.3 | 613.2 | 697.7 | 15.34 ± 1.17 |

---

## Notes

- All benchmarks are run with warm caches to measure steady-state performance
- Times shown are in milliseconds (ms)
- Speedup is calculated as: Composer time / Lectern time
- Each command is run multiple times (2 warmup + 5 test runs, or 1 warmup + 3 test runs for require/remove)

