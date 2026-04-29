//! Integration tests for protocol translation boundaries.

use oxmux::{
    CanonicalProtocolRequest, CanonicalProtocolResponse, CodexProtocolMetadata, CoreError,
    OpenAiProtocolMetadata, ProtocolBoundary, ProtocolFamily, ProtocolMetadata, ProtocolPayload,
    ProtocolPayloadBody, ProtocolResponseStatus, ProtocolTranslationDirection,
    ProtocolTranslationOutcome, ProtocolTranslator, ProviderSpecificProtocolMetadata,
};

#[test]
fn canonical_request_and_response_shapes_are_deterministic() -> Result<(), CoreError> {
    let first_request = CanonicalProtocolRequest::new(
        ProtocolMetadata::open_ai(),
        "gpt-4.1",
        ProtocolPayload::opaque("application/json", br#"{"messages":[]}"#.to_vec()),
    )?;
    let second_request = CanonicalProtocolRequest::new(
        ProtocolMetadata::open_ai(),
        "gpt-4.1",
        ProtocolPayload::opaque("application/json", br#"{"messages":[]}"#.to_vec()),
    )?;

    assert_eq!(first_request, second_request);
    assert_eq!(first_request.protocol.family(), ProtocolFamily::OpenAi);
    assert_eq!(first_request.model, "gpt-4.1");
    assert_eq!(
        first_request.payload.body,
        ProtocolPayloadBody::Opaque(br#"{"messages":[]}"#.to_vec())
    );

    let first_response = CanonicalProtocolResponse::new(
        ProtocolMetadata::claude(),
        ProtocolResponseStatus::success(),
        ProtocolPayload::empty(),
    )?;
    let second_response = CanonicalProtocolResponse::new(
        ProtocolMetadata::claude(),
        ProtocolResponseStatus::success(),
        ProtocolPayload::empty(),
    )?;

    assert_eq!(first_response, second_response);
    assert_eq!(first_response.protocol.family(), ProtocolFamily::Claude);
    assert_eq!(first_response.status.code, 200);

    Ok(())
}

#[test]
fn protocol_metadata_represents_supported_families_explicitly() -> Result<(), CoreError> {
    let metadata = [
        (ProtocolMetadata::open_ai(), ProtocolFamily::OpenAi),
        (ProtocolMetadata::gemini(), ProtocolFamily::Gemini),
        (ProtocolMetadata::claude(), ProtocolFamily::Claude),
        (ProtocolMetadata::codex(), ProtocolFamily::Codex),
        (
            ProtocolMetadata::provider_specific("local-lab", "responses-compatible")?,
            ProtocolFamily::ProviderSpecific,
        ),
    ];

    for (metadata, expected_family) in metadata {
        assert_eq!(metadata.family(), expected_family);
        metadata.validate()?;
    }

    let provider_specific = ProviderSpecificProtocolMetadata::new("local-lab", "custom-json")?;
    assert_eq!(provider_specific.provider_id, "local-lab");
    assert_eq!(provider_specific.format_name, "custom-json");

    Ok(())
}

#[test]
fn invalid_protocol_metadata_returns_structured_errors() {
    let invalid_request =
        CanonicalProtocolRequest::new(ProtocolMetadata::open_ai(), " ", ProtocolPayload::empty());
    assert!(matches!(
        invalid_request,
        Err(CoreError::ProtocolValidation { field: "model", .. })
    ));

    let invalid_provider_metadata = ProviderSpecificProtocolMetadata::new("provider", " ");
    assert!(matches!(
        invalid_provider_metadata,
        Err(CoreError::ProtocolValidation {
            field: "provider_specific.format_name",
            ..
        })
    ));

    let invalid_status = ProtocolResponseStatus::new(99, Some("Continue-ish".to_string()));
    assert!(matches!(
        invalid_status,
        Err(CoreError::ProtocolValidation {
            field: "status.code",
            ..
        })
    ));

    let invalid_metadata = ProtocolMetadata::OpenAi(OpenAiProtocolMetadata {
        api_version: Some(" ".to_string()),
    });
    assert!(matches!(
        invalid_metadata.validate(),
        Err(CoreError::ProtocolValidation {
            field: "openai.api_version",
            ..
        })
    ));
}

#[test]
fn translation_boundary_returns_deferred_outcomes_without_succeeding() -> Result<(), CoreError> {
    let request = CanonicalProtocolRequest::new(
        ProtocolMetadata::OpenAi(OpenAiProtocolMetadata {
            api_version: Some("2024-10-01".to_string()),
        }),
        "gpt-4.1",
        ProtocolPayload::empty(),
    )?;

    let request_outcome = ProtocolBoundary::translate_request(request, ProtocolMetadata::gemini())?;
    assert!(matches!(
        request_outcome,
        ProtocolTranslationOutcome::Deferred(ref deferred)
            if deferred.direction == ProtocolTranslationDirection::Request
                && deferred.source_family == ProtocolFamily::OpenAi
                && deferred.target_family == ProtocolFamily::Gemini
    ));

    if let ProtocolTranslationOutcome::Deferred(deferred) = request_outcome {
        assert!(matches!(
            deferred.into_error(),
            CoreError::ProtocolTranslationDeferred {
                direction: ProtocolTranslationDirection::Request,
                source_family: ProtocolFamily::OpenAi,
                target_family: ProtocolFamily::Gemini,
                ..
            }
        ));
    }

    let response = CanonicalProtocolResponse::new(
        ProtocolMetadata::codex(),
        ProtocolResponseStatus::success(),
        ProtocolPayload::empty(),
    )?;
    let translator = ProtocolBoundary;
    let response_outcome = translator.translate_response(
        response,
        ProtocolMetadata::Codex(CodexProtocolMetadata {
            api_version: Some("preview".to_string()),
        }),
    )?;

    assert!(matches!(
        response_outcome,
        ProtocolTranslationOutcome::Deferred(ref deferred)
            if deferred.direction == ProtocolTranslationDirection::Response
                && deferred.source_family == ProtocolFamily::Codex
                && deferred.target_family == ProtocolFamily::Codex
    ));

    Ok(())
}

#[test]
fn protocol_skeleton_is_usable_through_public_facade() -> Result<(), Box<dyn std::error::Error>> {
    let request = oxmux::CanonicalProtocolRequest::new(
        oxmux::ProtocolMetadata::provider_specific("lab-provider", "chat-json")?,
        "lab-model",
        oxmux::ProtocolPayload::empty(),
    )?;

    let outcome =
        oxmux::ProtocolBoundary::translate_request(request, oxmux::ProtocolMetadata::open_ai())?;

    assert!(matches!(
        outcome,
        oxmux::ProtocolTranslationOutcome::Deferred(deferred)
            if deferred.source_family == oxmux::ProtocolFamily::ProviderSpecific
                && deferred.target_family == oxmux::ProtocolFamily::OpenAi
    ));

    Ok(())
}
