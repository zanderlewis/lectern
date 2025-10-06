# Lectern vs Composer Performance Benchmark

> Generated on: 2025-10-06 14:34:41

This benchmark compares the performance of Lectern against Composer across various common operations.

## System Information

- **Lectern Version**: `lectern --version` output
- **Composer Version**: `composer --version` output
- **Benchmark Tool**: hyperfine with 2 warmup runs and 5 test runs (3 for require/remove)

## Results Summary

| Command | Lectern | Composer | Speedup |
|---------|---------|----------|---------|
| install | install` | install` | x |
| update | update` | update` | x |
| search | search` | search` | x |
| show | show` | show` | x |
| outdated | outdated` | outdated` | x |
| licenses | licenses` | licenses` | x |
| status | status` | show` | x |
| require | require` | require` | x |
| remove | remove` | remove` | x |

## Detailed Results

### Install Command
| `lectern install` | 56.1 ± 6.3 | 45.4 | 61.3 | 1.00 |
| `composer install` | 1191.3 ± 88.8 | 1130.4 | 1334.7 | 21.24 ± 2.86 |

### Update Command
| `lectern update` | 43.8 ± 2.7 | 39.7 | 46.6 | 1.00 |
| `composer update` | 895.0 ± 201.6 | 701.0 | 1203.9 | 20.43 ± 4.77 |

### Search Command
| `lectern search` | 3.9 ± 2.7 | 1.7 | 8.4 | 1.00 |
| `composer search` | 561.2 ± 28.2 | 532.7 | 600.3 | 145.78 ± 103.52 |

### Show Command
| `lectern show` | 4.9 ± 0.7 | 4.3 | 6.0 | 1.00 |
| `composer show` | 133.6 ± 31.4 | 105.8 | 167.7 | 27.35 ± 7.67 |

### Outdated Command
| `lectern outdated` | 13.4 ± 2.4 | 11.2 | 17.4 | 1.00 |
| `composer outdated` | 2375.9 ± 73.2 | 2249.9 | 2427.9 | 177.30 ± 32.47 |

### Licenses Command
| `lectern licenses` | 5.5 ± 0.8 | 4.6 | 6.5 | 1.00 |
| `composer licenses` | 160.6 ± 12.8 | 145.0 | 177.9 | 29.23 ± 4.93 |

### Status Command
| `lectern status` | 4.2 ± 0.9 | 3.1 | 5.1 | 1.00 |
| `composer show` | 109.1 ± 11.7 | 96.0 | 123.8 | 26.25 ± 6.13 |

### Require Command
| `lectern require` | 50.7 ± 6.3 | 45.6 | 57.7 | 1.00 |
| `composer require` | 764.1 ± 84.1 | 670.7 | 833.8 | 15.07 ± 2.49 |

### Remove Command
| `lectern remove` | 42.7 ± 5.5 | 38.3 | 48.8 | 1.00 |
| `composer remove` | 639.3 ± 56.3 | 600.6 | 704.0 | 14.97 ± 2.32 |
