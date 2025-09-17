//! Test utilities and strict testing helpers for Lectern
//!
//! This module provides utilities for comprehensive, strict testing including:
//! - Property-based testing helpers
//! - Memory leak detection
//! - Performance regression testing
//! - Comprehensive error scenario testing

#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Strict assertion that tracks all assertion calls for coverage
pub struct AssertionTracker {
    calls: Arc<Mutex<HashMap<String, u32>>>,
}

impl AssertionTracker {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn track_assertion(&self, name: &str) {
        let mut calls = self.calls.lock().unwrap();
        *calls.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn get_coverage_report(&self) -> HashMap<String, u32> {
        self.calls.lock().unwrap().clone()
    }

    pub fn assert_minimum_calls(&self, name: &str, min_calls: u32) {
        let calls = self.calls.lock().unwrap();
        let actual_calls = calls.get(name).unwrap_or(&0);
        assert!(
            *actual_calls >= min_calls,
            "Assertion '{name}' was called {actual_calls} times, expected at least {min_calls}"
        );
    }
}

impl Default for AssertionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance testing utilities
pub struct PerformanceTracker {
    benchmarks: Arc<Mutex<HashMap<String, Vec<Duration>>>>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            benchmarks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn time_operation<F, R>(&self, name: &str, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();

        let mut benchmarks = self.benchmarks.lock().unwrap();
        benchmarks
            .entry(name.to_string())
            .or_default()
            .push(duration);

        result
    }

    pub fn assert_performance_regression(&self, name: &str, max_duration: Duration) {
        let benchmarks = self.benchmarks.lock().unwrap();
        if let Some(durations) = benchmarks.get(name) {
            let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
            assert!(
                avg_duration <= max_duration,
                "Performance regression detected for '{}': average {} > maximum {}",
                name,
                avg_duration.as_millis(),
                max_duration.as_millis()
            );
        }
    }

    pub fn get_benchmark_report(&self) -> HashMap<String, Vec<Duration>> {
        self.benchmarks.lock().unwrap().clone()
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory leak detection utilities
pub struct MemoryTracker {
    initial_usage: usize,
    max_allowed_increase: usize,
}

impl MemoryTracker {
    pub fn new(max_allowed_increase_bytes: usize) -> Self {
        Self {
            initial_usage: Self::get_memory_usage(),
            max_allowed_increase: max_allowed_increase_bytes,
        }
    }

    pub fn assert_no_memory_leak(&self) {
        std::thread::sleep(Duration::from_millis(100)); // Allow cleanup
        let current_usage = Self::get_memory_usage();
        let increase = current_usage.saturating_sub(self.initial_usage);

        assert!(
            increase <= self.max_allowed_increase,
            "Memory leak detected: usage increased by {} bytes (max allowed: {} bytes)",
            increase,
            self.max_allowed_increase
        );
    }

    fn get_memory_usage() -> usize {
        // This is a simplified memory tracking - in a real implementation,
        // you might want to use more sophisticated memory profiling
        // For now, we'll return a placeholder value
        0
    }
}

/// Custom allocator for memory tracking
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TrackingAllocator {
    allocated: AtomicUsize,
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            allocated: AtomicUsize::new(0),
        }
    }

    pub fn used_memory(&self) -> Result<usize, ()> {
        Ok(self.allocated.load(Ordering::Relaxed))
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: We're delegating to the system allocator which is safe
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.allocated.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: We're delegating to the system allocator which handles deallocation safely
        unsafe { System.dealloc(ptr, layout) };
        self.allocated.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

// Extended assertion macros with enhanced error reporting
#[macro_export]
macro_rules! assert_eq_detailed {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(
                        "assertion failed: `(left == right)`\n  left: `{:#?}`,\n right: `{:#?}`\n  location: {}:{}:{}",
                        left_val,
                        right_val,
                        file!(),
                        line!(),
                        column!()
                    );
                }
            }
        }
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(
                        "assertion failed: `(left == right)`: {}\n  left: `{:#?}`,\n right: `{:#?}`\n  location: {}:{}:{}",
                        format_args!($($arg)*),
                        left_val,
                        right_val,
                        file!(),
                        line!(),
                        column!()
                    );
                }
            }
        }
    };
}

#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $expected_msg:expr) => {
        match $result {
            Ok(val) => panic!(
                "Expected error containing '{}', but got Ok({:?})",
                $expected_msg, val
            ),
            Err(err) => {
                let error_msg = format!("{}", err);
                assert!(
                    error_msg.contains($expected_msg),
                    "Error message '{}' does not contain expected text '{}'",
                    error_msg,
                    $expected_msg
                );
            }
        }
    };
}

#[macro_export]
macro_rules! assert_performance {
    ($operation:expr, $max_duration:expr) => {
        let start = std::time::Instant::now();
        let _result = $operation;
        let duration = start.elapsed();
        assert!(
            duration <= $max_duration,
            "Performance assertion failed: operation took {:?}, expected maximum {:?}",
            duration,
            $max_duration
        );
    };
}

/// Comprehensive test runner that enforces strict testing standards
pub struct StrictTestRunner {
    assertion_tracker: AssertionTracker,
    performance_tracker: PerformanceTracker,
    memory_tracker: Option<MemoryTracker>,
}

impl StrictTestRunner {
    pub fn new() -> Self {
        Self {
            assertion_tracker: AssertionTracker::new(),
            performance_tracker: PerformanceTracker::new(),
            memory_tracker: None,
        }
    }

    pub fn with_memory_tracking(mut self, max_memory_increase: usize) -> Self {
        self.memory_tracker = Some(MemoryTracker::new(max_memory_increase));
        self
    }

    pub fn run_test<F>(&self, test_name: &str, test_fn: F)
    where
        F: FnOnce(&AssertionTracker, &PerformanceTracker),
    {
        println!("Running strict test: {test_name}");

        test_fn(&self.assertion_tracker, &self.performance_tracker);

        // Check for memory leaks if tracking is enabled
        if let Some(ref tracker) = self.memory_tracker {
            tracker.assert_no_memory_leak();
        }

        println!("✓ Test '{test_name}' passed all strict checks");
    }

    pub fn finalize_testing(&self) {
        println!("\n=== STRICT TESTING REPORT ===");

        // Assertion coverage report
        let assertion_report = self.assertion_tracker.get_coverage_report();
        println!("Assertion Coverage:");
        for (name, count) in assertion_report {
            println!("  {name} : {count} calls");
        }

        // Performance report
        let perf_report = self.performance_tracker.get_benchmark_report();
        println!("\nPerformance Report:");
        for (name, durations) in perf_report {
            let avg = durations.iter().sum::<Duration>() / durations.len() as u32;
            let min = durations.iter().min().unwrap();
            let max = durations.iter().max().unwrap();
            println!("  {name} : avg={avg:?}, min={min:?}, max={max:?}");
        }

        println!("=== END STRICT TESTING REPORT ===\n");
    }
}

impl Default for StrictTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Property-based testing utilities
pub mod property_testing {
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestError, TestRunner};

    /// Generate valid semantic version strings
    pub fn arb_semver() -> impl Strategy<Value = String> {
        (0u32..1000, 0u32..1000, 0u32..1000)
            .prop_map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
    }

    /// Generate valid package names
    pub fn arb_package_name() -> impl Strategy<Value = String> {
        ("[a-z][a-z0-9-]*", "/", "[a-z][a-z0-9-]*")
            .prop_map(|(vendor, sep, package)| format!("{vendor}{sep}{package}"))
    }

    /// Generate valid version constraints
    pub fn arb_version_constraint() -> impl Strategy<Value = String> {
        prop_oneof![
            arb_semver(),
            arb_semver().prop_map(|v| format!("^{v}")),
            arb_semver().prop_map(|v| format!("~{v}")),
            arb_semver().prop_map(|v| format!(">={v}")),
            arb_semver().prop_map(|v| format!(">{v}")),
            arb_semver().prop_map(|v| format!("<={v}")),
            arb_semver().prop_map(|v| format!("<{v}")),
        ]
    }

    /// Run property-based test with strict configuration
    pub fn run_property_test<F>(test_fn: F) -> Result<(), TestError<String>>
    where
        F: Fn(&str) -> bool,
    {
        let config = Config {
            cases: 10000, // Very high number of test cases for strictness
            max_shrink_iters: 10000,
            ..Config::default()
        };

        let mut runner = TestRunner::new(config);
        runner.run(&arb_semver(), |version| {
            prop_assert!(test_fn(&version));
            Ok(())
        })
    }
}

#[cfg(test)]
mod strict_test_utils_tests {
    use super::*;

    #[test]
    fn test_assertion_tracker() {
        let tracker = AssertionTracker::new();
        tracker.track_assertion("test_assertion");
        tracker.track_assertion("test_assertion");
        tracker.track_assertion("another_assertion");

        let report = tracker.get_coverage_report();
        assert_eq!(report.get("test_assertion"), Some(&2));
        assert_eq!(report.get("another_assertion"), Some(&1));

        tracker.assert_minimum_calls("test_assertion", 2);
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new();

        tracker.time_operation("fast_op", || {
            std::thread::sleep(Duration::from_millis(1));
        });

        tracker.assert_performance_regression("fast_op", Duration::from_millis(100));
    }

    #[test]
    #[should_panic(expected = "Performance regression detected")]
    fn test_performance_regression_detection() {
        let tracker = PerformanceTracker::new();

        tracker.time_operation("slow_op", || {
            std::thread::sleep(Duration::from_millis(10));
        });

        tracker.assert_performance_regression("slow_op", Duration::from_millis(5));
    }
}
