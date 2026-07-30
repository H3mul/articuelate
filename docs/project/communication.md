# **Architecture Addendums: Telemetry & Playhead Logic**

This document addresses two specific refinements to the Audio Cue System: managing high-frequency media playback telemetry (progress bars, scrubbing) and formalizing the linear playhead behavior (Early GOs and Preemption).

## **1\. High-Frequency Media Telemetry & Scrubbing**

**The Decision:** Media telemetry (current playhead of a running audio file) must **bypass the Execution Thread** and go directly from the DSP Thread to the UI Thread.

If we force the Execution Engine to collate 60fps playhead progress, we would saturate the watch channel with continuous micro-updates, destroying the Engine's ability to efficiently process discrete state changes (like wait timers finishing).

### **A. The Telemetry Pipeline (DSP ![][image1] UI)**

To make the UI aware of what is playing, the CueId is carried all the way down to the audio layer.

1. **The Launch:** The Execution Engine sends DSPCommand::Play { cue\_id: CueId, file\_path: PathBuf, ... }. The DSP thread maps this CueId directly to the active audio node.
2. **The Telemetry Push:** Every 16ms (60fps), the DSP thread pushes a comprehensive telemetry packet into the ringbuf::Producer\<AudioTelemetry\>:  
   pub enum AudioTelemetry {  
   PlaybackState {  
   cue\_id: CueId,  
   current\_time\_sec: f32,  
   total\_duration\_sec: f32,  
   left\_peak: f32,  
   right\_peak: f32,  
   }  
   }

3. **UI Ingestion:** The UI Thread's 60Hz timer drains this ringbuffer. It updates a single Floem RwSignal\<HashMap\<CueId, PlaybackState\>\>.
4. **Reactivity:** The UI components (the progress bar in the cue row, and the active layer in the Media Sidebar) bind directly to this map. They animate smoothly without ever waking up the Tokio Execution Engine.

### **B. The Scrubbing Pipeline (UI ![][image1] DSP)**

When an operator manually drags the progress bar in the Media Sidebar, the data flows back down, utilizing the Execution Engine purely as a router.

1. **UI Action:** User drags the scrubber to 15.0s. UI sends UserIntent::ScrubMedia(CueId, 15.0) to the Execution Engine via the MPSC channel.
2. **Execution Routing:** The Execution Engine instantly translates this to DSPCommand::Seek(CueId, 15.0) and pushes it to the DSP ringbuffer.
3. **DSP Action:** The DSP thread updates the read-pointer of the underlying audio buffer to 15.0s.

## **2\. Linear Playhead & "Early GO" Preemption**

Instead of analyzing the cuelist for logical "chunks" to compile, the Playhead acts as a strict, linear UI cursor (usize index). This allows for organic show operation, specifically the "Early GO" (triggering the next event before the current one finishes).

### **A. Advancing the Playhead**

When the operator presses GO, the Execution Engine fires the cue currently under the playhead, and then immediately advances the playhead to the next **independent** cue.

- **Rule:** The playhead skips any cue with a Trigger::With condition (since those are slaved to the cue that just fired).
- **Rule:** The playhead **stops** on a cue with a Trigger::After condition, or a Trigger::Manual condition.

_Example:_

- Cue 1 (Manual) \<-- Playhead starts here
- Cue 2 (With Cue 1\)
- Cue 3 (After Cue 1\)
- Cue 4 (Manual)

_When GO is pressed:_ Cue 1 (and implicitly Cue 2\) starts. The Playhead immediately drops to **Cue 3**.

### **B. The "Early GO" (Preemption)**

Because the playhead now sits on Cue 3 (After Cue 1), the system anticipates that Cue 3 will fire automatically when Cue 1's post-wait finishes.

However, if the actors on stage move too fast, the operator can hit **GO** again immediately.

1. **The Action:** The operator presses GO. The UI sends UserIntent::GoPressed to the Execution Engine.
2. **The Resolution:** The Execution Engine looks at the playhead (Cue 3). It sees that Cue 3 is scheduled to fire After Cue 1, but the manual GO acts as an **Override**.
3. **The Result:** The Engine instantly aborts the pending After listener for Cue 3, spawns Cue 3's async actor immediately, and advances the playhead to Cue 4\.

This model correctly divorces the concept of "what is currently playing" from "what is the next thing in the list", allowing the operator to dynamically pace the show, skip waits, or preempt events on the fly.
