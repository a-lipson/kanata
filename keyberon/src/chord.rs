use crate::layout::Queue;

pub(crate) struct ChordsV2 {
    queue: Queue,
    // Active layers is a thing apparently
    chords: (), // 
    // release behaviour:
    // - associated fake coordinate
    // - when to release
    active_chords: (),
    layers_ignore_combo: (),
    // When a key leaves the combo queue without activating a chord,
    // this activates a timer during which keys cannot activate chords
    // and are always forwarded directly to the standard input queue.
    ticks_to_ignore_combo: (),
}

impl ChordsV2 {
    // TODO: a release that activates a tap+release should probably interact with
    // on_press_release_delay/on_press_release_delay in some way.

    // require_prior_idle_ms
    fn tick_get_outputs(&mut self) {
        todo!("return actions+coordinates. Make sure that delay is good
            releases from released chords,
            and queued events that are not to be processed as a chord")
    }

    // Update the times in the queue without activating anything.
    // Use when there are pending hold-taps...?
    // But still return keys that are found not to be chords.
    fn tick_no_action(&mut self) {
        todo!("return queued events that are not to be processed as a chord")
    }
}
