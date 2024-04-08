// TODO: removeme
#![allow(unused)]

use arraydeque::ArrayDeque;
use heapless::Vec as HVec;
use rustc_hash::FxHashMap;

use crate::{
    action::Action,
    key_code::KeyCode,
    layout::{CustomEvent, Event, Queue, Queued, QueuedAction},
};

enum ReleaseBehaviour {
    OnFirstRelease,
    OnLastRelease,
}

struct ChordV2<'a, T> {
    /// The action associated with this chord.
    action: Action<'a, T>,
    /// The full set of keys that need to be pressed to activate this chord.
    participating_keys: &'a [u16],
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
    mapping: FxHashMap<u16, ChordsForKey<'a, T>>,
}

struct ActiveChord<'a, T> {
    /// Chords uses a virtual coordinate in the keyberon state for an activated chord.
    /// This field tracks which coordinate to release when the chord itself is released.
    coordinate: u16,
    /// Keys left to release.
    /// For OnFirstRelease, this should have length 0.
    remaining_keys_to_release: HVec<u16, 10>,
    /// Necessary to include here make sure that, for OnFirstRelease,
    /// random other releases that are not part of this chord,
    /// do not release this chord.
    participating_keys: &'a [u16],
    /// Action associated with the active chord.
    /// This needs to be stored here
    action: &'a Action<'a, T>,
    /// In the case of Unread, this chord has not yet been consumed by the layout code.
    /// This might happen because of tap-hold-related delays.
    /// In the Releasable status, the active chord has been consumed and can be released.
    status: ActiveChordStatus,
    /// Tracks how old an action is.
    delay: u16,
}

fn tick_ach<T>(acc: &mut ActiveChord<T>) {
    acc.delay = acc.delay.saturating_add(1);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveChordStatus {
    /// -> UnreadPendingRelease if chord released before being consumed
    /// -> Releasable if consumed
    Unread,
    /// -> Released once consumed
    UnreadReleased,
    /// Can remove at any time.
    /// -> Released once released
    Releasable,
    /// Remove on next tick_chv2
    Released,
}
use ActiveChordStatus::*;

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
    /// Optimization: the below is part of skipping processing work - if this is has changed,
    /// then processing work cannot be skipped.
    prev_active_layer: u16,
}

impl<'a, T> ChordsV2<'a, T> {
    pub fn push_back_chv2(&mut self, item: Queued) -> Option<Queued> {
        self.queue.push_back(item)
    }

    pub(crate) fn get_action_chv2(&mut self) -> (QueuedAction<'a, T>, bool) {
        match self
            .active_chords
            .iter_mut()
            .find_map(|ach| match ach.status {
                Unread => {
                    ach.status = Releasable;
                    Some((Some(((0, ach.coordinate), ach.delay, ach.action)), false))
                }
                UnreadReleased => {
                    ach.status = Released;
                    Some((Some(((0, ach.coordinate), ach.delay, ach.action)), true))
                }
                Releasable | Released => None,
            }) {
            Some(v) => v,
            None => (None, false),
        }
    }

    /// Update the times in the queue without activating any chords yet.
    /// Returns keys that are found not to be useful in chords.
    pub(crate) fn tick_chv2(&mut self, active_layer: u16) -> SmolQueue {
        let mut q = SmolQueue::new();
        self.queue.iter_mut().for_each(Queued::tick_qd);
        self.active_chords.iter_mut().for_each(tick_ach);
        self.drain_unused_inputs(&mut q);
        self.clear_released_chords(&mut q);
        self.ticks_to_ignore_chord = self.ticks_to_ignore_chord.saturating_sub(1);
        q
    }

    fn drain_unused_inputs(&mut self, drainq: &mut SmolQueue) {
        if self.ticks_to_ignore_chord > 0 {
            drainq.extend(self.queue.drain(0..));
            return;
        }
        self.drain_releases_at_start_of_queue(drainq);

        todo!("handle presses!");

        if self.ticks_to_ignore_chord > 0 {
            drainq.extend(self.queue.drain(0..));
        }
    }

    fn drain_releases_at_start_of_queue(&mut self, drainq: &mut SmolQueue) {
        let mut press_found = false;
        let achs = &mut self.active_chords;
        self.queue.retain(|qd| {
            if press_found {
                true
            } else {
                match qd.event {
                    Event::Press(..) => {
                        press_found = true;
                        false
                    }
                    #[rustfmt::skip]
                    Event::Release(i, j) => {
                        achs.iter_mut().for_each(|ach| {
                            if !ach.participating_keys.contains(&j) {
                                return;
                            }
                            ach.remaining_keys_to_release.retain(|pk| *pk != j);
                            if ach.remaining_keys_to_release.is_empty() {
                                ach.status = match ach.status {
                                    Unread         => UnreadReleased,
                                    UnreadReleased => UnreadReleased,
                                    Releasable     => Released,
                                    Released       => Released,
                                }
                            }
                        });
                        false
                    }
                }
            }
        })
    }

    fn clear_released_chords(&mut self, drainq: &mut SmolQueue) {
        self.active_chords.retain(|ach| {
            if ach.status == Released {
                drainq.push_back(Queued {
                    event: Event::Release(0, ach.coordinate),
                    since: 0,
                });
                false
            } else {
                true
            }
        });
    }
}
