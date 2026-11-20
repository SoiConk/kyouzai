/// Events describing notable state transitions and user interactions
/// involving a companion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionEvent {
    /// The companion transitioned to another state.
    StateChanged,

    /// The companion was clicked.
    Clicked,

    /// The companion started being dragged.
    DragStarted,

    /// The companion finished being dragged.
    DragEnded,
}
