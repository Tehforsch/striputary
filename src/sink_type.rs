#[derive(Clone, Default)]
pub enum SinkType {
    /// Audio playback will not be audible while recording.
    #[default]
    Normal,
    /// Audio playback will be audible while recording.
    Monitor,
}
