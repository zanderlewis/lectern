use anyhow::Result;
use semver::{Version, VersionReq};

/// Parse a constraint string into a semver VersionReq
pub fn parse_constraint(spec: &str) -> Result<VersionReq> {
    let spec = spec.trim();
    
    // Handle special cases
    if spec == "*" || spec.is_empty() {
        return Ok(VersionReq::STAR);
    }

    // Handle OR constraints (both | and ||) by selecting the most permissive constraint
    if spec.contains('|') {
        let parts: Vec<&str> = if spec.contains("||") {
            spec.split("||").collect()
        } else {
            spec.split('|').collect()
        };
        
        // Try to parse each constraint and find the most permissive one
        let mut best_constraint = None;
        let mut best_score = 0;
        
        for part in &parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                if let Ok(constraint) = parse_simple_constraint(trimmed) {
                    // Score constraints by how permissive they are
                    let score = score_constraint_permissiveness(trimmed);
                    if score > best_score {
                        best_score = score;
                        best_constraint = Some(constraint);
                    }
                }
            }
        }
        
        if let Some(constraint) = best_constraint {
            return Ok(constraint);
        }
        
        // Fallback: just use the first valid constraint
        for part in &parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                if let Ok(constraint) = parse_simple_constraint(trimmed) {
                    return Ok(constraint);
                }
            }
        }
    }

    parse_simple_constraint(spec)
}

/// Score constraint permissiveness (higher = more permissive)
fn score_constraint_permissiveness(constraint: &str) -> i32 {
    // Prefer constraints that cover larger version ranges
    if constraint.starts_with(">=") && !constraint.contains('<') {
        return 100; // Very permissive (>=X.0.0)
    }
    if constraint.starts_with('^') {
        if let Some(version_part) = constraint.strip_prefix('^') {
            if let Ok(major) = version_part.split('.').next().unwrap_or("0").parse::<u32>() {
                return 50 + major as i32; // Higher major versions get higher scores
            }
        }
        return 50; // Caret constraints are generally permissive
    }
    if constraint.starts_with('~') {
        return 30; // Tilde constraints are less permissive
    }
    if constraint.starts_with('=') {
        return 10; // Exact constraints are least permissive
    }
    if constraint.starts_with(">=") && constraint.contains('<') {
        return 40; // Range constraints
    }
    20 // Default score
}

fn parse_simple_constraint(spec: &str) -> Result<VersionReq> {
    let spec = spec.trim();

    // Handle dev versions
    if spec.starts_with("dev-") {
        return Ok(VersionReq::parse(">=999.0.0-dev")?);
    }

    // Handle caret, tilde, and comparison operators
    if spec.starts_with('^') || spec.starts_with('~') || 
       spec.starts_with(">=") || spec.starts_with("<=") || 
       spec.starts_with('>') || spec.starts_with('<') {
        let normalized = normalize_version_in_constraint(spec)?;
        return Ok(VersionReq::parse(&normalized)?);
    }

    // Handle ranges like "1.0.0 - 2.0.0"
    if spec.contains(" - ") {
        let parts: Vec<&str> = spec.split(" - ").collect();
        if parts.len() == 2 {
            let start = normalize_semver_string(parts[0].trim())?;
            let end = normalize_semver_string(parts[1].trim())?;
            return Ok(VersionReq::parse(&format!(">={start}, <={end}"))?);
        }
    }

    // Handle comma-separated constraints (AND)
    if spec.contains(',') {
        return Ok(VersionReq::parse(spec)?);
    }

    // Treat as exact version
    let normalized = normalize_semver_string(spec)?;
    if Version::parse(&normalized).is_ok() {
        return Ok(VersionReq::parse(&format!("={normalized}"))?);
    }

    // Last resort
    Ok(VersionReq::parse(&normalized).unwrap_or(VersionReq::STAR))
}

fn normalize_version_in_constraint(constraint: &str) -> Result<String> {
    if let Some(version_part) = constraint.strip_prefix('^') {
        let normalized = normalize_semver_string(version_part)?;
        Ok(format!("^{normalized}"))
    } else if let Some(version_part) = constraint.strip_prefix('~') {
        let normalized = normalize_semver_string(version_part)?;
        Ok(format!("~{normalized}"))
    } else if let Some(version_part) = constraint.strip_prefix(">=") {
        let normalized = normalize_semver_string(version_part.trim())?;
        Ok(format!(">={normalized}"))
    } else if let Some(version_part) = constraint.strip_prefix("<=") {
        let normalized = normalize_semver_string(version_part.trim())?;
        Ok(format!("<={normalized}"))
    } else if let Some(version_part) = constraint.strip_prefix('>') {
        let normalized = normalize_semver_string(version_part.trim())?;
        Ok(format!(">{normalized}"))
    } else if let Some(version_part) = constraint.strip_prefix('<') {
        let normalized = normalize_semver_string(version_part.trim())?;
        Ok(format!("<{normalized}"))
    } else {
        Ok(constraint.to_string())
    }
}

/// Normalize a version string to be semver-compatible
fn normalize_semver_string(s: &str) -> Result<String> {
    let s = s.trim().strip_prefix('v').unwrap_or(s.trim());

    // Handle stability suffixes
    let (version_part, stability_suffix) = if let Some(idx) = s.find('-') {
        let (v, suffix) = s.split_at(idx);
        (v, Some(suffix))
    } else {
        (s, None)
    };

    // Split and validate version parts
    let parts: Vec<&str> = version_part.split('.').collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Invalid version: empty"));
    }

    let major = parts.first().unwrap_or(&"0");
    let minor = parts.get(1).unwrap_or(&"0");
    let patch = parts.get(2).unwrap_or(&"0");

    // Validate and clean each part
    let clean_part = |part: &str| -> Result<String> {
        if part.chars().all(char::is_numeric) && !part.is_empty() {
            Ok(part.parse::<u32>().unwrap_or(0).to_string())
        } else if part == "*" {
            Ok("0".to_string())
        } else {
            Err(anyhow::anyhow!("Invalid version part: {}", part))
        }
    };

    let major_clean = clean_part(major)?;
    let minor_clean = clean_part(minor)?;
    let patch_clean = clean_part(patch)?;

    let normalized = format!(
        "{}.{}.{}",
        major_clean,
        minor_clean,
        patch_clean
    );

    if let Some(suffix) = stability_suffix {
        Ok(format!("{normalized}{suffix}"))
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constraint() {
        assert!(parse_constraint("^1.2.3").is_ok());
        assert!(parse_constraint("~1.2").is_ok());
        assert!(parse_constraint(">=1.0.0").is_ok());
        assert!(parse_constraint("*").is_ok());
        assert!(parse_constraint("dev-master").is_ok());
    }

    #[test]
    fn test_or_constraints() {
        // These should pick the highest version
        assert!(parse_constraint("^2|^3").is_ok());
        assert!(parse_constraint("^1.0||^2.0").is_ok());
    }

    #[test]
    fn test_normalize_semver_string() {
        assert_eq!(normalize_semver_string("1.2.3").unwrap(), "1.2.3");
        assert_eq!(normalize_semver_string("v1.2.3").unwrap(), "1.2.3");
        assert_eq!(normalize_semver_string("1.2").unwrap(), "1.2.0");
        assert_eq!(normalize_semver_string("1").unwrap(), "1.0.0");
    }
}
