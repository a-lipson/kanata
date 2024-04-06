use arraydeque::ArrayDeque;

use crate::{action::Action, layout::{Queue, Queued, QueuedAction}};

/// Like the layout Queue but smaller. 10 is chosen as the total number of human digits
/// on both hands.
pub(crate) type SmolQueue = ArrayDeque<Queued, 10, arraydeque::behavior::Wrapping>;

pub(crate) struct ChordsV2<'a, T> {
    queue: Queue,
    // Active layers is a thing apparently
    chords: Action<'a, T>, //
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

impl<'a, T> ChordsV2<'a, T> {
    pub fn push_back_chv2(&mut self, item: Queued) -> Option<Queued> {
        self.queue.push_back(item)
    }

    // require_prior_idle_ms
    pub(crate) fn get_outputs_chv2(&mut self) -> (QueuedAction<'a, T>, bool) {
        // TODO: make sure to use `since` for QueuedAction delay field
        (todo!("return actions+coordinates. Make sure that delay is good
            releases from released chords,
            and queued events that are not to be processed as a chord"),
            self.pause_input_processing())
    }

    fn pause_input_processing(&self) -> bool {
        // TODO: a release that activates a tap+release should probably interact with
        // on_press_release_delay/on_press_release_delay in some way.
        // put proper logic here
        true
    }

    // Update the times in the queue without activating any chords yet.
    // Returns keys that are found not to be useful in chords.
    pub(crate) fn tick_chv2(&mut self) -> SmolQueue {
        let mut q = SmolQueue::new();
        self.queue.iter_mut().for_each(Queued::tick);
        self.drain_unused_inputs_chv2(&mut q);
        q
    }

    fn drain_unused_inputs_chv2(&mut self, drainq: &mut SmolQueue) {
        let retainlogic = |_qd| -> bool { todo!("logic for retaining an input") };
        self.queue.retain(|qd| if retainlogic(true) {
            true
        } else {
            drainq.push_back(*qd);
            false
        })
    }
}
