use lectern::resolver::dependency::{find_best_version, generate_content_hash};
use lectern::resolver::packagist::{P2Version, P2Dist, P2Source};
use lectern::resolver::version::parse_constraint;
use std::collections::BTreeMap;

#[cfg(test)]
mod dependency_tests {
    use super::*;

    fn create_test_version(version: &str, normalized: Option<&str>) -> P2Version {
        P2Version {
            version: version.to_string(),
            version_normalized: normalized.unwrap_or(version).to_string(),
            dist: Some(P2Dist {
                dtype: Some("zip".to_string()),
                url: Some(format!("https://example.com/{}.zip", version)),
                reference: Some(format!("ref-{}", version)),
                shasum: Some("".to_string()),
            }),
            source: Some(P2Source {
                stype: Some("git".to_string()),
                url: Some("https://github.com/test/repo.git".to_string()),
                reference: Some(format!("ref-{}", version)),
            }),
            require: Some({
                let mut req = BTreeMap::new();
                req.insert("php".to_string(), ">=7.4".to_string());
                req
            }),
            extra: None,
            other: serde_json::Map::new(),
        }
    }

    #[test]
    fn test_find_best_version_caret_constraint() {
        let versions = vec![
            create_test_version("1.0.0", Some("1.0.0.0")),
            create_test_version("1.2.0", Some("1.2.0.0")),
            create_test_version("1.5.3", Some("1.5.3.0")),
            create_test_version("2.0.0", Some("2.0.0.0")),
        ];

        let constraint = parse_constraint("^1.0").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should pick the highest 1.x version
        assert_eq!(best.version, "1.5.3");
    }

    #[test]
    fn test_find_best_version_tilde_constraint() {
        let versions = vec![
            create_test_version("1.2.0", Some("1.2.0.0")),
            create_test_version("1.2.3", Some("1.2.3.0")),
            create_test_version("1.2.9", Some("1.2.9.0")),
            create_test_version("1.3.0", Some("1.3.0.0")),
        ];

        let constraint = parse_constraint("~1.2.0").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should pick the highest 1.2.x version
        assert_eq!(best.version, "1.2.9");
    }

    #[test]
    fn test_find_best_version_or_constraint() {
        let versions = vec![
            create_test_version("1.5.0", Some("1.5.0.0")),
            create_test_version("2.3.0", Some("2.3.0.0")),
            create_test_version("3.1.0", Some("3.1.0.0")),
            create_test_version("4.0.0", Some("4.0.0.0")),
        ];

        let constraint = parse_constraint("^2|^3").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should pick the highest version that matches either ^2 or ^3
        assert_eq!(best.version, "3.1.0");
    }

    #[test]
    fn test_find_best_version_exact_match() {
        let versions = vec![
            create_test_version("1.0.0", Some("1.0.0.0")),
            create_test_version("1.2.3", Some("1.2.3.0")),
            create_test_version("2.0.0", Some("2.0.0.0")),
        ];

        let constraint = parse_constraint("1.2.3").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        assert_eq!(best.version, "1.2.3");
    }

    #[test]
    fn test_find_best_version_no_match() {
        let versions = vec![
            create_test_version("1.0.0", Some("1.0.0.0")),
            create_test_version("1.2.0", Some("1.2.0.0")),
        ];

        let constraint = parse_constraint("^2.0").unwrap();
        let result = find_best_version(&versions, &constraint);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_find_best_version_empty_list() {
        let versions = vec![];
        let constraint = parse_constraint("^1.0").unwrap();
        let result = find_best_version(&versions, &constraint);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_find_best_version_prefers_normalized() {
        let versions = vec![
            P2Version {
                version: "v1.2.3".to_string(),
                version_normalized: "1.2.3.0".to_string(),
                dist: None,
                source: None,
                require: None,
                extra: None,
                other: serde_json::Map::new(),
            }
        ];

        let constraint = parse_constraint("^1.2").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should match even though version has 'v' prefix
        assert_eq!(best.version, "v1.2.3");
    }

    #[test]
    fn test_find_best_version_semver_precedence() {
        let versions = vec![
            create_test_version("1.0.0", Some("1.0.0.0")),
            create_test_version("1.0.1", Some("1.0.1.0")),
            create_test_version("1.1.0", Some("1.1.0.0")),
            create_test_version("1.10.0", Some("1.10.0.0")),
        ];

        let constraint = parse_constraint("^1.0").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should pick 1.10.0, not 1.1.0 (proper semver sorting)
        assert_eq!(best.version, "1.10.0");
    }

    #[test]
    fn test_generate_content_hash_consistency() {
        let content1 = r#"{"name":"test/package","require":{"php":">=7.4"}}"#;
        let content2 = r#"{"name":"test/package","require":{"php":">=7.4"}}"#;
        let content3 = r#"{"name":"test/package","require":{"php":">=8.0"}}"#;

        let hash1 = generate_content_hash(content1);
        let hash2 = generate_content_hash(content2);
        let hash3 = generate_content_hash(content3);

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
        
        // Hash should be hex string of expected length (SHA-256 = 64 chars)
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_content_hash_empty() {
        let hash = generate_content_hash("");
        
        // Should still generate a valid hash for empty content
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_version_selection_with_prereleases() {
        let versions = vec![
            create_test_version("1.0.0", Some("1.0.0.0")),
            create_test_version("1.1.0-alpha", Some("1.1.0.0-alpha")),
            create_test_version("1.1.0-beta", Some("1.1.0.0-beta")),
            create_test_version("1.1.0", Some("1.1.0.0")),
        ];

        let constraint = parse_constraint("^1.0").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should prefer stable release over prereleases
        assert_eq!(best.version, "1.1.0");
    }

    #[test]
    fn test_version_with_v_prefix() {
        let versions = vec![
            create_test_version("v1.0.0", Some("1.0.0.0")),
            create_test_version("v1.2.0", Some("1.2.0.0")),
            create_test_version("v2.0.0", Some("2.0.0.0")),
        ];

        let constraint = parse_constraint("^1.0").unwrap();
        let best = find_best_version(&versions, &constraint).unwrap();
        
        // Should handle versions with 'v' prefix correctly
        assert_eq!(best.version, "v1.2.0");
    }
}
