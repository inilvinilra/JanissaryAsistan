use super::*;

#[test]
fn encrypted_file_round_trip_preserves_content() {
    let key = [42_u8; 32];
    let source = b"Jury project report: confidential";

    let encrypted = encrypt_file_bytes(&key, source).expect("file should encrypt");
    assert!(encrypted.starts_with(ENCRYPTED_FILE_PREFIX));
    assert_ne!(&encrypted[ENCRYPTED_FILE_PREFIX.len() + 12..], source);

    let decrypted = decrypt_file_bytes(&key, encrypted).expect("file should decrypt");
    assert_eq!(decrypted, source);
}

#[test]
fn competition_scope_is_extracted_only_from_competition_routes() {
    assert_eq!(
        competition_id_from_path("/competitions/42/stages"),
        Some(42)
    );
    assert_eq!(competition_id_from_path("/projects/42"), None);
    assert_eq!(competition_id_from_path("/competitions/not-a-number"), None);
}

#[test]
fn only_a_competition_manager_can_write_the_report_template() {
    let manager = models::UpsertReportTemplate {
        name: "TEKNOFEST Proje Raporu".into(),
        expected_language: "Turkish".into(),
        min_words: 500,
        max_words: 10_000,
        sections: vec![models::TemplateSection {
            key: "abstract".into(),
            title: "Özet".into(),
            aliases: vec![],
            min_words: 50,
            required: true,
        }],
    };
    assert!(validate_report_template(&manager).is_ok());

    let no_required = models::UpsertReportTemplate {
        sections: vec![models::TemplateSection {
            key: "abstract".into(),
            title: "Özet".into(),
            aliases: vec![],
            min_words: 50,
            required: false,
        }],
        ..manager
    };
    assert!(validate_report_template(&no_required).is_err());
}

#[test]
fn report_template_validation_rejects_bad_input() {
    let base = models::UpsertReportTemplate {
        name: "T".into(),
        expected_language: "Turkish".into(),
        min_words: 500,
        max_words: 10_000,
        sections: vec![
            models::TemplateSection {
                key: "a".into(),
                title: "Özet".into(),
                aliases: vec![],
                min_words: 10,
                required: true,
            },
            models::TemplateSection {
                key: "A".into(),
                title: "Sonuç".into(),
                aliases: vec![],
                min_words: 10,
                required: true,
            },
        ],
    };
    assert!(
        validate_report_template(&base)
            .unwrap_err()
            .contains("Duplicate section key")
    );

    let bad_language = models::UpsertReportTemplate {
        expected_language: "Klingon".into(),
        sections: vec![models::TemplateSection {
            key: "a".into(),
            title: "Özet".into(),
            aliases: vec![],
            min_words: 10,
            required: true,
        }],
        ..base
    };
    assert!(validate_report_template(&bad_language).is_err());

    let inverted = models::UpsertReportTemplate {
        expected_language: "Any".into(),
        min_words: 9_000,
        max_words: 100,
        sections: vec![models::TemplateSection {
            key: "a".into(),
            title: "Özet".into(),
            aliases: vec![],
            min_words: 10,
            required: true,
        }],
        ..bad_language
    };
    assert!(validate_report_template(&inverted).is_err());
}

#[test]
fn role_permissions_restrict_administrative_and_jury_routes() {
    let jury_member = AuthenticatedUser {
        id: 7,
        email: "jury@example.org".into(),
        role: "jury_member".into(),
        competition_id: Some(1),
        category: Some("software".into()),
    };
    let chief_judge = AuthenticatedUser {
        role: "chief_judge".into(),
        ..jury_member.clone()
    };

    assert!(!role_allows_request(&jury_member, &Method::GET, "/users"));
    assert!(!role_allows_request(
        &jury_member,
        &Method::PATCH,
        "/ranking"
    ));
    assert!(role_allows_request(
        &jury_member,
        &Method::POST,
        "/projects/3/jury-scores"
    ));
    assert!(!role_allows_request(
        &jury_member,
        &Method::GET,
        "/projects/3/ai-evaluation"
    ));
    assert!(role_allows_request(
        &chief_judge,
        &Method::PATCH,
        "/ranking"
    ));
    assert!(!role_allows_request(&chief_judge, &Method::GET, "/users"));
    assert!(!role_allows_request(
        &chief_judge,
        &Method::GET,
        "/notifications"
    ));
    assert!(role_allows_request(
        &jury_member,
        &Method::GET,
        "/projects/3/template-compliance"
    ));
    assert!(!role_allows_request(
        &jury_member,
        &Method::PUT,
        "/competitions/1/report-template"
    ));
    let observer = AuthenticatedUser {
        id: 3,
        email: "observer@example.test".into(),
        role: "observer".into(),
        competition_id: Some(1),
        category: None,
    };
    assert!(role_allows_request(&observer, &Method::GET, "/projects/1"));
    assert!(!role_allows_request(&observer, &Method::GET, "/users"));
    assert!(!role_allows_request(&observer, &Method::GET, "/audit"));
    assert!(!role_allows_request(
        &observer,
        &Method::GET,
        "/email-campaigns"
    ));
}

#[test]
fn event_stream_and_observer_safe_reads_follow_role_policy() {
    let observer = AuthenticatedUser {
        id: 3,
        email: "observer@example.test".into(),
        role: "observer".into(),
        competition_id: Some(1),
        category: None,
    };
    let jury_member = AuthenticatedUser {
        id: 4,
        email: "jury@example.test".into(),
        role: "jury_member".into(),
        competition_id: Some(1),
        category: Some("software".into()),
    };

    assert!(role_allows_request(&observer, &Method::GET, "/events"));
    assert!(role_allows_request(&jury_member, &Method::GET, "/events"));
    assert!(observer_can_read_path("/competitions/1/report"));
    assert!(!observer_can_read_path("/projects/1/jury-scores"));
    assert!(!observer_can_read_path("/projects/1/files"));
}

#[test]
fn competition_visibility_requires_an_assigned_competition() {
    let scoped_user = AuthenticatedUser {
        id: 1,
        email: "jury@example.test".into(),
        role: "jury_member".into(),
        competition_id: Some(10),
        category: None,
    };
    let unscoped_user = AuthenticatedUser {
        competition_id: None,
        ..scoped_user.clone()
    };
    assert!(competition_is_visible_to(&scoped_user, 10));
    assert!(!competition_is_visible_to(&scoped_user, 11));
    assert!(!competition_is_visible_to(&unscoped_user, 10));
}

#[test]
fn production_mode_disables_sample_data_by_default() {
    assert!(!sample_data_enabled(true, None));
    assert!(sample_data_enabled(false, None));
    assert!(sample_data_enabled(true, Some("true")));
}

#[test]
fn two_factor_secret_storage_supports_encrypted_and_legacy_values() {
    let stored = protect_totp_secret("JBSWY3DPEHPK3PXP").expect("secret should be stored");
    assert!(stored.starts_with("plain:"));
    assert_eq!(
        unprotect_totp_secret(&stored).expect("stored secret should be readable"),
        "JBSWY3DPEHPK3PXP"
    );
    assert_eq!(
        unprotect_totp_secret("JBSWY3DPEHPK3PXP").expect("legacy secret should be readable"),
        "JBSWY3DPEHPK3PXP"
    );
}

#[test]
fn production_requires_a_virus_scan_unless_explicitly_configured() {
    assert!(virus_scan_required(true, None));
    assert!(virus_scan_required(true, Some("")));
    assert!(!virus_scan_required(false, None));
    assert!(virus_scan_required(false, Some("true")));
}

#[test]
fn two_factor_policy_defaults_to_required_in_production() {
    assert!(two_factor_policy_enabled(true, None));
    assert!(!two_factor_policy_enabled(false, None));
    assert!(two_factor_policy_enabled(false, Some("true")));
    assert!(!two_factor_policy_enabled(true, Some("false")));
}

#[test]
fn rate_limit_ignores_spoofed_forwarded_headers_without_a_trusted_proxy() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-forwarded-for",
        HeaderValue::from_static("203.0.113.17, 10.0.0.1"),
    );
    assert_eq!(
        rate_limit_client(&headers, Some("127.0.0.1"), false),
        "127.0.0.1"
    );
    assert_eq!(
        rate_limit_client(&headers, Some("127.0.0.1"), true),
        "203.0.113.17"
    );
    assert_ne!(rate_limit_key("127.0.0.1".into()), "127.0.0.1");
}

#[test]
fn email_retry_delay_uses_bounded_exponential_backoff() {
    assert_eq!(email_retry_delay(1), Duration::from_secs(30));
    assert_eq!(email_retry_delay(2), Duration::from_secs(60));
    assert_eq!(email_retry_delay(8), Duration::from_secs(3600));
}

#[test]
fn password_reset_tokens_are_sha256_hashed() {
    let hash = password_reset_token_hash("reset-token-value");
    assert_eq!(hash.len(), 64);
    assert_ne!(hash, "reset-token-value");
    assert_eq!(hash, password_reset_token_hash("reset-token-value"));
}

#[test]
fn recovery_codes_are_normalized_and_generated_as_a_set() {
    let codes = generate_recovery_codes();
    assert_eq!(codes.len(), 10);
    assert!(
        codes
            .iter()
            .all(|code| code.len() == 11 && code.as_bytes()[5] == b'-')
    );
    assert_eq!(
        recovery_code_hash("abcde-f1234"),
        recovery_code_hash("ABCDE-F1234")
    );
}

#[test]
fn file_encryption_key_requires_a_base64_encoded_32_byte_value() {
    assert!(
        parse_file_encryption_key(None)
            .expect("missing key is valid outside production")
            .is_none()
    );
    assert!(
        parse_file_encryption_key(Some(""))
            .expect("empty key is treated as missing")
            .is_none()
    );
    assert!(parse_file_encryption_key(Some("invalid")).is_err());
    let encoded = STANDARD.encode([7_u8; 32]);
    assert_eq!(
        parse_file_encryption_key(Some(&encoded)).expect("valid key should parse"),
        Some([7_u8; 32])
    );
}

#[test]
fn research_analysis_marks_missing_external_evidence_as_advisory() {
    let document = models::Document {
        filename: "proposal.md".into(),
        file_type: models::FileType::Markdown,
        raw_text: "Project text".into(),
        word_count: 2,
        headings: vec![],
        keywords: vec!["robotics".into(), "vision".into()],
        references: vec!["https://example.org/reference".into()],
        has_references: true,
        has_abstract: false,
        has_conclusion: false,
        has_methodology: false,
        language: models::Language::english(),
        sections: vec![],
    };
    let analysis = build_project_research(7, Some(2), &document, &[]);

    assert_eq!(analysis.project_id, 7);
    assert_eq!(analysis.source_file_version, Some(2));
    assert_eq!(analysis.originality_score, 0.0);
    assert_eq!(analysis.originality_label, "Insufficient external evidence");
    assert_eq!(analysis.sources.len(), 1);
}

#[test]
fn research_analysis_calculates_keyword_overlap_for_external_sources() {
    let document = models::Document {
        filename: "proposal.md".into(),
        file_type: models::FileType::Markdown,
        raw_text: "Project text".into(),
        word_count: 2,
        headings: vec![],
        keywords: vec!["robotics".into(), "vision".into()],
        references: vec![],
        has_references: false,
        has_abstract: false,
        has_conclusion: false,
        has_methodology: false,
        language: models::Language::english(),
        sections: vec![],
    };
    let source = models::SearchResult {
        title: "Robotics vision paper".into(),
        url: "https://arxiv.org/abs/1234".into(),
        snippet: "Computer vision for robotics".into(),
        source_type: "academic".into(),
        fetched_content: None,
        http_status: 200,
    };
    let analysis = build_project_research(7, None, &document, &[source]);

    assert_eq!(analysis.sources.len(), 1);
    assert_eq!(
        analysis.sources[0].matched_terms,
        vec!["robotics", "vision"]
    );
    assert_eq!(analysis.sources[0].similarity, 1.0);
    assert_eq!(analysis.originality_score, 0.0);
}

#[test]
fn jury_members_cannot_access_ai_research_or_copilot() {
    let jury_member = AuthenticatedUser {
        id: 7,
        email: "jury@example.org".into(),
        role: "jury_member".into(),
        competition_id: Some(1),
        category: Some("software".into()),
    };
    assert!(!role_allows_request(
        &jury_member,
        &Method::GET,
        "/projects/3/research"
    ));
    assert!(!role_allows_request(
        &jury_member,
        &Method::POST,
        "/projects/3/copilot"
    ));
}

fn compliance_fixture(sections: Vec<(&str, bool, &str)>) -> models::TemplateCompliance {
    models::TemplateCompliance {
        project_id: 1,
        template_name: "TEKNOFEST Proje Detay Raporu".into(),
        template_version: 1,
        compliant: true,
        section_score: 100.0,
        sections: sections
            .into_iter()
            .map(|(key, required, status)| models::SectionFinding {
                key: key.into(),
                title: key.into(),
                required,
                status: status.into(),
                matched_heading: None,
                word_count: 0,
                min_words: 0,
                detail: String::new(),
            })
            .collect(),
        language_expected: "Turkish".into(),
        language_detected: "Turkish".into(),
        language_matches: true,
        word_count: 1200,
        min_words: 500,
        max_words: 10_000,
        word_count_within_range: true,
        summary: "özet".into(),
        evaluated_at: String::new(),
    }
}

/// The gate previously compared against a status the template module never
/// emits, so a fully compliant report still failed and no project could reach
/// evaluation.
#[test]
fn the_headings_gate_passes_a_fully_compliant_report() {
    let compliance = compliance_fixture(vec![
        ("abstract", true, "present"),
        ("conclusion", true, "present"),
        ("references", false, "missing"),
    ]);
    let (status, detail) = headings_content_gate(Some(&compliance));
    assert_eq!(status, "passed", "{detail}");
}

#[test]
fn the_headings_gate_counts_missing_and_thin_required_sections() {
    let compliance = compliance_fixture(vec![
        ("abstract", true, "present"),
        ("method", true, "thin"),
        ("conclusion", true, "missing"),
        ("references", false, "missing"),
    ]);
    let (status, detail) = headings_content_gate(Some(&compliance));
    assert_eq!(status, "failed");
    assert!(detail.starts_with("2 required"), "{detail}");
}

#[test]
fn both_gates_stay_pending_without_a_parsed_report() {
    assert_eq!(headings_content_gate(None).0, "pending");
    assert_eq!(language_template_gate(None).0, "pending");
}

#[test]
fn the_language_gate_fails_on_a_language_or_length_mismatch() {
    let mut compliance = compliance_fixture(vec![("abstract", true, "present")]);
    compliance.language_matches = false;
    assert_eq!(language_template_gate(Some(&compliance)).0, "failed");

    let mut compliance = compliance_fixture(vec![("abstract", true, "present")]);
    compliance.word_count_within_range = false;
    assert_eq!(language_template_gate(Some(&compliance)).0, "failed");
}
