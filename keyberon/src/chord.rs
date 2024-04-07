// TODO: removeme
#![allow(unused)]

use arraydeque::ArrayDeque;
use heapless::Vec as HVec;
use rustc_hash::FxHashMap;

use crate::{
    action::Action,
    key_code::KeyCode,
    layout::{CustomEvent, Queue, Queued, QueuedAction},
};

enum ReleaseBehaviour {
    OnFirstRelease,
    OnLastRelease,
}

struct ChordV2<'a, T> {
    /// The action associated with this chord.
    action: Action<'a, T>,
    /// The full set of keys that need to be pressed to activate this chord.
    paticipating_keys: &'a [u16],
    /// The number of ticks during which, after the first press of a participant,
    /// this chord can be activated if all participants get pressed.
    /// In other words, after the number of ticks defined by `pending_duration`
    /// elapses, this chord can no longer be completed.
    pending_duration: u16,
    /// The layers on which this chord is disabled.
    disabled_layers: &'a [u16],
    /// When should the action for this chord be released.
    release_behaviour: ReleaseBehaviour,
}

struct ChordsForKey<'a, T> {
    /// Chords that this key participates in.
    chords: Vec<&'a ChordV2<'a, T>>,
}

struct ChordsForKeys<'a, T> {
    mapping: FxHashMap<KeyCode, ChordsForKey<'a, T>>,
}

struct ActiveChord<'a, T> {
    /// Chords uses a virtual coordinate in the keyberon state for an activated chord.
    /// This field tracks which coordinate to release when the chord itself is released.
    coordinate: u16,
    /// Keys left to release.
    /// For OnFirstRelease, this should have length 0.
    remaining_keys_to_release: HVec<u16, 10>,
    /// Action associated with the active chord.
    action: &'a Action<'a, T>,
    /// In the case of Unread, this chord has not yet been consumed by the layout code.
    /// This might happen because of tap-hold-related delays. This is an unreleasable state.
    /// In the Releasable status, the active chord has been consumed and can be released.
    status: ActiveChordStatus,
    /// Tracks how old an action is.
    delay: u16,
}

fn tick_ach<T>(acc: &mut ActiveChord<T>) {
    acc.delay = acc.delay.saturating_add(1);
}

enum ActiveChordStatus {
    Unread,
    Releasable,
}

/// Like the layout Queue but smaller. 10 is chosen as the total number of human digits
/// on both hands.
pub(crate) type SmolQueue = ArrayDeque<Queued, 10, arraydeque::behavior::Wrapping>;

pub(crate) struct ChordsV2<'a, T> {
    /// Queued inputs that can potentially activate a chord but have not yet.
    /// Inputs will leave if they are determined that they will not activate a chord,
    /// or if a chord activates.
    queue: Queue,
    /// Information about what chords are possible.
    chords: ChordsForKeys<'a, T>,
    /// Chords that are active, i.e. ones that have not yet been released.
    active_chords: HVec<ActiveChord<'a, T>, 10>,
    /// When a key leaves the combo queue without activating a chord,
    /// this activates a timer during which keys cannot activate chords
    /// and are always forwarded directly to the standard input queue.
    ///
    /// This keeps track of the timer.
    ticks_to_ignore_chord: u16,
    /// Initial value for the above when the appropriate event happens.
    configured_ticks_to_ignore_chord: u16,
    /// Optimization: if there are no new inputs, the code can skip some processing work.
    /// This tracks the next time that a change will happen, so that the processing work
    /// is **not** skipped when something needs to be checked.
    ticks_until_next_state_change: u16,
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

    /// Update the times in the queue without activating any chords yet.
    /// Returns keys that are found not to be useful in chords.
    pub(crate) fn tick_chv2(&mut self, active_layer: u16) -> SmolQueue {
        let mut q = SmolQueue::new();
        self.queue.iter_mut().for_each(Queued::tick_qd);
        self.active_chords.iter_mut().for_each(tick_ach);
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
