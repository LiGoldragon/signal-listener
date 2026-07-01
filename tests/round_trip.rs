//! Round-trip witnesses for the Listener ordinary signal contract.

use nota::{NotaDecode, NotaEncode, NotaSource};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SubReply,
};
use signal_listener::{
    CaptureSession, CaptureStarted, CaptureStopped, DeliveredTo, Frame, FrameBody, Input,
    OperationKind, Output, Reason, RequestUnimplemented, StartCapture, StartedSession, StopCapture,
    StoppedSession, TranscriptText, UnimplementedOperationKind, UnimplementedReason,
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
}

#[test]
fn start_and_stop_requests_round_trip() {
    let start = Input::Start(StartCapture {});
    assert_eq!(start.operation_kind(), OperationKind::Start);
    ListenerFixture::assert_request_round_trips(start.clone());
    ListenerFixture::assert_nota_round_trips(&start);

    let stop = Input::Stop(StopCapture::new(CaptureSession::new(7)));
    assert_eq!(stop.operation_kind(), OperationKind::Stop);
    ListenerFixture::assert_request_round_trips(stop.clone());
    ListenerFixture::assert_nota_round_trips(&stop);
}

#[test]
fn reply_variants_round_trip() {
    let replies = [
        Output::Started(CaptureStarted::new(StartedSession::new(
            CaptureSession::new(7),
        ))),
        Output::Stopped(CaptureStopped {
            stopped_session: StoppedSession::new(CaptureSession::new(7)),
            transcript_text: TranscriptText::new("hello".to_owned()),
            delivered_to: DeliveredTo::new(signal_listener::OutputTarget::SystemClipboard),
        }),
        Output::RequestUnimplemented(RequestUnimplemented {
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
