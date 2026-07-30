# **Async Actor Execution Model (Cue-Based)**

This document defines the runtime execution architecture, pivoting away from a statically compiled "Task Graph" to a highly dynamic, event-driven **Async Actor Model**.

This approach guarantees that any UI edit—including structural trigger changes—takes effect immediately, even while a sequence is currently executing.

## **1\. The Core Philosophy: Cues as Async Actors**

Instead of an engine that compiles a rigid sequence of tasks, the Execution Engine acts as an **Orchestrator**. Every time a Cue is triggered, the Orchestrator spawns an independent, lightweight async task (an "Actor") to manage that specific Cue's lifecycle.

Because each Cue Actor reads from the shared ArcSwap state dynamically at each phase of its lifecycle, UI edits are inherently live.

## **2\. The Global Workspace State**

To make this work without locks, the Execution Engine holds a thread-safe, atomically swappable reference to the show data.

use arc\_swap::ArcSwap;  
use std::sync::Arc;  
use std::collections::HashMap;

// The UI publishes snapshots of this to the Execution Engine  
pub struct WorkspaceState {  
pub cues: HashMap\<CueId, Arc\<Cue\>\>,  
}

// The Execution Engine holds this globally  
lazy\_static\! {  
static ref SHOW\_STATE: ArcSwap\<WorkspaceState\> \= ArcSwap::from\_pointee(WorkspaceState::default());  
}

When the user makes _any_ edit in the Floem UI, the UI constructs a new WorkspaceState, wraps it in an Arc, and the Execution Engine simply swaps the SHOW\_STATE pointer.

## **3\. The Cue Lifecycle (The Async Task)**

When a Cue is triggered (either by GO or by a cascade), the Orchestrator spawns its lifecycle loop.

Notice how the SHOW\_STATE is queried _dynamically_ at every step. If the UI changes a parameter while the tokio::time::sleep is awaiting, the new parameter is instantly respected on the next line.

async fn execute\_cue\_lifecycle(cue\_id: CueId, dsp\_tx: Sender\<DSPCommand\>, event\_tx: Sender\<CueEvent\>) {  
// 1\. DYNAMIC FETCH: Get the absolute latest state of this cue  
let state \= SHOW\_STATE.load();  
let cue \= state.cues.get(\&cue\_id).expect("Cue must exist");

    // 2\. PRE-WAIT PHASE
    if cue.pre\_wait \> Duration::ZERO {
        tokio::time::sleep(cue.pre\_wait).await;
    }

    // 3\. ACTION PHASE (Send commands to lock-free Audio Ringbuffer)
    // We fetch state AGAIN just in case the UI edited volume during the pre-wait\!
    let latest\_cue \= SHOW\_STATE.load().cues.get(\&cue\_id).unwrap().clone();

    match \&latest\_cue.action {
        CueAction::PlayMedia { file, volume } \=\> {
            dsp\_tx.push(DSPCommand::Play(cue\_id, file.clone(), \*volume));
        }
        CueAction::Fade { target, new\_volume, duration } \=\> {
            dsp\_tx.push(DSPCommand::RampVolume(\*target, \*new\_volume, \*duration));
        }
    }

    // 4\. EMIT "ACTION STARTED" EVENT (Triggers "With" cues)
    let \_ \= event\_tx.send(CueEvent::ActionStarted(cue\_id)).await;

    // 5\. POST-WAIT PHASE
    if latest\_cue.post\_wait \> Duration::ZERO {
        tokio::time::sleep(latest\_cue.post\_wait).await;
    }

    // 6\. EMIT "ACTION FINISHED" EVENT (Triggers "After" cues)
    let \_ \= event\_tx.send(CueEvent::ActionFinished(cue\_id)).await;

}

## **4\. The Event Orchestrator (Dynamic Cascading)**

To handle structural triggers (With Cue X, After Cue X), we do not pre-compile a graph. Instead, we use an **Event Bus**.

The Orchestrator runs an infinite async loop listening to CueEvents emitted by the individual Cue Actors.

async fn run\_orchestrator(mut event\_rx: Receiver\<CueEvent\>, dsp\_tx: Sender\<DSPCommand\>) {  
while let Some(event) \= event\_rx.recv().await {

        // Load the absolute latest UI state
        let state \= SHOW\_STATE.load();

        match event {
            CueEvent::GoPressed(target\_id) \=\> {
                // Manual GO triggers the lifecycle directly
                tokio::spawn(execute\_cue\_lifecycle(target\_id, dsp\_tx.clone(), event\_tx.clone()));
            }

            CueEvent::ActionStarted(parent\_id) \=\> {
                // DYNAMIC LOOKUP: Find any cue currently set to trigger "With" the parent
                for (id, cue) in state.cues.iter() {
                    if cue.trigger \== Trigger::With(parent\_id) {
                        tokio::spawn(execute\_cue\_lifecycle(\*id, dsp\_tx.clone(), event\_tx.clone()));
                    }
                }
            }

            CueEvent::ActionFinished(parent\_id) \=\> {
                // DYNAMIC LOOKUP: Find any cue currently set to trigger "After" the parent
                for (id, cue) in state.cues.iter() {
                    if cue.trigger \== Trigger::After(parent\_id) {
                        tokio::spawn(execute\_cue\_lifecycle(\*id, dsp\_tx.clone(), event\_tx.clone()));
                    }
                }
            }
        }
    }

}

## **5\. Summary of Benefits over Task Graphs**

1. **Immunity to UI Edits:** If Cue A is running its post\_wait, and the operator quickly changes Cue B's trigger from "Manual" to "After Cue A", the Orchestrator will instantly catch it when Cue A emits ActionFinished. A static Task Graph would have missed this completely.
2. **Simplified State Squashing:** Fast-forwarding logic just requires skipping the tokio::time::sleep calls and suppressing DSP Play commands until the target timestamp is reached.
3. **True Separation of Concerns:** Cues encapsulate their own logic and timing. The Orchestrator just routes events. The UI just pushes ArcSwap updates.
