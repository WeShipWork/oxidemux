//! Integration tests for streaming response validation.

use oxmux::{
    CancellationReason, CanonicalProtocolResponse, CoreError, InvalidStreamSequence,
    OXMUX_KEEPALIVE_METADATA_KEY, OXMUX_RETRY_EXHAUSTED_METADATA_KEY,
    OXMUX_RETRY_SUMMARY_METADATA_KEY, OXMUX_TIMEOUT_METADATA_KEY, ProtocolMetadata,
    ProtocolPayload, ProtocolPayloadBody, ProtocolResponseStatus, ResponseMode, StreamContent,
    StreamEvent, StreamFailure, StreamMetadata, StreamTerminalState, StreamingCancellationPolicy,
    StreamingFailure, StreamingResponse, StreamingRobustnessPolicy,
};

#[test]
fn complete_response_mode_carries_canonical_response() -> Result<(), CoreError> {
    let response = canonical_response()?;
    let mode = ResponseMode::complete(response.clone());

    assert_eq!(mode.complete_response(), Some(&response));
    assert!(mode.streaming_response().is_none());

    Ok(())
}

#[test]
fn streaming_response_preserves_chunk_and_metadata_order() -> Result<(), CoreError> {
    let events = vec![
        StreamEvent::Metadata(StreamMetadata::new("request-id", "stream-1")?),
        StreamEvent::Content(StreamContent::new(
            ProtocolMetadata::open_ai(),
            ProtocolPayload::opaque("application/json", br#"{"delta":"one"}"#.to_vec()),
        )?),
        StreamEvent::Content(StreamContent::new(
            ProtocolMetadata::open_ai(),
            ProtocolPayload::opaque("application/json", br#"{"delta":"two"}"#.to_vec()),
        )?),
        StreamEvent::Terminal(StreamTerminalState::completed()),
    ];

    let response = StreamingResponse::new(events.clone())?;

    assert_eq!(response.events(), events.as_slice());
    assert_eq!(response.terminal(), Some(&StreamTerminalState::Completed));

    Ok(())
}

#[test]
fn empty_completed_stream_is_valid() -> Result<(), CoreError> {
    let response =
        StreamingResponse::new(vec![
            StreamEvent::Terminal(StreamTerminalState::completed()),
        ])?;

    assert_eq!(response.events().len(), 1);
    assert_eq!(response.terminal(), Some(&StreamTerminalState::Completed));

    Ok(())
}

#[test]
fn metadata_only_stream_is_valid() -> Result<(), CoreError> {
    let response = StreamingResponse::new(vec![
        StreamEvent::Metadata(StreamMetadata::new("rate-limit", "remaining=42")?),
        StreamEvent::Terminal(StreamTerminalState::completed()),
    ])?;

    assert!(matches!(response.events()[0], StreamEvent::Metadata(_)));
    assert_eq!(response.terminal(), Some(&StreamTerminalState::Completed));

    Ok(())
}

#[test]
fn streaming_policy_defaults_are_disabled_and_validated() -> Result<(), CoreError> {
    let default_policy = StreamingRobustnessPolicy::default();
    assert!(default_policy.is_disabled());
    assert_eq!(default_policy.max_attempts(), 1);

    let policy = StreamingRobustnessPolicy::new(
        Some(15_000),
        2,
        Some(120_000),
        StreamingCancellationPolicy::ClientDisconnect,
    )?;
    assert_eq!(policy.keepalive_interval_ms, Some(15_000));
    assert_eq!(policy.bootstrap_retry_count, 2);
    assert_eq!(policy.max_attempts(), 3);
    assert_eq!(policy.timeout_ms, Some(120_000));

    assert!(matches!(
        StreamingRobustnessPolicy::new(Some(0), 0, None, StreamingCancellationPolicy::Disabled),
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidPolicy {
                field: "streaming.keepalive-interval-ms",
                ..
            }
        })
    ));
    assert!(matches!(
        StreamingRobustnessPolicy::new(None, 0, None, StreamingCancellationPolicy::Timeout),
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidPolicy {
                field: "streaming.cancellation",
                ..
            }
        })
    ));

    Ok(())
}

#[test]
fn reserved_oxmux_metadata_requires_typed_helpers() -> Result<(), CoreError> {
    assert!(matches!(
        StreamMetadata::new(OXMUX_KEEPALIVE_METADATA_KEY, "true"),
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidSequence {
                reason: InvalidStreamSequence::ReservedMetadataKey { .. },
            }
        })
    ));

    let failure = StreamFailure::new("upstream_bootstrap", "upstream failed before first byte")?;
    let keepalive = StreamMetadata::keepalive();
    let timeout = StreamMetadata::timeout(30_000)?;
    let retry_summary = StreamMetadata::retry_summary(1, 2)?;
    let retry_exhausted = StreamMetadata::retry_exhausted(3, &failure)?;

    assert_eq!(keepalive.name(), OXMUX_KEEPALIVE_METADATA_KEY);
    assert_eq!(timeout.name(), OXMUX_TIMEOUT_METADATA_KEY);
    assert_eq!(retry_summary.name(), OXMUX_RETRY_SUMMARY_METADATA_KEY);
    assert_eq!(retry_exhausted.name(), OXMUX_RETRY_EXHAUSTED_METADATA_KEY);

    let response = StreamingResponse::new(vec![
        StreamEvent::Metadata(keepalive),
        StreamEvent::Metadata(timeout),
        StreamEvent::Terminal(StreamTerminalState::cancelled(CancellationReason::Timeout)),
    ])?;

    assert_eq!(response.events().len(), 3);
    assert!(matches!(
        response.terminal(),
        Some(StreamTerminalState::Cancelled {
            reason: CancellationReason::Timeout,
        })
    ));

    Ok(())
}

#[test]
fn cancelled_stream_carries_matchable_reason() -> Result<(), CoreError> {
    let reason = CancellationReason::ClientDisconnected;
    let response = StreamingResponse::new(vec![StreamEvent::Terminal(
        StreamTerminalState::cancelled(reason.clone()),
    )])?;

    assert!(matches!(
        response.terminal(),
        Some(StreamTerminalState::Cancelled { reason: returned_reason })
            if returned_reason == &reason
    ));

    let custom_reason = CancellationReason::other("policy_cancelled", "policy cancelled stream")?;
    assert!(matches!(
        custom_reason,
        CancellationReason::Other { ref code, .. } if code == "policy_cancelled"
    ));

    Ok(())
}

#[test]
fn errored_stream_carries_structured_failure_data() -> Result<(), CoreError> {
    let failure = StreamFailure::new("upstream_error", "upstream ended with error")?;
    let response = StreamingResponse::new(vec![StreamEvent::Terminal(
        StreamTerminalState::errored(failure.clone()),
    )])?;

    assert!(matches!(
        response.terminal(),
        Some(StreamTerminalState::Errored { failure: returned_failure })
            if returned_failure == &failure
    ));

    Ok(())
}

#[test]
fn constructor_rejects_missing_terminal_event_structurally() -> Result<(), CoreError> {
    let error = StreamingResponse::new(vec![StreamEvent::Content(StreamContent::new(
        ProtocolMetadata::open_ai(),
        ProtocolPayload::empty(),
    )?)]);

    assert!(matches!(
        error,
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidSequence {
                reason: InvalidStreamSequence::MissingTerminal,
            }
        })
    ));

    Ok(())
}

#[test]
fn constructor_rejects_multiple_terminal_events_structurally() {
    let error = StreamingResponse::new(vec![
        StreamEvent::Terminal(StreamTerminalState::completed()),
        StreamEvent::Terminal(StreamTerminalState::cancelled(
            CancellationReason::UserRequested,
        )),
    ]);

    assert!(matches!(
        error,
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidSequence {
                reason: InvalidStreamSequence::MultipleTerminals {
                    first_terminal_index: 0,
                    terminal_index: 1,
                },
            }
        })
    ));
}

#[test]
fn constructor_rejects_events_after_terminal_structurally() -> Result<(), CoreError> {
    let error = StreamingResponse::new(vec![
        StreamEvent::Terminal(StreamTerminalState::completed()),
        StreamEvent::Content(StreamContent::new(
            ProtocolMetadata::open_ai(),
            ProtocolPayload::empty(),
        )?),
    ]);

    assert!(matches!(
        error,
        Err(CoreError::Streaming {
            failure: StreamingFailure::InvalidSequence {
                reason: InvalidStreamSequence::EventAfterTerminal {
                    terminal_index: 0,
                    event_index: 1,
                },
            }
        })
    ));

    Ok(())
}

#[test]
fn streaming_error_display_is_human_readable() -> Result<(), CoreError> {
    let error = CoreError::Streaming {
        failure: StreamingFailure::PreStreamFailure {
            failure: StreamFailure::new("transport_unavailable", "transport unavailable")?,
        },
    };

    assert_eq!(error.to_string(), "streaming failed: transport unavailable");

    Ok(())
}

#[test]
fn pre_stream_retry_timeout_and_cancellation_failures_are_matchable() -> Result<(), CoreError> {
    let failure = StreamFailure::new("bootstrap_failed", "bootstrap failed")?;
    let retry_error = CoreError::Streaming {
        failure: StreamingFailure::RetryExhausted {
            total_attempts: 3,
            failure: failure.clone(),
        },
    };
    let timeout_error = CoreError::Streaming {
        failure: StreamingFailure::PreStreamTimeout {
            timeout_ms: 30_000,
            failure: failure.clone(),
        },
    };
    let cancellation_error = CoreError::Streaming {
        failure: StreamingFailure::PreStreamCancellation {
            reason: CancellationReason::ClientDisconnected,
            failure,
        },
    };

    assert!(matches!(
        retry_error,
        CoreError::Streaming {
            failure: StreamingFailure::RetryExhausted {
                total_attempts: 3,
                ..
            }
        }
    ));
    assert!(
        timeout_error
            .to_string()
            .contains("timed out before first event")
    );
    assert!(matches!(
        cancellation_error,
        CoreError::Streaming {
            failure: StreamingFailure::PreStreamCancellation {
                reason: CancellationReason::ClientDisconnected,
                ..
            }
        }
    ));

    Ok(())
}

fn canonical_response() -> Result<CanonicalProtocolResponse, CoreError> {
    CanonicalProtocolResponse::new(
        ProtocolMetadata::open_ai(),
        ProtocolResponseStatus::success(),
        ProtocolPayload {
            content_type: Some("application/json".to_string()),
            body: ProtocolPayloadBody::Opaque(br#"{"output":"ok"}"#.to_vec()),
        },
    )
}
