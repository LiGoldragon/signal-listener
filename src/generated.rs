#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::type_complexity
)]
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Toggling {
    Began(listener_library::JobId),
    Ended(listener_library::JobId),
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct NoSuchJob(pub listener_library::JobId);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub struct NoAudio(pub listener_library::JobId);
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Input {
    Toggle,
    ToggleCustom(listener_library::Options),
    Cancel,
    Recall,
    RecallJob(listener_library::JobId),
    Retranscribe(listener_library::JobId),
    Jobs,
    History,
    Subscribe,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Refused_Data {
    NothingRecording,
    NothingKept,
    NoBackend,
    UnknownBackend(listener_library::UnknownBackend),
    NoSuchJob(NoSuchJob),
    NoAudio(NoAudio),
    InProgress(listener_library::JobState),
    NoMicrophone,
    Unwritable,
}
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "datom", derive(datom_codec::Datomizable, datom_codec::Composing))]
pub enum Output {
    Toggled(Toggling),
    CustomToggled(Toggling),
    Cancelled(listener_library::JobId),
    Recalled(listener_library::Text),
    JobRecalled(listener_library::Text),
    Retranscribing(listener_library::JobId),
    JobsListed(std::vec::Vec<listener_library::JobState>),
    HistoryListed(std::vec::Vec<listener_library::Past>),
    Subscribed(
        signal::Frame<
            std::vec::Vec<listener_library::JobState>,
            listener_library::JobState,
        >,
    ),
    Refused(Refused_Data),
}
#[rustfmt::skip]
#[allow(clippy::large_enum_variant)]
pub enum Answer {
    Once(Output),
    Subscribed(
        signal::Stream<
            std::vec::Vec<listener_library::JobState>,
            listener_library::JobState,
        >,
    ),
}
#[rustfmt::skip]
#[allow(clippy::result_large_err, clippy::ptr_arg, clippy::borrowed_box)]
pub trait Answering {
    fn toggle(&self) -> std::result::Result<Toggling, Output>;
    fn toggle_custom(
        &self,
        options: &listener_library::Options,
    ) -> std::result::Result<Toggling, Output>;
    fn cancel(&self) -> std::result::Result<listener_library::JobId, Output>;
    fn recall(&self) -> std::result::Result<listener_library::Text, Output>;
    fn recall_job(
        &self,
        job_id: &listener_library::JobId,
    ) -> std::result::Result<listener_library::Text, Output>;
    fn retranscribe(
        &self,
        job_id: &listener_library::JobId,
    ) -> std::result::Result<listener_library::JobId, Output>;
    fn jobs(
        &self,
    ) -> std::result::Result<std::vec::Vec<listener_library::JobState>, Output>;
    fn history(
        &self,
    ) -> std::result::Result<std::vec::Vec<listener_library::Past>, Output>;
    fn subscribe(
        &self,
    ) -> std::result::Result<
        signal::Stream<
            std::vec::Vec<listener_library::JobState>,
            listener_library::JobState,
        >,
        Output,
    >;
}
#[rustfmt::skip]
pub trait Answerable {
    type Answer;
    fn answer<Nexus: Answering>(&self, nexus: &Nexus) -> Self::Answer;
}
#[rustfmt::skip]
impl Answerable for Input {
    type Answer = Answer;
    fn answer<Nexus: Answering>(&self, nexus: &Nexus) -> Answer {
        match self {
            Input::Toggle => {
                Answer::Once(
                    nexus.toggle().map_or_else(|refusal| refusal, Output::Toggled),
                )
            }
            Input::ToggleCustom(carried) => {
                Answer::Once(
                    nexus
                        .toggle_custom(carried)
                        .map_or_else(|refusal| refusal, Output::CustomToggled),
                )
            }
            Input::Cancel => {
                Answer::Once(
                    nexus.cancel().map_or_else(|refusal| refusal, Output::Cancelled),
                )
            }
            Input::Recall => {
                Answer::Once(
                    nexus.recall().map_or_else(|refusal| refusal, Output::Recalled),
                )
            }
            Input::RecallJob(carried) => {
                Answer::Once(
                    nexus
                        .recall_job(carried)
                        .map_or_else(|refusal| refusal, Output::JobRecalled),
                )
            }
            Input::Retranscribe(carried) => {
                Answer::Once(
                    nexus
                        .retranscribe(carried)
                        .map_or_else(|refusal| refusal, Output::Retranscribing),
                )
            }
            Input::Jobs => {
                Answer::Once(
                    nexus.jobs().map_or_else(|refusal| refusal, Output::JobsListed),
                )
            }
            Input::History => {
                Answer::Once(
                    nexus.history().map_or_else(|refusal| refusal, Output::HistoryListed),
                )
            }
            Input::Subscribe => {
                nexus.subscribe().map_or_else(Answer::Once, Answer::Subscribed)
            }
        }
    }
}
