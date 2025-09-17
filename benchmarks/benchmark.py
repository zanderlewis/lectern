#!/usr/bin/env python3
"""
Comprehensive Lectern vs Composer Benchmark Suite
Automatically tests all commands and generates detailed Markdown report
"""

import subprocess
import time
import sys
from datetime import datetime
from pathlib import Path
from typing import List, Tuple
import argparse
import matplotlib.pyplot as plt
import numpy as np
import matplotlib.patches as mpatches


class BenchmarkResult:
    def __init__(
        self,
        command: str,
        lectern_time: float,
        composer_time: float,
        lectern_success: bool = True,
        composer_success: bool = True,
        notes: str = "",
    ):
        self.command = command
        self.lectern_time = lectern_time
        self.composer_time = composer_time
        self.lectern_success = lectern_success
        self.composer_success = composer_success
        self.notes = notes

    @property
    def improvement(self) -> float:
        if self.lectern_time > 0 and self.composer_time > 0:
            return self.composer_time / self.lectern_time
        return 0.0

    @property
    def status(self) -> str:
        if not self.lectern_success:
            return "❌ Lectern Failed"
        if not self.composer_success:
            return "⚠️ Composer Failed"
        if self.improvement > 1:
            return f"🚀 {self.improvement:.1f}x faster"
        elif self.improvement < 1:
            return f"⚡ {1 / self.improvement:.1f}x slower"
        else:
            return "🟰 Similar performance"


class LecternBenchmark:
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.lectern_binary = project_root / "target" / "release" / "lectern"
        self.results: List[BenchmarkResult] = []
        self.test_projects = [
            "complex-app",
            "simple-laravel",
            "simple-test",
            "symfony-app",
        ]

    def run_command(
        self, cmd: List[str], cwd: Path = None, timeout: int = 60
    ) -> Tuple[float, bool, str]:
        """Run a command and return (execution_time, success, output)"""
        start_time = time.time()
        try:
            result = subprocess.run(
                cmd,
                cwd=cwd or self.project_root,
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            end_time = time.time()
            execution_time = end_time - start_time
            success = result.returncode == 0
            output = result.stdout + result.stderr
            return execution_time, success, output
        except subprocess.TimeoutExpired:
            end_time = time.time()
            execution_time = end_time - start_time
            return execution_time, False, f"Command timed out after {timeout}s"
        except Exception as e:
            end_time = time.time()
            execution_time = end_time - start_time
            return execution_time, False, str(e)

    def build_lectern(self):
        """Build Lectern in release mode"""
        print("🔨 Building Lectern in release mode...")
        _, success, output = self.run_command(
            ["cargo", "build", "--release"], timeout=120
        )
        if not success:
            print(f"❌ Failed to build Lectern: {output}")
            sys.exit(1)
        print("✅ Lectern built successfully")

    def backup_project_files(self, test_dir: Path) -> dict:
        """Backup important project files before modification"""
        backups = {}
        for file_name in ["composer.json", "composer.lock"]:
            file_path = test_dir / file_name
            if file_path.exists():
                backups[file_name] = file_path.read_text()
        return backups

    def restore_project_files(self, test_dir: Path, backups: dict):
        """Restore project files from backup"""
        for file_name, content in backups.items():
            file_path = test_dir / file_name
            file_path.write_text(content)

        # Remove any files that weren't in the backup
        for file_name in ["composer.json", "composer.lock"]:
            if file_name not in backups:
                file_path = test_dir / file_name
                if file_path.exists():
                    file_path.unlink()

    def test_command(
        self,
        command_name: str,
        lectern_cmd: List[str],
        composer_cmd: List[str],
        test_project: str = "complex-app",
        notes: str = "",
        use_backup: bool = False,
    ) -> BenchmarkResult:
        """Test a single command comparison"""
        print(f"🧪 Testing {command_name}...")

        # Set up test directory
        test_dir = self.project_root / "benchmarks" / test_project
        if not test_dir.exists():
            test_dir = self.project_root / "benchmarks" / "complex-app"  # fallback

        # Backup files if this test might modify them
        backups = {}
        if use_backup:
            backups = self.backup_project_files(test_dir)

        # Test Lectern
        lectern_full_cmd = [str(self.lectern_binary)] + lectern_cmd
        lectern_time, lectern_success, _ = self.run_command(
            lectern_full_cmd, test_dir
        )

        # Restore files before testing Composer
        if use_backup:
            self.restore_project_files(test_dir, backups)

        # Test Composer
        composer_full_cmd = ["composer"] + composer_cmd
        composer_time, composer_success, _ = self.run_command(
            composer_full_cmd, test_dir
        )

        # Restore files again after testing
        if use_backup:
            self.restore_project_files(test_dir, backups)

        result = BenchmarkResult(
            command_name,
            lectern_time,
            composer_time,
            lectern_success,
            composer_success,
            notes,
        )

        print(f"   Lectern: {lectern_time:.3f}s {'✅' if lectern_success else '❌'}")
        print(f"   Composer: {composer_time:.3f}s {'✅' if composer_success else '❌'}")
        print(f"   Result: {result.status}")
        print()

        return result

    def run_all_tests(self):
        """Run benchmark tests"""
        print("=== LECTERN VS COMPOSER BENCHMARK ===")
        print(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("Testing all Lectern commands with concurrent operations\n")

        # Build Lectern first
        self.build_lectern()
        print()

        # Core package management commands
        print("📦 CORE PACKAGE MANAGEMENT COMMANDS")
        print("=" * 50)

        # Real install test with backup/restore
        self.results.append(
            self.test_command(
                "Install Dependencies",
                ["install"],
                ["install"],
                "simple-test",
                "Real installation with backup/restore",
                use_backup=True,
            )
        )

        # Real update test with backup/restore
        self.results.append(
            self.test_command(
                "Update Dependencies",
                ["update"],
                ["update"],
                "simple-test",
                "Real update with backup/restore",
                use_backup=True,
            )
        )

        # Information and analysis commands
        print("📊 INFORMATION & ANALYSIS COMMANDS")
        print("=" * 50)

        self.results.append(
            self.test_command(
                "Search Packages", ["search", "laravel"], ["search", "laravel"]
            )
        )

        self.results.append(
            self.test_command(
                "Show Package Info",
                ["show", "laravel/framework"],
                ["show", "--available", "laravel/framework"],
            )
        )

        self.results.append(
            self.test_command("Check Outdated", ["outdated"], ["outdated"])
        )

        self.results.append(
            self.test_command("Show Licenses", ["licenses"], ["licenses"])
        )

        self.results.append(self.test_command("Show Status", ["status"], ["status"]))

        # Dependency management commands
        print("🔧 DEPENDENCY MANAGEMENT COMMANDS")
        print("=" * 50)

        # Real require test with backup/restore
        self.results.append(
            self.test_command(
                "Require Package",
                ["require", "psr/log"],
                ["require", "psr/log"],
                "simple-test",
                "Real package addition with backup/restore",
                use_backup=True,
            )
        )

        # Real remove test with backup/restore (remove a package that exists)
        self.results.append(
            self.test_command(
                "Remove Package",
                ["remove", "guzzlehttp/guzzle"],
                ["remove", "guzzlehttp/guzzle"],
                "simple-test",
                "Real package removal with backup/restore",
                use_backup=True,
            )
        )

        # Test across different project types
        print("🏗️ MULTI-PROJECT TESTING")
        print("=" * 50)

        for project in ["simple-laravel", "symfony-app"]:
            if (self.project_root / "benchmarks" / project).exists():
                self.results.append(
                    self.test_command(
                        f"Status Check ({project})",
                        ["status"],
                        ["status"],
                        project,
                        f"Testing on {project} project structure",
                    )
                )

                self.results.append(
                    self.test_command(
                        f"Outdated Check ({project})",
                        ["outdated"],
                        ["outdated"],
                        project,
                        f"Testing on {project} project structure",
                    )
                )

    def generate_performance_charts(self, output_dir: Path) -> List[str]:
        """Generate performance charts and return list of generated file paths"""
        plt.style.use('default')
        chart_files = []
        
        # Filter successful results for charting
        successful_results = [
            r for r in self.results if r.lectern_success and r.composer_success
        ]
        
        if not successful_results:
            print("⚠️ No successful results to chart")
            return chart_files
        
        # 1. Performance Comparison Bar Chart
        chart_files.append(self._generate_performance_comparison_chart(successful_results, output_dir))
        
        # 2. Execution Time Comparison Chart
        chart_files.append(self._generate_execution_time_chart(successful_results, output_dir))
        
        # 3. Performance Improvement Distribution
        chart_files.append(self._generate_improvement_distribution_chart(successful_results, output_dir))
        
        # 4. Command Category Performance
        chart_files.append(self._generate_category_performance_chart(successful_results, output_dir))
        
        return chart_files
    
    def _generate_performance_comparison_chart(self, results: List[BenchmarkResult], output_dir: Path) -> str:
        """Generate side-by-side performance comparison chart"""
        commands = [r.command for r in results]
        lectern_times = [r.lectern_time for r in results]
        composer_times = [r.composer_time for r in results]
        
        fig, ax = plt.subplots(figsize=(14, 8))
        
        x = np.arange(len(commands))
        width = 0.35
        
        bars1 = ax.bar(x - width/2, lectern_times, width, label='Lectern', color='#FF6B35', alpha=0.8)
        bars2 = ax.bar(x + width/2, composer_times, width, label='Composer', color='#2E86AB', alpha=0.8)
        
        ax.set_xlabel('Commands', fontsize=12)
        ax.set_ylabel('Execution Time (seconds)', fontsize=12)
        ax.set_title('Lectern vs Composer: Execution Time Comparison', fontsize=14, fontweight='bold')
        ax.set_xticks(x)
        ax.set_xticklabels(commands, rotation=45, ha='right')
        ax.legend()
        ax.grid(True, alpha=0.3, axis='y')
        
        # Add value labels on bars
        for bar in bars1:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{height:.3f}s', ha='center', va='bottom', fontsize=8)
        
        for bar in bars2:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{height:.3f}s', ha='center', va='bottom', fontsize=8)
        
        plt.tight_layout()
        
        chart_path = output_dir / "performance_comparison.png"
        plt.savefig(chart_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        return str(chart_path)
    
    def _generate_execution_time_chart(self, results: List[BenchmarkResult], output_dir: Path) -> str:
        """Generate execution time scatter plot with improvement indicators"""
        fig, ax = plt.subplots(figsize=(12, 8))
        
        lectern_times = [r.lectern_time for r in results]
        composer_times = [r.composer_time for r in results]
        improvements = [r.improvement for r in results]
        commands = [r.command for r in results]
        
        # Color based on improvement level
        colors = []
        for imp in improvements:
            if imp >= 10:
                colors.append('#00C851')  # Green for ultra-fast
            elif imp >= 2:
                colors.append('#FF8800')  # Orange for fast
            else:
                colors.append('#FF4444')  # Red for similar/slower
        
        scatter = ax.scatter(composer_times, lectern_times, c=colors, s=100, alpha=0.7, edgecolors='black')
        
        # Add diagonal line (y=x) for reference
        max_time = max(max(lectern_times), max(composer_times))
        ax.plot([0, max_time], [0, max_time], 'k--', alpha=0.5, label='Equal Performance')
        
        ax.set_xlabel('Composer Time (seconds)', fontsize=12)
        ax.set_ylabel('Lectern Time (seconds)', fontsize=12)
        ax.set_title('Execution Time Comparison: Lectern vs Composer', fontsize=14, fontweight='bold')
        
        # Add command labels
        for i, cmd in enumerate(commands):
            ax.annotate(cmd, (composer_times[i], lectern_times[i]), 
                       xytext=(5, 5), textcoords='offset points', fontsize=8, alpha=0.8)
        
        # Create custom legend
        ultra_fast = mpatches.Patch(color='#00C851', label='Ultra-fast (10x+)')
        fast = mpatches.Patch(color='#FF8800', label='Fast (2-10x)')
        similar = mpatches.Patch(color='#FF4444', label='Similar (0.5-2x)')
        equal_line = mpatches.Patch(color='black', label='Equal Performance')
        
        ax.legend(handles=[ultra_fast, fast, similar, equal_line], loc='upper left')
        ax.grid(True, alpha=0.3)
        
        plt.tight_layout()
        
        chart_path = output_dir / "execution_time_scatter.png"
        plt.savefig(chart_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        return str(chart_path)
    
    def _generate_improvement_distribution_chart(self, results: List[BenchmarkResult], output_dir: Path) -> str:
        """Generate performance improvement distribution histogram"""
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        
        improvements = [r.improvement for r in results]
        
        # Histogram
        ax1.hist(improvements, bins=15, color='#FF6B35', alpha=0.7, edgecolor='black')
        ax1.set_xlabel('Performance Improvement (x times faster)', fontsize=12)
        ax1.set_ylabel('Number of Commands', fontsize=12)
        ax1.set_title('Distribution of Performance Improvements', fontsize=14, fontweight='bold')
        ax1.grid(True, alpha=0.3, axis='y')
        
        # Add statistics text
        avg_improvement = np.mean(improvements)
        median_improvement = np.median(improvements)
        max_improvement = np.max(improvements)
        
        stats_text = f'Mean: {avg_improvement:.1f}x\nMedian: {median_improvement:.1f}x\nMax: {max_improvement:.1f}x'
        ax1.text(0.7, 0.8, stats_text, transform=ax1.transAxes, fontsize=10,
                bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8))
        
        # Box plot
        ax2.boxplot(improvements, vert=True, patch_artist=True,
                   boxprops=dict(facecolor='#2E86AB', alpha=0.7),
                   medianprops=dict(color='red', linewidth=2))
        ax2.set_ylabel('Performance Improvement (x times faster)', fontsize=12)
        ax2.set_title('Performance Improvement Distribution', fontsize=14, fontweight='bold')
        ax2.grid(True, alpha=0.3, axis='y')
        ax2.set_xticklabels(['All Commands'])
        
        plt.tight_layout()
        
        chart_path = output_dir / "improvement_distribution.png"
        plt.savefig(chart_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        return str(chart_path)
    
    def _generate_category_performance_chart(self, results: List[BenchmarkResult], output_dir: Path) -> str:
        """Generate performance chart by command category"""
        # Categorize commands
        categories = {
            'Package Management': ['Install Dependencies', 'Update Dependencies', 'Require Package', 'Remove Package'],
            'Information & Analysis': ['Search Packages', 'Show Package Info', 'Check Outdated', 'Show Licenses', 'Show Status'],
            'Multi-Project': [cmd for cmd in [r.command for r in results] if '(' in cmd]  # Commands with project names in parentheses
        }
        
        category_data = {}
        for category, commands in categories.items():
            category_results = [r for r in results if r.command in commands]
            if category_results:
                avg_improvement = np.mean([r.improvement for r in category_results])
                avg_lectern_time = np.mean([r.lectern_time for r in category_results])
                avg_composer_time = np.mean([r.composer_time for r in category_results])
                category_data[category] = {
                    'improvement': avg_improvement,
                    'lectern_time': avg_lectern_time,
                    'composer_time': avg_composer_time,
                    'count': len(category_results)
                }
        
        if not category_data:
            # Fallback: create simple categories based on command names
            category_data = {'All Commands': {
                'improvement': np.mean([r.improvement for r in results]),
                'lectern_time': np.mean([r.lectern_time for r in results]),
                'composer_time': np.mean([r.composer_time for r in results]),
                'count': len(results)
            }}
        
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        
        categories_list = list(category_data.keys())
        improvements = [data['improvement'] for data in category_data.values()]
        counts = [data['count'] for data in category_data.values()]
        
        # Performance improvement by category
        bars1 = ax1.bar(categories_list, improvements, color=['#FF6B35', '#2E86AB', '#00C851'][:len(categories_list)], alpha=0.8)
        ax1.set_ylabel('Average Performance Improvement (x)', fontsize=12)
        ax1.set_title('Average Performance by Command Category', fontsize=14, fontweight='bold')
        ax1.grid(True, alpha=0.3, axis='y')
        
        # Add value labels
        for bar, count in zip(bars1, counts):
            height = bar.get_height()
            ax1.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.1f}x\n({count} cmds)', ha='center', va='bottom', fontsize=10)
        
        # Execution time comparison by category
        lectern_times = [data['lectern_time'] for data in category_data.values()]
        composer_times = [data['composer_time'] for data in category_data.values()]
        
        x = np.arange(len(categories_list))
        width = 0.35
        
        bars2 = ax2.bar(x - width/2, lectern_times, width, label='Lectern', color='#FF6B35', alpha=0.8)
        bars3 = ax2.bar(x + width/2, composer_times, width, label='Composer', color='#2E86AB', alpha=0.8)
        
        ax2.set_ylabel('Average Execution Time (seconds)', fontsize=12)
        ax2.set_title('Average Execution Time by Category', fontsize=14, fontweight='bold')
        ax2.set_xticks(x)
        ax2.set_xticklabels(categories_list)
        ax2.legend()
        ax2.grid(True, alpha=0.3, axis='y')
        
        plt.tight_layout()
        
        chart_path = output_dir / "category_performance.png"
        plt.savefig(chart_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        return str(chart_path)
    
    def _generate_chart_markdown_section(self, chart_files: List[str], charts_dir: Path) -> str:
        """Generate markdown section with embedded charts"""
        chart_section = "\n## Performance Visualizations\n\n"
        chart_section += "The following charts provide visual insights into Lectern's performance compared to Composer:\n\n"
        
        chart_descriptions = {
            "performance_comparison.png": {
                "title": "Execution Time Comparison",
                "description": "Side-by-side comparison of execution times for each command, showing the absolute time difference between Lectern and Composer."
            },
            "execution_time_scatter.png": {
                "title": "Performance Scatter Plot",
                "description": "Scatter plot showing the relationship between Composer and Lectern execution times. Points below the diagonal line indicate faster Lectern performance."
            },
            "improvement_distribution.png": {
                "title": "Performance Improvement Distribution",
                "description": "Distribution analysis of performance improvements, showing both histogram and box plot views of the speed-up factors."
            },
            "category_performance.png": {
                "title": "Category Performance Analysis",
                "description": "Performance breakdown by command category, comparing average improvements and execution times across different types of operations."
            }
        }
        
        for chart_file in chart_files:
            chart_path = Path(chart_file)
            chart_filename = chart_path.name
            relative_path = f"charts/{chart_filename}"
            
            if chart_filename in chart_descriptions:
                info = chart_descriptions[chart_filename]
                chart_section += f"### {info['title']}\n\n"
                chart_section += f"{info['description']}\n\n"
                chart_section += f"![{info['title']}]({relative_path})\n\n"
            else:
                # Fallback for any unexpected chart files
                chart_section += f"### Performance Chart\n\n"
                chart_section += f"![Performance Chart]({relative_path})\n\n"
        
        return chart_section

    def generate_markdown_report(self, with_charts: bool = True) -> str:
        """Generate comprehensive Markdown report from template"""
        # Load template
        template_path = Path(__file__).parent / "report_template.md"
        with open(template_path, 'r') as f:
            template = f.read()
        
        # Generate performance charts if requested
        chart_section = ""
        if with_charts:
            print("📊 Generating performance charts...")
            charts_dir = Path(__file__).parent / "charts"
            charts_dir.mkdir(exist_ok=True)
            
            try:
                chart_files = self.generate_performance_charts(charts_dir)
                chart_section = self._generate_chart_markdown_section(chart_files, charts_dir)
                print(f"✅ Generated {len(chart_files)} charts")
            except Exception as e:
                print(f"⚠️ Failed to generate charts: {e}")
                chart_section = "\n*Charts could not be generated due to an error.*\n"
        
        successful_results = [
            r for r in self.results if r.lectern_success and r.composer_success
        ]
        total_improvements = [
            r.improvement for r in successful_results if r.improvement > 0
        ]
        avg_improvement = (
            sum(total_improvements) / len(total_improvements)
            if total_improvements
            else 0
        )

        # Build results table
        results_table = ""
        for result in self.results:
            results_table += f"| {result.command} | {result.lectern_time:.3f}s | {result.composer_time:.3f}s | {result.improvement:.1f}x | {result.status} |\n"

        # Performance categories
        fast_commands = [r for r in successful_results if r.improvement >= 10]
        medium_commands = [r for r in successful_results if 2 <= r.improvement < 10]
        similar_commands = [r for r in successful_results if 0.5 <= r.improvement < 2]

        # Ultra-fast commands section
        ultra_fast_commands = ""
        for cmd in fast_commands:
            ultra_fast_commands += f"- **{cmd.command}**: {cmd.improvement:.1f}x faster ({cmd.composer_time:.3f}s → {cmd.lectern_time:.3f}s)\n"

        # Fast commands section
        fast_commands_text = ""
        for cmd in medium_commands:
            fast_commands_text += f"- **{cmd.command}**: {cmd.improvement:.1f}x faster ({cmd.composer_time:.3f}s → {cmd.lectern_time:.3f}s)\n"

        # Similar commands section
        similar_commands_text = ""
        for cmd in similar_commands:
            similar_commands_text += f"- **{cmd.command}**: {cmd.improvement:.1f}x ({cmd.composer_time:.3f}s vs {cmd.lectern_time:.3f}s)\n"

        # Core commands analysis
        core_commands_analysis = ""
        core_commands = [
            "Install Dependencies",
            "Update Dependencies", 
            "Search Packages",
            "Show Package Info",
        ]
        for cmd_name in core_commands:
            result = next((r for r in self.results if r.command == cmd_name), None)
            if result:
                core_commands_analysis += f"""
#### {result.command}
- **Performance**: {result.improvement:.1f}x faster
- **Lectern**: {result.lectern_time:.3f}s
- **Composer**: {result.composer_time:.3f}s
- **Status**: {result.status}
- **Notes**: {result.notes or "Standard operation"}
"""

        # Issues section
        failed_results = [
            r for r in self.results if not r.lectern_success or not r.composer_success
        ]
        issues_section = ""
        if failed_results:
            issues_section = """
## Issues Found

The following tests encountered problems:

"""
            for result in failed_results:
                issues_section += f"""
### {result.command}
- **Lectern Success**: {"✅" if result.lectern_success else "❌"}
- **Composer Success**: {"✅" if result.composer_success else "❌"}
- **Notes**: {result.notes}
"""

        # Format the template with all variables
        return template.format(
            timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            total_tests=len(self.results),
            successful_tests=len(successful_results),
            avg_improvement=avg_improvement,
            best_performance=max(total_improvements) if total_improvements else 0,
            results_table=results_table.strip(),
            ultra_fast_count=len(fast_commands),
            ultra_fast_commands=ultra_fast_commands.strip(),
            fast_count=len(medium_commands),
            fast_commands=fast_commands_text.strip(),
            similar_count=len(similar_commands),
            similar_commands=similar_commands_text.strip(),
            core_commands_analysis=core_commands_analysis.strip(),
            issues_section=issues_section.strip(),
            test_projects=", ".join(self.test_projects),
            test_date=datetime.now().strftime("%Y-%m-%d"),
            charts_section=chart_section,
        )

    def save_report(self, report: str, filename: str = None):
        """Save the markdown report to file"""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"lectern_benchmark_report_{timestamp}.md"

        report_path = self.project_root / "benchmarks" / filename
        with open(report_path, "w") as f:
            f.write(report)

        print(f"📊 Benchmark report saved to: {report_path}")
        return report_path


def main():
    parser = argparse.ArgumentParser(
        description="Comprehensive Lectern vs Composer benchmark"
    )
    parser.add_argument("--output", "-o", help="Output filename for the report")
    parser.add_argument("--project", help="Test specific project only")
    parser.add_argument("--no-charts", action="store_true", help="Skip chart generation")

    args = parser.parse_args()

    # Find project root
    current_dir = Path.cwd()
    project_root = current_dir
    while project_root != project_root.parent:
        if (project_root / "Cargo.toml").exists():
            break
        project_root = project_root.parent
    else:
        print("❌ Could not find Lectern project root (no Cargo.toml found)")
        sys.exit(1)

    print(f"📁 Project root: {project_root}")

    # Run benchmark
    benchmark = LecternBenchmark(project_root)

    benchmark.run_all_tests()

    # Generate and save report
    print("\n📊 Generating comprehensive report...")
    report = benchmark.generate_markdown_report(with_charts=not args.no_charts)
    report_path = benchmark.save_report(report, args.output)

    # Print summary
    successful_results = [
        r for r in benchmark.results if r.lectern_success and r.composer_success
    ]
    total_improvements = [
        r.improvement for r in successful_results if r.improvement > 0
    ]
    avg_improvement = (
        sum(total_improvements) / len(total_improvements) if total_improvements else 0
    )

    print(f"""
🎉 BENCHMARK COMPLETE!

📈 Results Summary:
   • Tests completed: {len(benchmark.results)}
   • Successful comparisons: {len(successful_results)}
   • Average improvement: {avg_improvement:.1f}x faster
   • Report saved: {report_path.name}

🚀 Lectern performance advantage confirmed!
""")


if __name__ == "__main__":
    main()
