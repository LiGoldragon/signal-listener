//! Round-trip witnesses for the Listener ordinary signal contract.

use nota::{NotaDecode, NotaEncode, NotaSource};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SubReply,
};
use signal_listener::{
    AcquireMaintenanceLease, ActiveCapture, ActiveCaptureSession, AudioArtifactPath,
    CancellationRequestedSession, CancelledSession, CaptureAlreadyActive,
    CaptureCancellationRequested, CaptureCancelled, CaptureCompletionRequested, CaptureSession,
    CaptureSessionMismatch, CaptureStarted, CaptureStatus, CaptureStopped,
    CompletionRequestedSession, DaemonEpoch, DeliveredTo, DeliveryOutcome, DeliveryOutcomes,
    DurableAudioArtifact, Frame, FrameBody, Input, ListCapturesRequest, MaintenanceLeaseAbsent,
    MaintenanceLeaseCancellation, MaintenanceLeaseEpoch, MaintenanceLeaseRelease,
    NoActiveCapture, OperationKind, Output, OutputTarget, Reason, ReleaseMaintenanceLease,
    RequestUnimplemented, RequestedCaptureSession, RetryCapture, StartCapture, StartedSession,
    StatusRequest, StopCapture, StoppedSession, ToggleCapture, TranscriptText,
    UnimplementedOperationKind, UnimplementedReason, WirePath,
};

struct ListenerFixture;

impl ListenerFixture {
    fn exchange() -> ExchangeIdentifier {
        ExchangeIdentifier::new(
            SessionEpoch::new(1),
            ExchangeLane::Connector,
            LaneSequence::first(),
        )
    }

    fn assert_request_round_trips(request: Input) {
        let frame = Frame::new(FrameBody::Request {
            exchange: Self::exchange(),
            request: request.clone().into_request(),
        });
        let bytes = frame.encode_length_prefixed().expect("encode request");
        let decoded = Frame::decode_length_prefixed(&bytes).expect("decode request");
        match decoded.into_body() {
            FrameBody::Request {
                request: decoded_request,
                ..
            } => assert_eq!(decoded_request.payloads().head(), &request),
            other => panic!("expected request frame, got {other:?}"),
        }
    }

    fn assert_reply_round_trips(reply: Output) {
        let frame = Frame::new(FrameBody::Reply {
            exchange: Self::exchange(),
            reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply.clone()))),
        });
        let bytes = frame.encode_length_prefixed().expect("encode reply");
        let decoded = Frame::decode_length_prefixed(&bytes).expect("decode reply");
        match decoded.into_body() {
            FrameBody::Reply {
                reply: decoded_reply,
                ..
            } => match decoded_reply {
                Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                    SubReply::Ok(payload) => assert_eq!(payload, reply),
                    other => panic!("expected accepted reply payload, got {other:?}"),
                },
                Reply::Rejected { reason } => panic!("unexpected rejected reply: {reason:?}"),
            },
            other => panic!("expected reply frame, got {other:?}"),
        }
    }

    fn assert_nota_round_trips<Value>(value: &Value)
    where
        Value: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
    {
        let text = value.to_nota();
        let recovered = NotaSource::new(&text).parse::<Value>().expect("decode");
        assert_eq!(&recovered, value);
    }

    fn audio_artifact() -> DurableAudioArtifact {
        DurableAudioArtifact::new(AudioArtifactPath::new(WirePath::new(
            "/var/lib/persona/listener/captures/7.wav",
        )))
    }
}

#[test]
fn start_stop_and_status_requests_round_trip() {
    let start = Input::Start(StartCapture {});
    assert_eq!(start.operation_kind(), OperationKind::Start);
    ListenerFixture::assert_request_round_trips(start.clone());
    ListenerFixture::assert_nota_round_trips(&start);

    let stop = Input::Stop(StopCapture::new(CaptureSession::new(7)));
    assert_eq!(stop.operation_kind(), OperationKind::Stop);
    ListenerFixture::assert_request_round_trips(stop.clone());
    ListenerFixture::assert_nota_round_trips(&stop);

    let cancel = Input::cancel(CaptureSession::new(7));
    assert_eq!(cancel.operation_kind(), OperationKind::Cancel);
    ListenerFixture::assert_request_round_trips(cancel.clone());
    ListenerFixture::assert_nota_round_trips(&cancel);

    let status = Input::Status(StatusRequest {});
    assert_eq!(status.operation_kind(), OperationKind::Status);
    ListenerFixture::assert_request_round_trips(status.clone());
    ListenerFixture::assert_nota_round_trips(&status);

    let list = Input::ListCaptures(ListCapturesRequest {});
    assert_eq!(list.operation_kind(), OperationKind::ListCaptures);
    ListenerFixture::assert_request_round_trips(list.clone());
    ListenerFixture::assert_nota_round_trips(&list);

    let retry = Input::Retry(RetryCapture::new(CaptureSession::new(7)));
    assert_eq!(retry.operation_kind(), OperationKind::Retry);
    ListenerFixture::assert_request_round_trips(retry.clone());
    ListenerFixture::assert_nota_round_trips(&retry);

    let toggle = Input::Toggle(ToggleCapture {});
    assert_eq!(toggle.operation_kind(), OperationKind::Toggle);
    ListenerFixture::assert_request_round_trips(toggle.clone());
    ListenerFixture::assert_nota_round_trips(&toggle);

    let acquire = Input::AcquireMaintenance(AcquireMaintenanceLease {});
    assert_eq!(acquire.operation_kind(), OperationKind::AcquireMaintenance);
    ListenerFixture::assert_request_round_trips(acquire.clone());
    ListenerFixture::assert_nota_round_trips(&acquire);

    let release = Input::ReleaseMaintenance(ReleaseMaintenanceLease {});
    assert_eq!(release.operation_kind(), OperationKind::ReleaseMaintenance);
    ListenerFixture::assert_request_round_trips(release.clone());
    ListenerFixture::assert_nota_round_trips(&release);
}

#[test]
fn canonical_capture_control_nota_forms_round_trip() {
    let requests = [
        (Input::Toggle(ToggleCapture {}), "Toggle.{}"),
        (Input::cancel(CaptureSession::new(7)), "Cancel.7"),
        (Input::stop(CaptureSession::new(7)), "Stop.7"),
        (
            Input::AcquireMaintenance(AcquireMaintenanceLease {}),
            "AcquireMaintenance.{}",
        ),
        (
            Input::ReleaseMaintenance(ReleaseMaintenanceLease {}),
            "ReleaseMaintenance.{}",
        ),
    ];

    for (request, canonical_notation) in requests {
        assert_eq!(request.to_nota(), canonical_notation);
        ListenerFixture::assert_nota_round_trips(&request);
    }
}

#[test]
fn reply_variants_round_trip() {
    let replies = [
        Output::Started(CaptureStarted::new(StartedSession::new(
            CaptureSession::new(7),
        ))),
        Output::Stopped(CaptureStopped {
            stopped_session: StoppedSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
            transcript_text: TranscriptText::new("hello".to_owned()),
            delivery_outcomes: DeliveryOutcomes::new(vec![DeliveryOutcome::Delivered(
                DeliveredTo::new(OutputTarget::SystemClipboard),
            )]),
        }),
        Output::Cancelled(CaptureCancelled {
            cancelled_session: CancelledSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        }),
        Output::CancellationRequested(CaptureCancellationRequested {
            cancellation_requested_session: CancellationRequestedSession::new(CaptureSession::new(
                7,
            )),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        }),
        Output::status_reported(CaptureStatus::Capturing(ActiveCapture {
            active_capture_session: ActiveCaptureSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        })),
        Output::status_reported(CaptureStatus::Finalizing(ActiveCapture {
            active_capture_session: ActiveCaptureSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        })),
        Output::status_reported(CaptureStatus::Transcribing(ActiveCapture {
            active_capture_session: ActiveCaptureSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        })),
        Output::status_reported(CaptureStatus::Delivered(CaptureSession::new(7))),
        Output::status_reported(CaptureStatus::Error(CaptureSession::new(7))),
        Output::CompletionRequested(CaptureCompletionRequested {
            completion_requested_session: CompletionRequestedSession::new(CaptureSession::new(7)),
            durable_audio_artifact: ListenerFixture::audio_artifact(),
        }),
        Output::maintenance_lease_granted(MaintenanceLeaseEpoch::new(DaemonEpoch::new(42))),
        Output::maintenance_lease_released(MaintenanceLeaseRelease {}),
        Output::maintenance_lease_active(MaintenanceLeaseEpoch::new(DaemonEpoch::new(42))),
        Output::maintenance_lease_not_held(MaintenanceLeaseAbsent {}),
        Output::maintenance_lease_cancelled(MaintenanceLeaseCancellation {}),
        Output::AlreadyActive(CaptureAlreadyActive::new(ActiveCaptureSession::new(
            CaptureSession::new(7),
        ))),
        Output::NoActive(NoActiveCapture {}),
        Output::SessionMismatch(CaptureSessionMismatch {
            active_capture_session: ActiveCaptureSession::new(CaptureSession::new(7)),
            requested_capture_session: RequestedCaptureSession::new(CaptureSession::new(8)),
        }),
        Output::Unimplemented(RequestUnimplemented {
            unimplemented_operation_kind: UnimplementedOperationKind::new(OperationKind::Stop),
            reason: Reason::new(UnimplementedReason::NotBuiltYet),
        }),
    ];

    for reply in replies {
        ListenerFixture::assert_reply_round_trips(reply.clone());
        ListenerFixture::assert_nota_round_trips(&reply);
    }
}

#[test]
fn capture_session_projects_to_integer() {
    let session = CaptureSession::new(42);
    assert_eq!(session.value(), 42);
}

#[test]
fn audio_artifact_path_projects_to_string() {
    let artifact = ListenerFixture::audio_artifact();
    assert_eq!(
        artifact.path().as_str(),
        "/var/lib/persona/listener/captures/7.wav"
    );
}
