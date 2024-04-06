use arraydeque::ArrayDeque;

use crate::{
    action::Action,
    layout::{CustomEvent, Queue, Queued, QueuedAction},
};

/// Like the layout Queue but smaller. 10 is chosen as the total number of human digits
/// on both hands.
pub(crate) type SmolQueue = ArrayDeque<Queued, 10, arraydeque::behavior::Wrapping>;

pub(crate) struct ChordsV2<'a, T> {
    queue: Queue,
    chords: Action<'a, T>, //
    // release behaviour:
    // - associated fake coordinate
    // - when to release
    active_chords: (),
    layers_ignore_chord: (),
    // When a key leaves the combo queue without activating a chord,
    // this activates a timer during which keys cannot activate chords
    // and are always forwarded directly to the standard input queue.
    ticks_to_ignore_chord: u16,
    configured_ticks_to_ignore_chord: u16,
}

impl<'a, T> ChordsV2<'a, T> {
    pub fn push_back_chv2(&mut self, item: Queued) -> Option<Queued> {
        self.queue.push_back(item)
    }

    // require_prior_idle_ms
    pub(crate) fn get_action_chv2(&mut self) -> (QueuedAction<'a, T>, bool) {
        // TODO: make sure to use `since` for QueuedAction delay field
        (
            todo!(
                "return actions+coordinates. Make sure that delay is good
            releases from released chords,
            and queued events that are not to be processed as a chord"
            ),
            self.pause_input_processing(),
        )
    }

    // Update the times in the queue without activating any chords yet.
    // Returns keys that are found not to be useful in chords.
    pub(crate) fn tick_chv2(&mut self) -> SmolQueue {
        let mut q = SmolQueue::new();
        self.queue.iter_mut().for_each(Queued::tick_qd);
        self.drain_unused_inputs(&mut q);
        self.ticks_to_ignore_chord = self.ticks_to_ignore_chord.saturating_sub(1);
        q
    }

    fn pause_input_processing(&self) -> bool {
        // TODO: a release that activates a tap+release should probably interact with
        // on_press_release_delay/on_press_release_delay in some way.
        // put proper logic here
        true
    }

    fn drain_unused_inputs(&mut self, drainq: &mut SmolQueue) {
        let retain_input = |_qd: &_| -> bool {
            if self.ticks_to_ignore_chord > 0 {
                false
            } else {
                todo!("check logic woa")
            }
        };
        self.queue.retain(|qd| {
            if retain_input(qd) {
                true
            } else {
                drainq.push_back(*qd);
                false
            }
        });
        if !drainq.is_empty() {
            self.ticks_to_ignore_chord = self.configured_ticks_to_ignore_chord;
        }
    }
}
